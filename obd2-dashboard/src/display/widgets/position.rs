use alloc::{borrow::Cow, string::ToString as _};
use core::fmt::Write;

use display_interface::DisplayError;
use embedded_graphics::{
    image::Image,
    pixelcolor::Gray4,
    prelude::*,
    primitives::{Rectangle, StyledDrawable as _},
};
use embedded_iconoir::prelude::IconoirNewIcon as _;

#[derive(Clone, Debug)]
pub struct Position {
    position: Point,

    redraw: bool,

    last_position: bool,
}

impl Default for Position {
    fn default() -> Self {
        Self { position: Point::zero(), redraw: true, last_position: false }
    }
}

impl Position {
    pub fn new(position: Point) -> Self {
        Self { position, ..Default::default() }
    }

    pub fn update_last_position(&mut self, last_position: bool) {
        if self.last_position == last_position {
            return;
        }
        self.last_position = last_position;

        self.redraw = true;
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let color = if self.last_position { GrayColor::WHITE } else { Gray4::new(0x04) };
            let icon = embedded_iconoir::icons::size18px::maps::Position::new(color);
            let image = Image::new(&icon, self.position);
            image.draw(target)?;

            self.redraw = false;
        }

        Ok(())
    }
}
