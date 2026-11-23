use embassy_nrf::gpio::Input;
use embassy_time::{Duration, with_timeout};

#[embassy_executor::task]
pub async fn run(mut ing: Input<'static>) {
    let mut last_state = None;

    loop {
        let high = ing.is_high();
        if last_state != Some(high) {
            let level = if high { "high" } else { "low" };
            if last_state.is_none() {
                warn!("initial ing is {}", level);
            } else {
                info!("ing is {}", level);
            }
            last_state = Some(high);
        }

        // Waiting for a level also catches a change that happened during logging.
        // Recheck after one second even if no input wakeup arrives.
        let _ = with_timeout(Duration::from_secs(1), async {
            if high {
                ing.wait_for_low().await;
            } else {
                ing.wait_for_high().await;
            }
        })
        .await;
    }
}
