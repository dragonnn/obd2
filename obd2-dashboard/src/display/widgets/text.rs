use alloc::{borrow::Cow, string::ToString as _};
use core::fmt::Write;

use display_interface::DisplayError;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
        MonoFont, MonoTextStyle,
    },
    pixelcolor::Gray4,
    prelude::*,
    primitives::*,
    text::{Alignment, LineHeight, Text as EmbeddedText, TextStyleBuilder},
};
use heapless::String;
use num_traits::float::FloatCore;
use profont::*;

use crate::display::RotatedDrawTarget;

#[derive(Clone, Debug)]
pub struct Text {
    position: Point,

    font: &'static MonoFont<'static>,
    text: Cow<'static, str>,
    selected: bool,

    redraw: bool,
    bounding_box: Option<Rectangle>,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            position: Point::zero(),
            font: &PROFONT_9_POINT,
            redraw: true,
            selected: false,
            bounding_box: None,
            text: Cow::Borrowed(""),
        }
    }
}

impl Text {
    pub fn new(position: Point, font: &'static MonoFont, initial_str: Option<&'static str>) -> Self {
        let mut ret =
            Self { position, redraw: true, bounding_box: None, font, text: Cow::Borrowed(""), selected: false };
        if let Some(str) = initial_str {
            ret.update_str(str);
        }
        ret
    }

    pub fn update_str(&mut self, str: &'static str) {
        if self.text != str {
            self.text = Cow::Borrowed(str);
            self.redraw = true;
        }
    }

    pub fn update_string(&mut self, str: &str) {
        if self.text != str {
            self.text = Cow::Owned(str.to_string());
            self.redraw = true;
        }
    }

    pub fn update_selected(&mut self, selected: bool) {
        if self.selected != selected {
            self.selected = selected;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let character_style = MonoTextStyle::new(&self.font, Gray4::WHITE);

            // Create a new text style.
            let mut text_style =
                TextStyleBuilder::new().alignment(Alignment::Left).line_height(LineHeight::Percent(100)).build();

            let mut text =
                EmbeddedText::with_text_style(self.text.as_ref(), self.position, character_style, text_style);
            if self.selected {
                text.character_style.background_color = Some(Gray4::WHITE);
                text.character_style.text_color = Some(Gray4::BLACK);
            }
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
