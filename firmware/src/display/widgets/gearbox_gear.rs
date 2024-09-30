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
pub struct GearboxGear {
    position: Point,

    gear: &'static str,

    redraw: bool,
    bounding_box: Option<Rectangle>,
}

impl GearboxGear {
    pub fn new(position: Point) -> Self {
        Self { position, gear: "U", redraw: true, bounding_box: None }
    }

    pub fn update_gear(&mut self, gear: &'static str) {
        if self.gear != gear {
            self.gear = gear;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let mut text: String<16> = String::new();
            write!(text, "{}", self.gear).ok();

            let character_style = MonoTextStyle::new(&PROFONT_18_POINT, Gray4::WHITE);

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
