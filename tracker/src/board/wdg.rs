use embassy_nrf::{
    peripherals,
    wdt::{Config, Watchdog, WatchdogHandle},
};
use embassy_time::{Duration, Timer};

pub struct Wdg(WatchdogHandle);

impl Wdg {
    pub async fn new(wdt: peripherals::WDT) -> Self {
        let mut config = Config::default();

        #[cfg(not(debug_assertions))]
        {
            config.timeout_ticks = 32768 * 120;
            config.run_during_debug_halt = false;
        }

        #[cfg(debug_assertions)]
        {
            config.timeout_ticks = 32768 * 120;
        }

        let (_wdt, [handle]) = match Watchdog::try_new(wdt, config) {
            Ok(x) => x,
            Err(_) => {
                defmt::error!("watchdog already active with wrong config, waiting for it to timeout...");
                loop {
                    Timer::after(Duration::from_millis(100)).await;
                }
            }
        };

        Self(handle)
    }

    pub async fn pet(&mut self) {
        self.0.pet();
    }
}
