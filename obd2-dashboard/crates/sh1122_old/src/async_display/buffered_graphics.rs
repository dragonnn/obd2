//! Buffered graphics mode.

use display_interface::{AsyncWriteOnlyDataCommand, DisplayError};
use embedded_graphics::prelude::*;

use crate::{consts::*, display::DisplayRotation, AsyncDisplay};
/// Buffered graphics mode.
///
/// This mode keeps a pixel buffer in system memory, up to 1024 bytes for 128x64px displays. This
/// buffer is drawn to by [`set_pixel`](Ssd1306::set_pixel) commands or
/// [`embedded-graphics`](https://docs.rs/embedded-graphics) commands. The display can then be
/// updated using the [`flush`](Ssd1306::flush) method.
#[derive(Clone, Debug)]
pub struct AsyncBufferedGraphicsMode {
    buffer: [u8; (NUM_PIXEL_COLS as usize * NUM_PIXEL_ROWS as usize) / 2],
}

impl AsyncBufferedGraphicsMode {
    /// Create a new buffered graphics mode instance.
    pub(crate) fn new() -> Self {
        Self { buffer: [0u8; (NUM_PIXEL_COLS as usize * NUM_PIXEL_ROWS as usize) / 2] }
    }
}

impl<DI> DisplayConfig for AsyncDisplay<DI, AsyncBufferedGraphicsMode>
where
    DI: AsyncWriteOnlyDataCommand,
{
    type Error = DisplayError;

    /// Set the display rotation
    ///
    /// This method resets the cursor but does not clear the screen.
    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        //self.set_rotation(rot)
        Ok(())
    }

    /// Initialise and clear the display in graphics mode.
    fn init(&mut self) -> Result<(), DisplayError> {
        // /self.init(None)
        Ok(())
    }
}

impl<DI> AsyncDisplay<DI, AsyncBufferedGraphicsMode>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Clear the display buffer. You need to call `disp.flush()` for any effect on the screen
    pub fn clear(&mut self) {
        for b in self.mode.buffer.as_mut() {
            *b = 0;
        }
    }

    /// Write out data to a display.
    ///
    /// This only updates the parts of the display that have changed since the last flush.
    pub async fn flush(&mut self) -> Result<(), DisplayError> {
        //Command::SetColumnAddress(0).async_send(&mut self.iface).await?;
        //Command::SetRowAddress(0).async_send(&mut self.iface).await?;
        Self::draw_iface(&mut self.iface, &self.mode.buffer).await
    }

    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: Gray4) {
        let value = value.into_storage();
        let rotation = self.rotation;

        let idx = match rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                let idx = (y as usize) * NUM_PIXEL_COLS as usize + (x as usize);

                idx
            }
        };

        if let Some(byte) = self.mode.buffer.as_mut().get_mut(idx / 2) {
            #[allow(arithmetic_overflow)]
            {
                if idx % 2 != 0 {
                    *byte = *byte & 0xF0 | value;
                } else {
                    *byte = *byte & 0x0F | (value << 4);
                }
            }
        }
    }
}

use embedded_graphics::pixelcolor::Gray4;
#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{Dimensions, OriginDimensions},
    Pixel,
};

use super::Command;
use crate::mode::DisplayConfig;

#[cfg(feature = "graphics")]
impl<DI> DrawTarget for AsyncDisplay<DI, AsyncBufferedGraphicsMode>
where
    DI: AsyncWriteOnlyDataCommand,
{
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

#[cfg(feature = "graphics")]
impl<DI> OriginDimensions for AsyncDisplay<DI, AsyncBufferedGraphicsMode>
where
    DI: AsyncWriteOnlyDataCommand,
{
    fn size(&self) -> Size {
        let (w, h) = self.dimensions();

        Size::new(w.into(), h.into())
    }
}
