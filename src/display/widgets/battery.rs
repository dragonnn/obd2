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

pub enum BatteryOrientation {
    VerticalTop,
    VerticalDown,
    HorizontalLeft,
    HorizontalRight,
}

pub struct Battery {
    temp: f64,
    voltage: f64,
    percentage: f64,
    size: Size,
    position: Point,
    orientation: BatteryOrientation,
    cap: Option<Size>,
    bars: i32,
    inited: Option<(Point, Size)>,
    redraw: bool,
    text: bool,
}

impl Battery {
    pub fn new(
        position: Point,
        size: Size,
        orientation: BatteryOrientation,
        cap: Option<Size>,
        bars: i32,
        text: bool,
    ) -> Self {
        Self {
            position,
            size,
            orientation,
            percentage: 0.0,
            temp: 0.0,
            voltage: 0.0,
            cap,
            bars,
            inited: None,
            text,
            redraw: true,
            //D: DrawTarget<Color = Gray4>,_marker: core::marker::PhantomData::default(),
        }
    }

    fn cap_draw<D: DrawTarget<Color = Gray4>>(&self, target: &mut D) -> Result<(Point, Size), D::Error> {
        use BatteryOrientation::*;

        Ok(if let Some(cap) = self.cap {
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Gray4::WHITE)
                .fill_color(Gray4::WHITE)
                .build();
            let mut size = self.size;
            match self.orientation {
                VerticalDown => {
                    size.height -= cap.height;
                    let mut position = self.position;
                    position.x += (self.size.width / 2) as i32 - (cap.width / 2) as i32;
                    position.y = self.size.height as i32 - cap.height as i32;

                    Rectangle::new(position, cap).draw_styled(&style, target)?;
                    (self.position, size)
                }
                VerticalTop => {
                    size.height -= cap.height + 2;
                    let mut position = self.position;
                    position.x += (self.size.width / 2) as i32 - (cap.width / 2) as i32;
                    let mut bar_position = self.position;
                    bar_position.y += cap.height as i32;

                    Rectangle::new(position, cap).draw_styled(&style, target)?;
                    (bar_position, size)
                }
                HorizontalRight => {
                    size.width -= cap.width;
                    let mut position = self.position;
                    position.x += self.size.width as i32 - cap.width as i32 - 6;
                    position.y += (self.size.height / 2) as i32 - (cap.height / 2) as i32;

                    Rectangle::new(position, cap).draw_styled(&style, target)?;
                    let mut bar_position = self.position;
                    bar_position.x -= cap.width as i32;

                    (bar_position, size)
                }
                HorizontalLeft => {
                    size.width -= cap.width;
                    let mut position = self.position;
                    position.x = self.size.width as i32 - cap.width as i32;
                    position.y += (self.size.height / 2) as i32 - (cap.height / 2) as i32;

                    Rectangle::new(position, cap).draw_styled(&style, target)?;
                    (self.position, size)
                }
            }
        } else {
            (self.position, self.size)
        })
    }

    fn init_draw<D: DrawTarget<Color = Gray4>>(&self, target: &mut D) -> Result<(Point, Size), D::Error> {
        let style =
            PrimitiveStyleBuilder::new().stroke_width(2).stroke_color(Gray4::WHITE).fill_color(Gray4::BLACK).build();

        let (mut position, mut size) = self.cap_draw(target)?;

        Rectangle::new(position, size).draw_styled(&style, target)?;
        size.width -= 8;
        size.height -= 8;

        position.x += 4;
        position.y += 4;

        Ok((position, size))
    }

    pub fn update_percentage(&mut self, percentage: f64) {
        if self.percentage != percentage {
            self.percentage = percentage;
            self.redraw = true;
        }
    }

    pub fn update_voltage(&mut self, voltage: f64) {
        if self.voltage != voltage {
            self.voltage = voltage;
            self.redraw = true;
        }
    }

    pub fn update_temp(&mut self, temp: f64) {
        if self.temp != temp {
            self.temp = temp;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.inited.is_none() {
            self.inited = Some(self.init_draw(target)?);
        }

        if let Some((mut position, mut size)) = self.inited
            && self.redraw
        {
            use BatteryOrientation::*;
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Gray4::WHITE)
                .fill_color(Gray4::WHITE)
                .build();
            let mut style_black = style;
            style_black.fill_color = Some(Gray4::BLACK);
            style_black.stroke_color = Some(Gray4::BLACK);
            Rectangle::new(position, size).draw_styled(&style_black, target)?;
            let org_size = size;
            let org_position = position;
            let mut bar_style = style;
            bar_style.stroke_color = Some(Gray4::new(0x01));
            bar_style.fill_color = Some(Gray4::new(0x01));
            match self.orientation {
                VerticalDown => {
                    size.height = ((size.height as f64 * self.percentage) / 100.0).round() as u32;
                    Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                }
                VerticalTop => {
                    size.height = ((size.height as f64 * self.percentage) / 100.0).round() as u32;
                    position.y += org_size.height as i32 - size.height as i32;
                    Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                }
                HorizontalRight => {
                    size.width = ((size.width as f64 * self.percentage) / 100.0).round() as u32;
                    Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                }
                HorizontalLeft => {
                    size.width = ((size.width as f64 * self.percentage) / 100.0).round() as u32;
                    position.x += org_size.width as i32 - size.width as i32;
                    Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                }
            }
            if self.bars > 2 {
                match self.orientation {
                    VerticalDown | VerticalTop => {
                        let bar_size = org_size.height as i32 / self.bars;
                        for bar in 0..self.bars {
                            let mut bar_position = org_position;
                            bar_position.y += bar_size * (bar + 1) - (bar_size / 2 - 2) - (bar * 2);
                            Rectangle::new(bar_position, Size { width: size.width, height: 1 })
                                .draw_styled(&style_black, target)?;
                        }
                    }
                    HorizontalLeft | HorizontalRight => {
                        let bar_size = ((org_size.width as i32 - self.bars * 2) as f64 / (self.bars + 1) as f64).floor()
                            as i32
                            + self.bars * 2;

                        for bar in 0..(self.bars - 1) {
                            let bar_translate = Point::new((bar_size + 2) * bar - 2 + 1 + bar_size - 2, 0);

                            Rectangle::new(org_position + bar_translate, Size { width: 1, height: size.height })
                                .draw_styled(&style_black, target)?;
                        }
                    }
                }
            }

            if self.text {
                let mut text: String<16> = String::new();
                write!(text, "{:.1}%", self.percentage).unwrap();

                let character_style = MonoTextStyle::new(&PROFONT_14_POINT, Gray4::WHITE);

                // Create a new text style.
                let mut text_style =
                    TextStyleBuilder::new().alignment(Alignment::Center).line_height(LineHeight::Percent(100)).build();

                // Create a text at position (20, 30) and draw it using the previously defined style.
                let mut text_position = org_position;
                text_position.x += org_size.width as i32 / 2 / 2 + org_size.width as i32 / 2;
                text_position.y += org_size.height as i32 / 2 + 5;

                Text::with_text_style(text.as_str(), text_position, character_style, text_style).draw(target)?;

                let character_style = MonoTextStyle::new(&PROFONT_12_POINT, Gray4::WHITE);

                text_style.alignment = Alignment::Left;

                let mut text_position = org_position;
                text_position.x += 2;
                text_position.y += org_size.height as i32 / 2 / 2 + org_size.height as i32 / 2 + 6;
                text.clear();
                write!(text, "{:1}V", self.voltage).unwrap();

                Text::with_text_style(text.as_str(), text_position, character_style, text_style).draw(target)?;

                let mut text_position = org_position;
                text_position.x += 2;
                text_position.y += org_size.height as i32 / 2 / 2 + 6;
                text.clear();
                write!(text, "{:1}°C", self.temp).unwrap();

                Text::with_text_style(text.as_str(), text_position, character_style, text_style).draw(target)?;
            }

            self.redraw = false;
        }

        Ok(())
    }
}
