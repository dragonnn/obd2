mod adp5360;
mod adxl362;
mod adxl372;
mod bh1749nuc;
mod button;
mod buzzer;
mod destruct_twim;
mod gnss;
mod modem;
mod rgb;
mod rtc;
mod wdg;

pub use adp5360::{Adp5360, ChargerStatus, InterputEvent};
pub use adxl362::Adxl362;
pub use adxl372::Adxl372;
pub use bh1749nuc::Bh1749nuc;
pub use button::Button;
use defmt::error;
use embassy_embedded_hal::shared_bus::asynch::{i2c::I2cDevice, spi::SpiDevice};
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pin, Pull},
    pac::UARTE0,
    peripherals::{P0_07, P0_08, PWM0, PWM1, PWM2, SERIAL0, SERIAL1, SERIAL2, SERIAL3, TIMER0, TIMER1},
    spim::{self, Spim},
    twim::{self, Twim},
    uarte::{self, Uarte, UarteRxWithIdle, UarteTx},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
pub use gnss::Gnss;
pub use modem::Modem;
pub use rgb::Rgb;
use static_cell::StaticCell;
pub use wdg::Wdg;

static TWIM2: StaticCell<Mutex<CriticalSectionRawMutex, destruct_twim::DestructTwim>> = StaticCell::new();
static SPIM3: StaticCell<Mutex<CriticalSectionRawMutex, Spim<SERIAL3>>> = StaticCell::new();

//pub type I2cBus = destruct_twim::DestructTwim;
pub type I2cBus = embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<
    'static,
    CriticalSectionRawMutex,
    destruct_twim::DestructTwim,
>;
pub type Buzzer = buzzer::Buzzer<'static, PWM0>;
pub type Lightwell = Rgb<'static, PWM1>;
pub type Sense = Rgb<'static, PWM2>;
pub type Battery = Adp5360<I2cBus>;
pub type LowPowerAccelerometer =
    Adxl362<SpiDevice<'static, CriticalSectionRawMutex, Spim<'static, SERIAL3>, Output<'static>>>;
pub type HiGAccelerometer =
    Adxl372<SpiDevice<'static, CriticalSectionRawMutex, Spim<'static, SERIAL3>, Output<'static>>>;
pub type LightSensor = Bh1749nuc<I2cBus>;

/*bind_interrupts!(struct TwiIrqs {
    UARTE2_SPIM2_SPIS2_TWIM2_TWIS2 => twim::InterruptHandler<SERIAL2>;
});*/

bind_interrupts!(struct SpiIrqs {
    SERIAL3 => spim::InterruptHandler<SERIAL3>;
});

bind_interrupts!(struct UartIrqs {
    SERIAL1 => uarte::InterruptHandler<SERIAL1>;
    SERIAL0 => uarte::InterruptHandler<SERIAL0>;
});

pub type BoardUarteTx = UarteTx<'static, SERIAL1>;
pub type BoardUarteRx = UarteRxWithIdle<'static, SERIAL1, TIMER0>;
pub type BoardDebugUarteTx = UarteTx<'static, SERIAL0>;
pub type BoardGnssUarteRx = UarteRxWithIdle<'static, SERIAL0, TIMER1>;
pub type BoardGnssUarteTx = UarteTx<'static, SERIAL0>;

pub struct Board {
    pub buzzer: Buzzer,
    pub lightwell: Option<Lightwell>,
    pub sense: Option<Sense>,
    pub modem: Modem,
    pub button: Option<Button>,

    pub battery: Option<Battery>,

    pub low_power_accelerometer: Option<LowPowerAccelerometer>,
    pub hi_g_accelerometer: HiGAccelerometer,
    pub light_sensor: Option<LightSensor>,

    pub wdg: Option<Wdg>,
    pub uarte: Option<(BoardUarteTx, UarteRxWithIdle<'static, SERIAL1, TIMER0>)>,
    pub uarte_send: Option<Output<'static>>,
    pub uarte_receive: Option<Input<'static>>,
    pub uarte_reset: Option<Output<'static>>,
    pub charging_control: Option<Output<'static>>,
    pub uarte_tx_gnss: Option<(BoardGnssUarteTx, BoardGnssUarteRx)>,
    pub gnss_pss: Option<Input<'static>>,
    pub gnss_force_on: Option<Output<'static>>,
}

impl Board {
    pub async fn new() -> Board {
        let p = embassy_nrf::init(Default::default());

        defmt::info!("lightwell initializing");
        let mut lightwell = Rgb::new(p.PWM1, p.P0_29, p.P0_30, p.P0_31, true);
        lightwell.r(64);

        defmt::info!("wdg initializing");
        let wdg = Wdg::new(p.WDT).await;

        defmt::info!("buzzer initializing");
        let buzzer = Buzzer::new(p.PWM0, p.P0_28);

        defmt::info!("sense initializing");
        let sense = Rgb::new(p.PWM2, p.P0_00, p.P0_01, p.P0_02, true);
        defmt::info!("modem initializing");
        let modem = Modem::new().await;

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
        defmt::info!("battery initializing");

        Timer::after(Duration::from_millis(200)).await;

        let mut twi2 = destruct_twim::DestructTwim::new().await;
        twi2.reset(0xFF).await;

        /*let twi2 = Twim::new(p.SERIAL2, TwiIrqs, p.P0_11, p.P0_12, twi2_config);
        unsafe {
            let pac = nrf9160_pac::Peripherals::steal();
            pac.TWIM2_NS.frequency.write(|w| w.frequency().bits(2673868));
        }*/

        let twim2 = TWIM2.init(Mutex::<CriticalSectionRawMutex, _>::new(twi2));

        let twim2_dev1 = I2cDevice::new(twim2);

        let battery = Adp5360::new(twim2_dev1, p.P0_17.degrade()).await;

        let twim2_dev2 = I2cDevice::new(twim2);

        let light_sensor = LightSensor::new(twim2_dev2, p.P0_27.degrade()).await;

        let mut spim3_config = spim::Config::default();
        spim3_config.frequency = spim::Frequency::M8;
        spim3_config.mode = spim::MODE_0;

        let spim3 = spim::Spim::new(p.SERIAL3, SpiIrqs, p.P0_03, p.P0_05, p.P0_04, spim3_config);
        let spim3 = SPIM3.init(Mutex::<CriticalSectionRawMutex, _>::new(spim3));

        let spim3_dev1_cs = Output::new(p.P0_08, Level::High, OutputDrive::Standard);
        let spim3_dev1 = SpiDevice::new(spim3, spim3_dev1_cs);
        let spim3_dev2_cs = Output::new(p.P0_07, Level::High, OutputDrive::Standard);
        let spim3_dev2 = SpiDevice::new(spim3, spim3_dev2_cs);
        defmt::info!("low power accelerometer initalizing");
        let low_power_accelerometer = Adxl362::new(spim3_dev1, p.P0_09.degrade()).await;
        defmt::info!("hi g accelerometer initalizing");
        let hi_g_accelerometer = Adxl372::new(spim3_dev2, p.P0_06.degrade()).await;

        let button = Button::new(p.P0_26.degrade()).await;
        //rxd - p0.25 -> MCU_IF7
        //txd - p0.24 -> MCU_IF6
        let mut uart_config = uarte::Config::default();
        uart_config.baudrate = uarte::Baudrate::BAUD1M;
        uart_config.parity = uarte::Parity::INCLUDED;
        let uarte = Uarte::new(
            p.SERIAL1,
            UartIrqs,
            p.P0_24, //rxd
            p.P0_25, //txd
            uart_config.clone(),
        )
        .split_with_idle(p.TIMER0, p.PPI_CH0, p.PPI_CH1);
        //send - p0.23 -> MCU_IF5
        //receive - p0.22 -> MCU_IF4
        let uarte_send = Output::new(p.P0_23, Level::Low, OutputDrive::Standard);
        let uarte_receive = Input::new(p.P0_22, Pull::Down);
        let uarte_reset = Output::new(p.P0_10, Level::High, OutputDrive::Standard);

        uart_config.baudrate = uarte::Baudrate::BAUD9600;
        uart_config.parity = uarte::Parity::EXCLUDED;

        let uarte_tx_gnss = Uarte::new(p.SERIAL0, UartIrqs, p.P0_13, p.P0_16, uart_config)
            .split_with_idle(p.TIMER1, p.PPI_CH2, p.PPI_CH3);

        lightwell.r(0);

        let charging_control = Output::new(p.P0_14, Level::High, OutputDrive::Standard);
        let gnss_pss = Input::new(p.P0_21, Pull::Down);
        let gnss_force_on = Output::new(p.P0_15, Level::High, OutputDrive::Standard);

        Self {
            modem,
            buzzer,
            lightwell: Some(lightwell),
            button: Some(button),
            sense: Some(sense),
            battery: Some(battery),
            low_power_accelerometer: Some(low_power_accelerometer),
            light_sensor: Some(light_sensor),
            hi_g_accelerometer,
            wdg: Some(wdg),
            uarte: Some(uarte),
            uarte_send: Some(uarte_send),
            uarte_receive: Some(uarte_receive),
            uarte_reset: Some(uarte_reset),
            charging_control: Some(charging_control),
            uarte_tx_gnss: Some(uarte_tx_gnss),
            gnss_pss: Some(gnss_pss),
            gnss_force_on: Some(gnss_force_on),
        }
    }
}
