use defmt::Format;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    mutex::Mutex,
    pubsub::{PubSubChannel, Subscriber},
};
use embassy_time::Instant;

use super::TASKS_SUBSCRIBERS;
use crate::board::LowPowerAccelerometer;

static CHANNEL: PubSubChannel<ThreadModeRawMutex, (), TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1> = PubSubChannel::new();

pub type MontionDetectionSubscriper =
    Subscriber<'static, ThreadModeRawMutex, (), TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1>;

const THRESHOLD_ACT: (u16, u16) = (110, 250);
const THRESHOLD_INACT: (u16, u16) = (95, 100);
const SAMPLES: u8 = 4;

#[derive(Format, Clone, Copy)]
pub struct State(pub u64);

impl State {
    pub async fn get() -> State {
        let mut state = STATE.lock().await;
        let count = state.0;
        state.0 = 0;
        State(count)
    }

    pub async fn subscribe() -> MontionDetectionSubscriper {
        CHANNEL.subscriber().unwrap()
    }
}

static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State(0));

#[embassy_executor::task]
pub async fn task(mut montion_detection: LowPowerAccelerometer) {
    let mut current_threshold_act = THRESHOLD_ACT.0;
    let mut current_threshold_inact = THRESHOLD_INACT.0;
    let mut calibrate = false;

    montion_detection.setup_montion_detection(current_threshold_act, current_threshold_inact, SAMPLES).await;
    let montion_detection_pub = CHANNEL.publisher().unwrap();

    let mut last_montion_detect = Instant::now();
    let mut last_montion_count: u8 = 0;

    loop {
        if montion_detection.montion_detection_irq().await {
            defmt::info!("montion detect");
            if last_montion_detect.elapsed().as_secs() < 5 {
                if last_montion_count > 5 {
                    defmt::warn!("too much montion detect, reseting");
                    montion_detection.reset().await;
                    if current_threshold_act < THRESHOLD_ACT.1 && !calibrate {
                        current_threshold_act += 1;
                        defmt::trace!("setting current_threshold_act: {}", current_threshold_act);
                    }
                    montion_detection
                        .setup_montion_detection(current_threshold_act, current_threshold_inact, SAMPLES)
                        .await;
                    last_montion_count = 0;
                }
                last_montion_count += 1;
            } else {
                montion_detection_pub.publish(()).await;
                STATE.lock().await.0 += 1;
                last_montion_count = 0;
                if last_montion_detect.elapsed().as_secs() > 60 {
                    defmt::info!("montion detect calibrated");
                    calibrate = true;
                }
            }
            last_montion_detect = Instant::now();
        } else {
            defmt::warn!("montion detect irq run not from montion detect");
        }
    }
}
