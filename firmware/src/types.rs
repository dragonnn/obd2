use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use esp_hal::{gpio::*, spi::FullDuplexMode, Async};

pub type Spi = Mutex<
    CriticalSectionRawMutex,
    esp_hal::spi::master::SpiDma<'static, esp_hal::peripherals::SPI2, esp_hal::dma::Channel0, FullDuplexMode, Async>,
>;

pub use crate::hal::SpiBus;

pub type Mcp2515 = crate::mcp2515::Mcp2515<
    SpiDeviceWithConfig<'static, CriticalSectionRawMutex, SpiBus, Output<'static, GpioPin<17>>>,
    Input<'static, GpioPin<4>>,
>;

pub type Sh1122<const CS: u8> = sh1122::AsyncDisplay<
    display_interface_spi::SPIInterface<
        SpiDeviceWithConfig<'static, CriticalSectionRawMutex, SpiBus, Output<'static, GpioPin<CS>>>,
        Output<'static, GpioPin<23>>,
    >,
    sh1122::async_display::buffered_graphics::AsyncBufferedGraphicsMode,
>;

pub type Cap1188 = crate::cap1188::Cap1188<
    SpiDeviceWithConfig<'static, CriticalSectionRawMutex, SpiBus, Output<'static, GpioPin<20>>>,
    Input<'static, GpioPin<3>>,
>;

pub type UsbSerial = esp_hal::usb_serial_jtag::UsbSerialJtag<'static, esp_hal::Async>;

pub type IngGpio = Input<'static, GpioPin<5>>;

pub type Display2 = Sh1122<19>;
pub type Display1 = Sh1122<18>;

pub type Led = Output<'static, GpioPin<0>>;
pub type Rs = Output<'static, GpioPin<22>>;
