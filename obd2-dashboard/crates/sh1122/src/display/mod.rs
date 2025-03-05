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

use display_interface::{AsyncWriteOnlyDataCommand, DataFormat::U8, DisplayError, WriteOnlyDataCommand};

use crate::{
    command::*,
    config::PersistentConfig,
    consts::*,
    mode::{BasicMode, BufferedGraphicsMode, TerminalMode},
    Config,
};

/// A pixel coordinate pair of `column` and `row`. `column` must be in the range [0,
/// `consts::PIXEL_COL_MAX`], and `row` must be in the range [0, `consts::PIXEL_ROW_MAX`].
#[derive(Clone, Copy, Debug)]
pub struct PixelCoord(pub i16, pub i16);

/// A driver for an SSD1322 display.
pub struct Display<DI, MODE>
where
    DI: WriteOnlyDataCommand,
{
    pub iface: DI,
    pub(crate) mode: MODE,
    pub(crate) rotation: DisplayRotation,
    display_size: PixelCoord,
    display_offset: PixelCoord,
    pub(crate) persistent_config: Option<PersistentConfig>,
}
impl<DI> Display<DI, BasicMode>
where
    DI: WriteOnlyDataCommand,
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
        Display {
            iface: iface,
            mode: BasicMode,
            rotation,
            display_size: display_size,
            display_offset: display_offset,
            persistent_config: None,
        }
    }
}

impl<DI, MODE> Display<DI, MODE>
where
    DI: WriteOnlyDataCommand,
{
    /// Convert the display into another interface mode.
    fn into_mode<MODE2>(self, mode: MODE2) -> Display<DI, MODE2> {
        Display {
            iface: self.iface,
            mode,
            rotation: self.rotation,
            display_size: self.display_size,
            display_offset: self.display_offset,
            persistent_config: None,
        }
    }

    /// Convert the display into a buffered graphics mode, supporting
    /// [embedded-graphics](https://crates.io/crates/embedded-graphics).
    ///
    /// See [BufferedGraphicsMode] for more information.
    pub fn into_buffered_graphics_mode(self) -> Display<DI, BufferedGraphicsMode> {
        self.into_mode(BufferedGraphicsMode::new())
    }

    /// Convert the display into a text-only, terminal-like mode.
    ///
    /// See [TerminalMode] for more information.
    pub fn into_terminal_mode(self) -> Display<DI, TerminalMode> {
        self.into_mode(TerminalMode::new())
    }

    /// Initialize the display with a config message.
    pub fn init(&mut self, config: Option<Config>) -> Result<(), DisplayError> {
        self.sleep(true)?;
        Command::SetStartLine(0).send(&mut self.iface)?;
        match self.rotation {
            DisplayRotation::Rotate0 => {}
            DisplayRotation::Rotate180 => {
                /*Command::SetRemapping(
                    IncrementAxis::Horizontal,
                    ColumnRemap::Forward,
                    NibbleRemap::Reverse,
                    ComScanDirection::RowZeroLast,
                    ComLayout::Progressive,
                )
                .send(&mut self.iface)?;*/
            }
        }

        Command::SetScanDirection(0x0).send(&mut self.iface)?;
        Command::SetContrastCurrent(0x80).send(&mut self.iface)?;
        Command::SetMultiplexRatio(0x3F).send(&mut self.iface)?;
        Command::SetDCDCSetting(DCDCSetting::from_bytes([0x81])).send(&mut self.iface)?;
        Command::SetClockDivider(0x50).send(&mut self.iface)?;
        Command::SetDisplayOffset(0x00).send(&mut self.iface)?;
        Command::SetSecondPrechargePeriod(0x22).send(&mut self.iface)?;
        Command::SetComDeselectVoltage(0x35).send(&mut self.iface)?;
        Command::SetPreChargeVoltage(0x35).send(&mut self.iface)?;
        Command::SetDischargeLevel(0x0).send(&mut self.iface)?;
        self.sleep(false)
    }

    /// Control sleep mode.
    pub fn sleep(&mut self, enabled: bool) -> Result<(), DisplayError> {
        Command::SetSleepMode(enabled).send(&mut self.iface)
    }

    /// Control the master contrast.
    pub fn contrast(&mut self, contrast: u8) -> Result<(), DisplayError> {
        Command::SetContrastCurrent(contrast).send(&mut self.iface)
    }

    /// Set the display brightness look-up table.
    pub fn gray_scale_table(&mut self, table: &[u8]) -> Result<(), DisplayError> {
        BufCommand::SetGrayScaleTable(table).send(&mut self.iface)
    }

    /// Set the vertical pan.
    ///
    /// This uses the `Command::SetStartLine` feature to shift the display RAM row addresses
    /// relative to the active set of COM lines, allowing any display-height-sized window of the
    /// entire 128 rows of display RAM to be made visible.
    pub fn vertical_pan(&mut self, offset: u8) -> Result<(), DisplayError> {
        Command::SetStartLine(offset).send(&mut self.iface)
    }

    pub fn dimensions(&self) -> (u16, u8) {
        (NUM_PIXEL_COLS, NUM_PIXEL_ROWS)
    }

    pub fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), DisplayError> {
        self.rotation = rotation;
        //Command::SetSegmentRemap(self.rotation).send(&mut self.iface)
        Ok(())
    }

    pub fn draw(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.iface.send_data(U8(&buffer))
    }

    pub fn draw_iface(iface: &mut DI, buffer: &[u8]) -> Result<(), DisplayError> {
        iface.send_data(U8(&buffer))
    }
}

/// Display rotation.
#[derive(Copy, Clone, Debug)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 180 degress clockwise
    Rotate180,
}
