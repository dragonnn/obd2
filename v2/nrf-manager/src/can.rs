//! Board configuration around the mcp251xfd async driver.
//! CAN1/U16 + U6 is OBD2 CAN; CAN2/U15 + U8 is CAN LISTEN.

use crate::board::SpiDevice;
use defmt::info;
use embassy_time::Delay;
use mcp251xfd::{
    CiInt, ClockConfig, Config, Fifo, FifoLayout, Filter, FilterMatch, MCP251xFdAsync,
    NominalBitTiming, OperationMode, PayloadSize, Variant,
};
pub type Error = mcp251xfd::Error<<SpiDevice as embedded_hal::spi::ErrorType>::Error>;

// Initial classic CAN configuration; transceivers stay shut down at boot.
const CONFIG: Config = Config {
    clock: ClockConfig::MHZ40,
    nominal: NominalBitTiming::KBPS500_40MHZ,
    data: None,
};

pub const LISTEN_RX: Fifo = Fifo::F1;
pub const OBD2_TX: Fifo = Fifo::F1;
pub const OBD2_RX: Fifo = Fifo::F2;
pub const LISTEN_LAYOUT: FifoLayout = FifoLayout::new().rx_fifo(LISTEN_RX, PayloadSize::B8, 32);
pub const OBD2_LAYOUT: FifoLayout = FifoLayout::new()
    .tx_fifo(OBD2_TX, PayloadSize::B8, 8)
    .rx_fifo(OBD2_RX, PayloadSize::B8, 32);

/// Board configuration around the crate's async driver.
pub struct Can {
    pub driver: MCP251xFdAsync<SpiDevice>,
    layout: &'static FifoLayout,
    rx_fifo: Fifo,
}

impl Can {
    pub fn new(spi: SpiDevice, layout: &'static FifoLayout, rx_fifo: Fifo) -> Self {
        Self {
            driver: MCP251xFdAsync::new(spi),
            layout,
            rx_fifo,
        }
    }

    pub async fn init(&mut self) -> Result<Variant, Error> {
        let mut delay = Delay;
        info!("before variant");
        let variant = self.driver.init(&CONFIG, &mut delay).await?;
        info!("before apply layout");
        //self.driver.apply_layout(self.layout).await?;
        info!("before set filter");
        /*self.driver
        .set_filter(Filter::F0, FilterMatch::accept_all(), self.rx_fifo)
        .await?;*/
        info!("before set configure interrupts");
        self.driver
            .configure_interrupts(CiInt(0).with_rxie(true))
            .await?;
        Ok(variant)
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
        Ok(())
    }
}
