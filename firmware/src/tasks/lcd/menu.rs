use defmt::unwrap;
use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
use embedded_iconoir::prelude::*;

use crate::{
    display::widgets::DebugScroll,
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdMenuState {}

impl LcdMenuState {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        display1.clear();
        display2.clear();
        let icon = embedded_iconoir::icons::size48px::devices::Computer::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point::zero());
        image.draw(display1).unwrap();
        let icon = embedded_iconoir::icons::size48px::weather::SnowFlake::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52, y: 0 });
        image.draw(display1).unwrap();
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
