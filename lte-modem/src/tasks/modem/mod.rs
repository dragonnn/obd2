use core::{fmt::Write, write};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select;
use embassy_time::{Duration, Instant, Ticker};
use futures::StreamExt;
use heapless::String;
use persistent_buff::PersistentBuff;
use serde::{Deserialize, Serialize};

pub mod link;
mod persistent;
mod sms;

use persistent::PeristentManager;
use sms::send_state;

use crate::{
    board::Modem,
    tasks::{
        battery::State as BatteryState,
        button::subscribe as button_subscribe,
        gnss::{Fix, State as GnssState},
    },
};

// /#[embassy_executor::task]
pub async fn task(mut modem: Modem, spawner: &Spawner) {
    unwrap!(spawner.spawn(link::task()));

    let mut persistent_manager = PeristentManager::new();

    let imei = modem.imei().await.unwrap();
    defmt::info!("imei: {}", imei);

    let dbm = modem.dbm().await.unwrap();
    defmt::info!("dbm: {}", dbm);

    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut battery_state = BatteryState::get().await;

    if !persistent_manager.get_booted() {
        if let Err(err) = send_state(&modem, "booting..", false, true, persistent_manager.get_restarts()).await {
            defmt::error!("error sending sms: {:?}", defmt::Debug2Format(&err));
        }
        persistent_manager.update_booted(true);
    } else {
        persistent_manager.add_restarts();
    }

    let mut fix_sub = GnssState::subscribe().await;
    let mut fix = persistent_manager.get_fix();
    if fix.is_none() {
        fix = GnssState::get_current_fix().await;
    }

    let mut ticker = Ticker::every(Duration::from_secs(30));

    let mut button_sub = button_subscribe().await;

    let mut distance = persistent_manager.get_distance();
    let mut secs = persistent_manager.get_secs();

    loop {
        match select::select4(battery_state_sub.next(), ticker.next(), fix_sub.next(), button_sub.next()).await {
            select::Either4::First(new_battery_state) => {
                if let Some(new_battery_state) = new_battery_state {
                    if new_battery_state.charging != battery_state.charging {
                        send_charging_state(
                            &new_battery_state,
                            &mut modem,
                            distance,
                            distance / (secs / 3600.0),
                            persistent_manager.get_restarts(),
                        )
                        .await;
                        distance = 0.0;
                        secs = 0.0;
                        persistent_manager.update_distance(distance);
                        persistent_manager.update_secs(secs);
                    }
                    if new_battery_state.low_voltage != battery_state.low_voltage && new_battery_state.low_voltage {
                        send_charging_state(
                            &new_battery_state,
                            &mut modem,
                            distance,
                            distance / (secs / 3600.0),
                            persistent_manager.get_restarts(),
                        )
                        .await;
                    }
                    battery_state = new_battery_state;
                }
            }
            select::Either4::Second(_) => {
                defmt::info!("ticker running");
                #[cfg(feature = "modem-send")]
                if with_timeout(Duration::from_secs(120), send::send_signle(&modem)).await.is_err() {
                    defmt::error!("send single err");
                }
            }
            select::Either4::Third(new_fix) => {
                if let (Some(old_fix), Some(new_fix)) = (fix, new_fix) {
                    distance += (old_fix - new_fix) / 1000.0;
                    persistent_manager.update_distance(distance);

                    let old_instant = Instant::from_ticks(old_fix.elpased);
                    let new_instant = Instant::from_ticks(new_fix.elpased);
                    if new_instant > old_instant {
                        secs += (new_instant - old_instant).as_millis() as f64 / 1000.0;
                        persistent_manager.update_secs(secs);
                    }
                }
                fix =
                    process_new_fix(&battery_state, &fix, new_fix, &mut modem, persistent_manager.get_restarts()).await;
                persistent_manager.update_fix(fix);
            }
            select::Either4::Fourth(_) => {
                defmt::info!("sending button press");
                send_state(&modem, "button pressed...", false, false, persistent_manager.get_restarts()).await.ok();
            }
        }
    }
}

async fn send_charging_state(
    battery_state: &BatteryState,
    modem: &mut Modem,
    distance: f64,
    speed: f64,
    restarts: u32,
) {
    let mut discharging_event: String<32> = String::new();
    write!(&mut discharging_event, "discharging.. {:.2}km {:.1}km/h...", distance, speed).ok();

    if battery_state.charging {
        if send_state(modem, "charging..", true, false, restarts).await.is_err() {
            defmt::error!("error sending sms");
        }
    } else if battery_state.low_voltage {
        if send_state(modem, "low voltage..", true, false, restarts).await.is_err() {
            defmt::error!("error sending sms");
        }
    } else if send_state(modem, &discharging_event, true, false, restarts).await.is_err() {
        defmt::error!("error sending sms");
    }
}

async fn process_new_fix(
    battery_state: &BatteryState,
    old_fix: &Option<Fix>,
    new_fix: Option<Fix>,
    modem: &mut Modem,
    restarts: u32,
) -> Option<Fix> {
    match (old_fix, new_fix) {
        (Some(old_fix), Some(new_fix)) => {
            if !battery_state.charging {
                let mut fix_distance = *old_fix - new_fix;
                if fix_distance > 300.0 {
                    if let Some(accurate_fix) = GnssState::wait_for_fix(Duration::from_secs(120)).await {
                        fix_distance = *old_fix - accurate_fix;
                        if fix_distance > 300.0 {
                            let mut fix_distance_event: String<32> = String::new();
                            write!(&mut fix_distance_event, "movement on battery {:.2}m...", fix_distance).ok();
                            send_state(modem, &fix_distance_event, false, false, restarts).await.ok();
                        }
                        return Some(accurate_fix);
                    } else {
                        return None;
                    }
                }
            }
        }
        (Some(_fix), None) => {
            send_state(modem, "lost fix...", false, false, restarts).await.ok();
        }
        (None, Some(_new_fix)) => {
            send_state(modem, "found fix...", false, false, restarts).await.ok();
        }
        (None, None) => {}
    }
    new_fix
}
