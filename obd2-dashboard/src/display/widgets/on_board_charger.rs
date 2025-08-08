use core::fmt::Write;

use display_interface::DisplayError;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
        MonoTextStyle,
    },
    pixelcolor::Gray4,
    prelude::*,
    primitives::*,
    text::{Alignment, LineHeight, Text, TextStyleBuilder},
};
use heapless::String;
use num_traits::float::FloatCore;
use profont::*;

use crate::display::RotatedDrawTarget;

#[derive(Clone, Copy, Debug, Default)]
pub struct OnBoardCharger {
    position: Point,

    gear: &'static str,
    redraw: bool,
    bounding_box: Option<Rectangle>,
}

impl OnBoardCharger {
    pub fn new(position: Point) -> Self {
        Self { position, redraw: true, ..Default::default() }
    }

    pub fn force_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            self.redraw = false;
        }

        Ok(())
    }
}
