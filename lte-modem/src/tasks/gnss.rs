use defmt::{unwrap, warn, Format};
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

use super::{
    modem::link::{self, tx_channel_pub},
    uarte::state_channel_sub,
    TASKS_SUBSCRIBERS,
};
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
        with_timeout(Duration::from_secs(10), STATE.lock()).await.ok().map(|l| l.fix).flatten()
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
    let mut state_channel_sub = state_channel_sub();
    let mut current_state = None;
    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut battery_state = BatteryState::get().await;

    let fix_pub = CHANNEL.publisher().unwrap();
    let mut last_fix: Option<embassy_time::Instant> = None;
    let mut gnss_half_duration_delay: Option<embassy_time::Timer> = None;
    //let mut last_modem_fix_send: Option<Instant> = None;

    loop {
        let result = select::select3(
            async {
                match current_state {
                    None
                    | Some(types::State::Charging)
                    | Some(types::State::IgnitionOff)
                    | Some(types::State::CheckCharging) => {
                        if let Some(last_fix) = last_fix {
                            if last_fix.elapsed().as_secs() > 60 * 60 * 4 {
                                warn!("no fix for 4 hours, reseting GNSS");
                                gnss.next().await
                            } else {
                                futures::future::pending().await
                            }
                        } else {
                            last_fix = Some(embassy_time::Instant::now());
                            futures::future::pending().await
                        }
                    }
                    Some(types::State::Shutdown(duration)) => {
                        if gnss_half_duration_delay.is_none() {
                            let duration: embassy_time::Duration = unwrap!(duration.try_into());
                            gnss_half_duration_delay = Some(embassy_time::Timer::after(duration / 2));
                        }

                        let mut clear_gnss_half_duration_delay = false;
                        if let Some(gnss_half_duration_delay) = &mut gnss_half_duration_delay {
                            gnss_half_duration_delay.await;
                            clear_gnss_half_duration_delay = true;
                        }

                        if clear_gnss_half_duration_delay {
                            gnss_half_duration_delay = None;
                        }
                        warn!("getting fix in shutdown state");
                        if let Some(last_fix) = &last_fix {
                            if last_fix.elapsed().as_secs() < 60 * 15 {
                                warn!("last fix was less than 15 minutes ago, waiting");
                                embassy_time::Timer::after(Duration::from_secs(60 * 15) - last_fix.elapsed()).await;
                            }
                        }
                        gnss.next().await
                    }
                    Some(types::State::IgnitionOn) => {
                        warn!("getting fix in ignition on state");
                        gnss.next().await
                    }
                }
            },
            state_channel_sub.next_message_pure(),
            REQUEST.wait(),
        )
        .await;
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
                        fix_pub.publish_immediate(fix);
                    } else {
                        defmt::warn!("found duplicated fix");
                    }
                }
            }
            select::Either3::Second(new_state) => {
                if current_state != Some(new_state.clone()) {
                    gnss_set_duration(&battery_state, &mut gnss, &new_state).await;
                }
                current_state = Some(new_state.clone());
            }
            select::Either3::Third(_) => {
                warn!("got request for new fix");
                if let Ok(Some(new_gnss_frame)) = gnss.next().await {
                    last_fix = Some(embassy_time::Instant::now());
                    let fix: FromFix = new_gnss_frame.into();
                    let fix = fix.0;
                    let mut state = STATE.lock().await;
                    state.fix = Some(fix);
                    fix_pub.publish_immediate(fix);

                    battery_state = BatteryState::get().await;
                    if let Some(state) = &current_state {
                        gnss_set_duration(&battery_state, &mut gnss, state).await
                    }
                }
            }
        }
    }
}

async fn gnss_set_duration(battery_state: &BatteryState, gnss: &mut Gnss, state: &types::State) {
    let long_duration = Duration::from_secs(60 * 60);
    let short_duration = Duration::from_secs(1);

    match state {
        types::State::Charging
        | types::State::CheckCharging
        | types::State::IgnitionOff
        | types::State::Shutdown(_) => gnss.conf(long_duration, true).await,
        types::State::IgnitionOn => gnss.conf(short_duration, false).await,
    }
}
