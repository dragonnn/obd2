use defmt::*;
use embassy_futures::select::{select, Either::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal};
use embassy_time::{Duration, Timer};
use embedded_graphics::geometry::{Point, Size};
use heapless::String;
use statig::prelude::*;

use crate::{
    debug::DEBUG_STRING_LEN,
    display::widgets::{Battery, BatteryOrientation, DebugScroll},
    event::Obd2Event,
    types::{Display1, Display2, Sh1122},
};

mod debug;
mod main;

use debug::LcdDebugState;
use main::LcdMainState;

pub static EVENTS: Channel<CriticalSectionRawMutex, LcdEvent, 16> = Channel::new();

pub struct LcdContext {}

#[derive(Format, PartialEq)]
pub enum LcdEvent {
    PowerOff,
    Main,
    Debug,
    Menu,
    DebugLine(String<DEBUG_STRING_LEN>),
    Obd2Event(Obd2Event),
}

#[derive()]
pub struct LcdState {
    display1: Display1,
    display2: Display2,
    display_on: bool,
    is_debug: bool,
}

impl LcdState {
    pub fn new(display1: Display1, display2: Display2) -> Self {
        Self { display1, display2, display_on: false, is_debug: false }
    }

    pub fn is_debug(&self) -> bool {
        self.is_debug
    }
}

#[state_machine(
    initial = "State::init()",
    state(derive()),
    superstate(derive()),
    on_transition = "Self::on_transition",
    on_dispatch = "Self::on_dispatch"
)]
impl LcdState {
    async fn display_on(&mut self) {
        if self.display_on {
            warn!("display already on");
            return;
        }
        info!("display on");
        let lock = crate::locks::SPI_BUS.lock().await;
        info!("display on got spi lock");

        self.display1.clear();
        self.display2.clear();
        unwrap!(self.display1.sleep(false).await);
        unwrap!(self.display2.sleep(false).await);
        self.display_on = true;
        Timer::after(Duration::from_millis(100)).await;
    }

    async fn display_off(&mut self) {
        if !self.display_on {
            return;
        }
        info!("display off");
        let lock = crate::locks::SPI_BUS.lock().await;
        info!("display off got spi lock");

        unwrap!(self.display1.sleep(true).await);
        unwrap!(self.display2.sleep(true).await);
        self.display_on = false;
    }

    #[superstate]
    async fn state_dispatch(&mut self, event: &LcdEvent) -> Response<State> {
        //let lock = crate::locks::SPI_BUS.lock().await;
        match event {
            LcdEvent::Main => Transition(State::main(LcdMainState::new())),
            LcdEvent::Debug => Transition(State::debug(LcdDebugState::new())),
            LcdEvent::PowerOff => Transition(State::init()),
            _ => Handled,
        }
    }

    #[action]
    async fn enter_init(&mut self) {
        self.display_off().await;
    }

    #[state(entry_action = "enter_init")]
    async fn init(&mut self, event: &LcdEvent) -> Response<State> {
        match event {
            LcdEvent::Main => Transition(State::main(LcdMainState::new())),
            _ => Handled,
        }
    }

    #[action]
    async fn enter_main(&mut self, main: &mut LcdMainState) {
        self.display_on().await;
        let lock = crate::locks::SPI_BUS.lock().await;
        self.display1.clear();
        self.display2.clear();
        warn!("enter_main");
        main.draw(&mut self.display1, &mut self.display2).await;
    }

    #[state(entry_action = "enter_main", superstate = "state_dispatch")]
    async fn main(&mut self, main: &mut LcdMainState, event: &LcdEvent) -> Response<State> {
        info!("lcd main got event: {:?}", event);
        let lock = crate::locks::SPI_BUS.lock().await;
        info!("lcd main got spi block");
        let ret = match event {
            LcdEvent::Obd2Event(Obd2Event::BmsPid(bms_pid)) => {
                main.update_bms_pid(bms_pid);
                main.draw(&mut self.display1, &mut self.display2).await;
                Handled
            }
            _ => Super,
        };

        ret
    }

    #[action]
    async fn enter_debug(&mut self, debug: &mut LcdDebugState) {
        let lock = crate::locks::SPI_BUS.lock().await;
        warn!("enter_debug");
        self.display_on().await;
        debug.draw(&mut self.display1, &mut self.display2).await;
    }

    #[state(entry_action = "enter_debug", superstate = "state_dispatch")]
    async fn debug(
        &mut self,
        context: &mut LcdContext,
        debug: &mut LcdDebugState,
        event: &LcdEvent,
    ) -> Response<State> {
        let lock = crate::locks::SPI_BUS.lock().await;
        match event {
            LcdEvent::DebugLine(line) => {
                debug.add_line(line);
                debug.draw(&mut self.display1, &mut self.display2).await;
                Handled
            }
            _ => Super,
        }
    }
}

impl LcdState {
    // The `on_transition` callback that will be called after every transition.
    fn on_transition(&mut self, source: &State, target: &State) {
        //info!("lcd transitioned from `{}` to `{}`", source, target);
        self.is_debug = false;
        match target {
            State::Debug { debug: _ } => self.is_debug = true,
            _ => {}
        }
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &LcdEvent) {
        //info!("lcd dispatching `{}` to `{}`", event, defmt::Debug2Format(&state));
    }
}

#[embassy_executor::task]
pub async fn run(mut display1: Display1, mut display2: Display2) {
    unwrap!(display1.init(None).await);
    unwrap!(display2.init(None).await);
    let mut context = LcdContext {};
    let mut state =
        LcdState::new(display1, display2).uninitialized_state_machine().init_with_context(&mut context).await;

    loop {
        match state.state() {
            State::Debug { debug: _ } => match select(EVENTS.receive(), crate::debug::receive()).await {
                First(event) => state.handle_with_context(&event, &mut context).await,
                Second(line) => state.handle_with_context(&LcdEvent::DebugLine(line), &mut context).await,
            },
            _ => {
                let event = EVENTS.receive().await;
                state.handle_with_context(&event, &mut context).await;
            }
        }
    }
}