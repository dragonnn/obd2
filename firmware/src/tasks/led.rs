#[embassy_executor::task]
pub async fn run(mut led: crate::types::Led) {
    loop {
        led.toggle();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(250)).await;
    }
}
