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

pub struct Fuel<D> {
    max_temp: f64,
    min_temp: f64,
    current_temp: f64,
    current_temp_percentage: f64,

    size: Size,
    position: Point,

    bars: i32,
    redraw: bool,

    _marker: core::marker::PhantomData<D>,
}

impl<D> Fuel<D>
where
    D: DrawTarget<Color = Gray4>,
{
    pub fn new(position: Point, size: Size, min: f64, max: f64, bars: i32) -> Self {
        Self {
            position,
            size,
            current_temp: 0.0,
            current_temp_percentage: 0.0,
            max_temp: max,
            min_temp: min,

            bars,
            redraw: true,
            _marker: core::marker::PhantomData::default(),
        }
    }

    pub fn update_temp(&mut self, temp: f64) {
        if self.current_temp != temp {
            self.current_temp = temp;
            self.current_temp_percentage = (temp - self.min_temp) / (self.max_temp - self.min_temp);
            if self.current_temp > self.max_temp {
                self.current_temp_percentage = 1.0;
            } else if self.current_temp < self.min_temp {
                self.current_temp_percentage = 0.0;
            }
            //esp_println::println!("current_temp_percentage: {:.1}", self.current_temp_percentage);
            self.redraw = true;
        }
    }

    pub fn draw(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let color = Gray4::new(4);

            let mut style = PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Gray4::WHITE)
                .fill_color(Gray4::BLACK)
                .build();
            let mut size = self.size;
            size.width /= 2;
            size.height -= self.size.width / 2;

            let mut area = Rectangle::new(self.position + Point::new(self.size.width as i32 / 2, 0), size);

            area.draw_styled(&style, target)?;
            let mut circle_bottom = Circle::with_center(
                self.position
                    + Point::new(self.size.width as i32 / 4, self.size.height as i32 - (self.size.width as i32 / 2))
                    + Point::new(size.width as i32 - 1, -2),
                self.size.width,
            );

            circle_bottom.draw_styled(&style, target)?;

            let circle =
                Circle::with_center(self.position + Point::new(self.size.width as i32 / 2 + 3, 4), self.size.width / 2);
            style.fill_color = Some(Gray4::BLACK);
            let mut circle_box = circle.bounding_box();

            circle_box.size.width += 2;
            circle_box.size.height -= 1;
            circle_box.top_left.x -= 1;
            circle_box.top_left.y -= 1;

            target.fill_solid(&circle_box, Gray4::BLACK)?;

            circle.draw_styled(&style, target)?;

            style.stroke_color = Some(color);

            area.size.height -= 4;
            area.size.width -= 2;

            area.top_left.x += 1;
            area.top_left.y += 3;

            let mut area_clipped = target.clipped(&area.bounding_box());

            area_clipped.fill_solid(&area, Gray4::BLACK)?;

            let mut size = self.size;
            size.width /= 2;
            size.height = ((self.size.height as f64 - 8.0) * self.current_temp_percentage).round() as u32;
            size.width -= 6;

            style.fill_color = Some(color);
            let mut position = self.position + Point::new(self.size.width as i32 / 2, 0);
            position.x += 3;
            position.y = self.position.y + (self.size.height - 6) as i32 - size.height as i32 + 2;
            //esp_println::println!("self.position.y:{} position.y: {}", self.position.y, position.y);

            let mut area_filled = Rectangle::new(position, size);

            area_filled.draw_styled(&style, target)?;

            area_filled.size.width = self.size.width;
            area_filled.top_left.x -= self.size.width as i32 / 2;
            area_filled.top_left.y -= 1;
            area_filled.size.height += 1;

            let mut area_filled = target.clipped(&area_filled);

            circle_bottom.diameter -= 6;
            circle_bottom.top_left.y += 3;
            circle_bottom.top_left.x += 3;

            circle_bottom.draw_styled(&style, &mut area_filled)?;

            let mut text: String<16> = String::new();

            write!(text, "{:.1}°C", self.current_temp).unwrap();

            let character_style = MonoTextStyle::new(&PROFONT_9_POINT, Gray4::WHITE);

            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Center).line_height(LineHeight::Percent(100)).build();

            let mut rotate_target = RotatedDrawTarget::new(target);

            let text_position = Point::new(20, 26);

            let text = Text::with_text_style(text.as_str(), text_position, character_style, text_style);
            let text_box = Rectangle::with_center(text_position - Point::new(1, 2), Size::new(42, 12));
            rotate_target.fill_solid(&text_box, Gray4::BLACK)?;
            text.draw(&mut rotate_target)?;

            self.redraw = false;
        }

        Ok(())
    }
}
