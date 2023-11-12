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

pub struct Battery12V<D> {
    voltage: f64,
    position: Point,
    redraw: bool,
    inited: bool,
    _marker: core::marker::PhantomData<D>,
}

impl<D> Battery12V<D>
where
    D: DrawTarget<Color = Gray4>,
{
    pub fn new(position: Point) -> Self {
        Self { position, voltage: 12.5, redraw: true, inited: false, _marker: core::marker::PhantomData::default() }
    }

    pub fn update_voltage(&mut self, voltage: f64) {
        if self.voltage != voltage {
            self.voltage = voltage;
            self.redraw = true;
        }
    }

    pub fn init(&mut self, target: &mut D) -> Result<(), D::Error> {
        let cap_height = 7;
        let cap_width = 10;
        let main_width = 32;
        let main_height = 32;
        let mut style =
            PrimitiveStyleBuilder::new().stroke_width(2).stroke_color(Gray4::WHITE).fill_color(Gray4::BLACK).build();
        let main_rectangle = Rectangle::new(
            self.position + Point::new(0, cap_height),
            Size::new(main_width, main_height - cap_height as u32),
        );

        main_rectangle.draw_styled(&style, target)?;

        let cap1_rectangle = Rectangle::with_center(
            self.position + Point::new(main_width as i32 / 2 / 2, cap_height / 2),
            Size::new(cap_width, cap_height as u32),
        );
        style.stroke_color = None;
        style.fill_color = Some(Gray4::WHITE);

        cap1_rectangle.draw_styled(&style, target)?;

        let cap2_rectangle =
            cap1_rectangle.translate(Point::new(main_width as i32 / 2 / 2 + cap_width as i32 / 2 + 1, 0));

        cap2_rectangle.draw_styled(&style, target)?;

        let minus_rectangle = Rectangle::with_center(
            Point::new(cap1_rectangle.center().x, main_rectangle.top_left.y + main_width as i32 / 2 / 2 - 2),
            Size::new(cap_width - 2, 3),
        );

        minus_rectangle.draw_styled(&style, target)?;

        let plus1_rectangle = Rectangle::with_center(
            Point::new(cap2_rectangle.center().x + 1, main_rectangle.top_left.y + main_width as i32 / 2 / 2 - 2),
            Size::new(cap_width - 1, 3),
        );

        let plus2_rectangle = Rectangle::with_center(
            Point::new(cap2_rectangle.center().x + 1, main_rectangle.top_left.y + main_width as i32 / 2 / 2 - 2),
            Size::new(3, cap_width - 1),
        );

        plus1_rectangle.draw_styled(&style, target)?;
        plus2_rectangle.draw_styled(&style, target)?;

        self.inited = true;
        Ok(())
    }

    pub fn draw(&mut self, target: &mut D) -> Result<(), D::Error> {
        if !self.inited {
            self.init(target)?;
        }

        if self.redraw {
            let style = PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build();

            let main_width = 32;
            let main_height = 30;

            let mut text: String<16> = String::new();
            write!(text, "{:.1}V", self.voltage).unwrap();

            let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);

            // Create a new text style.
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Center).line_height(LineHeight::Percent(100)).build();

            let text = Text::with_text_style(
                text.as_str(),
                self.position + Point::new(main_width / 2, main_height - 4),
                character_style,
                text_style,
            );

            let mut rectangle = text.bounding_box();
            rectangle.top_left.x = self.position.x + 1;
            rectangle.size.width = main_width as u32 - 2;

            rectangle.draw_styled(&style, target)?;

            text.draw(target)?;
            self.redraw = false;
        }

        Ok(())
    }
}
