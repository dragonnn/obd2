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
    clutch1_temp: f32,
    clutch2_temp: f32,

    redraw: bool,
    bounding_box: Option<Rectangle>,
    bounding_box2: Option<Rectangle>,
}

impl GearboxGear {
    pub fn new(position: Point) -> Self {
        Self { position, gear: "U", redraw: true, ..Default::default() }
    }

    pub fn update_gear(&mut self, gear: &'static str) {
        if self.gear != gear {
            self.gear = gear;
            self.redraw = true;
        }
    }

    pub fn update_clutch1_temp(&mut self, clutch1_temp: f32) {
        if self.clutch1_temp != clutch1_temp {
            self.clutch1_temp = clutch1_temp;
            self.redraw = true;
        }
    }

    pub fn update_clutch2_temp(&mut self, clutch2_temp: f32) {
        if self.clutch2_temp != clutch2_temp {
            self.clutch2_temp = clutch2_temp;
            self.redraw = true;
        }
    }

    pub fn force_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let mut text: String<16> = String::new();
            core::write!(text, "{}", self.gear).ok();

            let character_style = MonoTextStyle::new(&PROFONT_18_POINT, Gray4::WHITE);

            // Create a new text style.
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Center).line_height(LineHeight::Percent(100)).build();

            let text = Text::with_text_style(text.as_str(), self.position, character_style, text_style);
            let mut new_bounding_box = text.bounding_box();

            let mut text2: String<16> = String::new();
            core::write!(text2, "{:.0}°C\n{:.0}°C", self.clutch1_temp, self.clutch2_temp).ok();

            let character_style_small = MonoTextStyle::new(&PROFONT_9_POINT, Gray4::WHITE);
            let text2 = Text::with_text_style(
                text2.as_str(),
                self.position + Point::new(27, -6),
                character_style_small,
                text_style,
            );

            let new_bounding_box2 = text2.bounding_box();
            new_bounding_box.size.width += new_bounding_box2.size.width + 4;

            if new_bounding_box.size.width > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0) {
                self.bounding_box = Some(new_bounding_box);
            }

            if new_bounding_box2.size.width > self.bounding_box2.map(|bb| bb.size.width).unwrap_or(0) {
                self.bounding_box2 = Some(new_bounding_box2);
            }

            if let Some(bb) = self.bounding_box {
                bb.draw_styled(&PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build(), target)?;
            }

            if let Some(bb) = self.bounding_box2 {
                bb.draw_styled(&PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build(), target)?;
            }

            text.draw(target)?;
            text2.draw(target)?;

            self.redraw = false;
        }

        Ok(())
    }
}
