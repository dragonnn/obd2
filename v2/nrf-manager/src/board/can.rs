//! Board configuration around the mcp251xfd async driver.
//! CAN1/U16 + U6 is OBD2 CAN; CAN2/U15 + U8 is CAN LISTEN.

use crate::board::SpiDevice;
use defmt::info;
use embassy_nrf::gpio::Output;
use embassy_time::{Delay, Timer};
use mcp251xfd::{
    CiInt, ClockConfig, Config, Fifo, FifoLayout, Filter, FilterMatch, Frame, MCP251xFdAsync,
    NominalBitTiming, OperationMode, PayloadSize, RxFrame, Variant,
    registers::{Osc, addr},
};
pub type Error = mcp251xfd::Error<<SpiDevice as embedded_hal::spi::ErrorType>::Error>;

// Initial classic CAN configuration; transceivers stay shut down at boot.
const CONFIG: Config = Config {
    clock: ClockConfig::MHZ40,
    nominal: NominalBitTiming::KBPS500_40MHZ,
    data: None,
};

pub const TX: Fifo = Fifo::F1;
pub const RX: Fifo = Fifo::F2;
pub const LAYOUT: FifoLayout =
    FifoLayout::new()
        .tx_fifo(TX, PayloadSize::B8, 8)
        .rx_fifo(RX, PayloadSize::B8, 32);

/// Board configuration around the crate's async driver.
pub struct Can {
    driver: MCP251xFdAsync<SpiDevice>,
    transceiver_shutdown: Output<'static>,
}

impl Can {
    pub fn new(spi: SpiDevice, transceiver_shutdown: Output<'static>) -> Self {
        Self {
            driver: MCP251xFdAsync::new(spi),
            transceiver_shutdown,
        }
    }

    pub async fn init(&mut self) -> Result<Variant, Error> {
        let mut delay = Delay;
        info!("before variant");
        let variant = self.driver.init(&CONFIG, &mut delay).await?;
        info!("before apply layout");
        self.driver.apply_layout(&LAYOUT).await?;
        info!("before set filter");
        self.driver
            .set_filter(Filter::F0, FilterMatch::accept_all(), RX)
            .await?;
        info!("before set configure interrupts");
        self.driver
            .configure_interrupts(CiInt(0).with_rxie(true))
            .await?;
        Ok(variant)
    }

    /// Wake a configured controller and start classic CAN operation.
    pub async fn wake(&mut self) -> Result<(), Error> {
        let osc = Osc(self.driver.read_register_raw(addr::OSC).await?);
        if osc.osc_disabled() {
            self.driver
                .write_register_raw(addr::OSC, osc.with_osc_disabled(false).0)
                .await?;
        }

        for _ in 0..100 {
            let osc = Osc(self.driver.read_register_raw(addr::OSC).await?);
            if osc.osc_ready() && !osc.osc_disabled() {
                self.transceiver_shutdown.set_low();
                return self
                    .driver
                    .set_mode(OperationMode::Normal20, &mut Delay)
                    .await;
            }
            Timer::after_millis(1).await;
        }
        Err(Error::ClockNotReady)
    }

    pub async fn transmit(&mut self, frame: &Frame) -> Result<(), Error> {
        self.driver.transmit(TX, frame).await
    }

    pub async fn receive(&mut self) -> Result<RxFrame, Error> {
        self.driver.receive(RX).await
    }

    pub async fn sleep(&mut self) -> Result<(), Error> {
        self.driver
            .set_mode(OperationMode::Configuration, &mut Delay)
            .await?;
        self.driver
            .set_mode(OperationMode::Sleep, &mut Delay)
            .await?;
        // Give it time to finish the transition
        embassy_time::Timer::after_millis(10).await;

        // Normal Sleep allows SPI register inspection
        let osc = self
            .driver
            .read_register_raw(mcp251xfd::registers::addr::OSC)
            .await?;
        let con = self
            .driver
            .read_register_raw(mcp251xfd::registers::addr::C1CON)
            .await?;
        let intr = self
            .driver
            .read_register_raw(mcp251xfd::registers::addr::C1INT)
            .await?;

        defmt::info!(
            "OSC={:#010x}, CiINT={:#010x}, WAKIE={}, WAKIF={}",
            osc,
            intr,
            intr & (1 << 30) != 0,
            intr & (1 << 14) != 0
        );
        Ok(())
    }

    pub async fn idle(&mut self) -> Result<(), Error> {
        let osc = self
            .driver
            .read_register_raw(mcp251xfd::registers::addr::OSC)
            .await?;
        let con = self
            .driver
            .read_register_raw(mcp251xfd::registers::addr::C1CON)
            .await?;

        let osc_disabled = (osc & (1 << 2)) != 0;
        let osc_ready = (osc & (1 << 10)) != 0;
        let opmode = (con >> 21) & 0x07;
        let reqop = (con >> 24) & 0x07;

        let sleeping = osc_disabled && !osc_ready && opmode == 4;

        info!(
            "MCP sleep={}, OSC={:#010x}, CON={:#010x}, OPMOD={}, REQOP={}",
            sleeping, osc, con, opmode, reqop
        );
        self.transceiver_shutdown.set_high();
        Ok(())
    }
}
