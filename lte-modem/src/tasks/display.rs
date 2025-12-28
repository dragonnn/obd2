use core::fmt::Write as _;

use defmt::{error, info, warn, Format};
use embassy_futures::select::{select4, Either4::*};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_6X13 as FONT, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use heapless::String;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306Async};

use super::battery::State as BatteryState;
use crate::{board::BoardDisplay, tasks::gnss::CHANNEL as GNSS_CHANNEL};

#[embassy_executor::task]
pub async fn task(mut display: BoardDisplay) {
    let mut battery_state = BatteryState::get().await;
    let mut battery_state_sub = BatteryState::subscribe().await;
    let mut gnss_fix_sub = GNSS_CHANNEL.subscriber().unwrap();

    let mut line_buffer = String::<128>::new();

    let mut gnss_fix = None;
    let mut dbm = None;
    let mut dbm_sub = crate::tasks::modem::dbm_channel_sub();

    let mut display_off_timeout = None;

    loop {
        if let Err(err) = display.init().await {
            error!("Display init error: {:?}", err);
            Timer::after_secs(10).await;
        } else {
            display.flush().await.ok();

            loop {
                match select4(
                    Timer::after_secs(1),
                    battery_state_sub.next_message_pure(),
                    gnss_fix_sub.next_message_pure(),
                    dbm_sub.next_message_pure(),
                )
                .await
                {
                    First(_) => {
                        if battery_state.charging {
                            display_off_timeout = None;
                        } else if !battery_state.charging && display_off_timeout.is_none() {
                            display_off_timeout = Some(Instant::now());
                        } else if !battery_state.charging && display_off_timeout.is_some() {
                            if let Some(timeout) = display_off_timeout {
                                if Instant::now() + Duration::from_secs(30) > timeout {
                                    display.set_display_on(false).await.ok();
                                    info!("Device is not charging");
                                    battery_state = battery_state_sub.next_message_pure().await;
                                }
                            } else {
                                display_off_timeout = Some(Instant::now());
                            }
                        } else {
                            warn!("Display off timeout logic error");
                        }
                    }
                    Second(new_battery_state) => {
                        info!("New battery state: {:?}", new_battery_state);
                        battery_state = new_battery_state;
                    }
                    Third(fix) => {
                        info!("New GNSS fix: {:?}", fix);
                        gnss_fix = Some(fix);
                    }
                    Fourth(new_dbm) => {
                        info!("New DBM: {:?}", new_dbm);
                        dbm = new_dbm;
                    }
                }

                line_buffer.clear();
                display.clear_buffer();
                display.set_display_on(true).await.ok();
                let text_style = MonoTextStyleBuilder::new().font(&FONT).text_color(BinaryColor::On).build();

                write!(
                    &mut line_buffer,
                    "B:{}% {}mV dBm:{}",
                    battery_state.capacity,
                    battery_state.voltage,
                    dbm.unwrap_or_default()
                )
                .unwrap();

                Text::with_baseline(&line_buffer, Point::zero(), text_style, Baseline::Top).draw(&mut display).unwrap();

                line_buffer.clear();
                if let Some(gnss_fix) = &gnss_fix {
                    write!(
                        &mut line_buffer,
                        "{}:{}:{} {} {}",
                        gnss_fix.hour, gnss_fix.minute, gnss_fix.seconds, gnss_fix.latitude, gnss_fix.longitude
                    )
                    .unwrap();
                } else {
                    write!(&mut line_buffer, "No GNSS fix").unwrap();
                }

                Text::with_baseline(&line_buffer, Point::new(0, 16), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
                display.flush().await.unwrap();
            }
        }
    }
}
