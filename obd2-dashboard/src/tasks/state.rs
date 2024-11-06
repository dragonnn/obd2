use defmt::*;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use statig::prelude::*;

use super::obd2::{set_obd2_sets, Obd2Debug, Obd2PidSets};
use crate::{
    event::{LcdEvent, Obd2Event, LCD_EVENTS},
    tasks::buttons::{Action, Button},
};

pub static EVENTS: Channel<CriticalSectionRawMutex, KiaEvent, 128> = Channel::new();

pub struct KiaContext {}

#[derive(Format, PartialEq, Clone)]
pub enum KiaEvent {
    InitIgnitionOff,
    InitIgnitionOn,
    Shutdown,
    Button(crate::tasks::buttons::Action),
    Obd2Event(Obd2Event),
    Obd2Debug(Obd2Debug),
    Obd2LoopEnd,
}

#[derive(Default)]
pub struct KiaState {}

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
            KiaEvent::InitIgnitionOff => {
                LCD_EVENTS.send(LcdEvent::PowerOff).await;
                Handled
            }
            KiaEvent::InitIgnitionOn => {
                LCD_EVENTS.send(LcdEvent::Main).await;
                Transition(State::ignition_on())
            }
            _ => Handled,
        }
    }

    #[action]
    async fn enter_ignition_on(&mut self) {
        set_obd2_sets(Obd2PidSets::IgnitionOn).await;
    }

    #[state(entry_action = "enter_ignition_on")]
    async fn ignition_on(&mut self, event: &KiaEvent) -> Response<State> {
        match event {
            KiaEvent::Shutdown => {
                LCD_EVENTS.send(LcdEvent::PowerOff).await;
                Transition(State::check_charging())
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
        set_obd2_sets(Obd2PidSets::Charging).await;
    }

    #[state(entry_action = "enter_charging")]
    async fn check_charging(&mut self, event: &KiaEvent) -> Response<State> {
        Transition(State::ignition_off())
    }

    #[action]
    async fn enter_ignition_off(&mut self) {
        set_obd2_sets(Obd2PidSets::IgnitionOff).await;
    }

    #[state(entry_action = "enter_ignition_off")]
    async fn ignition_off(&mut self, event: &KiaEvent) -> Response<State> {
        Handled
    }
}

impl KiaState {
    // The `on_transition` callback that will be called after every transition.
    fn on_transition(&mut self, source: &State, target: &State) {
        info!("kia transitioned from `{}` to `{}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &KiaEvent) {
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
    let mut state = KiaState::default().uninitialized_state_machine().init().await;

    loop {
        let event = EVENTS.receive().await;
        state.handle(&event).await;
    }
}
