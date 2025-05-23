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
pub enum BatteryOrientation {
    #[default]
    VerticalTop,
    VerticalDown,
    HorizontalLeft,
    HorizontalRight,
}

#[derive(Default)]
pub struct Battery {
    min_temp: f32,
    max_temp: f32,
    voltage: f32,
    cell_voltage_deviation: f32,
    cell_voltage: f32,
    percentage: f32,
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
            min_temp: 0.0,
            max_temp: 0.0,
            voltage: 0.0,
            cell_voltage_deviation: 0.0,
            cell_voltage: 0.0,
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

    pub fn update_percentage(&mut self, percentage: f32) {
        if self.percentage != percentage {
            self.percentage = percentage;
            self.redraw = true;
        }
    }

    pub fn update_voltage(&mut self, voltage: f32) {
        if self.voltage != voltage {
            self.voltage = voltage;
            self.redraw = true;
        }
    }

    pub fn update_cell_voltage_deviation(&mut self, cell_voltage_deviation: f32) {
        if self.cell_voltage_deviation != cell_voltage_deviation {
            self.cell_voltage_deviation = cell_voltage_deviation;
            self.redraw = true;
        }
    }

    pub fn update_cell_voltage(&mut self, cell_voltage: f32) {
        if self.cell_voltage != cell_voltage {
            self.cell_voltage = cell_voltage;
            self.redraw = true;
        }
    }

    pub fn update_min_temp(&mut self, min_temp: f32) {
        if self.min_temp != min_temp {
            self.min_temp = min_temp;
            self.redraw = true;
        }
    }

    pub fn update_max_temp(&mut self, max_temp: f32) {
        if self.max_temp != max_temp {
            self.max_temp = max_temp;
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
            bar_style.stroke_color = Some(Gray4::new(0x02));
            bar_style.fill_color = Some(Gray4::new(0x02));
            match self.orientation {
                VerticalDown => {
                    size.height = ((size.height as f32 * self.percentage) / 100.0).round() as u32;
                    Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                }
                VerticalTop => {
                    size.height = ((size.height as f32 * self.percentage) / 100.0).round() as u32;
                    position.y += org_size.height as i32 - size.height as i32;
                    Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                }
                HorizontalRight => {
                    size.width = ((size.width as f32 * self.percentage) / 100.0).round() as u32;
                    Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                }
                HorizontalLeft => {
                    size.width = ((size.width as f32 * self.percentage) / 100.0).round() as u32;
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
                        let bar_size = ((org_size.width as i32 - self.bars * 2) as f32 / (self.bars + 1) as f32).floor()
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
                let mut text: String<32> = String::new();
                core::write!(text, "{:.1}%", self.percentage).ok();

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
                core::write!(text, "{:.1}V {:.2}±{:.0}", self.voltage, self.cell_voltage, self.cell_voltage_deviation)
                    .ok();

                Text::with_text_style(text.as_str(), text_position, character_style, text_style).draw(target)?;

                let mut text_position = org_position;
                text_position.x += 2;
                text_position.y += org_size.height as i32 / 2 / 2 + 2;
                text.clear();
                core::write!(text, "{:.0}/{:.0}°C", self.min_temp, self.max_temp).ok();

                Text::with_text_style(text.as_str(), text_position, character_style, text_style).draw(target)?;
            }

            self.redraw = false;
        }

        Ok(())
    }
}
