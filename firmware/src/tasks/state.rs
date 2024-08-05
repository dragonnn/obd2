use defmt::*;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use statig::prelude::*;

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
    async fn init(&mut self, context: &mut KiaContext, event: &KiaEvent) -> Response<State> {
        info!("init got event: {:?}", event);
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

    #[state()]
    async fn ignition_on(&mut self, context: &mut KiaContext, event: &KiaEvent) -> Response<State> {
        info!("ignition_on got event: {:?}", event);
        match event {
            KiaEvent::Shutdown => {
                LCD_EVENTS.send(LcdEvent::PowerOff).await;
                Transition(State::init())
            }
            KiaEvent::Obd2Event(obd2_event) => {
                LCD_EVENTS.send(LcdEvent::Obd2Event(obd2_event.clone())).await;
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
}

impl KiaState {
    // The `on_transition` callback that will be called after every transition.
    fn on_transition(&mut self, source: &State, target: &State) {
        info!("kia transitioned from `{}` to `{}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &KiaEvent) {
        info!("kia dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
    }
}

pub async fn run() {
    let mut context = KiaContext {};
    let mut state = KiaState::default().uninitialized_state_machine().init_with_context(&mut context).await;

    loop {
        let event = EVENTS.receive().await;
        state.handle_with_context(&event, &mut context).await;
    }
}
