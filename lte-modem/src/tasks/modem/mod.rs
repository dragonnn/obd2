use core::{fmt::Write, write};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select4, Either4};
use embassy_time::{Duration, Instant, Ticker};
use futures::StreamExt;
use heapless::String;
use link::tx_channel_pub;
use persistent_buff::PersistentBuff;
use serde::{Deserialize, Serialize};

pub mod link;
mod persistent;
mod sms;

use persistent::PeristentManager;
use sms::send_state;
use types::{TxFrame, TxMessage};

use crate::{
    board::Modem,
    tasks::{
        battery::State as BatteryState,
        button::subscribe as button_subscribe,
        gnss::{Fix, State as GnssState},
        uarte::{set_current_state, state_channel_pub, state_channel_sub},
    },
};

// /#[embassy_executor::task]
pub async fn task(mut modem: Modem, spawner: &Spawner) {
    unwrap!(spawner.spawn(link::send_task(*spawner)));

    let mut persistent_manager = PeristentManager::new();

    let imei = modem.imei().await.unwrap();
    defmt::info!("imei: {}", imei);

    let dbm = modem.dbm().await.unwrap();
    defmt::info!("dbm: {}", dbm);
    let hw = modem.hw().await.unwrap();
    defmt::info!("hw: {}", hw);

    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut battery_state = BatteryState::get().await;
    let mut state_channel_sub = state_channel_sub();
    let state_channel_pub = state_channel_pub();

    if !persistent_manager.get_booted() {
        if let Err(err) = send_state(&modem, "booting..", false, true, persistent_manager.get_restarts(), false).await {
            defmt::error!("error sending sms: {:?}", defmt::Debug2Format(&err));
        }
        persistent_manager.update_booted(true);
    } else {
        persistent_manager.add_restarts();
    }

    if let Some(state) = persistent_manager.get_state() {
        state_channel_pub.publish_immediate(state.clone());
        set_current_state(state).await;
    }

    let mut fix_sub = GnssState::subscribe().await;
    let mut fix = persistent_manager.get_fix();
    if fix.is_none() {
        fix = GnssState::get_current_fix().await;
    }

    let mut button_sub = button_subscribe().await;
    let tx_channel_pub = tx_channel_pub();

    let mut distance = persistent_manager.get_distance();
    let mut secs = persistent_manager.get_secs();
    let mut last_button_press = Instant::now();

    loop {
        match select4(
            battery_state_sub.next(),
            fix_sub.next(),
            button_sub.next(),
            state_channel_sub.next_message_pure(),
        )
        .await
        {
            Either4::First(new_battery_state) => {
                if let Some(new_battery_state) = new_battery_state {
                    /*if new_battery_state.charging != battery_state.charging {
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
                    }*/
                    battery_state = new_battery_state;
                }
            }
            Either4::Second(new_fix) => {
                fix =
                    process_new_fix(&battery_state, &fix, new_fix, &mut modem, persistent_manager.get_restarts()).await;
                let mut current_distance = 0.0;
                if let (Some(old_fix), Some(new_fix)) = (fix, new_fix) {
                    current_distance = (old_fix - new_fix) / 1000.0;
                    distance += current_distance;
                    persistent_manager.update_distance(distance);

                    let old_instant = Instant::from_ticks(old_fix.elapsed);
                    let new_instant = Instant::from_ticks(new_fix.elapsed);
                    if new_instant > old_instant {
                        secs += (new_instant - old_instant).as_millis() as f64 / 1000.0;
                        persistent_manager.update_secs(secs);
                    }
                }
                if let Some(fix) = fix {
                    if link::connected() || current_distance > 0.5 {
                        tx_channel_pub.publish_immediate(TxMessage::new(TxFrame::Modem(types::Modem::GnssFix(fix))));
                    }
                }
                persistent_manager.update_fix(fix);
            }
            Either4::Third(_) => {
                defmt::info!("sending button press");
                if last_button_press.elapsed().as_secs() > 5 {
                    send_state(&modem, "button pressed...", false, false, persistent_manager.get_restarts(), false)
                        .await
                        .ok();
                } else {
                    warn!("button press ignored");
                }
                last_button_press = Instant::now();
            }
            Either4::Fourth(state) => {
                if persistent_manager.get_state() != Some(state.clone()) {
                    persistent_manager.update_state(Some(state.clone()));
                    if state == types::State::IgnitionOn || state == types::State::CheckCharging {
                        embassy_time::Timer::after(Duration::from_secs(5)).await;
                        warn!("sending obd2 state");
                        send_obd2_state(
                            &battery_state,
                            &mut modem,
                            distance,
                            distance / (secs / 3600.0),
                            persistent_manager.get_restarts(),
                            &state,
                        )
                        .await
                        .ok();
                    }
                }
            }
        }
    }
}

async fn send_obd2_state(
    battery_state: &BatteryState,
    modem: &mut Modem,
    distance: f64,
    speed: f64,
    restarts: u32,
    state: &types::State,
) -> Result<(), ()> {
    let mut parked_event: String<32> = String::new();
    write!(&mut parked_event, "parked.. {:.2}km {:.1}km/h...", distance, speed).ok();

    match state {
        types::State::CheckCharging => {
            if send_state(modem, &parked_event, true, false, restarts, true).await.is_err() {
                defmt::error!("error sending sms");
            }
        }
        types::State::IgnitionOn => {
            if send_state(modem, "driving..", true, false, restarts, true).await.is_err() {
                defmt::error!("error sending sms");
            }
        }
        _ => {}
    }

    Ok(())
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
                            send_state(modem, &fix_distance_event, false, false, restarts, true).await.ok();
                        }
                        return Some(accurate_fix);
                    } else {
                        return None;
                    }
                }
            }
        }
        (Some(_fix), None) => {
            send_state(modem, "lost fix...", false, false, restarts, true).await.ok();
        }
        (None, Some(_new_fix)) => {
            send_state(modem, "found fix...", false, false, restarts, true).await.ok();
        }
        (None, None) => {}
    }
    new_fix
}
