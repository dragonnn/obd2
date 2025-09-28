use defmt::warn;
use embassy_nrf::{
    peripherals,
    wdt::{Config, Instance, SleepConfig, Watchdog, WatchdogHandle},
    Peri,
};
use embassy_time::{Duration, Timer};

pub struct Wdg(WatchdogHandle);
//pub struct Wdg;

impl Wdg {
    pub async fn new<T: Instance>(wdt: Peri<'static, T>) -> Self {
        let mut config = Config::default();

        config.timeout_ticks = 32768 * 300;
        //config.action_during_sleep = SleepConfig::PAUSE;

        let (_wdt, [handle]) = match Watchdog::try_new(wdt, config) {
            Ok(x) => x,
            Err(_) => {
                defmt::error!("watchdog already active with wrong config, waiting for it to timeout...");
                loop {
                    Timer::after(Duration::from_millis(100)).await;
                }
            }
        };

        let mut ret = Self(handle);
        //let mut ret = Self;
        ret.pet().await;
        ret
    }

    pub async fn pet(&mut self) {
        //warn!("wdg pet");
        self.0.pet();
    }
}
