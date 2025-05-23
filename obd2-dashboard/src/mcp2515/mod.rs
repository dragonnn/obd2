use defmt::{error, info, warn};
use display_interface_spi::SPIInterface;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Delay, Duration, Timer};
use embedded_can::{Frame as _, StandardId};
use embedded_hal_async::spi::{Operation, SpiDevice};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{gpio::InputPin, peripherals::SPI2, spi::master::SpiDma};

mod bitrates;
mod config;
mod frame;
mod idheader;
mod registers;

pub use bitrates::*;
pub use config::*;
pub use frame::*;
pub use idheader::*;
pub use registers::*;

use crate::event::{KiaEvent, KIA_EVENTS};

pub struct Mcp2515<SPI, INT> {
    spi: SPI,
    int: INT,
}

impl<SPI, INT> Mcp2515<SPI, INT>
where
    SPI: SpiDevice<u8>,
    INT: embedded_hal_async::digital::Wait,
{
    pub fn new(spi: SPI, int: INT) -> Self {
        Self { spi, int }
    }

    pub async fn apply_canctrl(&mut self, canctrl: CANCTRL, mut debug: bool) -> bool {
        self.write_register(canctrl).await.ok();
        let mut canctrl_read = [0u8; 1];
        self.read_registers(CANCTRL::ADDRESS, &mut canctrl_read).await.ok();
        let written_canctrl: u8 = canctrl.into();
        let canctrl_read_parsed = CANCTRL::from_bytes(canctrl_read);
        if canctrl_read[0] != written_canctrl
            && !(canctrl_read_parsed.reqop() == OperationMode::ListenOnly && canctrl.reqop() == OperationMode::Sleep)
        {
            if debug {
                error!(
                    "MCP2515 canctrl config failed, expected: {:b}, got: {:b} with mode: {:?}",
                    written_canctrl,
                    canctrl_read[0],
                    canctrl_read_parsed.reqop()
                );
            }
            false
        } else {
            true
        }
    }

    pub async fn apply_config(&mut self, config: &Config<'_>, obd2: bool) -> Result<(), SPI::Error> {
        let mut ok_inits = 0u8;
        let mut previous_canctrl = 0;
        let mut last_init_state_event = embassy_time::Instant::now();
        loop {
            self.reset().await?;
            let mut canctrl = [0u8; 1];
            embassy_time::Timer::after_millis(100).await;
            self.read_registers(0x0F, &mut canctrl).await?;
            let mut debug = false;
            if previous_canctrl != canctrl[0] {
                debug = true;
                previous_canctrl = canctrl[0];

                info!("canctrl: {:b}", canctrl[0]);
                if canctrl[0] != 0b10000111 {
                    error!("MCP2515 is not in configuration mode");
                }
            }

            self.set_bitrate(config.cnf).await?;
            self.write_register(config.rxb0ctrl).await?;
            self.write_register(config.rxb1ctrl).await?;
            for &(filter, id_header) in config.filters {
                self.set_filter(filter, id_header).await?;
            }

            if self.apply_canctrl(config.canctrl, debug).await {
                ok_inits += 1;
                if ok_inits >= 2 {
                    break;
                }
            } else {
                if obd2 && last_init_state_event.elapsed().as_secs() > 10 {
                    KIA_EVENTS.send(KiaEvent::Obd2Init(false)).await;
                    last_init_state_event = embassy_time::Instant::now();
                }
                Timer::after(Duration::from_millis(10)).await;
            }
        }
        Ok(())
    }

    pub async fn apply_interrupts_config(&mut self, interputs_config: CANINTE) -> Result<(), SPI::Error> {
        let caninte: u8 = interputs_config.into();
        self.write_register(interputs_config).await?;
        let mut caninte_read = [0u8; 1];
        self.read_registers(CANINTE::ADDRESS, &mut caninte_read).await?;
        if caninte_read[0] != caninte {
            error!("MCP2515 interrupts config failed");
        }
        Ok(())
    }

    pub async fn clear_interrupts(&mut self) -> Result<(), SPI::Error> {
        let mut data = [0u8; 1];
        data[0] = 0x00;
        self.write_registers(CANINTF::ADDRESS, &data).await?;
        Ok(())
    }

    pub async fn set_bitrate(&mut self, cnf: CNF) -> Result<(), SPI::Error> {
        let bytes = cnf.into_bytes();
        self.write_registers(CNF3::ADDRESS, &bytes).await?;
        Ok(())
    }

    pub async fn set_filter(&mut self, filter: AcceptanceFilter, id: IdHeader) -> Result<(), SPI::Error> {
        self.write_registers(filter as u8, &id.into_bytes()).await?;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), SPI::Error> {
        let mut reset_buf = [0; 1];
        reset_buf[0] = Instruction::Reset as u8;

        self.spi.write(&reset_buf).await?;
        embassy_time::Timer::after_millis(50).await;
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.reset().await;

        let mut config = crate::mcp2515::Config::default().mode(OperationMode::Configuration);
        config.canctrl.set_abat(true);
        info!("Set config mode");
        self.apply_canctrl(config.canctrl, true).await;
        info!("Set config mode end");
        self.write_registers(CNF3::ADDRESS, &[0]).await;
        self.write_registers(CNF2::ADDRESS, &[0]).await;
        self.write_registers(CNF1::ADDRESS, &[0]).await;
        config.canctrl.set_reqop(OperationMode::Sleep);
        info!("Shutting down MCP2515");
        info!("config.canctrl.clken: {:?}", config.canctrl.clken());
        while !self.apply_canctrl(config.canctrl, true).await {
            self.reset().await;
            Timer::after(Duration::from_secs(1)).await;
        }
    }

    pub async fn rx_status(&mut self) -> Result<RxStatusResponse, SPI::Error> {
        let mut rx_status_buf = [0; 1];
        rx_status_buf[0] = Instruction::RxStatus as u8;
        let mut buf = [0];
        self.spi.transaction(&mut [Operation::Write(&rx_status_buf), Operation::Read(&mut buf)]).await?;
        Ok(RxStatusResponse::from_bytes(buf))
    }

    pub async fn status(&mut self) -> Result<ReadStatusResponse, SPI::Error> {
        let mut status_buf = [0; 1];
        status_buf[0] = Instruction::ReadStatus as u8;
        let mut buf = [0];
        self.spi.transaction(&mut [Operation::Write(&status_buf), Operation::Read(&mut buf)]).await?;
        Ok(ReadStatusResponse::from_bytes(buf))
    }

    pub async fn errors(&mut self) -> Result<EFLG, SPI::Error> {
        let mut eflg_buf = [0; 1];
        eflg_buf[0] = Instruction::Read as u8 | EFLG::ADDRESS;
        let mut buf = [0];
        self.spi.transaction(&mut [Operation::Write(&eflg_buf), Operation::Read(&mut buf)]).await?;

        self.modify_register(EFLG::new(), 1 << 7 as u8).await?;
        self.modify_register(EFLG::new(), 1 << 6 as u8).await?;

        Ok(EFLG::from_bytes(buf))
    }

    pub async fn read_rx_buffer(&mut self, buf_idx: RxBuffer) -> Result<frame::CanFrame, SPI::Error> {
        let mut frame_buffer = [0; core::mem::size_of::<frame::CanFrame>()];

        let mut rx_buf = [0; 1];
        rx_buf[0] = Instruction::ReadRxBuffer as u8 | (buf_idx as u8 * 2);
        self.spi.transaction(&mut [Operation::Write(&rx_buf), Operation::Read(&mut frame_buffer)]).await?;

        let mut frame: frame::CanFrame = unsafe { core::mem::transmute(frame_buffer) };
        let mut dlc = frame.dlc();
        if dlc > 8 {
            dlc = 8;
            frame.dlc.set_dlc(8);
        }

        //self.modify_register(CANINTF::new(), 1 << buf_idx as u8)
        //    .await?;

        Ok(frame)
    }

    pub async fn load_tx_buffer(&mut self, buf_idx: TxBuffer, frame: &frame::CanFrame) -> Result<(), SPI::Error> {
        let mut load_tx_buf = [0; 1];
        load_tx_buf[0] = Instruction::LoadTxBuffer as u8 | (buf_idx as u8 * 2);

        let data = &frame.as_bytes()[0..5 + frame.dlc()];

        self.spi.transaction(&mut [Operation::Write(&load_tx_buf), Operation::Write(data)]).await?;

        Ok(())
    }

    pub async fn request_to_send(&mut self, buf_idx: TxBuffer) -> Result<(), SPI::Error> {
        let mut request_to_send_buf = [0; 1];
        request_to_send_buf[0] = Instruction::Rts as u8 | (1 << buf_idx as u8);

        self.spi.transaction(&mut [Operation::Write(&request_to_send_buf)]).await?;

        Ok(())
    }

    async fn read_registers(&mut self, start_address: u8, buf: &mut [u8]) -> Result<(), SPI::Error> {
        let mut read_buf = [0; 2];
        read_buf[0] = Instruction::Read as u8;
        read_buf[1] = start_address;
        self.spi.transaction(&mut [Operation::Write(&read_buf), Operation::Read(buf)]).await?;
        Ok(())
    }

    async fn write_register<R: Register + Into<u8>>(&mut self, r: R) -> Result<(), SPI::Error> {
        let mut write_buf = [0; 3];
        write_buf[0] = Instruction::Write as u8;
        write_buf[1] = R::ADDRESS;
        write_buf[2] = r.into();

        self.spi.transaction(&mut [Operation::Write(&write_buf)]).await?;
        Ok(())
    }

    async fn write_registers(&mut self, start_address: u8, data: &[u8]) -> Result<(), SPI::Error> {
        let mut write_buf = [0; 2];
        write_buf[0] = Instruction::Write as u8;
        write_buf[1] = start_address;

        self.spi.transaction(&mut [Operation::Write(&write_buf), Operation::Write(data)]).await?;
        Ok(())
    }

    pub async fn modify_register<R: Register + Modify + Into<u8>>(
        &mut self,
        reg: R,
        mask: u8,
    ) -> Result<(), SPI::Error> {
        let mut modify_buf = [0; 4];
        modify_buf[0] = Instruction::BitModify as u8;
        modify_buf[1] = R::ADDRESS;
        modify_buf[2] = mask;
        modify_buf[3] = reg.into();

        self.spi.write(&modify_buf).await?;
        Ok(())
    }

    pub async fn interrupt(&mut self) {
        self.int.wait_for_falling_edge().await.ok();
    }
}

/// Filters and Masks of the two receive buffers
#[derive(Copy, Clone, Debug)]
pub enum AcceptanceFilter {
    /// Associated with Receive Buffer 0
    Filter0 = 0x00,
    /// Associated with Receive Buffer 0
    Filter1 = 0x04,
    /// Associated with Receive Buffer 1
    Filter2 = 0x08,
    /// Associated with Receive Buffer 1
    Filter3 = 0x10,
    /// Associated with Receive Buffer 1
    Filter4 = 0x14,
    /// Associated with Receive Buffer 1
    Filter5 = 0x18,
    /// Associated with Receive Buffer 0
    Mask0 = 0x20,
    /// Associated with Receive Buffer 1
    Mask1 = 0x24,
}

/// Transmit buffer
#[derive(Copy, Clone, Debug)]
pub enum TxBuffer {
    /// Transmit buffer 0
    TXB0 = 0,
    /// Transmit buffer 1
    TXB1 = 1,
    /// Transmit buffer 2
    TXB2 = 2,
}

/// Receive buffer
#[derive(Copy, Clone, Debug)]
pub enum RxBuffer {
    /// Receive Buffer 0
    RXB0 = 0,
    /// Receive Buffer 1
    RXB1 = 1,
}

/// Instruction supported by the CAN controller
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Instruction {
    /// Resets internal registers to the default state, sets Configuration mode.
    Reset = 0b1100_0000,
    /// Reads data from the register beginning at the selected address.
    Read = 0b0000_0011,
    /// Writes data to the register beginning at the selected address.
    Write = 0b0000_0010,
    /// Instructs the controller to begin the message transmission sequence for
    /// any of the transmit buffers specified in `0b1000_0nnn`.
    Rts = 0b1000_0000,
    /// Quick polling command that reads several Status bits for transmit and receive functions.
    ReadStatus = 0b1010_0000,
    /// Allows the user to set or clear individual bits in a particular register.
    ///
    /// Note: Not all registers can be bit modified with this command.
    /// Executing this command on registers that are not bit modifiable will force the mask to FFh.
    ///
    /// Registers that can be modified with this command implement [`Modify`].
    BitModify = 0b0000_0101,

    /// Quick polling command that indicates a filter match and message type
    /// (standard, extended and/or remote) of the received message.
    RxStatus = 0b1011_0000,
    /// When reading a receive buffer, reduces the overhead of a normal `Read`
    /// command by placing the Address Pointer at one of four locations, as
    /// indicated by ‘nm’ in `0b1001_0nm0`.
    ///
    /// Note: The associated RX flag bit (`rxNif` bits in the [`CANINTF`] register) will be cleared after bringing CS high.
    ReadRxBuffer = 0b1001_0000,

    /// When loading a transmit buffer, reduces the overhead of a normal `Write`
    /// command by placing the Address Pointer at one of six locations, as
    /// indicated by ‘abc’ in `0b0100_0abc`.
    LoadTxBuffer = 0b0100_0000,
}
