use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    pubsub::{PubSubChannel, Subscriber},
};
use embassy_time::{Duration, Instant, Timer};

use super::TASKS_SUBSCRIBERS;
use crate::board::Button;

static CHANNEL: PubSubChannel<ThreadModeRawMutex, (), TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1> = PubSubChannel::new();

#[embassy_executor::task]
pub async fn task(mut button: Button) {
    let button_pub = CHANNEL.publisher().unwrap();
    let mut button_last_press: Option<Instant> = None;

    loop {
        button.pressed().await;
        defmt::info!("button press detected");
        if let Some(button_last_press) = button_last_press {
            if button_last_press.elapsed().as_secs() > 15 {
                button_pub.publish_immediate(());
            }
        } else {
            button_pub.publish_immediate(());
        }
        button_last_press = Some(Instant::now());
        Timer::after(Duration::from_millis(500)).await;
    }
}

pub async fn subscribe() -> Subscriber<'static, ThreadModeRawMutex, (), TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1> {
    CHANNEL.subscriber().unwrap()
}
