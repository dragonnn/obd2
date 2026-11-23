use embassy_nrf::gpio::Input;
use embassy_time::{Duration, with_timeout};

use crate::prelude::*;

const DEBOUNCE: Duration = Duration::from_millis(50);

#[embassy_executor::task]
pub async fn run(mut ing: Input<'static>) {
    let mut car_events_sender = car_event_sender();
    let mut last_state = None;

    loop {
        let high = ing.is_high();
        let timeout = if last_state != Some(high) {
            DEBOUNCE
        } else {
            Duration::from_secs(1)
        };

        // A new level must stay stable for the debounce window.
        // Accepted levels retain the one-second fallback check.
        let changed = with_timeout(timeout, async {
            if high {
                ing.wait_for_low().await;
            } else {
                ing.wait_for_high().await;
            }
        })
        .await
        .is_ok();

        if changed || ing.is_high() != high {
            continue;
        }

        if last_state != Some(high) {
            let level = if high { "high" } else { "low" };
            if last_state.is_none() {
                warn!("initial ing is {}", level);
            } else {
                info!("ing is {}", level);
            }
            if high {
                car_events_sender.send(CarEvent::IgnitionOn).await.ok();
            } else {
                car_events_sender.send(CarEvent::IgnitionOff).await.ok();
            }

            last_state = Some(high);
        }
    }
}
