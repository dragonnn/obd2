//! Waveshare 2.13-inch monochrome HAT (FPC-A002, SSD1680).

use core::convert::Infallible;

use defmt::{info, warn};
use embassy_nrf::{
    Peri,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    pac::{self, gpio, shared, spim},
    peripherals::{P0_31, P1_04, P1_06},
};
use embassy_time::{Delay, Duration, block_for};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use epdsi::{bus::EpdBusError, prelude::*};

use super::{SpiBus, SpiDevice};

pub const WIDTH: u32 = 122;
pub const HEIGHT: u32 = 250;
pub const FRAME_BYTES: usize = WIDTH.div_ceil(8) as usize * HEIGHT as usize;

struct Waveshare213;

impl EpdPanel for Waveshare213 {
    const WIDTH: u32 = WIDTH;
    const HEIGHT: u32 = HEIGHT;
}

const DC_PIN: usize = 23; // P0.23 / J6.4, shared with the CAN controllers' SDO.
// Two 1 MHz SPI clock periods on the 64 MHz nRF52840 core.
const CAN_SDO_RELEASE_CYCLES: u32 = 128;

type DisplaySpi = SpiDevice<ChipSelect>;
type DisplayBus = SpiBusWrapper<DisplaySpi, DataCommand, Reset, Busy>;
pub type Error = EpdBusError<
    <SpiDevice as embedded_hal::spi::ErrorType>::Error,
    Infallible,
    Infallible,
    Infallible,
>;

pub struct Eink {
    driver: EpdDriver<DisplayBus, Ssd1680Controller, Waveshare213>,
}

impl Eink {
    pub(super) fn new(
        spi: &'static SpiBus,
        cs: Peri<'static, P0_31>,
        irq: Peri<'static, P1_04>,
        rst: Peri<'static, P1_06>,
    ) -> Self {
        let busy = Busy(Input::new(irq, Pull::None));
        let cs = ChipSelect {
            pin: Output::new(cs, Level::High, OutputDrive::HighDrive),
            miso_psel: pac::SPIM3.psel().miso().read(),
            miso_config: pac::P0.pin_cnf(DC_PIN).read(),
        };
        let reset = Reset(Output::new(rst, Level::High, OutputDrive::Standard));
        let bus = SpiBusWrapper::new(SpiDevice::new(spi, cs), DataCommand, reset, busy);
        let controller = Ssd1680Controller::for_panel::<Waveshare213>();
        Self {
            driver: EpdDriver::new(bus, controller),
        }
    }

    /// Reset and initialize the display. Call again after deep sleep.
    pub async fn init(&mut self) -> Result<(), Error> {
        self.driver.init(&mut Delay).await
    }

    /// Refresh from a full monochrome frame with 16 bytes per row and 1 for white.
    pub async fn display_frame(&mut self, frame: &[u8; FRAME_BYTES]) -> Result<(), Error> {
        self.driver.set_cursor(0, 0).await?;
        self.driver
            .display_frame(ColorChannel::BlackWhite, frame, &mut Delay)
            .await
    }

    pub async fn sleep(&mut self) -> Result<(), Error> {
        self.driver.sleep(&mut Delay).await
    }
}

// epdsi sets DC before taking the SPI mutex. These writes only preload the
// GPIO output latch. P0.23 stays an input until the e-ink CS hook owns the bus.
struct DataCommand;

impl ErrorType for DataCommand {
    type Error = Infallible;
}

impl OutputPin for DataCommand {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        pac::P0.outclr().write(|w| w.set_pin(DC_PIN, true));
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        pac::P0.outset().write(|w| w.set_pin(DC_PIN, true));
        Ok(())
    }
}

struct Busy(Input<'static>);

impl ErrorType for Busy {
    type Error = Infallible;
}

impl InputPin for Busy {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let high = self.0.is_high();
        info!("E-ink BUSY high: {}", high);
        Ok(high)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let low = self.0.is_low();
        info!("E-ink BUSY low: {}", low);
        Ok(low)
    }
}

impl Wait for Busy {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.0.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        defmt::trace!("E-ink BUSY high before idle wait: {}", self.0.is_high());
        self.0.wait_for_low().await;
        defmt::trace!("E-ink BUSY released");
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.0.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.0.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.0.wait_for_any_edge().await;
        Ok(())
    }
}

struct ChipSelect {
    pin: Output<'static>,
    miso_psel: shared::regs::Psel,
    miso_config: gpio::regs::PinCnf,
}

impl ErrorType for ChipSelect {
    type Error = Infallible;
}

impl OutputPin for ChipSelect {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        // SpiDevice holds the bus mutex here, so both CAN chip selects are high.
        cortex_m::asm::delay(CAN_SDO_RELEASE_CYCLES);
        let spi = pac::SPIM3;
        spi.enable()
            .write(|w| w.set_enable(spim::vals::Enable::Disabled));
        spi.psel()
            .miso()
            .modify(|w| w.set_connect(shared::vals::Connect::Disconnected));
        pac::P0.pin_cnf(DC_PIN).write(|w| {
            w.set_dir(gpio::vals::Dir::Output);
            w.set_input(gpio::vals::Input::Disconnect);
            w.set_drive(gpio::vals::Drive::H0h1);
        });
        spi.enable()
            .write(|w| w.set_enable(spim::vals::Enable::Enabled));
        warn!("chip select assert");
        self.pin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let spi = pac::SPIM3;
        // SpiDevice also calls this on cancellation. Stop EasyDMA before
        // changing the pin assignment if the write future was dropped.
        if spi.events_end().read() == 0 {
            spi.events_stopped().write_value(0);
            spi.tasks_stop().write_value(1);
            while spi.events_stopped().read() == 0 {}
        }
        warn!("chip select deassert");
        self.pin.set_high();
        spi.enable()
            .write(|w| w.set_enable(spim::vals::Enable::Disabled));
        pac::P0.pin_cnf(DC_PIN).write_value(self.miso_config);
        spi.psel().miso().write_value(self.miso_psel);
        spi.enable()
            .write(|w| w.set_enable(spim::vals::Enable::Enabled));
        Ok(())
    }
}

struct Reset(Output<'static>);

impl ErrorType for Reset {
    type Error = Infallible;
}

impl OutputPin for Reset {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        // Waveshare's HAT uses a 2 ms pulse. Holding RST low for epdsi's
        // full 10 ms delay can cut panel power on the HAT.
        self.0.set_low();
        block_for(Duration::from_millis(2));
        self.0.set_high();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high();
        Ok(())
    }
}
