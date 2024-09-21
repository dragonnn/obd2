use embedded_graphics::{prelude::*, primitives::Rectangle};

pub mod widgets;

#[derive(Debug)]
pub struct RotatedDrawTarget<'a, T>
where
    T: DrawTarget,
{
    parent: &'a mut T,
}

impl<'a, T> RotatedDrawTarget<'a, T>
where
    T: DrawTarget,
{
    pub fn new(parent: &'a mut T) -> Self {
        Self { parent }
    }
}

impl<T> DrawTarget for RotatedDrawTarget<'_, T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let parent_width = self.parent.bounding_box().size.width as i32;
        //esp_println::println!("parent_width: {}", parent_width);

        self.parent.draw_iter(pixels.into_iter().map(|Pixel(p, c)| Pixel(Point::new(parent_width - p.y, p.x), c)))
    }
}

impl<T> Dimensions for RotatedDrawTarget<'_, T>
where
    T: DrawTarget,
{
    fn bounding_box(&self) -> Rectangle {
        let parent_bb = self.parent.bounding_box();
        Rectangle::new(parent_bb.top_left, Size::new(parent_bb.size.height, parent_bb.size.width))
    }
}
