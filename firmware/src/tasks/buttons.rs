use defmt::{error, info, unwrap, warn, Format};
use embassy_futures::select::select;
use embassy_time::Timer;

use crate::{
    event::{KiaEvent, LcdEvent, KIA_EVENTS, LCD_EVENTS},
    tasks::power::get_shutdown_signal,
    types::Cap1188,
};

#[derive(Format, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
}

#[derive(Format, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Pressed(Button),
    Released(Button),
}

#[embassy_executor::task]
pub async fn run(mut cap1188: Cap1188) {
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    cap1188.reset().await.ok();
    loop {
        match cap1188.init().await {
            Ok(true) => {
                info!("cap1188 init success");
                break;
            }
            Ok(false) => {
                info!("cap1188 init failed");
                Timer::after(embassy_time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                info!("cap1188 init error: {:?}", e);
                Timer::after(embassy_time::Duration::from_secs(1)).await;
            }
        }
    }
    info!("cap1188 task started");
    let mut old_touched = unwrap!(cap1188.touched().await);
    let mut old_touched_bytes = old_touched.into_bytes()[0];
    let mut last_touched = embassy_time::Instant::now();
    let mut fast_loops = 0;
    info!("cap1188 task running");
    select(
        async {
            loop {
                if old_touched_bytes > 0 {
                    embassy_time::with_timeout(embassy_time::Duration::from_millis(100), cap1188.wait_for_touched())
                        .await
                        .ok();
                    warn!("cap1188 touched timeout");
                } else {
                    cap1188.wait_for_touched().await;
                }
                let new_touched = unwrap!(cap1188.touched().await);
                let new_touched_bytes = new_touched.into_bytes()[0];
                if new_touched_bytes != old_touched_bytes {
                    if new_touched.b0() != old_touched.b0() {
                        if new_touched.b0() {
                            info!("button b0 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B0))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B0))).await;
                        } else {
                            info!("button b0 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B0))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B0))).await;
                        }
                    }
                    if new_touched.b1() != old_touched.b1() {
                        if new_touched.b1() {
                            info!("button b1 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B1))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B1))).await;
                        } else {
                            info!("button b1 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B1))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B1))).await;
                        }
                    }
                    if new_touched.b2() != old_touched.b2() {
                        if new_touched.b2() {
                            info!("button b2 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B2))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B2))).await;
                        } else {
                            info!("button b2 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B2))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B2))).await;
                        }
                    }
                    if new_touched.b3() != old_touched.b3() {
                        if new_touched.b3() {
                            info!("button b3 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B3))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B3))).await;
                        } else {
                            info!("button b3 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B3))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B3))).await;
                        }
                    }
                    if new_touched.b4() != old_touched.b4() {
                        if new_touched.b4() {
                            info!("button b4 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B4))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B4))).await;
                        } else {
                            info!("button b4 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B4))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B4))).await;
                        }
                    }
                    if new_touched.b5() != old_touched.b5() {
                        if new_touched.b5() {
                            info!("button b5 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B5))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B5))).await;
                        } else {
                            info!("button b5 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B5))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B5))).await;
                        }
                    }
                    if new_touched.b6() != old_touched.b6() {
                        if new_touched.b6() {
                            info!("button b6 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B6))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B6))).await;
                        } else {
                            info!("button b6 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B6))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B6))).await;
                        }
                    }
                    if new_touched.b7() != old_touched.b7() {
                        if new_touched.b7() {
                            info!("button b7 pressed");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B7))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Pressed(Button::B7))).await;
                        } else {
                            info!("button b7 released");
                            KIA_EVENTS.send(KiaEvent::Button(Action::Released(Button::B7))).await;
                            LCD_EVENTS.send(LcdEvent::Button(Action::Released(Button::B7))).await;
                        }
                    }
                }
                old_touched = new_touched;
                old_touched_bytes = new_touched_bytes;
                if last_touched.elapsed() < embassy_time::Duration::from_millis(50) {
                    embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;
                    fast_loops += 1;
                    if fast_loops > 10 {
                        embassy_time::with_timeout(embassy_time::Duration::from_secs(1), cap1188.wait_for_released())
                            .await
                            .ok();
                        fast_loops = 0;
                    }
                }
                last_touched = embassy_time::Instant::now();
            }
        },
        get_shutdown_signal().next_message(),
    )
    .await;
    cap1188.shutdown().await;
}
