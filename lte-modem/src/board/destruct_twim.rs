use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::SERIAL2,
    twim::{self, Twim},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::i2c::{Operation, SevenBitAddress};
use static_cell::StaticCell;

use crate::tasks::reset::request_reset;

bind_interrupts!(struct TwiIrqs {
    UARTE2_SPIM2_SPIS2_TWIM2_TWIS2 => twim::InterruptHandler<SERIAL2>;
});

pub type Sda = embassy_nrf::peripherals::P0_11;
pub type Scl = embassy_nrf::peripherals::P0_12;

pub type I2cBus = Twim<'static, SERIAL2>;

pub struct DestructTwim {
    i2c_bus: Option<I2cBus>,
    errors: u8,
    lifetime: Instant,
}

impl DestructTwim {
    pub async fn new() -> Self {
        unsafe {
            let twi2 = Self::get_bus();

            Self { i2c_bus: Some(twi2), errors: 0, lifetime: Instant::now() }
        }
    }

    unsafe fn get_bus() -> I2cBus {
        let sda = Sda::steal();
        let scl = Scl::steal();
        let mut twi2_config = twim::Config::default();
        twi2_config.scl_high_drive = true;
        twi2_config.sda_high_drive = true;
        twi2_config.scl_pullup = true;
        twi2_config.sda_pullup = true;
        twi2_config.frequency = twim::Frequency::K100;
        let serial = SERIAL2::steal();
        let twi2 = Twim::new(serial, TwiIrqs, sda, scl, twi2_config);

        let pac = nrf9160_pac::Peripherals::steal();
        pac.TWIM2_NS.frequency.write(|w| w.frequency().bits(2673868));
        twi2
    }

    pub async fn reset(&mut self) {
        self.i2c_bus = None;

        error!("twi2_reset");
        self.errors += 1;
        unsafe {
            let mut twim2_scl = Output::new(Scl::steal(), Level::High, OutputDrive::Standard);
            let mut twim2_sda = Output::new(Sda::steal(), Level::High, OutputDrive::Standard);
            for _ in 0..12 {
                twim2_scl.set_low();
                Timer::after(Duration::from_micros(100)).await;
                twim2_scl.set_high();
                Timer::after(Duration::from_micros(100)).await;
            }
            Input::new(Scl::steal(), Pull::Up);
            Input::new(Sda::steal(), Pull::Up);
            Timer::after(Duration::from_millis(5)).await;
            self.i2c_bus = Some(Self::get_bus());
        }
        if self.errors > 50 && self.lifetime.elapsed().as_secs() > 5 * 60 {
            request_reset();
        }
    }
}

impl embedded_hal::i2c::ErrorType for DestructTwim {
    type Error = twim::Error;
}

impl embedded_hal_async::i2c::I2c for DestructTwim {
    async fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        if let Some(i2c_bus) = &mut self.i2c_bus {
            let ret = i2c_bus.transaction(address, operations).await;
            if ret.is_err() {
                self.reset().await;
            }
            ret
        } else {
            Err(twim::Error::Receive)
        }
    }

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        if let Some(i2c_bus) = &mut self.i2c_bus {
            let ret = i2c_bus.read(address, read).await;
            if ret.is_err() {
                self.reset().await;
            }
            ret
        } else {
            Err(twim::Error::Receive)
        }
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        if let Some(i2c_bus) = &mut self.i2c_bus {
            let ret = i2c_bus.write(address, write).await;
            if ret.is_err() {
                self.reset().await;
            }
            ret
        } else {
            Err(twim::Error::Receive)
        }
    }
    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        if let Some(i2c_bus) = &mut self.i2c_bus {
            let ret = i2c_bus.write_read(address, write, read).await;
            if ret.is_err() {
                self.reset().await;
            }
            ret
        } else {
            Err(twim::Error::Receive)
        }
    }
}

pub trait I2cBusReset {
    async fn reset(&mut self);
}

impl I2cBusReset
    for embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'static, CriticalSectionRawMutex, DestructTwim>
{
    async fn reset(&mut self) {}
}
