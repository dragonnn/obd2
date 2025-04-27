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
pub struct Icon<I> {
    position: Point,
    size: u32,

    redraw: bool,

    last_enabled: bool,

    _icon: core::marker::PhantomData<I>,
}

impl<I> Default for Icon<I> {
    fn default() -> Self {
        Self { size: 0, position: Point::zero(), redraw: true, last_enabled: true, _icon: core::marker::PhantomData }
    }
}

impl<I: embedded_iconoir::prelude::IconoirIcon> Icon<I> {
    pub fn new(position: Point) -> Self {
        Self { position, ..Default::default() }
    }

    pub fn enabled(&mut self, enabled: bool) {
        if self.last_enabled == enabled {
            return;
        }
        self.last_enabled = enabled;

        self.redraw = true;
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            if self.last_enabled {
                let icon = I::new(GrayColor::WHITE);
                let image = Image::new(&icon, self.position);
                image.draw(target)?;
                self.size = image.bounding_box().size.width;
            } else {
                if self.size != 0 {
                    let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                        .stroke_width(0)
                        .stroke_color(Gray4::BLACK)
                        .fill_color(Gray4::BLACK)
                        .build();

                    let bounding_box = Rectangle::new(self.position, Size::new(18, 18));
                    bounding_box.draw_styled(&style, target)?;
                }
            }

            self.redraw = false;
        }

        Ok(())
    }
}
