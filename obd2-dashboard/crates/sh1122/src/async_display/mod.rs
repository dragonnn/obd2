//! The main API to the display driver. It provides a builder API to configure the display, and
//! methods for obtaining `Region` instances which can be used to write image data to the display.

// This has to be here in order to be usable by mods declared afterwards.
#[cfg(test)]
#[macro_use]
pub mod testing {
    macro_rules! send {
        ([$($d:tt),*]) => {Sent::Data(vec![$($d,)*])};
        ($c:tt) => {Sent::Cmd($c)};
    }
    macro_rules! sends {
        ($($e:tt),*) => {&[$(send!($e),)*]};
    }
}

pub mod buffered_graphics;

use display_interface::{AsyncWriteOnlyDataCommand, DataFormat::U8, DisplayError};

use self::buffered_graphics::AsyncBufferedGraphicsMode;
use crate::{
    command::*,
    config::PersistentConfig,
    consts::*,
    display::DisplayRotation,
    mode::{BasicMode, TerminalMode},
    Config, PixelCoord,
};

/// A driver for an SSD1322 display.
pub struct AsyncDisplay<DI, MODE>
where
    DI: AsyncWriteOnlyDataCommand,
{
    pub iface: DI,
    pub(crate) mode: MODE,
    pub(crate) rotation: DisplayRotation,
    display_size: PixelCoord,
    display_offset: PixelCoord,
    contrast: u8,
    pub(crate) persistent_config: Option<PersistentConfig>,
}
impl<DI> AsyncDisplay<DI, BasicMode>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Construct a new display driver for a display with viewable dimensions `display_size`, which
    /// is connected to the interface `iface`.
    ///
    /// Some display modules with resolution lower than the maximum supported by the chip will
    /// connect column driver or COM lines starting in the middle rather than from 0 for mechanical
    /// PCB layout reasons.
    ///
    /// For such modules, `display_offset` allows automatically removing these offsets when drawing
    /// image data to the display. It describes the number of pixels of offset the display column
    /// numbering has relative to the driver and COM line numbering: `display_offset.0` indicates
    /// the driver line column which corresponds to pixel column 0 of the display, and
    /// `display_offset.1` indicates which COM line corresponds to pixel row 0 of the display.
    pub fn new(iface: DI, display_size: PixelCoord, display_offset: PixelCoord, rotation: DisplayRotation) -> Self {
        if false
            || display_size.0 > NUM_PIXEL_COLS as i16
            || display_size.1 > NUM_PIXEL_ROWS as i16
            || display_offset.0 + display_size.0 > NUM_PIXEL_COLS as i16
            || display_offset.1 + display_size.1 > NUM_PIXEL_ROWS as i16
            || display_size.0.rem_euclid(2) != 0
            || display_offset.0.rem_euclid(2) != 0
        {
            panic!("Display size or column offset not supported by SH1222");
        }
        AsyncDisplay {
            iface: iface,
            mode: BasicMode,
            rotation,
            display_size: display_size,
            display_offset: display_offset,
            persistent_config: None,
            contrast: 15,
        }
    }
}

impl<DI, MODE> AsyncDisplay<DI, MODE>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Convert the display into another interface mode.
    fn into_mode<MODE2>(self, mode: MODE2) -> AsyncDisplay<DI, MODE2> {
        AsyncDisplay {
            iface: self.iface,
            mode,
            rotation: self.rotation,
            display_size: self.display_size,
            display_offset: self.display_offset,
            contrast: 15,
            persistent_config: None,
        }
    }

    /// Convert the display into a buffered graphics mode, supporting
    /// [embedded-graphics](https://crates.io/crates/embedded-graphics).
    ///
    /// See [BufferedGraphicsMode] for more information.
    pub fn into_buffered_graphics_mode(self) -> AsyncDisplay<DI, AsyncBufferedGraphicsMode> {
        self.into_mode(AsyncBufferedGraphicsMode::new())
    }

    /// Convert the display into a text-only, terminal-like mode.
    ///
    /// See [TerminalMode] for more information.
    pub fn into_terminal_mode(self) -> AsyncDisplay<DI, TerminalMode> {
        self.into_mode(TerminalMode::new())
    }

    /// Initialize the display with a config message.
    pub async fn init(&mut self, config: Option<Config>) -> Result<(), DisplayError> {
        self.sleep(true).await?;

        match self.rotation {
            DisplayRotation::Rotate0 => {
                Command::SetSegmentRemap(self.rotation).async_send(&mut self.iface).await?;
                Command::SetScanDirection(0x00).async_send(&mut self.iface).await?;
            }
            DisplayRotation::Rotate180 => {
                Command::SetSegmentRemap(self.rotation).async_send(&mut self.iface).await?;
                Command::SetScanDirection(0x08).async_send(&mut self.iface).await?;
                // No idea why this is need in that rotation
                Command::SetRowAddress(32).async_send(&mut self.iface).await?;
            }
        }
        Command::SetStartLine(0).async_send(&mut self.iface).await?;

        //Command::SetScanDirection(0x00)
        //    .async_send(&mut self.iface)
        //    .await?;
        Command::SetContrastCurrent(0).async_send(&mut self.iface).await?;
        Command::SetMultiplexRatio(0x3F).async_send(&mut self.iface).await?;
        Command::SetDCDCSetting(DCDCSetting::new().with_dc_dc_enable(false).with_frequency(DCDCFrequency::Sf10))
            .async_send(&mut self.iface)
            .await?;

        Command::SetClockDivider(0b0001_0000).async_send(&mut self.iface).await?;
        Command::SetDisplayOffset(0x00).async_send(&mut self.iface).await?;
        Command::SetSecondPrechargePeriod(0x00).async_send(&mut self.iface).await?;
        Command::SetComDeselectVoltage(0x00).async_send(&mut self.iface).await?;
        Command::SetPreChargeVoltage(0x00).async_send(&mut self.iface).await?;
        Command::SetDischargeLevel(0x00).async_send(&mut self.iface).await?;
        embassy_time::Timer::after_millis(10).await;
        self.sleep(false).await;
        embassy_time::Timer::after_millis(10).await;
        Ok(())
    }

    /// Control sleep mode.
    pub async fn sleep(&mut self, enabled: bool) -> Result<(), DisplayError> {
        Command::SetSleepMode(enabled).async_send(&mut self.iface).await
    }

    /// Control the master contrast.
    pub async fn set_contrast(&mut self, contrast: u8) -> Result<(), DisplayError> {
        Command::SetContrastCurrent(contrast).async_send(&mut self.iface).await?;
        self.contrast = contrast;
        Ok(())
    }

    pub fn get_contrast(&self) -> u8 {
        self.contrast
    }

    /// Set the vertical pan.
    ///
    /// This uses the `Command::SetStartLine` feature to shift the display RAM row addresses
    /// relative to the active set of COM lines, allowing any display-height-sized window of the
    /// entire 128 rows of display RAM to be made visible.
    pub async fn vertical_pan(&mut self, offset: u8) -> Result<(), DisplayError> {
        Command::SetStartLine(offset).async_send(&mut self.iface).await
    }

    pub fn dimensions(&self) -> (u16, u8) {
        (NUM_PIXEL_COLS, NUM_PIXEL_ROWS)
    }

    pub async fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), DisplayError> {
        self.rotation = rotation;
        Command::SetSegmentRemap(self.rotation).async_send(&mut self.iface).await
    }

    pub async fn draw(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.iface.send_data(U8(&buffer)).await
    }

    pub async fn draw_iface(iface: &mut DI, buffer: &[u8]) -> Result<(), DisplayError> {
        iface.send_data(U8(&buffer)).await
    }
}
