use defmt::{info, warn, Format};
use embassy_futures::select::{select3, Either3::*};
use embassy_nrf::gpio::Output;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::Mutex,
    pubsub::{PubSubChannel, Subscriber},
};
use embassy_time::{Duration, Instant, Timer};
use types::{Modem, TxFrame, TxMessage};

use super::{modem::link::tx_channel_pub, uarte::state_channel_sub, TASKS_SUBSCRIBERS};
use crate::board::{Battery, ChargerStatus, InterputEvent};

#[derive(Format, Clone, Default)]
pub struct State {
    pub charging: bool,
    pub charger_state: ChargerStatus,
    pub low_voltage: bool,
    pub capacity: u8,
    pub voltage: u16,
}

pub type StateSubscriper = Subscriber<'static, CriticalSectionRawMutex, State, TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1>;

impl State {
    pub async fn new(battery: &mut Battery, low_voltage: bool) -> Self {
        let charger_state = battery.charger_status().await.unwrap();
        info!("charger state: {:?}", charger_state);
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

impl Into<types::Modem> for State {
    fn into(self) -> types::Modem {
        types::Modem::Battery {
            voltage: self.voltage as f64 / 1000.0,
            low_voltage: self.low_voltage,
            soc: self.capacity,
            charging: self.charging,
        }
    }
}

static STATE: Mutex<CriticalSectionRawMutex, State> = Mutex::new(State {
    charging: false,
    charger_state: ChargerStatus::Off,
    low_voltage: false,
    capacity: 0,
    voltage: 0,
});

static CHANNEL: PubSubChannel<CriticalSectionRawMutex, State, TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, 1> =
    PubSubChannel::new();

#[embassy_executor::task]
pub async fn task(mut battery: Battery, mut charging_control: Output<'static>) {
    let mut current_charging = true;
    let mut charging_control = async |charing: bool, battery: &mut Battery| {
        if charing && !current_charging {
            warn!("enable charging");
            Timer::after_secs(1).await;
            charging_control.set_high();
            Timer::after_secs(5).await;
            battery.enable_charging().await;
            Timer::after_secs(5).await;
        } else if !charing && current_charging {
            warn!("disable charging");
            Timer::after_secs(1).await;
            battery.disable_charging().await;
            Timer::after_secs(5).await;
            charging_control.set_low();
            Timer::after_secs(5).await;
        }
        current_charging = charing;
    };

    let mut state_channel_sub = state_channel_sub();
    let mut current_state = None;

    embassy_time::Timer::after(Duration::from_secs(1)).await;
    let mut timeout = Duration::from_secs(60);
    let mut low_voltage = false;
    let mut state = State::new(&mut battery, low_voltage).await;
    let tx_channel_pub = tx_channel_pub();

    let mut last_modem_battery_send: Option<Instant> = None;
    let charging_pub = CHANNEL.publisher().unwrap();
    let mut low_capacity_forced_charging = false;

    loop {
        let new_state = State::new(&mut battery, low_voltage).await;
        if new_state.charging != state.charging {
            charging_pub.publish_immediate(new_state.clone());
            state = new_state.clone();
            if new_state.charging {
                timeout = Duration::from_secs(5);
            } else {
                timeout = Duration::from_secs(60);
            }
        }

        if new_state.capacity < 15 {
            warn!("low battery capacity: {}, charging", new_state.capacity);
            charging_control(true, &mut battery).await;
            low_capacity_forced_charging = true;
        } else if new_state.capacity > 85
            && (current_state != Some(types::State::IgnitionOn) && current_state != Some(types::State::Charging))
            && low_capacity_forced_charging
        {
            charging_control(false, &mut battery).await;
            low_capacity_forced_charging = false;
        }

        *STATE.lock().await = new_state;

        match select3(battery.irq(), Timer::after(timeout), state_channel_sub.next_message_pure()).await {
            First(event) => {
                defmt::info!("battery event: {:?}", event);
                if let Ok(InterputEvent::LowVoltage) = event {
                    defmt::error!("battery low voltage");
                    low_voltage = true;
                }
            }
            Second(_) => {}
            Third(new_state) => {
                info!("new state: {:?}", new_state);
                if let types::State::Charging = new_state {
                    charging_control(true, &mut battery).await;
                    low_capacity_forced_charging = false;
                } else if let types::State::CheckCharging = new_state {
                    charging_control(true, &mut battery).await;
                    low_capacity_forced_charging = false;
                } else if let types::State::IgnitionOn = new_state {
                    charging_control(true, &mut battery).await;
                    low_capacity_forced_charging = false;
                } else {
                    if !low_capacity_forced_charging {
                        charging_control(false, &mut battery).await;
                    }
                }
                current_state = Some(new_state);
            }
        }
        let battery_voltage = battery.voltage().await;
        let battery_soc = battery.battery_soc().await;
        defmt::info!("battery voltage: {} soc: {} low_voltage: {}", battery_voltage, battery_soc, low_voltage);
        if last_modem_battery_send.map(|l| l.elapsed().as_secs() > 60).unwrap_or(true) {
            last_modem_battery_send = Some(Instant::now());
            tx_channel_pub.publish_immediate(TxMessage::new(TxFrame::Modem(Modem::Battery {
                voltage: battery_voltage as f64 / 1000.0,
                low_voltage,
                soc: battery_soc,
                charging: state.charging,
            })));
        }
    }
}
