use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::{gpio::*, spi::FullDuplexMode};

pub type Mcp2515 = crate::mcp2515::Mcp2515<
    embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice<
        'static,
        CriticalSectionRawMutex,
        esp_hal::spi::master::dma::SpiDma<
            'static,
            esp_hal::peripherals::SPI2,
            esp_hal::dma::Channel0,
            FullDuplexMode,
        >,
        GpioPin<Output<PushPull>, 8>,
    >,
    GpioPin<Input<PullDown>, 21>,
>;

pub type Sh1122<const CS: u8> = sh1122::AsyncDisplay<
    display_interface_spi::SPIInterface<
        embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice<
            'static,
            CriticalSectionRawMutex,
            esp_hal::spi::master::dma::SpiDma<
                'static,
                esp_hal::peripherals::SPI2,
                esp_hal::dma::Channel0,
                FullDuplexMode,
            >,
            GpioPin<Output<PushPull>, CS>,
        >,
        GpioPin<Output<PushPull>, 9>,
    >,
    sh1122::async_display::buffered_graphics::AsyncBufferedGraphicsMode,
>;

pub type Cap1188 = crate::cap1188::Cap1188<
    embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice<
        'static,
        CriticalSectionRawMutex,
        esp_hal::spi::master::dma::SpiDma<
            'static,
            esp_hal::peripherals::SPI2,
            esp_hal::dma::Channel0,
            FullDuplexMode,
        >,
        GpioPin<Output<PushPull>, 3>,
    >,
>;
