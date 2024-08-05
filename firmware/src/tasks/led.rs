#[embassy_executor::task]
pub async fn run(mut led: crate::types::Led) {
    loop {
        led.set_low();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        led.set_high();
        embassy_time::Timer::after(embassy_time::Duration::from_secs(2)).await;
    }
}
