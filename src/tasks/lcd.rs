use defmt::{error, info, unwrap, Format};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal};
use embedded_graphics::geometry::{Point, Size};
use statig::prelude::*;

use crate::{
    display::widgets::{Battery, BatteryOrientation},
    event::Obd2Event,
    types::Sh1122,
};

pub static EVENTS: Channel<CriticalSectionRawMutex, LcdEvent, 16> = Channel::new();

pub struct LcdContext {}

#[derive(Format, PartialEq)]
pub enum LcdEvent {
    PowerOff,
    Main,
    Obd2Event(Obd2Event),
}

#[derive()]
pub struct LcdState {
    display1: Sh1122<10>,
    display2: Sh1122<1>,
}

impl LcdState {
    pub fn new(display1: Sh1122<10>, display2: Sh1122<1>) -> Self {
        Self { display1, display2 }
    }
}

#[derive(Default)]
pub struct LcdMainState {
    hv_battery: Battery,
}

#[state_machine(
    // This sets the initial state to `led_on`.
    initial = "State::init()",
    // Derive the Debug trait on the `State` enum.
    state(derive()),
    // Derive the Debug trait on the `Superstate` enum.
    superstate(derive()),
    // Set the `on_transition` callback.
    on_transition = "Self::on_transition",
    // Set the `on_dispatch` callback.
    on_dispatch = "Self::on_dispatch"
)]
impl LcdState {
    #[action]
    async fn enter_init(&mut self, context: &mut LcdContext) {
        unwrap!(self.display1.init(None).await);
        unwrap!(self.display2.init(None).await);

        self.display1.clear();
        self.display2.clear();

        unwrap!(self.display1.sleep(true).await);
        unwrap!(self.display2.sleep(true).await);
    }

    #[state(entry_action = "enter_init")]
    async fn init(&mut self, context: &mut LcdContext, event: &LcdEvent) -> Response<State> {
        info!("init got event: {:?}", event);
        match event {
            LcdEvent::Main => Transition(State::main(LcdMainState {
                hv_battery: Battery::new(
                    Point::new(9, 1),
                    Size::new(128, 62),
                    BatteryOrientation::HorizontalRight,
                    Some(Size::new(8, 32)),
                    4,
                    true,
                ),
                ..Default::default()
            })),
            _ => Handled,
        }
    }

    #[action]
    async fn enter_main(&mut self, context: &mut LcdContext, main: &mut LcdMainState) {
        unwrap!(self.display1.sleep(false).await);
        unwrap!(self.display2.sleep(false).await);
        unwrap!(main.hv_battery.draw(&mut self.display2));
        unwrap!(self.display1.flush().await);
        unwrap!(self.display2.flush().await);
    }

    #[state(entry_action = "enter_main")]
    async fn main(&mut self, context: &mut LcdContext, main: &mut LcdMainState, event: &LcdEvent) -> Response<State> {
        info!("main got event: {:?}", event);
        let ret = match event {
            LcdEvent::PowerOff => Transition(State::init()),
            LcdEvent::Obd2Event(Obd2Event::BmsPid(bms_pid)) => {
                main.hv_battery.update_voltage(bms_pid.dc_voltage);
                main.hv_battery.update_min_temp(bms_pid.min_temp);
                main.hv_battery.update_max_temp(bms_pid.max_temp);
                main.hv_battery.update_percentage(bms_pid.soc);
                unwrap!(main.hv_battery.draw(&mut self.display2));
                Handled
            }
            _ => Handled,
        };

        unwrap!(self.display1.flush().await);
        unwrap!(self.display2.flush().await);

        ret
    }
}

impl LcdState {
    // The `on_transition` callback that will be called after every transition.
    fn on_transition(&mut self, source: &State, target: &State) {
        //info!("lcd transitioned from `{}` to `{}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &LcdEvent) {
        //info!("lcd dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
    }
}

#[embassy_executor::task]
pub async fn run(mut display1: Sh1122<10>, mut display2: Sh1122<1>) {
    let mut context = LcdContext {};
    let mut state =
        LcdState::new(display1, display2).uninitialized_state_machine().init_with_context(&mut context).await;

    loop {
        let event = EVENTS.receive().await;
        state.handle_with_context(&event, &mut context).await;
    }
}
