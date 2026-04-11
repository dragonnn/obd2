use core::cell::UnsafeCell;

use defmt::trace;
use display_interface::DisplayError;
use embassy_time::Instant;
use embedded_graphics::{
    image::Image,
    pixelcolor::{Gray4, Rgb565},
    prelude::*,
    primitives::*,
};
use once_cell::sync::{Lazy, OnceCell};
use static_cell::StaticCell;
use tinybmp::Bmp;

#[derive(Debug, Clone)]
pub struct Grid {
    grid_im: Image<'static, Bmp<'static, Rgb565>>,
    needs_update: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(Point::zero())
    }
}

impl Grid {
    pub fn new(position: Point) -> Self {
        static GRID_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
        let grid_bmp =
            GRID_BMP.get_or_init(|| unsafe { Bmp::from_slice(include_bytes!("../../../assets/grid.bmp")).unwrap() });

        let grid_im: Image<Bmp<Rgb565>> = Image::new(grid_bmp, position);

        Self { grid_im, needs_update: true }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.needs_update {
            let now = Instant::now();
            self.grid_im.draw(&mut target.color_converted())?;
            let elapsed_us = now.elapsed().as_micros() as u32;
            trace!("Grid draw: {=u32},{=u32:03}ms", elapsed_us / 1000, elapsed_us % 1000);
            self.needs_update = false;
        }

        Ok(())
    }
}
