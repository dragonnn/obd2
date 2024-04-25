use display_interface::DisplayError;
use embedded_graphics::{
    image::Image,
    pixelcolor::{Gray4, Rgb565},
    prelude::*,
    primitives::*,
};
use static_cell::StaticCell;
use tinybmp::Bmp;

static MOTOR_BMP: StaticCell<Bmp<Rgb565>> = StaticCell::new();
static MOTOR_ON_BMP: StaticCell<Bmp<Rgb565>> = StaticCell::new();
static MOTOR_OFF_BMP: StaticCell<Bmp<Rgb565>> = StaticCell::new();

pub struct MotorIce<D> {
    motor_im: Image<'static, Bmp<'static, Rgb565>>,
    motor_on_im: Image<'static, Bmp<'static, Rgb565>>,
    motor_off_im: Image<'static, Bmp<'static, Rgb565>>,

    on: bool,
    needs_update: bool,
    _marker: core::marker::PhantomData<D>,
}

impl<D> MotorIce<D>
where
    D: DrawTarget<Color = Gray4>,
{
    pub fn new(position: Point) -> Self {
        let motor_bmp = MOTOR_BMP.init(Bmp::from_slice(include_bytes!("../../../assets/motor_ice_body.bmp")).unwrap());
        let motor_on_bmp =
            MOTOR_ON_BMP.init(Bmp::from_slice(include_bytes!("../../../assets/motor_ice_symbol_on.bmp")).unwrap());
        let motor_off_bmp =
            MOTOR_OFF_BMP.init(Bmp::from_slice(include_bytes!("../../../assets/motor_ice_symbol_off.bmp")).unwrap());

        let motor_im: Image<Bmp<Rgb565>> = Image::new(motor_bmp, position);
        let motor_on_im: Image<Bmp<Rgb565>> = Image::new(motor_on_bmp, position).translate(Point::new(5, 24));
        let motor_off_im: Image<Bmp<Rgb565>> = Image::new(motor_off_bmp, position).translate(Point::new(5, 24));

        Self {
            motor_im,
            motor_on_im,
            motor_off_im,
            on: false,
            needs_update: true,
            _marker: core::marker::PhantomData::default(),
        }
    }

    pub fn update_on(&mut self, on: bool) {
        if self.on != on {
            self.on = on;
            self.needs_update = true;
        }
    }

    pub fn draw(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.needs_update {
            esp_println::println!("motor ice redraw");
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
