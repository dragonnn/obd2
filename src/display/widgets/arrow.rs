use core::fmt::Write;

use display_interface::DisplayError;
use embedded_graphics::{
    draw_target::Clipped,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    Forward,
    Reverse,
}

pub struct Arrow<D> {
    size: Size,
    position: Point,
    arrow_width: u32,
    offset: f64,
    old_offest: i32,
    force_update: bool,
    color: u8,
    speed: f64,
    direction: ArrowDirection,
    _marker: core::marker::PhantomData<D>,
}

impl<D> Arrow<D>
where
    D: DrawTarget<Color = Gray4>,
{
    pub fn new(position: Point, size: Size, arrow_width: u32, direction: ArrowDirection) -> Self {
        Self {
            position,
            size,
            arrow_width,
            old_offest: i32::MAX,
            force_update: true,
            color: 0,
            offset: 0.0,
            speed: 0.0,
            direction,
            _marker: core::marker::PhantomData::default(),
        }
    }

    pub fn update_direction(&mut self, direction: ArrowDirection) {
        if self.direction != direction {
            self.direction = direction;
            self.force_update = true;
        }
    }

    pub fn update_speed(&mut self, speed: f64) {
        let old_speed = self.speed;
        if speed > 0.0 {
            self.speed = speed / 100.0 * 3.5 + 1.0;
        } else {
            self.speed = 0.0;
        }
        self.color = (speed / 100.0 * 16.0).round() as u8;
        if speed != 0.0 && self.color == 0 {
            self.color = 1;
        }
        if speed != old_speed {
            self.force_update = true;
        }
        if self.color > 15 {
            self.color = 15;
        }
        if self.speed > 4.5 {
            self.speed = 4.5;
        }
        esp_println::println!("color: {}, speed: {}", self.color, self.speed);
    }

    pub fn draw(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.offset >= self.arrow_width as f64 {
            self.offset = self.speed;
        }

        let new_offest = self.offset.ceil() as i32;
        if new_offest != self.old_offest || self.force_update {
            let mut size = self.size;
            size.height += 1;
            let style_black = PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Gray4::BLACK)
                .fill_color(Gray4::BLACK)
                .build();
            let area = Rectangle::new(self.position, size);
            area.draw_styled(&style_black, target)?;
            let mut area = target.clipped(&area);

            let style = PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Gray4::new(self.color))
                .fill_color(Gray4::new(self.color))
                .build();

            let triangle_offset = match self.direction {
                ArrowDirection::Forward => -1,
                ArrowDirection::Reverse => 1,
            };

            let triangle = Triangle::new(
                Point::new(self.position.x, self.position.y),
                Point::new(
                    self.position.x - (self.arrow_width as i32 - 6) * triangle_offset,
                    self.position.y + self.size.height as i32 / 2,
                ),
                Point::new(self.position.x, self.position.y + self.size.height as i32),
            )
            .translate(Point::new(-(triangle_offset * new_offest), 0));
            if self.direction == ArrowDirection::Forward {
                for a in (-1..(self.size.width / self.arrow_width) as i32 + 2).rev() {
                    self.draw_triangle(&mut area, &style, &style_black, triangle, triangle_offset, a)?;
                }
            } else {
                for a in 0..(self.size.width / self.arrow_width) as i32 + 4 {
                    self.draw_triangle(&mut area, &style, &style_black, triangle, triangle_offset, a)?;
                }
            }
            self.old_offest = new_offest;
            self.force_update = false;
        }
        self.offset += self.speed;

        Ok(())
    }

    fn draw_triangle(
        &mut self,
        area: &mut Clipped<D>,
        style: &PrimitiveStyle<Gray4>,
        style_black: &PrimitiveStyle<Gray4>,
        triangle: Triangle,
        triangle_offset: i32,
        a: i32,
    ) -> Result<(), D::Error> {
        let triangle_a = triangle.translate(Point::new((self.arrow_width as f64 / 1.2).ceil() as i32 * a, 0));
        triangle_a.draw_styled(style, area)?;
        triangle_a
            .translate(Point::new(triangle_offset * (self.arrow_width as i32 / 3), 0))
            .draw_styled(style_black, area)?;
        Ok(())
    }
}
