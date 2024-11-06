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
use types::{Modem, TxFrame};

use super::{modem::link::tx_channel_pub, TASKS_SUBSCRIBERS};
use crate::{board::Gnss, tasks::battery::State as BatteryState};

const FIXES_BUFFER_SIZE: usize = 10;

pub use types::GnssFix as Fix;

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

pub struct FromFix(Fix);

impl From<nrf_modem_gnss_pvt_data_frame> for FromFix {
    fn from(value: nrf_modem_gnss_pvt_data_frame) -> Self {
        FromFix(Fix {
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
            elapsed: Instant::now().as_ticks(),
        })
    }
}

static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State { fix: None, fixes: Vec::new() });

static CHANNEL: PubSubChannel<ThreadModeRawMutex, Fix, TASKS_SUBSCRIBERS, TASKS_SUBSCRIBERS, FIXES_BUFFER_SIZE> =
    PubSubChannel::new();

static REQUEST: Signal<ThreadModeRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn task(mut gnss: Gnss) {
    let tx_channel_pub = tx_channel_pub();
    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut battery_state = BatteryState::get().await;

    gnss_set_duration(&battery_state, &mut gnss).await;

    let fix_pub = CHANNEL.publisher().unwrap();
    //let mut last_modem_fix_send: Option<Instant> = None;

    loop {
        gnss_set_duration(&battery_state, &mut gnss).await;
        let result = select::select3(gnss.next(), battery_state_sub.next(), REQUEST.wait()).await;
        match result {
            select::Either3::First(new_gnss_frame) => {
                if let Ok(Some(new_gnss_frame)) = new_gnss_frame {
                    let fix: FromFix = new_gnss_frame.into();
                    let fix = fix.0;
                    defmt::info!("fix: {:?}", fix);
                    let mut state = STATE.lock().await;
                    if state.fix != Some(fix) {
                        state.fix = Some(fix);
                        if state.fixes.is_full() {
                            state.fixes.pop();
                        }
                        state.fixes.push(fix).ok();
                        fix_pub.publish(fix).await;
                        //if last_modem_fix_send.map(|l| l.elapsed().as_secs() > 60).unwrap_or(true) {
                        tx_channel_pub.publish(TxFrame::Modem(Modem::GnssFix(fix))).await;
                        //    last_modem_fix_send = Some(Instant::now());
                        //}
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
                    let fix: FromFix = new_gnss_frame.into();
                    let fix = fix.0;
                    let mut state = STATE.lock().await;
                    state.fix = Some(fix);
                    fix_pub.publish(fix).await;
                    //if last_modem_fix_send.map(|l| l.elapsed().as_secs() > 60).unwrap_or(true) {
                    tx_channel_pub.publish(TxFrame::Modem(Modem::GnssFix(fix))).await;
                    //    last_modem_fix_send = Some(Instant::now());
                    //}

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
        if battery_state.capacity < 50 {
            gnss.conf(Duration::from_secs(60 * 60), true).await;
        } else {
            gnss.conf(Duration::from_secs(15 * 60), true).await;
        }
    }
}
