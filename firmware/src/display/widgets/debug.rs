use core::{fmt::Write, str::FromStr as _};

use defmt::info;
use display_interface::DisplayError;
use embassy_time::Instant;
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

use crate::debug::{DEBUG_CHANNEL_LEN, DEBUG_STRING_LEN};

#[derive(Default)]
pub struct DebugScroll {
    text_buffer: [heapless::String<DEBUG_STRING_LEN>; DEBUG_CHANNEL_LEN],
    redraw: bool,
}

impl DebugScroll {
    pub fn new() -> Self {
        Self { text_buffer: Default::default(), redraw: false }
    }

    pub fn add_line(&mut self, line: &str) {
        for i in 0..self.text_buffer.len() - 1 {
            self.text_buffer[i] = self.text_buffer[i + 1].clone();
        }
        self.text_buffer[self.text_buffer.len() - 1] = String::from_str(line).unwrap();
        self.redraw = true;
    }

    pub fn draw<D: DrawTarget<Color = Gray4>, D2: DrawTarget<Color = Gray4>>(
        &mut self,
        target: &mut D,
        target2: &mut D2,
    ) -> Result<(), ()> {
        if self.redraw {
            //let now = Instant::now();
            target.clear(Gray4::BLACK).map_err(|_| ())?;
            target2.clear(Gray4::BLACK).map_err(|_| ())?;
            let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Left).line_height(LineHeight::Percent(100)).build();
            let mut position = Point::new(0, 4);
            let mut text_buffer_chunks = self.text_buffer.chunks(DEBUG_CHANNEL_LEN / 2);
            for text in text_buffer_chunks.next().unwrap() {
                let text = Text::with_text_style(text, position, character_style, text_style);
                text.draw(target).map_err(|_| ())?;
                position += Point::new(0, 8);
            }

            let mut position = Point::new(0, 4);
            for text in text_buffer_chunks.next().unwrap() {
                let text = Text::with_text_style(text, position, character_style, text_style);

                text.draw(target2).map_err(|_| ())?;
                position += Point::new(0, 8);
            }

            //info!("draw took: {:?}ms", now.elapsed().as_millis());

            self.redraw = false;
        }
        Ok(())
    }
}
