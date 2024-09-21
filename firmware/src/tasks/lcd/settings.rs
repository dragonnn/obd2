use core::sync::atomic::AtomicBool;

use defmt::*;
use embedded_graphics::prelude::*;
use statig::Response::{self, Handled, Transition};

use super::State;
use crate::{
    display::widgets::{DebugScroll, Obd2DebugSelector, Slider, Text},
    tasks::{buttons::Action, lcd::menu::LcdMenuState, obd2::Obd2Debug},
    types::{Display1, Display2},
};

#[derive(Format, PartialEq, Eq, Clone, Copy, Default)]
pub enum LcdSettingsEdit {
    #[default]
    Contrast,
}

#[derive(Default)]
pub struct LcdSettingsState {
    contrast: u8,
    contrast_text: Text,
    contrast_slider: Slider,
    current_contrast: u8,

    init: bool,
    edit: LcdSettingsEdit,
}

impl LcdSettingsState {
    pub fn new() -> Self {
        error!("LcdSettingsState::new()");
        let mut ret = Self {
            contrast: 128,
            current_contrast: 128,
            contrast_slider: Slider::new(Point::new(128, 0), Size::new(128, 10)),
            contrast_text: Text::new(
                Point::new(30, 7),
                &embedded_graphics::mono_font::ascii::FONT_6X10,
                Some("Contrast: "),
            ),
            init: true,
            edit: LcdSettingsEdit::Contrast,
        };
        ret.contrast_text.update_selected(true);
        ret
    }

    pub fn handle_button(&mut self, button: &Action) -> Option<Response<State>> {
        use crate::tasks::buttons::{Action::*, Button::*};
        info!("settings button: {:?}", button);
        match button {
            Pressed(B0) => {
                self.edit = LcdSettingsEdit::Contrast;
                None
            }
            Pressed(B1) => {
                self.edit = LcdSettingsEdit::Contrast;
                None
            }
            Pressed(B4) => {
                return Some(Transition(State::menu(LcdMenuState::new())));
                None
            }
            Pressed(_) => {
                self.handle_edit_contrast(button);
                None
            }
            _ => Some(0),
        };
        None
    }

    fn handle_edit_contrast(&mut self, button: &Action) {
        use crate::tasks::buttons::{Action::*, Button::*};
        match button {
            Pressed(B6) => {
                self.contrast = self.contrast.saturating_sub(5);
                self.contrast_slider.update_percentage(self.contrast as f64 / 255.0 * 100.0);
            }
            Pressed(B7) => {
                self.contrast = self.contrast.saturating_add(5);
                self.contrast_slider.update_percentage(self.contrast as f64 / 255.0 * 100.0);
            }
            _ => {}
        }
        info!("contrast: {}", self.contrast);
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        if self.init {
            self.contrast = display1.get_contrast();
            self.init = false;
        }
        if self.current_contrast != self.contrast {
            display1.set_contrast(self.contrast).await;
            display2.set_contrast(self.contrast).await;
            self.current_contrast = self.contrast;
        }
        self.contrast_text.draw(display1).ok();
        self.contrast_slider.draw(display1).ok();
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
