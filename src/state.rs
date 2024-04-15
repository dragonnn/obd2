use defmt::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use statig::prelude::*;

const EVENTS: Channel<CriticalSectionRawMutex, KiaEvent, 10> = Channel::new();

pub struct KiaContext {}

#[derive(Format, PartialEq, Eq, Clone, Copy)]
pub enum KiaEvent {
    Init,
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
        Transition(State::init())
    }
}

impl KiaState {
    // The `on_transition` callback that will be called after every transition.
    fn on_transition(&mut self, source: &State, target: &State) {
        info!("transitioned from `{}` to `{}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &KiaEvent) {
        info!(
            "dispatching `{}` to `{}`",
            event,
            defmt::Debug2Format(&state)
        );
    }
}

pub async fn run() {
    let mut context = KiaContext {};
    let mut state = KiaState::default()
        .uninitialized_state_machine()
        .init_with_context(&mut context)
        .await;
    state
        .handle_with_context(&KiaEvent::Init, &mut context)
        .await;

    loop {
        let event = EVENTS.receive().await;
        state.handle_with_context(&event, &mut context).await;
    }
}
