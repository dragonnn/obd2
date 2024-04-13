use defmt::info;
use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Executor;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_backtrace as _;
use esp_hal::spi::master::prelude::_esp_hal_spi_master_dma_WithDmaSpi2;
//use esp_hal::spi::master::SpiBusController;
use esp_hal::{
    clock::ClockControl,
    dma::Dma,
    dma::DmaDescriptor,
    dma::DmaPriority,
    embassy,
    peripherals::Peripherals,
    prelude::*,
    riscv::singleton,
    spi::{
        master::{dma::SpiDma, Spi},
        FullDuplexMode, SpiMode,
    },
    Delay, IO,
};
use sh1122::{
    async_display::buffered_graphics::AsyncBufferedGraphicsMode, display::DisplayRotation,
    AsyncDisplay, PixelCoord,
};
use static_cell::{make_static, StaticCell};

use crate::cap1188::Cap1188;
use crate::mcp2515::Mcp2515;
use crate::types;

pub struct Hal {
    pub display1: types::Sh1122<10>,
    pub display2: types::Sh1122<1>,
    pub buttons: types::Cap1188,
    pub obd2: types::Mcp2515,
}

pub fn init() -> Hal {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    info!("starting");
    //#[cfg(feature = "embassy-time-systick")]
    embassy::init(
        &clocks,
        esp_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    esp_hal::interrupt::enable(
        esp_hal::peripherals::Interrupt::DMA_CH0,
        esp_hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let sclk = io.pins.gpio6;
    let mosi = io.pins.gpio7;
    let miso = io.pins.gpio5;

    let tx_descriptors = make_static!([DmaDescriptor::EMPTY; 8]);
    let rx_descriptors = make_static!([DmaDescriptor::EMPTY; 8]);

    let spi = Spi::new(peripherals.SPI2, 6u32.MHz(), SpiMode::Mode0, &clocks)
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso)
        .with_dma(dma_channel.configure(
            false,
            tx_descriptors,
            rx_descriptors,
            DmaPriority::Priority0,
        ));

    let mut dc = io.pins.gpio9.into_push_pull_output();
    let mut cs_display1 = io.pins.gpio10.into_push_pull_output();
    let mut cs_display2 = io.pins.gpio1.into_push_pull_output();
    let mut cs_cap1188 = io.pins.gpio3.into_push_pull_output();
    let mut cs_mcp2515 = io.pins.gpio8.into_push_pull_output();
    let int_mcp2515 = io.pins.gpio21.into_pull_down_input();
    let mut rs = io.pins.gpio4.into_push_pull_output();

    let mut delay = Delay::new(&clocks);
    dc.set_high().unwrap();
    rs.set_low().unwrap();
    cs_display1.set_high().unwrap();
    cs_display2.set_high().unwrap();
    cs_cap1188.set_high().unwrap();
    cs_mcp2515.set_high().unwrap();
    delay.delay_ms(1u32);
    rs.set_high().unwrap();

    delay.delay_ms(1u32);

    rs.set_low().unwrap();
    delay.delay_ms(1u32);
    rs.set_high().unwrap();
    delay.delay_ms(1u32);

    let dc2 = unsafe { core::ptr::read(&dc) };

    static SPI_BUS: StaticCell<
        Mutex<
            CriticalSectionRawMutex,
            esp_hal::spi::master::dma::SpiDma<
                '_,
                esp_hal::peripherals::SPI2,
                esp_hal::dma::Channel0,
                FullDuplexMode,
            >,
        >,
    > = StaticCell::new();
    let spi_bus = SPI_BUS.init(Mutex::new(spi));

    let display1_spi = SpiDevice::new(spi_bus, cs_display1);
    let display2_spi = SpiDevice::new(spi_bus, cs_display2);
    let cap1188_spi = SpiDevice::new(spi_bus, cs_cap1188);
    let mcp2515_spi = SpiDevice::new(spi_bus, cs_mcp2515);
    let interface1 = SPIInterface::new(display1_spi, dc);
    let interface2 = SPIInterface::new(display2_spi, dc2);

    let display1 = AsyncDisplay::new(
        interface1,
        PixelCoord(256, 64),
        PixelCoord(0, 0),
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    let display2 = AsyncDisplay::new(
        interface2,
        PixelCoord(256, 64),
        PixelCoord(0, 0),
        DisplayRotation::Rotate180,
    )
    .into_buffered_graphics_mode();

    let cap1188 = Cap1188::new(cap1188_spi);
    let mcp2515 = Mcp2515::new(mcp2515_spi, int_mcp2515);

    Hal {
        display1,
        display2,
        buttons: cap1188,
        obd2: mcp2515,
    }
}
