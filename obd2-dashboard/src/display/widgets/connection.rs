use alloc::{borrow::Cow, string::ToString as _};
use core::fmt::Write;

use display_interface::DisplayError;
use embedded_graphics::{image::Image, pixelcolor::Gray4, prelude::*};
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
            self.redraw = false;

            match (self.last_receive, self.last_send) {
                (true, true) => {
                    let icon = embedded_iconoir::icons::size24px::connectivity::DataTransferBoth::new(GrayColor::WHITE);
                    let image = Image::new(&icon, Point::zero());
                    image.draw(target)?;
                }
                (true, false) => {
                    let icon = embedded_iconoir::icons::size24px::connectivity::DataTransferDown::new(GrayColor::WHITE);
                    let image = Image::new(&icon, Point::zero());
                    image.draw(target)?;
                }
                (false, true) => {
                    let icon = embedded_iconoir::icons::size24px::connectivity::DataTransferUp::new(GrayColor::WHITE);
                    let image = Image::new(&icon, Point::zero());
                    image.draw(target)?;
                }
                (false, false) => {
                    let icon =
                        embedded_iconoir::icons::size24px::connectivity::DataTransferWarning::new(GrayColor::WHITE);
                    let image = Image::new(&icon, Point::zero());
                    image.draw(target)?;
                }
            };
        }

        Ok(())
    }
}
