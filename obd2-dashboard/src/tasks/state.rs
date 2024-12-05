use defmt::*;
use embassy_futures::select::{select, Either::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{with_timeout, Duration, Instant};
use esp_hal_procmacros::ram;
use statig::prelude::*;
use types::OnBoardChargerPid;

use super::{
    ieee802154::{extra_txframes_pub, TxFramePub},
    obd2::{set_obd2_sets, Obd2Debug, Obd2PidSets},
    power::{get_power_events_pub, PowerEvent, PowerEventPublisher},
};
use crate::{
    event::{LcdEvent, Obd2Event, LCD_EVENTS},
    tasks::{
        buttons::{Action, Button},
        ieee802154,
    },
};

pub static EVENTS: Channel<CriticalSectionRawMutex, KiaEvent, 128> = Channel::new();

#[derive(Format, PartialEq, Clone)]
pub enum KiaEvent {
    IgnitionOff,
    IgnitionOn,
    Button(crate::tasks::buttons::Action),
    Obd2Event(Obd2Event),
    Obd2Debug(Obd2Debug),
    Obd2LoopEnd(bool),
    Ticker,
}

#[ram(rtc_fast, persistent)]
static mut LAST_IGNITION_ON: i64 = 0;

#[derive()]
pub struct KiaState {
    pub power_events_pub: PowerEventPublisher,
    pub tx_frame_pub: TxFramePub,
    pub rtc: crate::types::Rtc,
}

#[state_machine(
    // This sets the initial state to `led_on`.
    initial = "State::init()",
    // Derive the Debug trait on the `State` enum.
    state(derive(Format, Debug)),
    // Derive the Debug trait on the `Superstate` enum.
    superstate(derive(Format, Debug)),
    // Set the `on_transition` callback.
    on_transition = "Self::on_transition",
    // Set the `on_dispatch` callback.
    on_dispatch = "Self::on_dispatch"
)]
impl KiaState {
    #[state()]
    async fn init(&mut self, event: &KiaEvent) -> Response<State> {
        //info!("init got event: {:?}", event);
        match event {
            KiaEvent::IgnitionOff => Transition(State::ignition_off(Instant::now())),
            KiaEvent::IgnitionOn => Transition(State::ignition_on()),
            _ => Handled,
        }
    }

    #[action]
    async fn enter_ignition_on(&mut self) {
        unsafe {
            LAST_IGNITION_ON = self.rtc.lock().await.current_time().and_utc().timestamp();
            info!("last ignition on: {}", LAST_IGNITION_ON);
        }

        ieee802154::send_now();
        LCD_EVENTS.send(LcdEvent::Main).await;
        set_obd2_sets(Obd2PidSets::IgnitionOn).await;
        self.tx_frame_pub.publish_immediate(types::TxFrame::State(types::State::IgnitionOn));
    }

    #[state(entry_action = "enter_ignition_on")]
    async fn ignition_on(&mut self, event: &KiaEvent) -> Response<State> {
        match event {
            KiaEvent::IgnitionOff => {
                LCD_EVENTS.send(LcdEvent::PowerOff).await;
                Transition(State::check_charging(Default::default(), Instant::now()))
            }
            KiaEvent::Obd2Event(obd2_event) => {
                LCD_EVENTS.send(LcdEvent::Obd2Event(obd2_event.clone())).await;
                Handled
            }
            KiaEvent::Obd2Debug(obd2_debug) => {
                LCD_EVENTS.send(LcdEvent::Obd2Debug(obd2_debug.clone())).await;
                Handled
            }
            KiaEvent::Obd2LoopEnd(_all) => {
                LCD_EVENTS.send(LcdEvent::Render).await;
                Handled
            }
            KiaEvent::Button(action) => {
                info!("button action: {:?}", action);
                match action {
                    /*Action::Pressed(Button::B4) | Action::Released(Button::B4) => {
                        LCD_EVENTS.send(LcdEvent::Main).await;
                    }
                    Action::Pressed(Button::B5) | Action::Released(Button::B5) => {
                        LCD_EVENTS.send(LcdEvent::Debug).await;
                    }
                    Action::Pressed(Button::B3) | Action::Released(Button::B3) => {
                        LCD_EVENTS.send(LcdEvent::Menu).await;
                    }*/
                    _ => {
                        warn!("unhandled button action: {:?}", action);
                    }
                }
                Handled
            }

            _ => Handled,
        }
    }

    #[action]
    async fn enter_check_charging(&mut self) {
        ieee802154::send_now();
        set_obd2_sets(Obd2PidSets::Charging).await;
        self.tx_frame_pub.publish_immediate(types::TxFrame::State(types::State::CheckCharging));
        embassy_time::Timer::after_secs(1).await;
    }

    #[state(entry_action = "enter_check_charging")]
    async fn check_charging(
        &mut self,
        event: &KiaEvent,
        obc_pid: &mut Option<OnBoardChargerPid>,
        timeout: &Instant,
    ) -> Response<State> {
        match event {
            KiaEvent::IgnitionOn => Transition(State::ignition_on()),
            KiaEvent::Obd2Event(Obd2Event::OnBoardChargerPid(new_obc_pid)) => {
                *obc_pid = Some(new_obc_pid.clone());
                Handled
            }
            KiaEvent::Obd2LoopEnd(_all) => {
                if let Some(obc_pid) = obc_pid {
                    if obc_pid.ac_input_current > 0.0 {
                        Transition(State::charging(None, 0))
                    } else {
                        if timeout.elapsed().as_secs() > 5 * 60 {
                            Transition(State::ignition_off(Instant::now()))
                        } else {
                            Handled
                        }
                    }
                } else {
                    if timeout.elapsed().as_secs() > 5 * 60 {
                        Transition(State::ignition_off(Instant::now()))
                    } else {
                        Handled
                    }
                }
            }
            _ => Handled,
        }
    }

    #[action]
    async fn enter_charging(&mut self) {
        ieee802154::send_now();
        set_obd2_sets(Obd2PidSets::Charging).await;
        self.tx_frame_pub.publish_immediate(types::TxFrame::State(types::State::Charging));
    }

    #[state(entry_action = "enter_charging")]
    async fn charging(
        &mut self,
        event: &KiaEvent,
        obc_pid: &mut Option<OnBoardChargerPid>,
        obc_pid_wait: &mut u8,
    ) -> Response<State> {
        match event {
            KiaEvent::Obd2Event(Obd2Event::OnBoardChargerPid(new_obc_pid)) => {
                let ret = if new_obc_pid.ac_input_current == 0.0 {
                    warn!("ac input current is zero");
                    Transition(State::check_charging(None, Instant::now()))
                } else {
                    Handled
                };
                *obc_pid = Some(new_obc_pid.clone());
                ret
            }
            KiaEvent::IgnitionOn => Transition(State::ignition_on()),
            KiaEvent::Obd2LoopEnd(_all) => {
                if obc_pid.is_none() {
                    if *obc_pid_wait > 50 {
                        Transition(State::check_charging(None, Instant::now()))
                    } else {
                        *obc_pid_wait += 1;
                        Handled
                    }
                } else {
                    *obc_pid = None;
                    Handled
                }
            }
            _ => Handled,
        }
    }

    #[action]
    async fn enter_ignition_off(&mut self) {
        ieee802154::send_now();
        LCD_EVENTS.send(LcdEvent::PowerOff).await;
        set_obd2_sets(Obd2PidSets::IgnitionOff).await;
        self.tx_frame_pub.publish_immediate(types::TxFrame::State(types::State::IgnitionOff));
    }

    #[state(entry_action = "enter_ignition_off")]
    async fn ignition_off(&mut self, event: &KiaEvent, timeout: &Instant) -> Response<State> {
        let now = self.rtc.lock().await.current_time().and_utc().timestamp();
        let last_ignition_on = unsafe { LAST_IGNITION_ON };
        let shutdown_duration = if last_ignition_on != 0 && now - last_ignition_on > 60 * 60 {
            Duration::from_secs(60 * 60)
        } else {
            Duration::from_secs(15 * 60)
        };
        warn!(
            "shutdown duration: {}min: last_ignition_on:{}sec now: {}sec",
            shutdown_duration.as_secs() / 60,
            last_ignition_on,
            now
        );

        match event {
            KiaEvent::Obd2Event(Obd2Event::Icu3Pid(icu3_pid)) => {
                if icu3_pid.on_board_charger_wakeup_output {
                    Transition(State::check_charging(None, Instant::now()))
                } else {
                    Handled
                }
            }
            KiaEvent::Obd2Event(Obd2Event::OnBoardChargerPid(obc_pid)) => {
                if obc_pid.ac_input_current > 0.0 {
                    Transition(State::check_charging(None, Instant::now()))
                } else {
                    Handled
                }
            }
            KiaEvent::IgnitionOn => Transition(State::ignition_on()),
            KiaEvent::Obd2LoopEnd(all) => {
                if timeout.elapsed().as_secs() > 2 * 60 || (*all && timeout.elapsed().as_secs() > 10) {
                    Transition(State::shutdown(shutdown_duration))
                } else {
                    Handled
                }
            }
            _ => {
                if timeout.elapsed().as_secs() > 5 * 60 {
                    Transition(State::shutdown(shutdown_duration))
                } else {
                    Handled
                }
            }
        }
    }

    #[action]
    async fn enter_shutdown(&mut self, duration: &embassy_time::Duration) {
        ieee802154::send_now();
        self.tx_frame_pub.publish_immediate(types::TxFrame::State(types::State::Shutdown((*duration).into())));
        embassy_time::Timer::after_millis(200).await;
        self.power_events_pub.publish_immediate(PowerEvent::Shutdown(*duration));
        ieee802154::send_now();
    }

    #[state(entry_action = "enter_shutdown")]
    async fn shutdown(&mut self, duration: &Duration) -> Response<State> {
        Handled
    }
}

impl KiaState {
    // The `on_transition` callback that will be called after every transition.
    fn on_transition(&mut self, source: &State, target: &State) {
        info!("kia transitioned from `{}` to `{}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &KiaEvent) {
        self.power_events_pub.publish_immediate(PowerEvent::RwdtFeed);
        if let KiaEvent::Obd2Event(_) = event {
            trace!("kia dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
        } else {
            match event {
                KiaEvent::Obd2Debug(_) | KiaEvent::Obd2LoopEnd(_) => {
                    trace!("kia dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
                }
                _ => {
                    info!("kia dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
                }
            }
        }
    }
}

pub async fn run(rtc: crate::types::Rtc) {
    let mut state = KiaState { power_events_pub: get_power_events_pub(), tx_frame_pub: extra_txframes_pub(), rtc }
        .uninitialized_state_machine()
        .init()
        .await;

    loop {
        match with_timeout(Duration::from_secs(5), EVENTS.receive()).await {
            Ok(event) => {
                state.handle(&event).await;
            }
            Err(_) => {
                state.handle(&KiaEvent::Ticker).await;
            }
        }
    }
}
