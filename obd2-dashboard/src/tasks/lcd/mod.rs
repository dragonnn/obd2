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
mod menu;
mod obd2_pids;
mod settings;

use debug::LcdDebugState;
use main::LcdMainState;
use menu::LcdMenuState;
use obd2_pids::LcdObd2Pids;
use settings::LcdSettingsState;

use super::{
    buttons::{Action, Button},
    obd2::Obd2Debug,
};

pub static EVENTS: Channel<CriticalSectionRawMutex, LcdEvent, 128> = Channel::new();
pub use obd2_pids::obd2_debug_pids_enabled;

pub struct LcdContext {}

#[derive(Format, PartialEq, Clone)]
pub enum LcdEvent {
    PowerOff,
    Main,
    Debug,
    Menu,
    Render,
    DebugLine(String<DEBUG_STRING_LEN>),
    Obd2Event(Obd2Event),
    Obd2Debug(Obd2Debug),
    Button(crate::tasks::buttons::Action),
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
        debug!("display on");
        let lock = crate::locks::SPI_BUS.lock().await;
        info!("display on got spi lock");

        self.display1.clear();
        self.display2.clear();
        self.display1.flush().await.ok();
        self.display2.flush().await.ok();
        unwrap!(self.display1.sleep(false).await);
        unwrap!(self.display2.sleep(false).await);
        crate::tasks::buttons::init();
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

        self.display1.clear();
        self.display2.clear();
        self.display1.flush().await.ok();
        self.display2.flush().await.ok();
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
            LcdEvent::Button(Action::Pressed(pressed)) => {
                if *pressed != Button::B3 {
                    Transition(State::menu(LcdMenuState::new()))
                } else {
                    Handled
                }
            }
            LcdEvent::Menu => Transition(State::menu(LcdMenuState::new())),
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
        let lock = crate::locks::SPI_BUS.lock().await;
        let ret = match event {
            LcdEvent::Obd2Event(obd2_event) => {
                main.handle_obd2_event(obd2_event);
                Handled
            }
            LcdEvent::Render => {
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
        self.display1.clear();
        self.display2.clear();
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

    #[action]
    async fn enter_menu(&mut self, menu: &mut LcdMenuState) {
        let lock = crate::locks::SPI_BUS.lock().await;
        warn!("enter_debug");
        self.display_on().await;
        self.display1.clear();
        self.display2.clear();
        menu.draw(&mut self.display1, &mut self.display2).await;
    }

    #[state(entry_action = "enter_menu", superstate = "state_dispatch")]
    async fn menu(&mut self, context: &mut LcdContext, menu: &mut LcdMenuState, event: &LcdEvent) -> Response<State> {
        let lock = crate::locks::SPI_BUS.lock().await;
        match event {
            LcdEvent::Button(action) => {
                if let Some(transition) = menu.handle_button(action) {
                    return transition;
                }
                menu.draw(&mut self.display1, &mut self.display2).await;
                Handled
            }
            _ => Super,
        }
    }

    #[action]
    async fn enter_obd2_pids(&mut self, obd2_pids: &mut LcdObd2Pids) {
        let lock = crate::locks::SPI_BUS.lock().await;
        warn!("enter_debug");
        self.display_on().await;
        self.display1.clear();
        self.display2.clear();
        obd2_pids.draw(&mut self.display1, &mut self.display2).await;
    }

    #[state(entry_action = "enter_obd2_pids", superstate = "state_dispatch")]
    async fn obd2_pids(
        &mut self,
        context: &mut LcdContext,
        obd2_pids: &mut LcdObd2Pids,
        event: &LcdEvent,
    ) -> Response<State> {
        let lock = crate::locks::SPI_BUS.lock().await;
        match event {
            LcdEvent::Obd2Debug(obd2_debug) => {
                obd2_pids.handle_obd2_debug(obd2_debug);
                obd2_pids.draw(&mut self.display1, &mut self.display2).await;
                Handled
            }
            _ => Super,
        }
    }

    #[action]
    async fn enter_settings(&mut self, settings: &mut LcdSettingsState) {
        let lock = crate::locks::SPI_BUS.lock().await;
        warn!("enter_debug");
        self.display_on().await;
        self.display1.clear();
        self.display2.clear();
        settings.draw(&mut self.display1, &mut self.display2).await;
    }

    #[state(entry_action = "enter_settings", superstate = "state_dispatch")]
    async fn settings(
        &mut self,
        context: &mut LcdContext,
        settings: &mut LcdSettingsState,
        event: &LcdEvent,
    ) -> Response<State> {
        let lock = crate::locks::SPI_BUS.lock().await;
        match event {
            LcdEvent::Button(action) => {
                if let Some(transition) = settings.handle_button(action) {
                    return transition;
                }
                settings.draw(&mut self.display1, &mut self.display2).await;
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
    info!("lcd init start");
    unwrap!(display1.init(None).await);
    unwrap!(display2.init(None).await);

    display1.set_contrast(50).await.ok();
    display2.set_contrast(50).await.ok();

    display1.clear();
    display2.clear();

    display1.flush().await.ok();
    display2.flush().await.ok();
    info!("lcd init end");
    let mut context = LcdContext {};
    let mut state =
        LcdState::new(display1, display2).uninitialized_state_machine().init_with_context(&mut context).await;
    info!("lcd state machine initialized");
    let mut render_ticker = embassy_time::Ticker::every(Duration::from_millis(1000 / 12));
    loop {
        match state.state() {
            State::Debug { debug: _ } => match select(EVENTS.receive(), crate::debug::receive()).await {
                First(event) => state.handle_with_context(&event, &mut context).await,
                Second(line) => state.handle_with_context(&LcdEvent::DebugLine(line), &mut context).await,
            },
            _ => {
                let event = match select(EVENTS.receive(), render_ticker.next()).await {
                    First(event) => {
                        //render_ticker.reset();
                        event
                    }
                    Second(_) => LcdEvent::Render,
                };
                state.handle_with_context(&event, &mut context).await;
            }
        }
    }
}
