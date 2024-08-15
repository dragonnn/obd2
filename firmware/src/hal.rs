use defmt::info;
//use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::{SpiDevice, SpiDeviceWithConfig};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use esp_backtrace as _;
use esp_hal::{
    clock::{ClockControl, Clocks},
    delay::Delay,
    dma::{Dma, DmaPriority},
    dma_descriptors,
    gpio::{Input, Io, Output, Pull},
    peripherals::Peripherals,
    prelude::*,
    rtc_cntl::Rtc,
    spi::{
        master::{prelude::_esp_hal_spi_master_dma_WithDmaSpi2, Spi},
        FullDuplexMode, SpiMode,
    },
    system::SystemControl,
    timer::{timg::TimerGroup, ErasedTimer, OneShotTimer},
    usb_serial_jtag::UsbSerialJtag,
    Async,
};
use sh1122::{display::DisplayRotation, AsyncDisplay, PixelCoord};
use static_cell::StaticCell;

use crate::{cap1188::Cap1188, mcp2515::Mcp2515, obd2, power, types};

defmt::timestamp!("{=u32:us}", { embassy_time::Instant::from_ticks(0).elapsed().as_micros() as u32 });

pub struct Hal {
    pub display1: types::Display1,
    pub display2: types::Display2,
    pub buttons: types::Cap1188,
    pub obd2: obd2::Obd2,
    pub usb_serial: types::UsbSerial,
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

pub struct SpiBus {
    spi: esp_hal::spi::master::dma::SpiDma<
        'static,
        esp_hal::peripherals::SPI2,
        esp_hal::dma::DmaChannel0,
        FullDuplexMode,
        Async,
    >,
    clocks: &'static Clocks<'static>,
}

impl SpiBus {
    pub fn new(
        spi: esp_hal::spi::master::dma::SpiDma<
            'static,
            esp_hal::peripherals::SPI2,
            esp_hal::dma::DmaChannel0,
            FullDuplexMode,
            Async,
        >,
        clocks: &'static Clocks<'static>,
    ) -> Self {
        Self { spi, clocks }
    }
}

impl embedded_hal_async::spi::ErrorType for SpiBus {
    type Error = esp_hal::spi::Error;
}

impl embedded_hal_async::spi::SpiBus for SpiBus {
    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.read(words).await
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.spi.transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.transfer_in_place(words).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.spi.flush().await
    }
}

impl embassy_embedded_hal::SetConfig for SpiBus {
    type Config = u32;

    type ConfigError = ();

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.spi.change_bus_frequency(config.MHz(), self.clocks);
        Ok(())
    }
}

pub fn init() -> Hal {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);
    delay.delay_micros(100u32);

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let timer0: ErasedTimer = timg0.timer0.into();
    let timers = [OneShotTimer::new(timer0)];
    let timers = mk_static!([OneShotTimer<ErasedTimer>; 1], timers);
    esp_hal_embassy::init(&clocks, timers);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let rtc = Rtc::new(peripherals.LPWR);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let sclk = io.pins.gpio6;
    let mosi = io.pins.gpio7;
    let miso = io.pins.gpio2;

    let (descriptors, rx_descriptors) = dma_descriptors!(32000);

    let spi = Spi::new(peripherals.SPI2, 6.MHz(), SpiMode::Mode0, &clocks)
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso)
        .with_dma(dma_channel.configure_for_async(false, DmaPriority::Priority0), descriptors, rx_descriptors);

    let mut dc = Output::new(io.pins.gpio23, false.into());
    let mut cs_display1 = Output::new(io.pins.gpio18, false.into());
    let mut cs_display2 = Output::new(io.pins.gpio19, false.into());
    let mut cs_cap1188 = Output::new(io.pins.gpio20, false.into());
    let mut cs_mcp2515 = Output::new(io.pins.gpio17, false.into());
    let int_mcp2515 = Input::new(io.pins.gpio4, Pull::Up);
    let mut rs = Output::new(io.pins.gpio22, false.into());
    let ing = Input::new(io.pins.gpio5, Pull::Up);
    let int_cap1188 = Input::new(io.pins.gpio3, Pull::Up);
    let led = Output::new(io.pins.gpio0, false.into());

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

    let dc2 = unsafe { core::ptr::read(&dc) };

    static CLOCKS: StaticCell<Clocks> = StaticCell::new();
    let clocks = CLOCKS.init(clocks);
    static SPI_BUS: StaticCell<Mutex<CriticalSectionRawMutex, SpiBus>> = StaticCell::new();
    let spi_bus = SPI_BUS.init(Mutex::new(SpiBus::new(spi, clocks)));

    let display1_spi = SpiDeviceWithConfig::new(spi_bus, cs_display1, 25);
    let display2_spi = SpiDeviceWithConfig::new(spi_bus, cs_display2, 25);
    let cap1188_spi = SpiDeviceWithConfig::new(spi_bus, cs_cap1188, 6);
    let mcp2515_spi = SpiDeviceWithConfig::new(spi_bus, cs_mcp2515, 6);
    let interface1 = SPIInterface::new(display1_spi, dc);
    let interface2 = SPIInterface::new(display2_spi, dc2);

    let display1 = AsyncDisplay::new(interface1, PixelCoord(256, 64), PixelCoord(0, 0), DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    let display2 = AsyncDisplay::new(interface2, PixelCoord(256, 64), PixelCoord(0, 0), DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    let cap1188 = Cap1188::new(cap1188_spi, int_cap1188);
    let mcp2515 = Mcp2515::new(mcp2515_spi, int_mcp2515);

    let usb_serial = UsbSerialJtag::new_async(peripherals.USB_DEVICE);

    info!("HAL initialized");

    Hal {
        display1,
        display2,
        buttons: cap1188,
        obd2: obd2::Obd2::new(mcp2515),
        usb_serial,
        power: power::Power::new(ing, delay, rtc),
        led,
    }
}
