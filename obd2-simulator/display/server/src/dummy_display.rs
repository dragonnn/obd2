use embedded_graphics::{Pixel, pixelcolor::Gray4, prelude::*, primitives::Rectangle};
use ipc::DisplayIndex;
use tokio::sync::mpsc::UnboundedSender;

/// The maximum supported display width in pixels.
pub const NUM_PIXEL_COLS: u16 = 256;

/// The maximum supported display height in pixels.
pub const NUM_PIXEL_ROWS: u8 = 64;

pub struct DummyDisplay {
    buffer: [u8; NUM_PIXEL_COLS as usize * NUM_PIXEL_ROWS as usize],
    needs_flush: bool,
    index: DisplayIndex,
    display_tx: UnboundedSender<(DisplayIndex, Vec<u8>)>,
}

impl DummyDisplay {
    pub fn new(index: DisplayIndex, display_tx: UnboundedSender<(DisplayIndex, Vec<u8>)>) -> Self {
        DummyDisplay {
            buffer: [0; NUM_PIXEL_COLS as usize * NUM_PIXEL_ROWS as usize],
            needs_flush: true,
            index,
            display_tx,
        }
    }

    pub async fn init(&mut self, conf: Option<i32>) -> Result<(), ()> {
        self.needs_flush = true;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.needs_flush = true;
        self.buffer.fill(0);
    }

    pub async fn flush(&mut self) -> Result<(), ()> {
        if self.needs_flush {
            let data = self.buffer.to_vec();
            self.display_tx.send((self.index, data)).map_err(|_| ())?;
            info!("Flushing display");
            self.needs_flush = false;
        }
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

    pub fn set_pixel(&mut self, x: u32, y: u32, value: Gray4) {
        let value = value.into_storage();

        let idx = (y as usize) * NUM_PIXEL_COLS as usize + (x as usize);

        if let Some(byte) = self.buffer.as_mut().get_mut(idx) {
            /*if !value > 0 {
                *byte = 0xFF
            } else {
                info!("Setting pixel: x: {}, y: {}, value: {}", x, y, value);
                *byte = 0x20;
            }*/
            *byte = value * 17;
        } else {
            error!("Pixel out of bounds: x: {}, y: {}", x, y);
        }
        self.needs_flush = true;
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
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| self.set_pixel(pos.x as u32, pos.y as u32, color));
        Ok(())
    }
}

impl Dimensions for DummyDisplay {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::zero(),
            Size::new(NUM_PIXEL_COLS.into(), NUM_PIXEL_ROWS.into()),
        )
    }
}
