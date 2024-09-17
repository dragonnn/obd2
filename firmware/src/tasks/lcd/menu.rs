use defmt::{info, unwrap};
use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
use embedded_iconoir::prelude::*;
use statig::Response::{self, Transition};

use super::State;
use crate::{
    display::widgets::DebugScroll,
    tasks::{
        buttons::{Action, Button},
        lcd::{debug::LcdDebugState, main::LcdMainState, obd2_pids::LcdObd2Pids},
    },
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdMenuState {}

impl LcdMenuState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_button(&mut self, button: &Action) -> Option<Response<State>> {
        info!("menu button: {:?}", button);
        match button {
            Action::Pressed(Button::B4) => Some(Transition(State::main(LcdMainState::new()))),
            Action::Pressed(Button::B2) => Some(Transition(State::debug(LcdDebugState::new()))),
            Action::Pressed(Button::B1) => Some(Transition(State::obd2_pids(LcdObd2Pids::new()))),
            Action::Pressed(Button::B3) => {
                esp_hal::reset::software_reset();
                None
            }
            _ => None,
        }
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        let icon = embedded_iconoir::icons::size48px::devices::Computer::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point::zero());
        image.draw(display1).unwrap();
        let icon = embedded_iconoir::icons::size48px::weather::SnowFlake::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52, y: 0 });
        image.draw(display1).unwrap();
        let icon = embedded_iconoir::icons::size48px::editor::List::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 * 2, y: 0 });
        image.draw(display2).unwrap();
        let icon = embedded_iconoir::icons::size48px::development::CodeBrackets::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 * 3, y: 0 });
        image.draw(display2).unwrap();
        let icon = embedded_iconoir::icons::size48px::actions::Restart::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 * 4, y: 0 });
        image.draw(display2).unwrap();
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
