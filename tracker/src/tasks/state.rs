use defmt::Format;
use embassy_futures::select;
use embassy_time::{Duration, Instant, Timer};
use futures::StreamExt;

use super::TASKS_SUBSCRIBERS;
use crate::{
    board::{LightSensor, Lightwell, Sense, Wdg},
    tasks::{battery::State as BatteryState, montion_detection::State as MontionDetectionState},
};

#[derive(Format, Clone, Default)]
pub struct State {
    montion_detect: bool,
    battery: BatteryState,
}

#[embassy_executor::task]
pub async fn task(mut sense: Sense, mut lightwell: Lightwell, mut wdg: Wdg, mut light_sensor: LightSensor) {
    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut montion_detection_sub = MontionDetectionState::subscribe().await;
    let mut state = State { battery: BatteryState::get().await, ..Default::default() };

    loop {
        wdg.pet().await;
        match select::select3(
            state_loop(&mut state, &mut sense, &mut lightwell, &mut wdg, &mut light_sensor),
            battery_state_sub.next(),
            montion_detection_sub.next(),
        )
        .await
        {
            select::Either3::Second(new_battery_state) => {
                if let Some(new_battery_state) = new_battery_state {
                    state.battery = new_battery_state
                }
            }
            select::Either3::Third(_) => {
                state.montion_detect = true;
            }
            _ => {}
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

            if w < 15 {
                lightwell_step_max = 12.0;
                lightwell_step_min = 0.0;
                lightwell_step_size = 0.24;
            } else {
                lightwell_step_max = 100.0;
                lightwell_step_min = 5.0;
                lightwell_step_size = 1.0;
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

            Timer::after(Duration::from_millis(25)).await;
        } else if state.montion_detect {
            if w > 15 {
                lightwell.b(255);
                Timer::after(Duration::from_millis(250)).await;
                lightwell.b(0);
            }
            state.montion_detect = false;
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
                Timer::after(Duration::from_secs(30)).await;
                w = light_sensor.w().await;
                light_sensor.shutdown().await;
            }
        }
    }
}
