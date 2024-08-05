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
        let icon = embedded_iconoir::icons::size48px::docs::Journal::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52, y: 0 });
        image.draw(display1).unwrap();
        let icon = embedded_iconoir::icons::size48px::docs::PageSearch::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 + 52, y: 0 });
        image.draw(display1).unwrap();
        let icon = embedded_iconoir::icons::size48px::docs::PrivacyPolicy::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 + 52 + 52, y: 0 });
        image.draw(display1).unwrap();
        let icon = embedded_iconoir::icons::size48px::weather::SnowFlake::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 52 + 52 + 52 + 52, y: 0 });
        image.draw(display1).unwrap();

        let icon = embedded_iconoir::icons::size16px::activities::Book::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 0, y: 0 });
        image.draw(display2).unwrap();

        let icon = embedded_iconoir::icons::size16px::weather::Cloud::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 18, y: 0 });
        image.draw(display2).unwrap();

        let icon = embedded_iconoir::icons::size16px::weather::CloudSunny::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 18 + 18, y: 0 });
        image.draw(display2).unwrap();

        let icon = embedded_iconoir::icons::size16px::weather::Rain::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 18 + 18 + 18, y: 0 });
        image.draw(display2).unwrap();

        let icon = embedded_iconoir::icons::size16px::weather::HeavyRain::new(GrayColor::WHITE);
        let image = Image::new(&icon, Point { x: 18 + 18 + 18 + 18, y: 0 });
        image.draw(display2).unwrap();
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
