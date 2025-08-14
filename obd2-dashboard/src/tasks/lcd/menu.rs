use defmt::{info, unwrap};
use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
use embedded_iconoir::prelude::*;
use statig::{self, Outcome as Response, Outcome::Transition};

use super::State;
use crate::{
    display::widgets::DebugScroll,
    tasks::{
        buttons::{Action, Button},
        lcd::{
            ac::LcdAcState, debug::LcdDebugState, main::LcdMainState, obd2_pids::LcdObd2Pids,
            settings::LcdSettingsState,
        },
    },
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdMenuState {}

impl defmt::Format for LcdMenuState {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "LcdMenuState {{  }}");
    }
}

impl LcdMenuState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_button(&mut self, button: &Action) -> Option<Response<State>> {
        info!("menu button: {:?}", button);
        match button {
            Action::Pressed(Button::B4) => Some(Transition(State::main(LcdMainState::new()))),
            Action::Pressed(Button::B5) => Some(Transition(State::ac(LcdAcState::new()))),
            Action::Pressed(Button::B2) => Some(Transition(State::debug(LcdDebugState::new()))),
            Action::Pressed(Button::B1) => Some(Transition(State::obd2_pids(LcdObd2Pids::new()))),
            Action::Pressed(Button::B0) => Some(Transition(State::settings(LcdSettingsState::new()))),
            Action::Pressed(Button::B3) => {
                crate::hal::reset();
                None
            }
            _ => None,
        }
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        let icon = embedded_iconoir::icons::size48px::devices::Computer::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point::zero());
        unwrap!(image.draw(display1));
        let icon = embedded_iconoir::icons::size48px::weather::SnowFlake::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52, y: 0 });
        unwrap!(image.draw(display1));
        let icon = embedded_iconoir::icons::size48px::system::Settings::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 * 1, y: 0 });
        unwrap!(image.draw(display2));
        let icon = embedded_iconoir::icons::size48px::editor::List::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 * 2, y: 0 });
        unwrap!(image.draw(display2));
        let icon = embedded_iconoir::icons::size48px::development::CodeBrackets::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 * 3, y: 0 });
        unwrap!(image.draw(display2));
        let icon = embedded_iconoir::icons::size48px::actions::Restart::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 * 4, y: 0 });
        unwrap!(image.draw(display2));
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
