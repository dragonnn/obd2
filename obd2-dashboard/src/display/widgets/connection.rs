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
pub struct Connection {
    position: Point,

    redraw: bool,

    last_send: bool,
    last_receive: bool,
}

impl Default for Connection {
    fn default() -> Self {
        Self { position: Point::zero(), redraw: true, last_send: false, last_receive: false }
    }
}

impl Connection {
    pub fn new(position: Point) -> Self {
        Self { position, ..Default::default() }
    }

    pub fn update_last_send(&mut self, last_send: bool) {
        if self.last_send == last_send {
            return;
        }
        self.last_send = last_send;
        self.redraw = true;
    }

    pub fn update_last_receive(&mut self, last_receive: bool) {
        if self.last_receive == last_receive {
            return;
        }
        self.last_receive = last_receive;
        self.redraw = true;
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let icon = embedded_iconoir::icons::size18px::connectivity::DataTransferBoth::new(Gray4::new(0x04));
            let image = Image::new(&icon, self.position);

            let icon_enabled = embedded_iconoir::icons::size18px::connectivity::DataTransferBoth::new(GrayColor::WHITE);
            let image_enabled = Image::new(&icon_enabled, self.position);

            image.draw(target)?;

            let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                .stroke_width(0)
                .stroke_color(Gray4::BLACK)
                .fill_color(Gray4::BLACK)
                .build();

            if self.last_receive {
                let bounding_box = Rectangle::new(self.position + Point::new(0, 0), Size::new(18 / 2, 18));
                let mut clipped = target.clipped(&bounding_box);
                image_enabled.draw(&mut clipped)?;
            }

            if self.last_send {
                let bounding_box = Rectangle::new(self.position + Point::new(18 / 2, 0), Size::new(18 / 2, 18));
                let mut clipped = target.clipped(&bounding_box);
                image_enabled.draw(&mut clipped)?;
            }

            self.redraw = false;
        }

        Ok(())
    }
}
