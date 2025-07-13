use core::{fmt::Write, write};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, select4, Either, Either4};
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
use sms::{send_state_delayed as send_state, SmsEvent};
use types::{TxFrame, TxMessage};

use crate::{
    board::Modem,
    tasks::{
        battery::State as BatteryState,
        button::subscribe as button_subscribe,
        gnss::{Fix, State as GnssState},
        uarte::{pid_channel_sub, set_current_state, state_channel_pub, state_channel_sub},
    },
};

// /#[embassy_executor::task]
pub async fn task(mut modem: Modem, spawner: &Spawner) {
    unwrap!(spawner.spawn(link::send_task(*spawner)));
    unwrap!(spawner.spawn(sms::task(modem)));

    let mut persistent_manager = PeristentManager::new();

    let imei = modem.imei().await.unwrap();
    defmt::info!("imei: {}", imei);

    let dbm = modem.dbm().await.unwrap();
    defmt::info!("dbm: {}", dbm);
    let hw = modem.hw().await.unwrap();
    defmt::info!("hw: {}", hw);
    let fw = modem.fw().await.unwrap();

    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut battery_state = BatteryState::get().await;
    let mut state_channel_sub = state_channel_sub();
    let mut pid_channel_sub = pid_channel_sub();
    let state_channel_pub = state_channel_pub();

    if !persistent_manager.get_booted() {
        embassy_time::Timer::after(Duration::from_secs(5)).await;
        if let Err(err) =
            send_state(SmsEvent::Custom("booting.."), false, true, persistent_manager.get_restarts(), false).await
        {
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
            select(state_channel_sub.next_message_pure(), pid_channel_sub.next_message_pure()),
        )
        .await
        {
            Either4::First(new_battery_state) => {
                if let Some(new_battery_state) = new_battery_state {
                    battery_state = new_battery_state;
                }
            }
            Either4::Second(new_fix) => {
                fix =
                    process_new_fix(&fix, new_fix, persistent_manager.get_restarts(), &persistent_manager.get_state())
                        .await;
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
                    send_state(SmsEvent::Custom("button..."), false, false, persistent_manager.get_restarts(), false)
                        .await
                        .ok();
                } else {
                    warn!("button press ignored");
                }
                last_button_press = Instant::now();
            }
            Either4::Fourth(state) => match state {
                Either::First(state) => {
                    let old_state = persistent_manager.get_state();
                    if persistent_manager.get_state() != Some(state.clone()) {
                        persistent_manager.update_state(Some(state.clone()));
                        if state == types::State::IgnitionOn || state == types::State::CheckCharging {
                            embassy_time::Timer::after(Duration::from_secs(5)).await;
                            warn!("sending obd2 state");
                            send_obd2_state(persistent_manager.get_restarts(), &state, &old_state).await.ok();
                        }
                    }
                }
                Either::Second(state) => {
                    if let types::Pid::Icu2Pid(icu2_pid) = state {
                        info!("checking icu2 pid");
                        let old_icu2_pid = persistent_manager.get_icu2_pid();

                        if let Some(old_icu2_pid) = old_icu2_pid {
                            let mut should_send_icu2_pid_state = false;
                            if old_icu2_pid.actuator_back_door_passenger_side_unlock
                                != icu2_pid.actuator_back_door_passenger_side_unlock
                                || old_icu2_pid.actuator_back_door_passenger_side_unlock
                                    != icu2_pid.actuator_back_door_passenger_side_unlock
                            {
                                should_send_icu2_pid_state = true;
                            }

                            if old_icu2_pid.trunk_open != icu2_pid.trunk_open {
                                should_send_icu2_pid_state = true;
                            }

                            if old_icu2_pid.engine_hood_open != icu2_pid.engine_hood_open {
                                should_send_icu2_pid_state = true;
                            }

                            if should_send_icu2_pid_state {
                                if let Err(err) =
                                    send_icu2_pid_state(&icu2_pid, persistent_manager.get_restarts()).await
                                {
                                    defmt::error!("error sending icu2 pid state: {:?}", defmt::Debug2Format(&err));
                                }
                            }
                        }

                        persistent_manager.update_icu2_pid(Some(icu2_pid.clone()));
                    }
                }
            },
        }
    }
}

async fn send_obd2_state(restarts: u32, state: &types::State, old_state: &Option<types::State>) -> Result<(), ()> {
    match (state, old_state) {
        (types::State::CheckCharging, Some(types::State::IgnitionOn)) => {
            if send_state(SmsEvent::Driving(false), true, false, restarts, true).await.is_err() {
                defmt::error!("error sending sms");
            }
        }
        (types::State::IgnitionOn, _old_state) => {
            if send_state(SmsEvent::Driving(true), true, false, restarts, true).await.is_err() {
                defmt::error!("error sending sms");
            }
        }
        _ => {}
    }

    Ok(())
}

async fn send_icu2_pid_state(icu2_pid: &types::Icu2Pid, restarts: u32) -> Result<(), ()> {
    if let Err(err) = send_state(
        SmsEvent::Closed {
            trunk_open: icu2_pid.trunk_open,
            engine_hood_open: icu2_pid.engine_hood_open,
            actuator_back_door_passenger_side_unlock: icu2_pid.actuator_back_door_passenger_side_unlock,
            actuator_back_door_driver_side_unlock: icu2_pid.actuator_back_door_driver_side_unlock,
        },
        false,
        false,
        restarts,
        true,
    )
    .await
    {
        defmt::error!("error sending icu2 pid state: {:?}", defmt::Debug2Format(&err));
        return Err(());
    }

    Ok(())
}

async fn process_new_fix(
    old_fix: &Option<Fix>,
    new_fix: Option<Fix>,
    restarts: u32,
    state: &Option<types::State>,
) -> Option<Fix> {
    match (old_fix, new_fix) {
        (Some(old_fix), Some(new_fix)) => {
            if state != &Some(types::State::IgnitionOn) {
                let mut fix_distance = *old_fix - new_fix;
                if fix_distance > 300.0 {
                    if let Some(accurate_fix) = GnssState::wait_for_fix(Duration::from_secs(120)).await {
                        fix_distance = *old_fix - accurate_fix;
                        if fix_distance > 300.0 {
                            send_state(SmsEvent::MovementOnBattery(fix_distance), false, false, restarts, true)
                                .await
                                .ok();
                        }
                        return Some(accurate_fix);
                    } else {
                        return None;
                    }
                }
            }
        }
        (Some(_fix), None) => {
            send_state(SmsEvent::Fix(false), false, false, restarts, true).await.ok();
        }
        (None, Some(_new_fix)) => {
            send_state(SmsEvent::Fix(true), false, false, restarts, true).await.ok();
        }
        (None, None) => {}
    }
    new_fix
}
