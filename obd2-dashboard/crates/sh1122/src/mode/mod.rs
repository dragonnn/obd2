//! Display modes.

mod buffered_graphics;
mod terminal;

use crate::{display::DisplayRotation, Display};
pub use buffered_graphics::*;
use display_interface::{DisplayError, WriteOnlyDataCommand};
pub use terminal::*;

/// Common functions to all display modes.
pub trait DisplayConfig {
    /// Error.
    type Error;

    /// Set display rotation.
    fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), Self::Error>;

    /// Initialise and configure the display for the given mode.
    fn init(&mut self) -> Result<(), Self::Error>;
}

/// A mode with no additional functionality beyond that provided by the base [`Ssd1306`] struct.
#[derive(Debug, Copy, Clone)]
pub struct BasicMode;

impl<DI> Display<DI, BasicMode>
where
    DI: WriteOnlyDataCommand,
{
    /// Clear the display.
    pub fn clear(&mut self) -> Result<(), DisplayError> {
        //self.set_draw_area((0, 0), self.dimensions())?;

        // TODO: If const generics allows this, replace `1024` with computed W x H value for current
        // `SIZE`.
        //self.draw(&[0u8; 1024])
        Ok(())
    }
}

impl<DI> DisplayConfig for Display<DI, BasicMode>
where
    DI: WriteOnlyDataCommand,
{
    type Error = DisplayError;

    /// Set the display rotation.
    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_rotation(rot)
    }

    /// Initialise in horizontal addressing mode.
    fn init(&mut self) -> Result<(), DisplayError> {
        //self.init_with_addr_mode(AddrMode::Horizontal)
        Ok(())
    }
}
