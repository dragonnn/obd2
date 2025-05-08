use alloc::{borrow::Cow, string::ToString as _};
use core::fmt::Write;

use defmt::info;
use display_interface::DisplayError;
use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyle,
    pixelcolor::{Gray4, Rgb565},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable as _},
    text::{Alignment, LineHeight, Text, TextStyleBuilder},
};
use embedded_iconoir::prelude::IconoirNewIcon as _;
use heapless::String;
use num_traits::Float;
use once_cell::sync::{Lazy, OnceCell};
use profont::PROFONT_9_POINT;
use tinybmp::Bmp;

#[derive(Clone, Debug)]
pub struct Humidity {
    position: Point,

    humidity: u8,
    evaporator_temp: f32,
    compressor: bool,

    redraw: bool,

    text_bounding_box: Option<Rectangle>,
}

impl Default for Humidity {
    fn default() -> Self {
        Self {
            position: Point::zero(),
            redraw: true,
            humidity: 0,
            compressor: false,
            text_bounding_box: None,
            evaporator_temp: 0.0,
        }
    }
}

impl Humidity {
    pub fn new(position: Point) -> Self {
        Self { position, ..Default::default() }
    }

    pub fn update_humidity(&mut self, humidity: u8) {
        if self.humidity != humidity {
            self.humidity = humidity;
            self.redraw = true;
        }
    }

    pub fn update_compressor(&mut self, compressor: bool) {
        if self.compressor != compressor {
            self.compressor = compressor;
            self.redraw = true;
        }
    }

    pub fn update_evaporator_temp(&mut self, evaporator_temp: f32) {
        if self.evaporator_temp != evaporator_temp {
            self.evaporator_temp = evaporator_temp;
            self.redraw = true;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if self.redraw {
            let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                .stroke_width(0)
                .stroke_color(Gray4::BLACK)
                .fill_color(Gray4::BLACK)
                .build();

            static DROPLET_SOLID: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
            let droplet_solid = DROPLET_SOLID.get_or_init(|| unsafe {
                Bmp::from_slice(include_bytes!("../../../assets/droplet_solid.bmp")).unwrap_unchecked()
            });

            let droplet = embedded_iconoir::size32px::design_tools::Droplet::new(GrayColor::WHITE);
            let compressor = embedded_iconoir::icons::size18px::weather::SnowFlake::new(Gray4::new(0xF - 2));
            //let compressor_black = embedded_iconoir::icons::size18px::weather::SnowFlake::new(Gray4::new(0x02));

            let droplet = Image::new(&droplet, self.position);
            let mut bounding_box = droplet.bounding_box();
            let compressor = Image::with_center(&compressor, bounding_box.center() + Point::new(0, 3));
            //let compressor_black = Image::with_center(&compressor_black, bounding_box.center() + Point::new(0, 2));
            let droplet_solid = Image::new(droplet_solid, self.position);

            bounding_box.draw_styled(&style, target)?;

            //if self.compressor {
            //    compressor.draw(target)?;
            //}

            let org_height = bounding_box.size.height;

            bounding_box.size.height = (org_height as f32 * self.humidity as f32 / 100.0).round() as u32;
            bounding_box.top_left.y += (org_height - bounding_box.size.height) as i32;

            let mut clipped_target = target.clipped(&bounding_box);
            droplet_solid.draw(&mut clipped_target.color_converted())?;
            droplet.draw(target)?;
            if self.compressor {
                //compressor_black.draw(&mut clipped_target)?;
                compressor.draw(target)?;
            }

            info!("Humidity bounding box: {:?}", bounding_box.size.height);

            let mut text: String<16> = String::new();
            core::write!(text, "{:.0}%\n{:.0}°C", self.humidity, self.evaporator_temp).ok();

            let text_style =
                TextStyleBuilder::new().alignment(Alignment::Right).line_height(LineHeight::Percent(100)).build();

            let character_style_small = MonoTextStyle::new(&PROFONT_9_POINT, Gray4::WHITE);
            let text = Text::with_text_style(
                text.as_str(),
                self.position + Point::new(-1, 10),
                character_style_small,
                text_style,
            );

            let text_bounding_box = text.bounding_box();

            if text_bounding_box.size.width > self.text_bounding_box.map(|bb| bb.size.width).unwrap_or(0) {
                self.text_bounding_box = Some(text_bounding_box);
            }

            if let Some(bb) = self.text_bounding_box {
                bb.draw_styled(&PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build(), target)?;
            }

            text.draw(target)?;

            self.redraw = false;
        }

        Ok(())
    }
}
