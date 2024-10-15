mod adp5360;
mod adxl362;
mod adxl372;
mod bh1749nuc;
mod button;
mod buzzer;
mod gnss;
mod modem;
mod rgb;
mod rtc;
mod wdg;

pub use adp5360::{Adp5360, ChargetStatus, InterputEvent};
pub use adxl362::Adxl362;
pub use adxl372::Adxl372;
pub use bh1749nuc::Bh1749nuc;
pub use button::Button;
use embassy_embedded_hal::shared_bus::asynch::{i2c::I2cDevice, spi::SpiDevice};
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive, Pin},
    peripherals::{P0_07, P0_08, PWM0, PWM1, PWM2, SERIAL2, SERIAL3},
    spim::{self, Spim},
    twim::{self, Twim},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
pub use gnss::Gnss;
pub use modem::Modem;
pub use rgb::Rgb;
use static_cell::StaticCell;
pub use wdg::Wdg;

static TWIM2: StaticCell<Mutex<ThreadModeRawMutex, Twim<SERIAL2>>> = StaticCell::new();
static SPIM3: StaticCell<Mutex<ThreadModeRawMutex, Spim<SERIAL3>>> = StaticCell::new();

pub type Buzzer = buzzer::Buzzer<'static, PWM0>;
pub type Lightwell = Rgb<'static, PWM1>;
pub type Sense = Rgb<'static, PWM2>;
pub type Battery = Adp5360<
    embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'static, ThreadModeRawMutex, Twim<'static, SERIAL2>>,
>;
pub type LowPowerAccelerometer =
    Adxl362<SpiDevice<'static, ThreadModeRawMutex, Spim<'static, SERIAL3>, Output<'static, P0_08>>>;
pub type HiGAccelerometer =
    Adxl372<SpiDevice<'static, ThreadModeRawMutex, Spim<'static, SERIAL3>, Output<'static, P0_07>>>;
pub type LightSensor = Bh1749nuc<
    embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'static, ThreadModeRawMutex, Twim<'static, SERIAL2>>,
>;

bind_interrupts!(struct TwiIrqs {
    UARTE2_SPIM2_SPIS2_TWIM2_TWIS2 => twim::InterruptHandler<SERIAL2>;
});

bind_interrupts!(struct SpiIrqs {
    UARTE3_SPIM3_SPIS3_TWIM3_TWIS3 => spim::InterruptHandler<SERIAL3>;
});

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
}

impl Board {
    pub async fn new() -> Board {
        let p = embassy_nrf::init(Default::default());

        defmt::info!("lightwell initalizing");
        let mut lightwell = Rgb::new(p.PWM1, p.P0_29, p.P0_30, p.P0_31, true);
        lightwell.r(64);

        defmt::info!("wdg initalizing");
        let wdg = Wdg::new(p.WDT).await;

        defmt::info!("buzzer initalizing");
        let buzzer = Buzzer::new(p.PWM0, p.P0_28);

        defmt::info!("sense initalizing");
        let sense = Rgb::new(p.PWM2, p.P0_00, p.P0_01, p.P0_02, true);
        defmt::info!("modem initalizing");
        let modem = Modem::new().await;

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
        defmt::info!("battery initalizing");

        twi2_reset_gpio().await;
        Timer::after(Duration::from_millis(200)).await;

        let mut twi2_config = twim::Config::default();
        twi2_config.scl_high_drive = true;
        twi2_config.sda_high_drive = true;
        twi2_config.scl_pullup = true;
        twi2_config.sda_pullup = true;
        let twi2 = Twim::new(p.SERIAL2, TwiIrqs, p.P0_11, p.P0_12, twi2_config);

        let twim2 = TWIM2.init(Mutex::<ThreadModeRawMutex, _>::new(twi2));

        let twim2_dev1 = I2cDevice::new(twim2);

        let battery = Adp5360::new(twim2_dev1, p.P0_17.degrade(), p.P0_10.degrade()).await;

        let twim2_dev2 = I2cDevice::new(twim2);

        let light_sensor = LightSensor::new(twim2_dev2, p.P0_27.degrade()).await;

        let mut spim3_config = spim::Config::default();
        spim3_config.frequency = spim::Frequency::M8;
        spim3_config.mode = spim::MODE_0;

        let spim3 = spim::Spim::new(p.SERIAL3, SpiIrqs, p.P0_03, p.P0_05, p.P0_04, spim3_config);
        let spim3 = SPIM3.init(Mutex::<ThreadModeRawMutex, _>::new(spim3));

        let spim3_dev1_cs = Output::new(p.P0_08, Level::High, OutputDrive::Standard);
        let spim3_dev1 = SpiDevice::new(spim3, spim3_dev1_cs);
        let spim3_dev2_cs = Output::new(p.P0_07, Level::High, OutputDrive::Standard);
        let spim3_dev2 = SpiDevice::new(spim3, spim3_dev2_cs);
        defmt::info!("low power accelerometer initalizing");
        let low_power_accelerometer = Adxl362::new(spim3_dev1, p.P0_09.degrade()).await;
        defmt::info!("hi g accelerometer initalizing");
        let hi_g_accelerometer = Adxl372::new(spim3_dev2, p.P0_06.degrade()).await;

        let button = Button::new(p.P0_26.degrade()).await;

        lightwell.r(0);

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
        }
    }
}

pub async fn twi2_reset_true() {
    /*let _lock = TWIM2_RESET_MUTEX.lock().await;

    let now = embassy_time::Instant::now();
    unsafe {
        let _twim2_lock = TWIM2_API.as_ref().unwrap().lock().await;

        let _critical = embassy_nrf::interrupt::CriticalSection::new();
        let twim2_scl = &mut TWIM2_SCL;
        let mut state_high = true;
        for _ in 0..9 {
            if state_high {
                twim2_scl.as_mut().unwrap().set_low();
                state_high = false;
            } else {
                twim2_scl.as_mut().unwrap().set_high();
                state_high = true;
            }
            for _ in 0..160 {
                core::arch::arm::__nop();
            }
        }
    }*/

    //defmt::info!("i2c reset took: {}", now.elapsed().as_micros());
}

pub async fn twi2_init() -> Twim<'static, SERIAL2> {
    todo!()
    /*unsafe {
        let p = Peripherals::steal();
        let twi2_irq = &TWI2_IRQ;
        let twim2_irq_owned = ptr::read(twi2_irq).unwrap();
        let mut twi2_config = twim::Config::default();
        twi2_config.scl_high_drive = true;
        twi2_config.sda_high_drive = true;
        twi2_config.scl_pullup = true;
        twi2_config.sda_pullup = true;
        Twim::new(p.SERIAL2, twim2_irq_owned, p.P0_11, p.P0_12, twi2_config)
    }*/
}

use core::sync::atomic::{AtomicU32, Ordering};

pub static TWI2_RESETS: AtomicU32 = AtomicU32::new(0);
pub async fn twi2_reset() {
    panic!("twi reset");
    TWI2_RESETS.fetch_add(1, Ordering::SeqCst);

    /*let _lock = TWIM2_RESET_MUTEX.lock().await;
    unsafe {
        let mut _twim2_lock = TWIM2_API.as_ref().unwrap().lock().await;
        let twim2_old = &mut *_twim2_lock;
        let twim2_owned = ptr::read(twim2_old);
        drop(twim2_owned);
        twi2_reset_gpio();
        let twi2 = twi2_init().await;
        //core::mem::forget(twi2);
        core::ptr::replace(twim2_old, twi2);
    }*/

    twi2_reset_gpio().await;

    if TWI2_RESETS.load(Ordering::Relaxed) > 10 {
        panic!("twi2 resets over 10");
    }

    //panic!("twi2 error");
}

pub async fn twi2_reset_gpio() {
    unsafe {
        let mut twim2_scl = Output::new(embassy_nrf::peripherals::P0_12::steal(), Level::High, OutputDrive::Standard);
        let mut state_high = true;
        for _ in 0..9 {
            if state_high {
                twim2_scl.set_low();
                state_high = false;
            } else {
                twim2_scl.set_high();
                state_high = true;
            }
            for _ in 0..160 {
                core::arch::arm::__nop();
            }
        }
    }
}
