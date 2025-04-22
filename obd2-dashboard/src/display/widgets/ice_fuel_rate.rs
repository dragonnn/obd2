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
pub struct IceFuelRate {
    position: Point,

    ice_fuel_rate: f64,
    vehicle_speed: f64,

    redraw: bool,
    bounding_box: Option<Rectangle>,
}

impl IceFuelRate {
    pub fn new(position: Point) -> Self {
        Self { position, ice_fuel_rate: 0.0, vehicle_speed: 0.0, redraw: true, bounding_box: None }
    }

    pub fn update_ice_fuel_rate(&mut self, ice_fuel_rate: f64) {
        if self.ice_fuel_rate != ice_fuel_rate {
            self.ice_fuel_rate = ice_fuel_rate;
            self.redraw = true;
        }
    }

    pub fn update_vehicle_speed(&mut self, vehicle_speed: f64) {
        if self.vehicle_speed != vehicle_speed {
            self.vehicle_speed = vehicle_speed;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let mut text: String<16> = String::new();

            let mut fuel_per_100km = 0.0;
            if self.vehicle_speed > 0.0 {
                fuel_per_100km = self.ice_fuel_rate / self.vehicle_speed * 100.0;
            }
            write!(text, "{:.1}", fuel_per_100km).ok();

            let character_style = MonoTextStyle::new(&PROFONT_10_POINT, Gray4::WHITE);

            // Create a new text style.
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Left).line_height(LineHeight::Percent(100)).build();

            let draw_text = Text::with_text_style(text.as_str(), self.position, character_style, text_style);
            let new_bounding_box = draw_text.bounding_box();
            if new_bounding_box.size.width > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0) {
                self.bounding_box = Some(new_bounding_box);
            }
            if let Some(bb) = self.bounding_box {
                bb.draw_styled(&PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build(), target)?;
            }

            draw_text.draw(target)?;

            text.clear();
            write!(text, "l/100").ok();

            let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);

            // Create a new text style.
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Left).line_height(LineHeight::Percent(100)).build();
            let draw_text = Text::with_text_style(
                text.as_str(),
                new_bounding_box.top_left + Point::new(new_bounding_box.size.width as i32 + 4, 8),
                character_style,
                text_style,
            );

            let new_bounding_box = draw_text.bounding_box();
            new_bounding_box.draw_styled(&PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build(), target)?;

            draw_text.draw(target)?;

            self.redraw = false;
        }

        Ok(())
    }
}
