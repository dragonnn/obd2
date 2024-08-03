use defmt::{info, unwrap, Format};
use embassy_time::Timer;

use super::state::{KiaEvent, EVENTS};
use crate::types::Cap1188;

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
    Timer::after(embassy_time::Duration::from_secs(5)).await;
    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B0))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B0))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B1))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B1))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B2))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B2))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B3))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B3))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B4))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B4))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B5))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B5))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B6))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B6))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    /*EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B7))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;
    EVENTS.send(KiaEvent::Button(Action::Released(Button::B7))).await;
    Timer::after(embassy_time::Duration::from_secs(1)).await;*/

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
    loop {
        cap1188.wait_for_touched().await;
        let new_touched = unwrap!(cap1188.touched().await);
        let new_touched_bytes = new_touched.into_bytes()[0];
        if new_touched_bytes != old_touched_bytes {
            if new_touched_bytes > 0 {
                info!("touched: {:?}", new_touched_bytes);
            } else {
                info!("released: {:?}", new_touched_bytes);
            }
            if new_touched.b0() != old_touched.b0() {
                if new_touched.b0() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B0))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B0))).await;
                }
            }
            if new_touched.b1() != old_touched.b1() {
                if new_touched.b1() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B1))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B1))).await;
                }
            }
            if new_touched.b2() != old_touched.b2() {
                if new_touched.b2() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B2))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B2))).await;
                }
            }
            if new_touched.b3() != old_touched.b3() {
                if new_touched.b3() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B3))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B3))).await;
                }
            }
            if new_touched.b4() != old_touched.b4() {
                if new_touched.b4() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B4))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B4))).await;
                }
            }
            if new_touched.b5() != old_touched.b5() {
                if new_touched.b5() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B5))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B5))).await;
                }
            }
            if new_touched.b6() != old_touched.b6() {
                if new_touched.b6() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B6))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B6))).await;
                }
            }
            if new_touched.b7() != old_touched.b7() {
                if new_touched.b7() {
                    EVENTS.send(KiaEvent::Button(Action::Pressed(Button::B7))).await;
                } else {
                    EVENTS.send(KiaEvent::Button(Action::Released(Button::B7))).await;
                }
            }
            old_touched = new_touched;
            old_touched_bytes = new_touched_bytes;
        }
    }
}
