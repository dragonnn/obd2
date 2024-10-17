use embassy_futures::select::select;

use super::power::get_shutdown_signal;

#[embassy_executor::task]
pub async fn run(mut led: crate::types::Led) {
    select(
        async {
            loop {
                led.set_low();
                embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
                led.set_high();
                embassy_time::Timer::after(embassy_time::Duration::from_secs(2)).await;
            }
        },
        get_shutdown_signal().next_message(),
    )
    .await;
    led.set_high();
}
