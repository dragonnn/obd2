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
pub struct Power {
    position: Point,

    power: f64,
    current: f64,

    redraw: bool,
    bounding_box: Option<Rectangle>,
}

impl Power {
    pub fn new(position: Point) -> Self {
        Self { position, power: 0.0, current: 0.0, redraw: true, bounding_box: None }
    }

    pub fn update_power(&mut self, power: f64) {
        if self.power != power {
            self.power = power;
            self.redraw = true;
        }
    }

    pub fn update_current(&mut self, current: f64) {
        if self.current != current {
            self.current = current;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let mut text: String<16> = String::new();
            write!(text, "{:.2}kW", self.power / 1000.0).ok();

            let character_style = MonoTextStyle::new(&PROFONT_12_POINT, Gray4::WHITE);

            // Create a new text style.
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Center).line_height(LineHeight::Percent(100)).build();

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
