use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use esp_hal::{gpio::*, spi::AnySpi, Async};

pub type Spi = Mutex<CriticalSectionRawMutex, esp_hal::spi::master::SpiDma<'static, Async>>;

pub use crate::hal::SpiBus;

pub type Mcp2515 = crate::mcp2515::Mcp2515<
    SpiDeviceWithConfig<'static, CriticalSectionRawMutex, SpiBus, Output<'static>>,
    Input<'static>,
>;

pub type Sh1122 = sh1122::AsyncDisplay<
    display_interface_spi::SPIInterface<
        SpiDeviceWithConfig<'static, CriticalSectionRawMutex, SpiBus, Output<'static>>,
        Output<'static>,
    >,
    sh1122::async_display::buffered_graphics::AsyncBufferedGraphicsMode,
>;

pub type Cap1188 = crate::cap1188::Cap1188<
    SpiDeviceWithConfig<'static, CriticalSectionRawMutex, SpiBus, Output<'static>>,
    Input<'static>,
>;

pub type UsbSerial = esp_hal::usb_serial_jtag::UsbSerialJtag<'static, esp_hal::Async>;

pub type IngGpio = Input<'static>;

pub type Display2 = Sh1122;
pub type Display1 = Sh1122;

pub type Led = Output<'static>;
pub type Rs = Output<'static>;
pub type Rtc = &'static Mutex<CriticalSectionRawMutex, esp_hal::rtc_cntl::Rtc<'static>>;
