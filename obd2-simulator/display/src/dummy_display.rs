use embedded_graphics::{
    Pixel,
    pixelcolor::Gray4,
    prelude::{Dimensions, DrawTarget},
    primitives::Rectangle,
};

pub struct DummyDisplay;

impl DummyDisplay {
    pub fn new() -> Self {
        DummyDisplay
    }

    pub async fn init(&mut self, conf: Option<i32>) -> Result<(), ()> {
        Ok(())
    }

    pub fn clear(&mut self) {}

    pub async fn flush(&mut self) -> Result<(), ()> {
        Ok(())
    }

    pub async fn sleep(&mut self, sleep: bool) -> Result<(), ()> {
        Ok(())
    }

    pub async fn set_contrast(&mut self, contrast: u8) -> Result<(), ()> {
        Ok(())
    }

    pub fn get_contrast(&self) -> u8 {
        10
    }
}

#[derive(defmt::Format)]
pub struct DisplayError;

impl DrawTarget for DummyDisplay {
    type Color = Gray4;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        Ok(())
    }
}

impl Dimensions for DummyDisplay {
    fn bounding_box(&self) -> Rectangle {
        todo!()
    }
}
