use defmt::{error, Format};
use embedded_hal_async::spi::{Operation, SpiDevice};
use esp_hal::gpio::InputPin;
use modular_bitfield::prelude::*;

#[bitfield]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Format, Default)]
pub struct Cap1188Inputs {
    pub b0: bool,
    pub b1: bool,
    pub b2: bool,
    pub b3: bool,
    pub b4: bool,
    pub b5: bool,
    pub b6: bool,
    pub b7: bool,
}

pub struct Cap1188<SPI, INT> {
    spi: SPI,
    int: INT,
}

///< The Sensor Input Status Register stores status bits that indicate a
///< touch has been detected. A value of ‘0’ in any bit indicates that no
///< touch has been detected. A value of ‘1’ in any bit indicates that a
///< touch has been detected.
const CAP1188_SENINPUTSTATUS: u8 = 0x3;
//< Multiple Touch Configuration register controls the settings for the
///< multiple touch detection circuitry. These settings determine the
///< number of simultaneous buttons that may be pressed before additional
///< buttons are blocked and the MULT status bit is set. [0/1]
const CAP1188_MTBLK: u8 = 0x2A;

const CAP1188_LEDLINK: u8 = 0x72; // Sensor Input LED Linking. Controls linking of sensor inputs to LED channels
const CAP1188_PRODID: u8 = 0xFD; // Product ID. Stores a fixed value that identifies each product.
const CAP1188_MANUID: u8 = 0xFE; // Manufacturer ID. Stores a fixed value that identifies SMSC
const CAP1188_STANDBYCFG: u8 = 0x41; // Standby Configuration. Controls averaging and cycle time while in standby.
const CAP1188_REV: u8 = 0xFF; // Revision register. Stores an 8-bit value that represents the part revision.
const CAP1188_MAIN: u8 = 0x00; // Main Control register. Controls the primary power state of the device.
const CAP1188_MAIN_INT: u8 = 0x01; // Main Control Int register. Indicates that there is an interrupt.
const CAP1188_LEDPOL: u8 = 0x73; // LED Polarity. Controls the output polarity of LEDs.

impl<SPI, INT> Cap1188<SPI, INT>
where
    SPI: SpiDevice<u8>,
    INT: embedded_hal_async::digital::Wait,
{
    pub fn new(spi: SPI, int: INT) -> Self {
        Self { spi, int }
    }

    pub async fn init(&mut self) -> Result<(), SPI::Error> {
        let mut prod_id = [0; 3];

        self.read_register(CAP1188_PRODID, &mut prod_id).await?;
        if prod_id[0] != 0x50 {
            error!("cap1188.rs: Invalid Product ID");
        }
        if prod_id[1] != 0x5d {
            error!("cap1188.rs: Invalid Manufacturer");
        }
        if prod_id[2] != 0x83 {
            error!("cap1188.rs: Revision");
        }

        self.write_register(CAP1188_MTBLK, &[0]).await?;
        self.write_register(CAP1188_LEDLINK, &[0xFF]).await?;
        self.write_register(CAP1188_STANDBYCFG, &[0x30]).await?;

        Ok(())
    }

    pub async fn touched(&mut self) -> Result<Cap1188Inputs, SPI::Error> {
        let mut touched = [0; 1];
        self.read_register(CAP1188_SENINPUTSTATUS, &mut touched).await?;

        if touched[0] != 0 {
            let mut main = [0; 1];
            self.read_register(CAP1188_MAIN, &mut main).await?;
            self.write_register(CAP1188_MAIN, &[main[0] & !CAP1188_MAIN_INT]).await?;
        }

        Ok(Cap1188Inputs::from_bytes(touched))
    }

    async fn read_register(&mut self, reg: u8, out_buf: &mut [u8]) -> Result<(), SPI::Error> {
        let init_buf = [0x7d, reg, 0x7f];

        out_buf.fill(0x7F);
        self.spi.transaction(&mut [Operation::Write(&init_buf), Operation::TransferInPlace(out_buf)]).await
    }

    async fn write_register(&mut self, reg: u8, in_buf: &[u8]) -> Result<(), SPI::Error> {
        let init_buf = [0x7d, reg, 0x7E, in_buf[0]];
        self.spi.transaction(&mut [Operation::Write(&init_buf)]).await
    }

    async fn reset(&mut self) -> Result<(), SPI::Error> {
        let mut buffer = [0; 2];

        buffer[0] = 0x7a;
        buffer[1] = 0x7a;

        self.spi.transaction(&mut [Operation::Write(&buffer)]).await
    }

    pub async fn wait_for_touched(&mut self) {
        self.int.wait_for_low().await;
    }
}
