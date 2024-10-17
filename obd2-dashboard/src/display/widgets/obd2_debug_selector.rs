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

use crate::{
    debug::{DEBUG_CHANNEL_LEN, DEBUG_STRING_LEN},
    tasks::obd2::Obd2Debug,
};

#[derive(Default)]
pub struct Obd2DebugSelector {
    pids: heapless::FnvIndexMap<&'static str, Obd2Debug, 16>,
    redraw: bool,
}

impl Obd2DebugSelector {
    pub fn new() -> Self {
        Self { pids: Default::default(), redraw: false }
    }

    pub fn handle_obd2_debug(&mut self, debug: &Obd2Debug) {
        if let Some(pid) = self.pids.get_mut(debug.type_id) {
            pid.data = debug.data.clone();
        } else {
            self.pids.insert(debug.type_id, debug.clone()).ok();
        }
        self.redraw = true;
    }

    pub fn draw<D: DrawTarget<Color = Gray4>, D2: DrawTarget<Color = Gray4>>(
        &mut self,
        target: &mut D,
        target2: &mut D2,
    ) -> Result<(), ()> {
        if self.redraw {
            target.clear(Gray4::BLACK).map_err(|_| ())?;
            target2.clear(Gray4::BLACK).map_err(|_| ())?;
            let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);
            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Left).line_height(LineHeight::Percent(100)).build();
            let mut position = Point::new(0, 6);

            for (pid, buffer) in &self.pids {
                let mut text = String::<64>::new();
                if let Some(data) = &buffer.data {
                    write!(text, "{}: {:x?}", pid, data).ok();
                } else {
                    write!(text, "{}: None", pid).ok();
                }
                let text = Text::with_text_style(&text, position, character_style, text_style);
                text.draw(target).map_err(|_| ())?;
                position += Point::new(0, 8);
            }

            self.redraw = false;
        }
        Ok(())
    }
}
