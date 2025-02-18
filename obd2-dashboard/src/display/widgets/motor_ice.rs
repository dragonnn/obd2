use display_interface::DisplayError;
use embedded_graphics::{
    image::Image,
    pixelcolor::{Gray4, Rgb565},
    prelude::*,
    primitives::*,
};
use once_cell::sync::OnceCell;
use static_cell::StaticCell;
use tinybmp::Bmp;

#[derive(Debug, Clone)]
pub struct MotorIce {
    motor_im: Image<'static, Bmp<'static, Rgb565>>,
    motor_on_im: Image<'static, Bmp<'static, Rgb565>>,
    motor_off_im: Image<'static, Bmp<'static, Rgb565>>,

    on: bool,
    needs_update: bool,
}

impl Default for MotorIce {
    fn default() -> Self {
        Self::new(Point::zero())
    }
}

impl MotorIce {
    pub fn new(position: Point) -> Self {
        static MOTOR_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
        let motor_bmp = MOTOR_BMP.get_or_init(|| unsafe {
            Bmp::from_slice(include_bytes!("../../../assets/motor_ice_body.bmp")).unwrap_unchecked()
        });

        static MOTOR_ON_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
        let motor_on_bmp = MOTOR_ON_BMP.get_or_init(|| unsafe {
            Bmp::from_slice(include_bytes!("../../../assets/motor_ice_symbol_on.bmp")).unwrap_unchecked()
        });

        static MOTOR_OFF_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
        let motor_off_bmp = MOTOR_OFF_BMP.get_or_init(|| unsafe {
            Bmp::from_slice(include_bytes!("../../../assets/motor_ice_symbol_off.bmp")).unwrap_unchecked()
        });

        let motor_im: Image<Bmp<Rgb565>> = Image::new(motor_bmp, position);
        let motor_on_im: Image<Bmp<Rgb565>> = Image::new(motor_on_bmp, position).translate(Point::new(5, 24));
        let motor_off_im: Image<Bmp<Rgb565>> = Image::new(motor_off_bmp, position).translate(Point::new(5, 24));

        Self { motor_im, motor_on_im, motor_off_im, on: false, needs_update: true }
    }

    pub fn update_on(&mut self, on: bool) -> bool {
        if self.on != on {
            self.on = on;
            self.needs_update = true;
        }
        self.needs_update
    }

    pub fn is_redraw(&self) -> bool {
        self.needs_update
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.needs_update {
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
