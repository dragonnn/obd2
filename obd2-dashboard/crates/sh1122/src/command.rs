//! The command set for the SSD1322.
//!
//! The display RAM of the SSD1322 is arranged in 128 rows and 120 columns, where each column is 4
//! adjacent pixels (segments) in the row for a total max resolution of 128x480. Each pixel is 4
//! bits/16 levels of intensity, so each column also refers to two adjacent bytes. Thus, anywhere
//! there is a "column" address, these refer to horizontal groups of 2 bytes driving 4 pixels.

use display_interface::{AsyncWriteOnlyDataCommand, DataFormat::U8, DisplayError, WriteOnlyDataCommand};
use modular_bitfield::{bitfield, prelude::B4, BitfieldSpecifier};

use self::consts::*;
use crate::display::DisplayRotation;

pub mod consts {
    //! Constants describing max supported display size and the display RAM layout.

    /// The maximum supported display width in pixels.
    pub const NUM_PIXEL_COLS: u16 = 256;

    /// The maximum supported display height in pixels.
    pub const NUM_PIXEL_ROWS: u8 = 64;

    /// The number of display RAM column addresses.
    pub const NUM_BUF_COLS: u8 = (NUM_PIXEL_COLS / 4) as u8;

    /// The highest valid pixel column index.
    pub const PIXEL_COL_MAX: u16 = NUM_PIXEL_COLS - 1;

    /// The highest valid pixel row index.
    pub const PIXEL_ROW_MAX: u8 = NUM_PIXEL_ROWS - 1;

    /// The highest valid display RAM column address.
    pub const BUF_COL_MAX: u8 = NUM_BUF_COLS - 1;
}

#[bitfield]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DCDCSetting {
    padding: B4,
    pub frequency: DCDCFrequency,
    pub dc_dc_enable: bool,
}

#[derive(BitfieldSpecifier, Copy, Clone, Debug)]
#[bits = 3]
pub enum DCDCFrequency {
    Sf06 = 0b000,
    Sf07 = 0b001,
    Sf08 = 0b010,
    Sf09 = 0b011,
    Sf10 = 0b100,
    Sf11 = 0b101,
    Sf12 = 0b110,
    Sf13 = 0b111,
}

/// The address increment orientation when writing image data. This configures how the SSD1322 will
/// auto-increment the row and column addresses when image data is written using the
/// `WriteImageData` command.
#[derive(Clone, Copy)]
pub enum IncrementAxis {
    /// The column address will increment as image data is written, writing pairs of bytes
    /// (horizontal groups of 4 pixels) from left to right in the range set by `SetColumnAddress`
    /// command, and then top to bottom in the range set by `SetRowAddress` command.
    Horizontal,
    /// The row address will increment as image data is written, writing pairs of bytes
    /// (*horizontal* groups of 4 pixels) from top to bottom in the range set by `SetRowAddress`
    /// command, and then left to right in the range set by `SetColumnAddress` command.
    Vertical,
}

/// Setting of column address remapping. This controls the direction of mapping display RAM column
/// addresses onto groups of pixel column driver lines.
#[derive(Clone, Copy)]
pub enum ColumnRemap {
    /// Column addresses 0->119 map to pixel columns 0,1,2,3->476,477,478,479.
    Forward,
    /// Column addresses 0->119 map to pixel columns 476,477,478,479->0,1,2,3. Note that the pixels
    /// within each column number in the same order; `NibbleRemap` controls the order of mapping
    /// pixels to nibbles within each column.
    Reverse,
}

/// Setting of data nibble remapping. This controls how the SSD1322 will interpret the nibble-wise
/// endianness of each 2-byte word, changing the order in which each group of 4 pixels is mapped
/// onto the 4 nibbles stored at the corresponding display RAM column address.
#[derive(Clone, Copy)]
pub enum NibbleRemap {
    /// The 2-byte sequence at each column address 0xABCD maps (in L->R order) to pixels 3,2,1,0.
    Reverse,
    /// The 2-byte sequence at each column address 0xABCD maps (in L->R order) to pixels 0,1,2,3.
    Forward,
}

/// Setting of the COM line scanning of rows. This controls the order in which COM lines are
/// scanned, leaving the order in which display RAM row addresses are scanned unchanged. Toggling
/// this setting will thus flip the displayed image vertically.
#[derive(Clone, Copy)]
pub enum ComScanDirection {
    /// COM lines scan row addresses top to bottom, so that row address 0 is the first row of the
    /// display.
    RowZeroFirst,
    /// COM lines scan row addresses bottom to top, so that row address 0 is the last row of the
    /// display.
    RowZeroLast,
}

/// Setting the layout of the COM lines to the display rows. This setting is dictated by how the
/// display module itself wires the OLED matrix to the driver chip, and changing it to anything
/// other than the correct setting for your module will yield a corrupted image. See the display
/// module datasheet for the correct value to use.
#[derive(Clone, Copy)]
pub enum ComLayout {
    /// COM lines are connected to display rows in a progressive arrangement, so that COM lines
    /// 0->127 map to display rows 0->127.
    Progressive,
    /// COM lines are connected to display rows in an interlaced arrangement, so that COM lines
    /// 0->63 map to *even* display rows 0->126, and COM lines 64->127 map to *odd* display rows
    /// 1->127.
    Interlaced,
    /// COM lines are connected to display rows in a dual-COM progressive arrangement, so that COM
    /// lines 0->63 map to display rows 0->63 for half of the columns, and COM lines 64->127 map to
    /// display rows 0->63 for the other half. The maximum displayable image size for this
    /// configuration is halved to 480x64 because each display row uses two COM lines.
    DualProgressive,
}

/// Setting of the display mode. The display mode controls whether the display is blanked, and
/// whether the pixel intensities are rendered normal or inverted.
#[derive(Clone, Copy)]
pub enum DisplayMode {
    /// The display is blanked with all pixels turned OFF (to grayscale level 0).
    BlankDark,
    /// The display is blanked with all pixels turned ON (to grayscale level 15).
    BlankBright,
    /// The display operates normally, showing the image in the display RAM.
    Normal,
    /// The display operates with inverse brightness, showing the image in the display RAM with the
    /// grayscale levels inverted (level 0->15, 1->14, ..., 15->0).
    Inverse,
}

/// Enumerates most of the valid commands that can be sent to the SSD1322 along with their
/// parameter values. Commands which accept an array of similar "arguments" as a slice are encoded
/// by `BufCommand` instead to avoid lifetime parameters on this enum.
#[derive(Clone, Copy)]
pub enum Command {
    /// Enable the gray scale gamma table (see `BufCommand::SetGrayScaleTable`).
    EnableGrayScaleTable,
    /// Set the column start and end address range when writing to the display RAM. The column
    /// address pointer is reset to the start column address such that `WriteImageData` will begin
    /// writing there. Range is 0-119. (Note 1)
    SetColumnAddress(u8),
    SetHighColumnAddress(u8),
    SetLowColumnAddress(u8),
    /// Set the row start address when writing to the display RAM. The row address
    /// pointer is reset to the start row address such that `WriteImageData` will begin writing
    /// there. Range is 0-127.
    SetRowAddress(u8),
    /// Set the direction of display address increment, column address remapping, data nibble
    /// remapping, COM scan direction, and COM line layout. See documentation for each enum for
    /// details.
    SetRemapping(IncrementAxis, ColumnRemap, NibbleRemap, ComScanDirection, ComLayout),
    /// Set the display start line. Setting this to e.g. 40 will cause the first row of pixels on
    /// the display to display row 40 or the display RAM, and rows 0-39 of the display RAM will be
    /// wrapped to the bottom, "rolling" the displayed image upwards.  This transformation is
    /// applied *before* the MUX ratio setting, meaning if the MUX ratio is set to 90, display rows
    /// 0->89 will always be active, and the "rolled" image will be rendered within those display
    /// rows. Range is 0-127.
    SetStartLine(u8),
    /// Set the display COM line offset. This has a similar effect to `SetStartLine`, rolling the
    /// displayed image upwards as the values increase, except that it is applied *after* the MUX
    /// ratio setting. This means both the image *and* the display rows seleced by the MUX ratio
    /// setting will be rolled upwards. Range is 0-127.
    SetDisplayOffset(u8),
    /// Set the display operating mode. See enum for details.
    SetDisplayMode(DisplayMode),
    /// Enable partial display mode. This selects an inclusive range of rows `start` and `end` in
    /// the display area which will be active, while all others remain inactive. Range is 0-127,
    /// where `start` must be <= `end`.
    EnablePartialDisplay(u8, u8),
    /// Disable partial display mode.
    DisablePartialDisplay,
    /// Control sleep mode. When sleep mode is enabled (`true`), the display multiplexer and driver
    /// circuits are powered off.
    SetSleepMode(bool),
    /// Set the refresh phase lengths. The first phase (reset) can be set from 5-31 DCLKs, and the
    /// second (first pre-charge) can be set from 3-15 DCLKs. The display module datasheet should
    /// have appropriate values.
    SetPhaseLengths(u8, u8),
    /// Set the display clock divider.
    /// set display clock divide ratio (lower 4 bit)/oscillator frequency (upper 4 bit)
    SetClockDivider(u8),
    /// Set the second pre-charge period. Range 0-15 DCLKs.
    /// pre charge (lower 4 bit) and discharge(higher 4 bit) period
    SetSecondPrechargePeriod(u8),
    /// Set the gray scale gamma table to the factory default.
    SetDefaultGrayScaleTable,
    /// Set the pre-charge voltage level, from 0.2*Vcc to 0.6*Vcc. Range 0-31.
    SetPreChargeVoltage(u8),
    /// Set the COM deselect voltage level, from 0.72*Vcc to 0.86*Vcc. Range 0-7.
    SetComDeselectVoltage(u8),
    /// Set the contrast current. Range 0-255.
    SetContrastCurrent(u8),
    /// Set the MUX ratio, which controls the number of COM lines that are active and thus the
    /// number of display pixel rows which are active. Which COM lines are active is controlled by
    /// `SetDisplayOffset`, and how the COM lines map to display RAM row addresses is controlled by
    /// `SetStartLine`. Range 16-128.
    SetMuxRatio(u8),
    /// Set whether the command lock is enabled or disabled. Enabling the command lock (`true`)
    /// blocks all commands except `SetCommandLock`.
    SetCommandLock(bool),
    /// use buildin DC-DC with 0.6 * 500 kHz
    SetDCDCSetting(DCDCSetting),
    /// discharge level
    SetDischargeLevel(u8),
    /// remap segments
    SetSegmentRemap(DisplayRotation),
    /// scan direction
    SetScanDirection(u8),
    /// multiplex ratio 1/64 Duty (0x0F~0x3F)
    SetMultiplexRatio(u8),
}

/// Enumerates commands that can be sent to the SSD1322 which accept a slice argument buffer. This
/// is separated from `Command` so that the lifetime parameter of the argument buffer slice does
/// not pervade code which never invokes these two commands.
pub enum BufCommand<'buf> {
    /// Set the gray scale gamma table. Each byte 0-14 can range from 0-180 and sets the pixel
    /// drive pulse width in DCLKs. Bytes 0->14 adjust the gamma setting for grayscale levels
    /// 1->15; grayscale level 0 cannot be modified. The gamma settings must monotonically
    /// increase.
    SetGrayScaleTable(&'buf [u8]),
    /// Write image data into display RAM. The image data will be written to the display RAM in the
    /// order specified by `SetRemapping` `IncrementAxis` setting. The data, once written, will be
    /// mapped onto the display pixels in a manner determined by `SetRemapping` `ColumnRemap`,
    /// `NibbleRemap`, `ComScanDirection`, and `ComLayout` settings.
    WriteImageData(&'buf [u8]),
}

macro_rules! ok_command {
    ($buf:ident, $cmd:expr,[]) => {
        Ok(($cmd, &$buf[..0]))
    };
    ($buf:ident, $cmd:expr,[$arg0:expr]) => {{
        $buf[0] = $arg0;
        Ok(($cmd, &$buf[..1]))
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        Ok(($cmd, &$buf[..2]))
    }};
}

impl Command {
    /// Transmit the command encoded by `self` to the display on interface `iface`.
    pub fn send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
    {
        let mut arg_buf = [0u8; 2];
        let (cmd, data) = match self {
            Command::EnableGrayScaleTable => ok_command!(arg_buf, 0x00, []),
            Command::SetColumnAddress(address) => match address {
                0..=BUF_COL_MAX => {
                    ok_command!(arg_buf, 0x10 | (address >> 4), [0x00 | (address & 0x0F)])
                }
                _ => Err(()),
            },
            Command::SetHighColumnAddress(address) => match address {
                0..=0x7F => ok_command!(arg_buf, 0x10 + (address >> 4), []),
                _ => Err(()),
            },
            Command::SetLowColumnAddress(address) => match address {
                0..=0x7F => ok_command!(arg_buf, 0x00 + (address & 0x0F), []),
                _ => Err(()),
            },
            Command::SetRowAddress(address) => match address {
                0..=0xFF => ok_command!(arg_buf, 0xB0, [address]),
                _ => Err(()),
            },
            //TODO: this should be part of SetRemapping
            Command::SetSegmentRemap(remap) => match remap {
                DisplayRotation::Rotate0 => ok_command!(arg_buf, 0xA0 | 0x00, []),
                DisplayRotation::Rotate180 => ok_command!(arg_buf, 0xA0 | 0x01, []),
            },
            Command::SetScanDirection(direction) => match direction {
                0..=0x8 => ok_command!(arg_buf, 0xC0 | direction, []),
                _ => Err(()),
            },
            Command::SetMultiplexRatio(ratio) => match ratio {
                0x0F..=0x3F => ok_command!(arg_buf, 0xA8, [ratio]),
                _ => Err(()),
            },
            Command::SetDCDCSetting(dcdc) => {
                let dcdc_bytes = dcdc.into_bytes()[0];
                match dcdc_bytes {
                    0x00..=0xFF => {
                        ok_command!(arg_buf, 0xAD, [dcdc_bytes])
                    }
                    _ => Err(()),
                }
            }
            Command::SetRemapping(increment_axis, column_remap, nibble_remap, com_scan_direction, com_layout) => {
                let ia = match increment_axis {
                    IncrementAxis::Horizontal => 0x00,
                    IncrementAxis::Vertical => 0x01,
                };
                let cr = match column_remap {
                    ColumnRemap::Forward => 0x00,
                    ColumnRemap::Reverse => 0x02,
                };
                let nr = match nibble_remap {
                    NibbleRemap::Reverse => 0x00,
                    NibbleRemap::Forward => 0x04,
                };
                let csd = match com_scan_direction {
                    ComScanDirection::RowZeroFirst => 0x00,
                    ComScanDirection::RowZeroLast => 0x10,
                };
                let (interlace, dual_com) = match com_layout {
                    ComLayout::Progressive => (0x00, 0x01),
                    ComLayout::Interlaced => (0x20, 0x01),
                    ComLayout::DualProgressive => (0x00, 0x11),
                };
                ok_command!(arg_buf, 0xA0 | (ia | cr | nr | csd | interlace), [])
            }
            Command::SetStartLine(line) => match line {
                0..=PIXEL_ROW_MAX => ok_command!(arg_buf, 0x40 | line, []),
                _ => Err(()),
            },
            Command::SetDisplayOffset(line) => match line {
                0..=PIXEL_ROW_MAX => ok_command!(arg_buf, 0xD3, [line]),
                _ => Err(()),
            },
            Command::SetDisplayMode(mode) => ok_command!(
                arg_buf,
                match mode {
                    DisplayMode::BlankDark => 0xA4,
                    DisplayMode::BlankBright => 0xA5,
                    DisplayMode::Normal => 0xA6,
                    DisplayMode::Inverse => 0xA7,
                },
                []
            ),
            Command::EnablePartialDisplay(start, end) => match (start, end) {
                (0..=PIXEL_ROW_MAX, 0..=PIXEL_ROW_MAX) if start <= end => {
                    ok_command!(arg_buf, 0xA8, [start, end])
                }
                _ => Err(()),
            },
            Command::DisablePartialDisplay => ok_command!(arg_buf, 0xA9, []),
            Command::SetSleepMode(ena) => ok_command!(
                arg_buf,
                match ena {
                    true => 0xAE,
                    false => 0xAF,
                },
                []
            ),
            Command::SetPhaseLengths(phase_1, phase_2) => match (phase_1, phase_2) {
                (5..=31, 3..=15) => {
                    let p1 = (phase_1 - 1) >> 1;
                    let p2 = 0xF0 & (phase_2 << 4);
                    ok_command!(arg_buf, 0xB1, [p1 | p2])
                }
                _ => Err(()),
            },
            Command::SetDischargeLevel(level) => match level {
                0..=0x0F => ok_command!(arg_buf, 0x30 | level, []),
                _ => Err(()),
            },
            Command::SetClockDivider(divider) => match divider {
                0..=0xFF => ok_command!(arg_buf, 0xD5, [divider]),
                _ => Err(()),
            },
            Command::SetSecondPrechargePeriod(period) => match period {
                0..=0x40 => ok_command!(arg_buf, 0xD9, [period]),
                _ => Err(()),
            },
            Command::SetDefaultGrayScaleTable => ok_command!(arg_buf, 0xB9, []),
            Command::SetPreChargeVoltage(voltage) => match voltage {
                0..=0x50 => ok_command!(arg_buf, 0xDC, [voltage]),
                _ => Err(()),
            },
            Command::SetComDeselectVoltage(voltage) => match voltage {
                0..=0x40 => ok_command!(arg_buf, 0xDB, [voltage]),
                _ => Err(()),
            },
            Command::SetContrastCurrent(current) => ok_command!(arg_buf, 0x81, [current]),
            Command::SetMuxRatio(ratio) => match ratio {
                16..=NUM_PIXEL_ROWS => ok_command!(arg_buf, 0xCA, [ratio - 1]),
                _ => Err(()),
            },
            Command::SetCommandLock(ena) => {
                let e = match ena {
                    true => 0x16,
                    false => 0x12,
                };
                ok_command!(arg_buf, 0xFD, [e])
            }
        }
        .map_err(|_| DisplayError::InvalidFormatError)?;

        if data.len() == 0 {
            iface.send_commands(U8(&[cmd]))?;
        } else {
            iface.send_commands(U8(&[cmd, data[0]]))?;
        }
        Ok(())
    }
}

impl Command {
    /// Transmit the command encoded by `self` to the display on interface `iface`.
    pub async fn async_send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: AsyncWriteOnlyDataCommand,
    {
        let mut arg_buf = [0u8; 2];
        let (cmd, data) = match self {
            Command::EnableGrayScaleTable => ok_command!(arg_buf, 0x00, []),
            Command::SetColumnAddress(address) => match address {
                0..=BUF_COL_MAX => {
                    ok_command!(arg_buf, 0x10 | (address >> 4), [0x00 | (address & 0x0F)])
                }
                _ => Err(()),
            },
            Command::SetHighColumnAddress(address) => match address {
                0..=0x7F => ok_command!(arg_buf, 0x10 + (address >> 4), []),
                _ => Err(()),
            },
            Command::SetLowColumnAddress(address) => match address {
                0..=0x7F => ok_command!(arg_buf, 0x00 + (address & 0x0F), []),
                _ => Err(()),
            },
            Command::SetRowAddress(address) => match address {
                0..=0xFF => ok_command!(arg_buf, 0xB0, [address]),
                _ => Err(()),
            },
            Command::SetSegmentRemap(remap) => match remap {
                DisplayRotation::Rotate0 => ok_command!(arg_buf, 0xA0 | 0x00, []),
                DisplayRotation::Rotate180 => ok_command!(arg_buf, 0xA0 | 0x01, []),
            },
            Command::SetScanDirection(direction) => match direction {
                0..=0x8 => ok_command!(arg_buf, 0xC0 | direction, []),
                _ => Err(()),
            },
            Command::SetMultiplexRatio(ratio) => match ratio {
                0x0F..=0x3F => ok_command!(arg_buf, 0xA8, [ratio]),
                _ => Err(()),
            },
            Command::SetDCDCSetting(dcdc) => {
                let dcdc_bytes = dcdc.into_bytes()[0];
                match dcdc_bytes {
                    0x00..=0xFF => {
                        ok_command!(arg_buf, 0xAD, [dcdc_bytes])
                    }
                    _ => Err(()),
                }
            }
            Command::SetRemapping(increment_axis, column_remap, nibble_remap, com_scan_direction, com_layout) => {
                let ia = match increment_axis {
                    IncrementAxis::Horizontal => 0x00,
                    IncrementAxis::Vertical => 0x01,
                };
                let cr = match column_remap {
                    ColumnRemap::Forward => 0x00,
                    ColumnRemap::Reverse => 0x02,
                };
                let nr = match nibble_remap {
                    NibbleRemap::Reverse => 0x00,
                    NibbleRemap::Forward => 0x04,
                };
                let csd = match com_scan_direction {
                    ComScanDirection::RowZeroFirst => 0x00,
                    ComScanDirection::RowZeroLast => 0x10,
                };
                let (interlace, dual_com) = match com_layout {
                    ComLayout::Progressive => (0x00, 0x01),
                    ComLayout::Interlaced => (0x20, 0x01),
                    ComLayout::DualProgressive => (0x00, 0x11),
                };
                ok_command!(arg_buf, 0xA0 | (ia | cr | nr | csd | interlace), [])
            }
            Command::SetStartLine(line) => match line {
                0..=PIXEL_ROW_MAX => ok_command!(arg_buf, 0x40 | line, []),
                _ => Err(()),
            },
            Command::SetDisplayOffset(line) => match line {
                0..=PIXEL_ROW_MAX => ok_command!(arg_buf, 0xD3, [line]),
                _ => Err(()),
            },
            Command::SetDisplayMode(mode) => ok_command!(
                arg_buf,
                match mode {
                    DisplayMode::BlankDark => 0xA4,
                    DisplayMode::BlankBright => 0xA5,
                    DisplayMode::Normal => 0xA6,
                    DisplayMode::Inverse => 0xA7,
                },
                []
            ),
            Command::EnablePartialDisplay(start, end) => match (start, end) {
                (0..=PIXEL_ROW_MAX, 0..=PIXEL_ROW_MAX) if start <= end => {
                    ok_command!(arg_buf, 0xA8, [start, end])
                }
                _ => Err(()),
            },
            Command::DisablePartialDisplay => ok_command!(arg_buf, 0xA9, []),
            Command::SetSleepMode(ena) => ok_command!(
                arg_buf,
                match ena {
                    true => 0xAE,
                    false => 0xAF,
                },
                []
            ),
            Command::SetPhaseLengths(phase_1, phase_2) => match (phase_1, phase_2) {
                (5..=31, 3..=15) => {
                    let p1 = (phase_1 - 1) >> 1;
                    let p2 = 0xF0 & (phase_2 << 4);
                    ok_command!(arg_buf, 0xB1, [p1 | p2])
                }
                _ => Err(()),
            },
            Command::SetDischargeLevel(level) => match level {
                0..=0x0F => ok_command!(arg_buf, 0x30 | level, []),
                _ => Err(()),
            },
            Command::SetClockDivider(divider) => match divider {
                0..=0xFF => ok_command!(arg_buf, 0xD5, [divider]),
                _ => Err(()),
            },
            Command::SetSecondPrechargePeriod(period) => match period {
                0..=0x40 => ok_command!(arg_buf, 0xD9, [period]),
                _ => Err(()),
            },
            Command::SetDefaultGrayScaleTable => ok_command!(arg_buf, 0xB9, []),
            Command::SetPreChargeVoltage(voltage) => match voltage {
                0..=0x50 => ok_command!(arg_buf, 0xDC, [voltage]),
                _ => Err(()),
            },
            Command::SetComDeselectVoltage(voltage) => match voltage {
                0..=0x40 => ok_command!(arg_buf, 0xDB, [voltage]),
                _ => Err(()),
            },
            Command::SetContrastCurrent(current) => ok_command!(arg_buf, 0x81, [current]),
            Command::SetMuxRatio(ratio) => match ratio {
                16..=NUM_PIXEL_ROWS => ok_command!(arg_buf, 0xCA, [ratio - 1]),
                _ => Err(()),
            },
            Command::SetCommandLock(ena) => {
                let e = match ena {
                    true => 0x16,
                    false => 0x12,
                };
                ok_command!(arg_buf, 0xFD, [e])
            }
        }
        .map_err(|_| DisplayError::InvalidFormatError)?;

        if data.len() == 0 {
            iface.send_commands(U8(&[cmd])).await?;
        } else {
            iface.send_commands(U8(&[cmd, data[0]])).await?;
        }
        Ok(())
    }
}

impl<'a> BufCommand<'a> {
    /// Transmit the command encoded by `self` to the display on interface `iface`.
    pub fn send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
    {
        let (cmd, data) = match self {
            BufCommand::SetGrayScaleTable(table) => {
                // Each element must be greater than the previous one, and all must be
                // between 0 and 180.
                let ok = table.len() == 15
                    && table[1..]
                        .iter()
                        .fold((true, 0), |(ok_so_far, prev), cur| (ok_so_far && prev < *cur && *cur <= 180, *cur))
                        .0
                    && table[0] <= table[1];
                if ok {
                    Ok((0xB8, table))
                } else {
                    Err(())
                }
            }
            BufCommand::WriteImageData(buf) => Ok((0x00, buf)),
        }
        .map_err(|_| DisplayError::InvalidFormatError)?;
        if cmd != 0x00 {
            iface.send_commands(U8(&[cmd]))?;
        }
        if data.len() != 0 {
            iface.send_data(U8(&data))?;
        }
        Ok(())
    }
}

impl<'a> BufCommand<'a> {
    /// Transmit the command encoded by `self` to the display on interface `iface`.
    pub async fn async_send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: AsyncWriteOnlyDataCommand,
    {
        let (cmd, data) = match self {
            BufCommand::SetGrayScaleTable(table) => {
                // Each element must be greater than the previous one, and all must be
                // between 0 and 180.
                let ok = table.len() == 15
                    && table[1..]
                        .iter()
                        .fold((true, 0), |(ok_so_far, prev), cur| (ok_so_far && prev < *cur && *cur <= 180, *cur))
                        .0
                    && table[0] <= table[1];
                if ok {
                    Ok((0xB8, table))
                } else {
                    Err(())
                }
            }
            BufCommand::WriteImageData(buf) => Ok((0x00, buf)),
        }
        .map_err(|_| DisplayError::InvalidFormatError)?;
        if cmd != 0x00 {
            iface.send_commands(U8(&[cmd])).await?;
        }
        if data.len() != 0 {
            iface.send_data(U8(&data)).await?;
        }
        Ok(())
    }
}
