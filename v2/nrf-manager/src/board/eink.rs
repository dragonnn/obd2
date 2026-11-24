//! Waveshare 2.13-inch monochrome HAT (FPC-A002, SSD1680).

use core::convert::Infallible;

use defmt::{info, warn};
use embassy_nrf::{
    Peri,
    gpio::{Flex, Level, Output, OutputDrive, Pull},
    peripherals::{P0_31, P1_04, P1_06},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::{Mutex, MutexGuard},
};
use embassy_time::{Delay, Duration, block_for};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use epdsi::{bus::EpdBusError, prelude::*};
use static_cell::StaticCell;

use super::{SpiBus, SpiDevice};

pub const WIDTH: u32 = 122;
pub const HEIGHT: u32 = 250;
pub const FRAME_BYTES: usize = WIDTH.div_ceil(8) as usize * HEIGHT as usize;

struct Waveshare213;

impl EpdPanel for Waveshare213 {
    const WIDTH: u32 = WIDTH;
    const HEIGHT: u32 = HEIGHT;
}

type DcBusyGpio = Mutex<CriticalSectionRawMutex, Flex<'static>>;
static DC_BUSY: StaticCell<DcBusyGpio> = StaticCell::new();

type DisplaySpi = SpiDevice<ChipSelect>;
type DisplayBus = SpiBusWrapper<DisplaySpi, DcBusy, Reset, DcBusy>;
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
        let mut irq = Flex::new(irq);
        irq.set_as_input(Pull::None);
        let dc_busy = DcBusy(DC_BUSY.init(Mutex::new(irq)));
        let cs = ChipSelect {
            pin: Output::new(cs, Level::High, OutputDrive::HighDrive),
            dc_busy,
        };
        let reset = Reset(Output::new(rst, Level::High, OutputDrive::Standard));
        let bus = SpiBusWrapper::new(SpiDevice::new(spi, cs), dc_busy, reset, dc_busy);
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

// IRQ connects directly to DC and through 2 kΩ to BUSY. epdsi sees distinct
// pins, while the SPI chip select controls when the shared GPIO drives DC.
#[derive(Clone, Copy)]
struct DcBusy(&'static DcBusyGpio);

impl DcBusy {
    fn pin(&self) -> MutexGuard<'_, CriticalSectionRawMutex, Flex<'static>> {
        // The driver issues DC, CS and BUSY operations sequentially.
        self.0.try_lock().unwrap()
    }
}

impl ErrorType for DcBusy {
    type Error = Infallible;
}

impl OutputPin for DcBusy {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin().set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin().set_high();
        Ok(())
    }
}

impl InputPin for DcBusy {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let pin = self.pin().is_high();
        info!("is_high dc read pin: {}", pin);
        Ok(pin)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let pin = self.pin().is_low();
        info!("is_low dc read pin: {}", pin);
        Ok(pin)
    }
}

impl Wait for DcBusy {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        let mut pin = self.0.lock().await;
        defmt::trace!("E-ink BUSY high before idle wait: {}", pin.is_high());
        pin.wait_for_low().await;
        defmt::trace!("E-ink BUSY released");
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.wait_for_any_edge().await;
        Ok(())
    }
}

struct ChipSelect {
    pin: Output<'static>,
    dc_busy: DcBusy,
}

impl ErrorType for ChipSelect {
    type Error = Infallible;
}

impl OutputPin for ChipSelect {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        warn!("chip select assert");
        self.dc_busy.pin().set_as_output(OutputDrive::HighDrive);
        self.pin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        warn!("chip select deasert");
        self.pin.set_high();
        self.dc_busy.pin().set_as_input(Pull::None);
        /*block_for(Duration::from_millis(2));
        // The shared SPI device also deasserts CS on cancellation.
        let mut dc_busy = self.dc_busy.pin();
        let dc_level = dc_busy.get_output_level();
        // Return DC low before releasing the line. With the HAT level shifter,
        // releasing it directly from high can leave the shared node high at idle.
        dc_busy.set_high();
        dc_busy.set_as_disconnected();
        dc_busy.set_as_input(Pull::None);
        // Preserve the requested DC level for consecutive SPI transactions.
        //dc_busy.set_level(dc_level);*/
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
