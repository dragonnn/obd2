use core::sync::atomic::{AtomicI32, Ordering};

use defmt::*;
use embassy_time::{with_timeout, Duration};
use embedded_io_async::Write as _;
use heapless::Vec;

static TEMPERATURE: AtomicI32 = AtomicI32::new(0);

#[embassy_executor::task]
pub async fn run(temperature: crate::types::TemperatureSensor) {
    loop {
        let temp = temperature.get_temperature().to_celsius();
        TEMPERATURE.store(temp as i32 * 1000, Ordering::Relaxed);

        embassy_time::Timer::after(Duration::from_secs(1)).await;
    }
}

pub fn get_temperature() -> f32 {
    TEMPERATURE.load(Ordering::Relaxed) as f32 / 1000.0
}
