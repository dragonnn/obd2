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

#[derive(Default)]
pub struct Slider {
    percentage: f64,
    size: Size,
    position: Point,
    redraw: bool,
}

impl Slider {
    pub fn new(position: Point, size: Size) -> Self {
        Self { position, size, percentage: 0.0, redraw: true }
    }

    pub fn update_percentage(&mut self, percentage: f64) {
        if self.percentage != percentage {
            self.percentage = percentage;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(Gray4::WHITE)
                .fill_color(Gray4::BLACK)
                .build();
            Rectangle::new(self.position, self.size).draw_styled(&style, target)?;
            let mut bar_style = style;
            bar_style.stroke_color = Some(Gray4::new(0x01));
            bar_style.fill_color = Some(Gray4::new(0x01));

            self.redraw = false;
        }

        Ok(())
    }
}
