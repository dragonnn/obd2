use defmt::{Format, error, info, unwrap, warn};
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::{
    cap1188::Cap1188Inputs,
    event::{KIA_EVENTS, KiaEvent, LCD_EVENTS, LcdEvent},
    tasks::power::get_shutdown_signal,
    types::Cap1188,
};

const BUTTON_DEBOUNCE_TIME: embassy_time::Duration = embassy_time::Duration::from_millis(30);
const BUTTON_DEBOUNCE_POLL_TIME: embassy_time::Duration = embassy_time::Duration::from_millis(5);

struct ButtonDebouncer {
    stable: u8,
    candidates: u8,
    candidate_since: [embassy_time::Instant; 8],
}

impl ButtonDebouncer {
    fn new(initial: Cap1188Inputs) -> Self {
        let initial = initial.into_bytes()[0];
        Self { stable: initial, candidates: initial, candidate_since: [embassy_time::Instant::now(); 8] }
    }

    fn has_pending(&self) -> bool {
        self.stable != self.candidates
    }

    async fn next_state(&mut self, cap1188: &mut Cap1188) -> Cap1188Inputs {
        loop {
            let sample = unwrap!(cap1188.touched().await).into_bytes()[0];
            let now = embassy_time::Instant::now();

            for index in 0..8 {
                let mask = 1 << index;
                if sample & mask != self.candidates & mask {
                    self.candidates ^= mask;
                    self.candidate_since[index] = now;
                }
            }

            let previous_stable = self.stable;
            for index in 0..8 {
                let mask = 1 << index;
                if self.stable & mask != self.candidates & mask
                    && self.candidate_since[index].elapsed() >= BUTTON_DEBOUNCE_TIME
                {
                    self.stable ^= mask;
                }
            }

            if self.stable != previous_stable || !self.has_pending() {
                return Cap1188Inputs::from_bytes([self.stable]);
            }

            Timer::after(BUTTON_DEBOUNCE_POLL_TIME).await;
        }
    }
}

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

static INIT_BUTTONS: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn run(mut cap1188: Cap1188) {
    let mut shutdown_on_init = false;
    /*if let Either::Second(_) = select(INIT_BUTTONS.wait(), get_shutdown_signal().next_message()).await {
        shutdown_on_init = true;
    }*/
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    cap1188.reset().await.ok();
    let mut init_attempts = 0;
    loop {
        match cap1188.init().await {
            Ok(true) => {
                info!("cap1188 init success");
                break;
            }
            Ok(false) => {
                info!("cap1188 init failed");
                init_attempts += 1;
                Timer::after(embassy_time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                info!("cap1188 init error: {:?}", e);
                init_attempts += 1;
                Timer::after(embassy_time::Duration::from_secs(1)).await;
            }
        }
        if init_attempts > 50 {
            error!("cap1188 failed to init after {} attempts, giving up", init_attempts);
            return;
        }
    }
    info!("cap1188 task started");
    cap1188.calibrate().await.ok();
    let mut old_touched = unwrap!(cap1188.touched().await);
    let mut old_touched_bytes = old_touched.into_bytes()[0];
    let mut debouncer = ButtonDebouncer::new(old_touched);
    info!("cap1188 task running");
    select(
        async {
            loop {
                if debouncer.has_pending() {
                    // Continue polling until each changing button has independently stabilized.
                } else if old_touched_bytes > 0 {
                    embassy_time::with_timeout(embassy_time::Duration::from_secs(250), cap1188.wait_for_touched())
                        .await
                        .ok();
                    warn!("cap1188 touched timeout on bytes: {:?}", old_touched_bytes);
                    cap1188.calibrate().await.ok();
                    old_touched_bytes = 0;
                    old_touched = Cap1188Inputs::default();
                } else {
                    cap1188.wait_for_touched().await;
                }
                let new_touched = debouncer.next_state(&mut cap1188).await;
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
            }
        },
        async {
            if !shutdown_on_init {
                get_shutdown_signal().next_message_pure().await
            }
        },
    )
    .await;
    cap1188.shutdown().await.ok();
}

pub fn init() {
    INIT_BUTTONS.signal(());
}
