use defmt::*;
use embassy_futures::select;
use embassy_time::{Duration, Instant, Timer};
use futures::StreamExt;
use types::GnssFix;

use super::TASKS_SUBSCRIBERS;
use crate::{
    board::{LightSensor, Lightwell, Sense, Wdg},
    tasks::{battery::State as BatteryState, gnss::Fix, montion_detection::State as MontionDetectionState},
};

#[derive(Format, Clone, Default)]
pub struct State {
    monition_detect: bool,
    button_detect: bool,
    battery: BatteryState,
}

#[embassy_executor::task]
pub async fn task(mut sense: Sense, mut lightwell: Lightwell, mut wdg: Wdg, mut light_sensor: LightSensor) {
    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut monition_detection_sub = MontionDetectionState::subscribe().await;
    let mut button_sub = crate::tasks::button::subscribe().await;
    let mut state = State { battery: BatteryState::get().await, ..Default::default() };

    loop {
        wdg.pet().await;
        warn!("State loop");
        match select::select4(
            state_loop(&mut state, &mut sense, &mut lightwell, &mut wdg, &mut light_sensor),
            battery_state_sub.next(),
            async {
                match select::select(monition_detection_sub.next(), button_sub.next()).await {
                    select::Either::First(_) => true,
                    select::Either::Second(_) => false,
                }
            },
            Timer::after_secs(20),
        )
        .await
        {
            select::Either4::Second(new_battery_state) => {
                info!("Battery state changed: {:?}", new_battery_state);
                if let Some(new_battery_state) = new_battery_state {
                    state.battery = new_battery_state
                }
            }
            select::Either4::Third(monition_or_button) => {
                info!("Monition detected");
                if monition_or_button {
                    state.monition_detect = true;
                } else {
                    state.button_detect = true;
                }
            }
            _ => {
                wdg.pet().await;
            }
        }
    }
}

async fn state_loop(
    state: &mut State,
    sense: &mut Sense,
    lightwell: &mut Lightwell,
    wdg: &mut Wdg,
    light_sensor: &mut LightSensor,
) {
    sense.off();
    lightwell.off();

    let mut lightwell_step: f32 = 10.0;
    let mut lightwell_step_size: f32 = 1.0;
    let mut lightwell_step_max = 50.0;
    let mut lightwell_step_min = 5.0;
    let mut lightwell_direction = true;

    let mut last_battery_state_time = Instant::now();

    let mut w = light_sensor.w().await;

    loop {
        wdg.pet().await;
        if last_battery_state_time.elapsed().as_secs() > 60 {
            state.battery = BatteryState::get().await;
            last_battery_state_time = Instant::now();
            if !state.battery.charging {
                w = light_sensor.w().await;
            }
        }
        let g = (((state.battery.capacity as f32 / 100.0) / 1.0) * 255.0) as u8;
        let r = ((((100 - state.battery.capacity) as f32 / 100.0) / 1.0) * 255.0) as u8;
        if state.battery.charging {
            let stepped_g = g as f32 * (lightwell_step / 100.0);
            let stepped_r = r as f32 * (lightwell_step / 100.0);

            lightwell.g(stepped_g as u8);
            lightwell.r(stepped_r as u8);

            if false {
                lightwell_step_max = 12.0;
                lightwell_step_min = 0.0;
                lightwell_step_size = 0.24;
                Timer::after_millis(200).await;
            } else {
                lightwell_step_max = 100.0;
                lightwell_step_min = 5.0;
                lightwell_step_size = 1.0 * 4.0;
            }

            if lightwell_direction {
                lightwell_step += lightwell_step_size;
                if lightwell_step >= lightwell_step_max {
                    lightwell_direction = false;
                }
            } else {
                lightwell_step -= lightwell_step_size;
                if lightwell_step <= lightwell_step_min {
                    lightwell_direction = true;
                    w = light_sensor.w().await;
                }
            }
            Timer::after(Duration::from_millis(100)).await;
        } else if state.monition_detect {
            if w > 15 {
                lightwell.b(255);
                Timer::after(Duration::from_millis(250)).await;
                lightwell.b(0);
            }
            state.monition_detect = false;
        } else if state.button_detect {
            warn!("Button pressed");
            lightwell.b(255);
            lightwell.r(255);
            lightwell.g(255);
            Timer::after(Duration::from_millis(250)).await;
            lightwell.b(0);
            lightwell.r(0);
            lightwell.g(0);

            state.button_detect = false;
        } else {
            light_sensor.shutdown().await;
            if w > 15 {
                sense.g(g);
                sense.r(r);
                Timer::after(Duration::from_millis(40)).await;
                sense.g(0);
                sense.r(0);
                Timer::after(Duration::from_millis(2000 - 40)).await;
            } else {
                warn!("Light sensor shutdown");
                Timer::after(Duration::from_secs(30)).await;
                w = light_sensor.w().await;
                light_sensor.shutdown().await;
            }
        }
    }
}
