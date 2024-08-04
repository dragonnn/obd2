use defmt::info;
//use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    dma::{Dma, DmaDescriptor, DmaPriority},
    dma_descriptors, gpio,
    gpio::{Input, Io, Output, Pull},
    interrupt::Priority,
    peripherals::Peripherals,
    prelude::*,
    riscv::singleton,
    rtc_cntl::{sleep::WakeupLevel, Rtc},
    spi::{
        master::{dma::SpiDma, prelude::_esp_hal_spi_master_dma_WithDmaSpi2, Spi},
        FullDuplexMode, SpiMode,
    },
    system::SystemControl,
    timer::{timg::TimerGroup, ErasedTimer, OneShotTimer},
    usb_serial_jtag::UsbSerialJtag,
    Async, Blocking,
};
use sh1122::{
    async_display::buffered_graphics::AsyncBufferedGraphicsMode, display::DisplayRotation, AsyncDisplay, PixelCoord,
};
use static_cell::{make_static, StaticCell};

use crate::{cap1188::Cap1188, mcp2515::Mcp2515, obd2, power, types};

// WARNING may overflow and wrap-around in long lived apps
/*defmt::timestamp!("{=u32:us}", {
    // NOTE(interrupt-safe) single instruction volatile read operation

    (esp_hal::systimer::SystemTimer::now() / (esp_hal::systimer::SystemTimer::TICKS_PER_SECOND / 1_000_000)) as u32
});*/

pub struct Hal {
    pub display1: types::Display1,
    pub display2: types::Display2,
    pub buttons: types::Cap1188,
    pub obd2: obd2::Obd2,
    //pub usb_serial: types::UsbSerial,
    pub power: power::Power,
    pub led: types::Led,
}

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

pub fn init() -> Hal {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);
    delay.delay_micros(100u32);

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let timer0: ErasedTimer = timg0.timer0.into();
    let timers = [OneShotTimer::new(timer0)];
    let timers = mk_static!([OneShotTimer<ErasedTimer>; 1], timers);
    esp_hal_embassy::init(&clocks, timers);
    info!("Embassy initialized");
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    info!("io initialized");
    let mut rtc = Rtc::new(peripherals.LPWR, None);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let sclk = io.pins.gpio6;
    let mosi = io.pins.gpio7;
    let miso = io.pins.gpio2;

    let (descriptors, rx_descriptors) = dma_descriptors!(32000);

    let spi = Spi::new(peripherals.SPI2, 6000.kHz(), SpiMode::Mode0, &clocks)
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso)
        .with_dma(dma_channel.configure_for_async(false, DmaPriority::Priority0), descriptors, rx_descriptors);

    /*let mut dc = io.pins.gpio23.into_push_pull_output();
    let mut cs_display1 = io.pins.gpio18.into_push_pull_output();
    let mut cs_display2 = io.pins.gpio19.into_push_pull_output();
    let mut cs_cap1188 = io.pins.gpio20.into_push_pull_output();
    let mut cs_mcp2515 = io.pins.gpio16.into_push_pull_output();
    let int_mcp2515 = io.pins.gpio4.into_pull_down_input();
    let mut rs = io.pins.gpio22.into_push_pull_output();
    let mut ing = io.pins.gpio5.into_pull_down_input();
    let mut int_cap1188 = io.pins.gpio3.into_pull_down_input();*/

    let mut dc = Output::new(io.pins.gpio23, false.into());
    info!("cs display1 init");
    let mut cs_display1 = Output::new(io.pins.gpio18, false.into());
    info!("cs display2 init");
    let mut cs_display2 = Output::new(io.pins.gpio19, false.into());
    info!("cs cap init");
    let mut cs_cap1188 = Output::new(io.pins.gpio20, false.into());
    info!("cs mcp init");
    let mut cs_mcp2515 = Output::new(io.pins.gpio17, false.into());
    info!("int mcp init");
    let int_mcp2515 = Input::new(io.pins.gpio4, Pull::Up);
    info!("int rs init");
    let mut rs = Output::new(io.pins.gpio22, false.into());
    info!("int ing init");
    let mut ing = Input::new(io.pins.gpio5, Pull::Down);
    info!("int cap init");
    let int_cap1188 = Input::new(io.pins.gpio3, Pull::Up);

    let mut led = Output::new(io.pins.gpio0, false.into());

    info!("delay init");

    dc.set_high();
    rs.set_low();
    cs_display1.set_high();
    cs_display2.set_high();
    cs_cap1188.set_high();
    cs_mcp2515.set_high();
    delay.delay_micros(1u32);
    rs.set_high();

    delay.delay_micros(1u32);

    rs.set_low();
    delay.delay_micros(1u32);
    rs.set_high();
    delay.delay_micros(1u32);
    /*for _ in 0..3000 {
        dc.set_low();
        delay.delay_micros(100u32);
        dc.set_high();
        delay.delay_micros(100u32);
    }*/

    let dc2 = unsafe { core::ptr::read(&dc) };

    static SPI_BUS: StaticCell<
        Mutex<
            CriticalSectionRawMutex,
            esp_hal::spi::master::dma::SpiDma<
                '_,
                esp_hal::peripherals::SPI2,
                esp_hal::dma::Channel0,
                FullDuplexMode,
                Async,
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

    let display1 = AsyncDisplay::new(interface1, PixelCoord(256, 64), PixelCoord(0, 0), DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    let display2 = AsyncDisplay::new(interface2, PixelCoord(256, 64), PixelCoord(0, 0), DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    let cap1188 = Cap1188::new(cap1188_spi, int_cap1188);
    let mcp2515 = Mcp2515::new(mcp2515_spi, int_mcp2515);

    //let mut usb_serial = UsbSerialJtag::new_async(peripherals.USB_DEVICE);

    info!("HAL initialized");

    Hal {
        display1,
        display2,
        buttons: cap1188,
        obd2: obd2::Obd2::new(mcp2515),
        //usb_serial,
        power: power::Power::new(ing, delay, rtc),
        led,
    }
}
