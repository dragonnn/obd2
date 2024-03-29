use defmt::error;
use defmt::info;
use display_interface_spi::SPIInterface;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Delay;
use embassy_time::Duration;
use embassy_time::Timer;
use embedded_hal_async::spi::{Operation, SpiDevice};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{
    dma::Channel0,
    peripherals::SPI2,
    spi::{master::dma::SpiDma, FullDuplexMode},
};

mod bitrates;
mod config;
mod idheader;
mod registers;

pub use bitrates::*;
pub use config::*;
pub use idheader::*;
pub use registers::*;

use self::registers::OperationMode;
use self::registers::Register;
use self::registers::CNF;
use self::registers::CNF3;

#[embassy_executor::task]
pub async fn run(
    mut mcp2515: Mcp2515<
        embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice<
            'static,
            CriticalSectionRawMutex,
            esp_hal::spi::master::dma::SpiDma<
                'static,
                esp_hal::peripherals::SPI2,
                esp_hal::dma::Channel0,
                FullDuplexMode,
            >,
            esp_hal::gpio::GpioPin<esp_hal::gpio::Output<esp_hal::gpio::PushPull>, 8>,
        >,
    >,
) {
    let config = Config::default()
        .mode(OperationMode::NormalOperation)
        .bitrate(clock_8mhz::CNF_10K_BPS)
        .receive_buffer_0(RXB0CTRL::default().with_rxm(RXM::ReceiveAny));
    mcp2515.apply_config(&config).await.unwrap();
    loop {
        info!("Hello world from embassy using esp-hal-async!");

        Timer::after(Duration::from_millis(1_000)).await;
    }
}

pub struct Mcp2515<SPI> {
    spi: SPI,
}

impl<SPI> Mcp2515<SPI>
where
    SPI: SpiDevice<u8>,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub async fn apply_config(&mut self, config: &Config<'_>) -> Result<(), SPI::Error> {
        self.reset().await?;
        self.set_bitrate(config.cnf).await?;
        self.write_register(config.rxb0ctrl).await?;
        self.write_register(config.rxb1ctrl).await?;
        for &(filter, id_header) in config.filters {
            self.set_filter(filter, id_header).await?;
        }
        self.write_register(config.canctrl).await?;
        Ok(())
    }

    pub async fn set_bitrate(&mut self, cnf: CNF) -> Result<(), SPI::Error> {
        self.write_registers(CNF3::ADDRESS, &cnf.into_bytes())
            .await?;
        Ok(())
    }

    pub async fn set_filter(
        &mut self,
        filter: AcceptanceFilter,
        id: IdHeader,
    ) -> Result<(), SPI::Error> {
        self.write_registers(filter as u8, &id.into_bytes()).await?;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), SPI::Error> {
        info!("mcp2515.rs: init()");
        let mut reset_buf = [0; 1];
        reset_buf[0] = Instruction::Reset as u8;

        self.spi.write(&reset_buf).await?;

        info!("mcp2515.rs: init() - done");
        Ok(())
    }

    async fn read_registers(
        &mut self,
        start_address: u8,
        buf: &mut [u8],
    ) -> Result<(), SPI::Error> {
        let mut read_buf = [0; 2];
        read_buf[0] = Instruction::Read as u8;
        read_buf[1] = start_address;
        self.spi
            .transaction(&mut [Operation::Write(&read_buf), Operation::Read(buf)])
            .await?;
        info!("mcp2515.rs: read_registers() - done");
        Ok(())
    }

    async fn write_register<R: Register + Into<u8>>(&mut self, r: R) -> Result<(), SPI::Error> {
        let mut write_buf = [0; 3];
        write_buf[0] = Instruction::Write as u8;
        write_buf[1] = R::ADDRESS;
        write_buf[2] = r.into();

        self.spi
            .transaction(&mut [Operation::Write(&write_buf)])
            .await?;
        info!("mcp2515.rs: write_register() - done");
        Ok(())
    }

    async fn write_registers(&mut self, start_address: u8, data: &[u8]) -> Result<(), SPI::Error> {
        let mut write_buf = [0; 2];
        write_buf[0] = Instruction::Write as u8;
        write_buf[1] = start_address;

        self.spi
            .transaction(&mut [Operation::Write(&write_buf), Operation::Write(data)])
            .await?;
        info!("mcp2515.rs: write_registers() - done");
        Ok(())
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

    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    #[cfg_attr(doc, doc(cfg(any(feature = "mcp2515", feature = "mcp25625"))))]
    /// Quick polling command that indicates a filter match and message type
    /// (standard, extended and/or remote) of the received message.
    RxStatus = 0b1011_0000,
    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    #[cfg_attr(doc, doc(cfg(any(feature = "mcp2515", feature = "mcp25625"))))]
    /// When reading a receive buffer, reduces the overhead of a normal `Read`
    /// command by placing the Address Pointer at one of four locations, as
    /// indicated by ‘nm’ in `0b1001_0nm0`.
    ///
    /// Note: The associated RX flag bit (`rxNif` bits in the [`CANINTF`] register) will be cleared after bringing CS high.
    ReadRxBuffer = 0b1001_0000,
    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    #[cfg_attr(doc, doc(cfg(any(feature = "mcp2515", feature = "mcp25625"))))]
    /// When loading a transmit buffer, reduces the overhead of a normal `Write`
    /// command by placing the Address Pointer at one of six locations, as
    /// indicated by ‘abc’ in `0b0100_0abc`.
    LoadTxBuffer = 0b0100_0000,
}
