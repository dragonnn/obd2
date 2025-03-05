use defmt::{info, unwrap, warn};
//use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::{SpiDevice, SpiDeviceWithConfig};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Instant;
use esp_backtrace as _;
use esp_hal::{
    aes::dma::AesDma,
    clock::{Clocks, CpuClock},
    delay::Delay,
    dma::{DmaChannel0, DmaPriority, DmaRxBuf, DmaTxBuf},
    dma_buffers, dma_descriptors,
    gpio::{Input, Io, Output, Pull},
    peripherals::Peripherals,
    rtc_cntl::{Rtc, Rwdt, RwdtStage},
    spi::{
        master::{Spi, SpiDmaBus},
        Mode as SpiMode,
    },
    time::RateExtU32,
    timer::{timg::TimerGroup, OneShotTimer},
    usb_serial_jtag::UsbSerialJtag,
    Async,
};
use esp_ieee802154::{Config, Frame, Ieee802154};
use fugit::{ExtU32, HertzU32};
use sh1122::{display::DisplayRotation, AsyncDisplay, PixelCoord};
use static_cell::StaticCell;

use crate::{cap1188::Cap1188, mcp2515::Mcp2515, obd2, power, types};

defmt::timestamp!("{=u32:us}", { embassy_time::Instant::from_ticks(0).elapsed().as_micros() as u32 });

pub struct Hal {
    pub display1: types::Display1,
    pub display2: types::Display2,
    pub buttons: types::Cap1188,
    pub obd2: obd2::Obd2,
    pub can_listen: types::Mcp2515,
    pub usb_serial: types::UsbSerial,
    pub power: power::Power,
    pub led: types::Led,
    pub ieee802154: Ieee802154<'static>,
    pub rtc: types::Rtc,
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
    spi: SpiDmaBus<'static, Async>,
    speed: Option<HertzU32>,
    elapsed: Instant,
}

impl SpiBus {
    pub fn new(spi: SpiDmaBus<'static, Async>) -> Self {
        Self { spi, speed: None, elapsed: Instant::now() }
    }
}

impl embedded_hal_async::spi::ErrorType for SpiBus {
    type Error = esp_hal::spi::Error;
}

impl embedded_hal_async::spi::SpiBus for SpiBus {
    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.read_async(words).await
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.spi.write_async(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.spi.transfer_async(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.transfer_in_place_async(words).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        //self.spi.flush_async().await
        Ok(())
    }
}

const SPI_RAMP_UP_SECS: u64 = 200;

impl embassy_embedded_hal::SetConfig for SpiBus {
    type Config = u32;

    type ConfigError = ();

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        let mut mhz = config.MHz();
        let mut elapsed_secs = self.elapsed.elapsed().as_secs();
        if elapsed_secs < SPI_RAMP_UP_SECS {
            if elapsed_secs < 1 {
                elapsed_secs = 1;
            }
            mhz = (mhz * elapsed_secs as u32) / SPI_RAMP_UP_SECS as u32;
        }

        if self.speed == Some(mhz) {
            return Ok(());
        }

        let config = esp_hal::spi::master::Config::default().with_frequency(config.MHz()).with_mode(SpiMode::_0);
        self.spi.apply_config(&config).ok();
        self.speed = Some(mhz);
        Ok(())
    }
}

pub fn init() -> Hal {
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    let peripherals = esp_hal::init(config);

    //let system = SystemControl::new(peripherals.SYSTEM);
    //let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new();
    info!("delay init");
    delay.delay_millis(10u32);
    info!("delay done");

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_hal_embassy::init(timg0.timer0);

    let mut rtc = Rtc::new(peripherals.LPWR);

    let sclk = peripherals.GPIO6;
    let mosi = peripherals.GPIO7;
    let miso = peripherals.GPIO2;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = unwrap!(DmaRxBuf::new(rx_descriptors, rx_buffer).ok());
    let dma_tx_buf = unwrap!(DmaTxBuf::new(tx_descriptors, tx_buffer).ok());

    let dma_channel = peripherals.DMA_CH0;
    let spi = Spi::new(peripherals.SPI2, esp_hal::spi::master::Config::default().with_frequency(1.MHz()))
        .unwrap()
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso)
        .with_dma(dma_channel)
        .with_buffers(dma_rx_buf, dma_tx_buf)
        .into_async();

    let mut dc = Output::new(peripherals.GPIO23, false.into());
    let mut cs_display1 = Output::new(peripherals.GPIO18, false.into());
    let mut cs_display2 = Output::new(peripherals.GPIO19, false.into());
    let mut cs_cap1188 = Output::new(peripherals.GPIO20, false.into());
    let mut cs_mcp2515 = Output::new(peripherals.GPIO17, false.into());
    let mut cs_mcp2515_2 = Output::new(peripherals.GPIO16, false.into());
    let int_mcp2515 = Input::new(peripherals.GPIO4, Pull::Up);
    let int_mcp2515_2 = Input::new(peripherals.GPIO1, Pull::Up);
    let mut rs = Output::new(peripherals.GPIO22, true.into());
    let ing = Input::new(peripherals.GPIO5, Pull::Up);
    let int_cap1188 = Input::new(peripherals.GPIO3, Pull::Up);
    let led = Output::new(peripherals.GPIO0, false.into());

    dc.set_high();
    rs.set_low();
    cs_display1.set_high();
    cs_display2.set_high();
    cs_cap1188.set_high();
    cs_mcp2515.set_high();
    cs_mcp2515_2.set_high();
    delay.delay_micros(2u32);
    rs.set_high();

    delay.delay_micros(2u32);

    rs.set_low();
    delay.delay_micros(2u32);
    rs.set_high();
    delay.delay_micros(2u32);

    let dc2 = unsafe { core::ptr::read(&dc) };

    static SPI_BUS: StaticCell<Mutex<CriticalSectionRawMutex, SpiBus>> = StaticCell::new();
    let spi_bus = SPI_BUS.init(Mutex::new(SpiBus::new(spi)));

    let display1_spi = SpiDeviceWithConfig::new(spi_bus, cs_display1, 20);
    let display2_spi = SpiDeviceWithConfig::new(spi_bus, cs_display2, 20);
    let cap1188_spi = SpiDeviceWithConfig::new(spi_bus, cs_cap1188, 5);
    let mcp2515_spi = SpiDeviceWithConfig::new(spi_bus, cs_mcp2515, 10);
    let mcp2515_2_spi = SpiDeviceWithConfig::new(spi_bus, cs_mcp2515_2, 10);
    let interface1 = SPIInterface::new(display1_spi, dc);
    let interface2 = SPIInterface::new(display2_spi, dc2);

    let display1 = AsyncDisplay::new(interface1, PixelCoord(256, 64), PixelCoord(0, 0), DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    let display2 = AsyncDisplay::new(interface2, PixelCoord(256, 64), PixelCoord(0, 0), DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    let cap1188 = Cap1188::new(cap1188_spi, int_cap1188);
    let mcp2515 = Mcp2515::new(mcp2515_spi, int_mcp2515);
    let mcp2515_2 = Mcp2515::new(mcp2515_2_spi, int_mcp2515_2);

    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();

    info!("HAL initialized");

    let ieee802154 = Ieee802154::new(peripherals.IEEE802154, peripherals.RADIO_CLK);

    //let mut rtc = Rtc::new(peripherals.LPWR);
    //rtc.set_interrupt_handler(interrupt_handler);

    rtc.rwdt.enable();
    rtc.rwdt.set_timeout(RwdtStage::Stage0, (5 * 60u32).secs().into());
    rtc.rwdt.listen();
    static RTC: StaticCell<Mutex<CriticalSectionRawMutex, Rtc<'static>>> = StaticCell::new();
    let rtc = RTC.init(Mutex::new(rtc));

    Hal {
        display1,
        display2,
        buttons: cap1188,
        obd2: obd2::Obd2::new(mcp2515),
        can_listen: mcp2515_2,
        usb_serial,
        power: power::Power::new(ing, delay, rtc, rs),
        led,
        ieee802154,
        rtc,
    }
}
