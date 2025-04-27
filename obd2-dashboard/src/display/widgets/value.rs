use core::fmt::Write;

use display_interface::DisplayError;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
        MonoFont, MonoTextStyle,
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

#[derive(Clone, Copy, Debug)]
pub struct Value {
    position: Point,

    value: f32,
    font: &'static MonoFont<'static>,
    unit: &'static str,
    precision: usize,

    redraw: bool,
    bounding_box: Option<Rectangle>,
}

impl Default for Value {
    fn default() -> Self {
        Self {
            position: Point::zero(),
            value: 0.0,
            font: &PROFONT_9_POINT,
            unit: "",
            precision: 0,
            redraw: true,
            bounding_box: None,
        }
    }
}

impl Value {
    pub fn new(position: Point, font: &'static MonoFont, unit: &'static str, precision: usize) -> Self {
        Self { position, value: 0.0, unit, redraw: true, bounding_box: None, precision, font }
    }

    pub fn update_value(&mut self, value: f32) {
        if self.value != value {
            self.value = value;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let mut text: String<16> = String::new();
            core::write!(text, "{:.1$}", self.value, self.precision).ok();
            core::write!(text, "{}", self.unit).ok();

            let character_style = MonoTextStyle::new(&self.font, Gray4::WHITE);

            // Create a new text style.
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Left).line_height(LineHeight::Percent(100)).build();

            let text = Text::with_text_style(text.as_str(), self.position, character_style, text_style);
            let new_bounding_box = text.bounding_box();
            if new_bounding_box.size.width > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0) {
                self.bounding_box = Some(new_bounding_box);
            }
            if let Some(bb) = self.bounding_box {
                bb.draw_styled(&PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build(), target)?;
            }

            text.draw(target)?;

            self.redraw = false;
        }

        Ok(())
    }
}
