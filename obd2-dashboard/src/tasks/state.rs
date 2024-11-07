use defmt::*;
use embassy_futures::select::{select, Either::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{with_timeout, Duration, Instant};
use statig::prelude::*;
use types::OnBoardChargerPid;

use super::{
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
    Obd2LoopEnd,
    Ticker,
}

#[derive()]
pub struct KiaState {
    pub power_events_pub: PowerEventPublisher,
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
            KiaEvent::IgnitionOff => {
                LCD_EVENTS.send(LcdEvent::PowerOff).await;
                Transition(State::ignition_off(Instant::now()))
            }
            KiaEvent::IgnitionOn => {
                LCD_EVENTS.send(LcdEvent::Main).await;
                Transition(State::ignition_on())
            }
            _ => Handled,
        }
    }

    #[action]
    async fn enter_ignition_on(&mut self) {
        ieee802154::send_now();
        set_obd2_sets(Obd2PidSets::IgnitionOn).await;
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
            KiaEvent::Obd2LoopEnd => {
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
    async fn enter_charging(&mut self) {
        ieee802154::send_now();
        set_obd2_sets(Obd2PidSets::Charging).await;
    }

    #[state(entry_action = "enter_charging")]
    async fn check_charging(
        &mut self,
        event: &KiaEvent,
        obc_pid: &mut Option<OnBoardChargerPid>,
        timeout: &Instant,
    ) -> Response<State> {
        match event {
            KiaEvent::Obd2Event(Obd2Event::OnBoardChargerPid(new_obc_pid)) => {
                *obc_pid = Some(new_obc_pid.clone());
                Handled
            }
            KiaEvent::Obd2LoopEnd => {
                if let Some(obc_pid) = obc_pid {
                    if obc_pid.ac_input_current > 0.0 {
                        Transition(State::charging(None))
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

    #[state(entry_action = "enter_charging")]
    async fn charging(&mut self, event: &KiaEvent, obc_pid: &mut Option<OnBoardChargerPid>) -> Response<State> {
        match event {
            KiaEvent::IgnitionOn => Transition(State::ignition_on()),
            KiaEvent::Obd2LoopEnd => {
                if obc_pid.is_none() {
                    Transition(State::check_charging(None, Instant::now()))
                } else {
                    *obc_pid = None;
                    Handled
                }
            }
            KiaEvent::Obd2Event(Obd2Event::OnBoardChargerPid(new_obc_pid)) => {
                let ret = if new_obc_pid.ac_input_current == 0.0 {
                    Transition(State::check_charging(None, Instant::now()))
                } else {
                    Handled
                };
                *obc_pid = Some(new_obc_pid.clone());
                ret
            }
            _ => Handled,
        }
    }

    #[action]
    async fn enter_ignition_off(&mut self) {
        ieee802154::send_now();
        set_obd2_sets(Obd2PidSets::IgnitionOff).await;
    }

    #[state(entry_action = "enter_ignition_off")]
    async fn ignition_off(&mut self, event: &KiaEvent, timeout: &Instant) -> Response<State> {
        match event {
            KiaEvent::IgnitionOn => Transition(State::ignition_on()),
            KiaEvent::Obd2LoopEnd => {
                if timeout.elapsed().as_secs() > 1 * 60 {
                    ieee802154::send_now();
                    self.power_events_pub
                        .publish(PowerEvent::Shutdown(embassy_time::Duration::from_secs(15 * 60)))
                        .await;
                }
                Handled
            }
            _ => {
                if timeout.elapsed().as_secs() > 5 * 60 {
                    ieee802154::send_now();
                    self.power_events_pub
                        .publish(PowerEvent::Shutdown(embassy_time::Duration::from_secs(5 * 60)))
                        .await;
                }
                Handled
            }
        }
    }
}

impl KiaState {
    // The `on_transition` callback that will be called after every transition.
    fn on_transition(&mut self, source: &State, target: &State) {
        info!("kia transitioned from `{}` to `{}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &KiaEvent) {
        self.power_events_pub.try_publish(PowerEvent::RwdtFeed).ok();
        if let KiaEvent::Obd2Event(_) = event {
            trace!("kia dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
        } else {
            match event {
                KiaEvent::Obd2Debug(_) | KiaEvent::Obd2LoopEnd => {
                    trace!("kia dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
                }
                _ => {
                    info!("kia dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
                }
            }
        }
    }
}

pub async fn run() {
    let mut state = KiaState { power_events_pub: get_power_events_pub() }.uninitialized_state_machine().init().await;

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
