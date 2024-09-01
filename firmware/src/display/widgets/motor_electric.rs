use core::cell::UnsafeCell;

use display_interface::DisplayError;
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
pub struct MotorElectric {
    motor_im: Image<'static, Bmp<'static, Rgb565>>,
    motor_on_im: Image<'static, Bmp<'static, Rgb565>>,
    motor_off_im: Image<'static, Bmp<'static, Rgb565>>,

    on: bool,
    needs_update: bool,
}

impl Default for MotorElectric {
    fn default() -> Self {
        Self::new(Point::zero())
    }
}

impl MotorElectric {
    pub fn new(position: Point) -> Self {
        static MOTOR_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
        let motor_bmp = MOTOR_BMP.get_or_init(|| unsafe {
            Bmp::from_slice(include_bytes!("../../../assets/motor_electric_body.bmp")).unwrap_unchecked()
        });

        static MOTOR_ON_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
        let motor_on_bmp = MOTOR_ON_BMP.get_or_init(|| unsafe {
            Bmp::from_slice(include_bytes!("../../../assets/motor_electric_symbol_on.bmp")).unwrap_unchecked()
        });

        static MOTOR_OFF_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
        let motor_off_bmp = MOTOR_OFF_BMP.get_or_init(|| unsafe {
            Bmp::from_slice(include_bytes!("../../../assets/motor_electric_symbol_off.bmp")).unwrap_unchecked()
        });

        let motor_im: Image<Bmp<Rgb565>> = Image::new(motor_bmp, position);
        let motor_on_im: Image<Bmp<Rgb565>> = Image::new(motor_on_bmp, position).translate(Point::new(34, 24));
        let motor_off_im: Image<Bmp<Rgb565>> = Image::new(motor_off_bmp, position).translate(Point::new(34, 24));

        Self { motor_im, motor_on_im, motor_off_im, on: false, needs_update: true }
    }

    pub fn update_on(&mut self, on: bool) {
        if self.on != on {
            self.on = on;
            self.needs_update = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.needs_update {
            esp_println::println!("motor electric redraw");
            self.motor_im.draw(&mut target.color_converted())?;
            if self.on {
                self.motor_on_im.draw(&mut target.color_converted())?;
            } else {
                self.motor_off_im.draw(&mut target.color_converted())?;
            }
            self.needs_update = false;
        }

        Ok(())
    }
}
