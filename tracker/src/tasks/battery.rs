use defmt::Format;
use embassy_futures::select;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    mutex::Mutex,
    pubsub::{PubSubChannel, Subscriber},
};
use embassy_time::{Duration, Timer};

use super::TASKS_SUBSCRIBERS;
use crate::board::{Battery, ChargetStatus, InterputEvent};

#[derive(Format, Clone, Default)]
pub struct State {
    pub charging: bool,
    pub charger_state: ChargetStatus,
    pub low_voltage: bool,
    pub capacity: u8,
    pub voltage: u16,
}

pub type StateSubscriper = Subscriber<'static, ThreadModeRawMutex, State, TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1>;

impl State {
    pub async fn new(battery: &mut Battery, low_voltage: bool) -> Self {
        let charger_state = battery.charger_status().await.unwrap();
        let charging = charger_state.is_charging();
        let capacity = battery.battery_soc().await;
        let voltage = battery.voltage().await;
        Self { charging, charger_state, low_voltage, capacity, voltage }
    }

    pub async fn get() -> Self {
        STATE.lock().await.clone()
    }

    pub async fn subscribe() -> StateSubscriper {
        CHANNEL.subscriber().unwrap()
    }
}

static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State {
    charging: false,
    charger_state: ChargetStatus::Off,
    low_voltage: false,
    capacity: 0,
    voltage: 0,
});

static CHANNEL: PubSubChannel<ThreadModeRawMutex, State, TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1> =
    PubSubChannel::new();

#[embassy_executor::task]
pub async fn task(mut battery: Battery) {
    let mut timeout = Duration::from_secs(60);
    let mut low_voltage = false;
    let mut state = State::new(&mut battery, low_voltage).await;

    //STATE.init(Mutex::new(State::new(&mut battery).await));
    let charging_pub = CHANNEL.publisher().unwrap();

    loop {
        let new_state = State::new(&mut battery, low_voltage).await;
        if new_state.charging != state.charging {
            charging_pub.publish(new_state.clone()).await;
            state = new_state.clone();
            if new_state.charging {
                timeout = Duration::from_secs(5);
            } else {
                timeout = Duration::from_secs(60);
            }
        }
        *STATE.lock().await = new_state;

        let result = select::select(battery.irq(), Timer::after(timeout)).await;
        match result {
            select::Either::First(event) => {
                defmt::info!("battery event: {:?}", event);
                if let Ok(InterputEvent::LowVoltage) = event {
                    defmt::error!("battery low voltage");
                    low_voltage = true;
                }
            }
            select::Either::Second(_) => {}
        }
        defmt::info!(
            "battery voltage: {} soc: {} low_voltage: {}",
            battery.voltage().await,
            battery.battery_soc().await,
            low_voltage
        );
    }
}
