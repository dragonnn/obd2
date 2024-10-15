use defmt::Format;
use embassy_futures::select;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    mutex::Mutex,
    pubsub::{PubSubChannel, Subscriber},
    signal::Signal,
};
use embassy_time::{with_timeout, Duration, Instant};
use futures::StreamExt;
use heapless::Vec;
use nrf_modem::nrfxlib_sys::nrf_modem_gnss_pvt_data_frame;
use num_traits::real::Real;
use serde::{Deserialize, Serialize};

use super::TASKS_SUBSCRIBERS;
use crate::{board::Gnss, tasks::battery::State as BatteryState};

const FIXES_BUFFER_SIZE: usize = 10;

#[derive(Format, Clone)]
pub struct State {
    fix: Option<Fix>,
    fixes: Vec<Fix, FIXES_BUFFER_SIZE>,
}

impl State {
    pub async fn get_current_fix() -> Option<Fix> {
        STATE.lock().await.fix
    }

    pub async fn wait_for_fix(timeout: Duration) -> Option<Fix> {
        REQUEST.signal(());
        with_timeout(timeout, Self::subscribe().await.next()).await.ok().flatten()
    }

    pub async fn get_buffred_fixes() -> Vec<Fix, FIXES_BUFFER_SIZE> {
        let mut state = STATE.lock().await;
        let fixes = state.fixes.clone();
        state.fixes.clear();
        fixes
    }

    pub async fn subscribe() -> FixSubscriper {
        CHANNEL.subscriber().unwrap()
    }
}

pub type FixSubscriper =
    Subscriber<'static, ThreadModeRawMutex, Fix, TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, FIXES_BUFFER_SIZE>;

#[derive(Format, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub struct Fix {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
    pub accuracy: f32,

    pub year: u16,
    pub month: u8,
    pub day: u8,

    pub hour: u8,
    pub minute: u8,
    pub seconds: u8,
    pub ms: u16,
    pub elpased: u64,
}

impl core::ops::Sub for Fix {
    type Output = f64;

    fn sub(self, other: Self) -> Self::Output {
        let r = 6378.137;
        let d_lat = (other.latitude * core::f64::consts::PI / 180.0) - (self.latitude * core::f64::consts::PI / 180.0);
        let d_lon =
            (other.longitude * core::f64::consts::PI / 180.0) - (self.longitude * core::f64::consts::PI / 180.0);
        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + (self.latitude * core::f64::consts::PI / 180.0).cos()
                * (other.latitude * core::f64::consts::PI / 180.0).cos()
                * (d_lon / 2.0).sin()
                * (d_lon / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let d = r * c;
        d * 1000.0
    }
}

impl From<nrf_modem_gnss_pvt_data_frame> for Fix {
    fn from(value: nrf_modem_gnss_pvt_data_frame) -> Self {
        Fix {
            latitude: value.latitude,
            longitude: value.longitude,
            altitude: value.altitude,
            accuracy: value.accuracy,

            year: value.datetime.year,
            month: value.datetime.month,
            day: value.datetime.day,

            hour: value.datetime.hour,
            minute: value.datetime.minute,
            seconds: value.datetime.seconds,
            ms: value.datetime.ms,
            elpased: Instant::now().as_ticks(),
        }
    }
}

static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State { fix: None, fixes: Vec::new() });

static CHANNEL: PubSubChannel<ThreadModeRawMutex, Fix, TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, FIXES_BUFFER_SIZE> =
    PubSubChannel::new();

static REQUEST: Signal<ThreadModeRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn task(mut gnss: Gnss) {
    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut battery_state = BatteryState::get().await;

    gnss_set_duration(&battery_state, &mut gnss).await;

    let fix_pub = CHANNEL.publisher().unwrap();

    loop {
        gnss_set_duration(&battery_state, &mut gnss).await;
        let result = select::select3(gnss.next(), battery_state_sub.next(), REQUEST.wait()).await;
        match result {
            select::Either3::First(new_gnss_frame) => {
                if let Ok(Some(new_gnss_frame)) = new_gnss_frame {
                    let fix: Fix = new_gnss_frame.into();
                    defmt::info!("fix: {:?}", fix);
                    let mut state = STATE.lock().await;
                    if state.fix != Some(fix) {
                        state.fix = Some(fix);
                        if state.fixes.is_full() {
                            state.fixes.pop();
                        }
                        state.fixes.push(fix).ok();
                        fix_pub.publish(fix).await;
                    } else {
                        defmt::warn!("found duplicated fix");
                    }
                }
            }
            select::Either3::Second(new_battery_state) => {
                defmt::info!("got new battery state: {:?}", new_battery_state);
                if let Some(new_battery_state) = new_battery_state {
                    if new_battery_state.charging != battery_state.charging {
                        battery_state = new_battery_state;
                        gnss_set_duration(&battery_state, &mut gnss).await;
                    }
                }
            }
            select::Either3::Third(_) => {
                defmt::info!("got request for new fix");
                gnss.conf(Duration::from_secs(1), false).await;
                if let Ok(Some(new_gnss_frame)) = gnss.next().await {
                    let fix: Fix = new_gnss_frame.into();
                    let mut state = STATE.lock().await;
                    state.fix = Some(fix);
                    fix_pub.publish(fix).await;

                    battery_state = BatteryState::get().await;
                    gnss_set_duration(&battery_state, &mut gnss).await;
                }
            }
        }
    }
}

async fn gnss_set_duration(battery_state: &BatteryState, gnss: &mut Gnss) {
    if battery_state.charging {
        gnss.conf(Duration::from_secs(1), false).await;
        defmt::debug!("gnss in charging state");
    } else {
        defmt::debug!("gnss in batter state");
        gnss.conf(Duration::from_secs(15 * 60), true).await;
    }
}
