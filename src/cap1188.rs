use display_interface_spi::SPIInterface;
use embassy_time::Delay;
use embassy_time::Timer;
use embedded_hal_async::spi::{Operation, SpiDevice};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{
    dma::Channel0,
    peripherals::SPI2,
    spi::{master::dma::SpiDma, FullDuplexMode},
};

#[embassy_executor::task]
pub async fn run(
    mut spi: ExclusiveDevice<
        SpiDma<'static, SPI2, Channel0, FullDuplexMode>,
        esp_hal::gpio::GpioPin<esp_hal::gpio::Output<esp_hal::gpio::PushPull>, 3>,
        NoDelay,
    >,
) {
    let mut cap1188 = Cap1188::new(spi);
    cap1188.init().await;
    loop {
        /*let mut spi_buffer_read = [0x7F; 256];
        spi_buffer_read.fill(0x7F);
        let mut spi_buffer = [0; 256];
        let mut spi_buffer_out = [0; 256];
        /*
        spi_buffer[0] = 0x7a;
        spi_buffer[1] = 0x7a;
        spi.transaction(&mut [Operation::Write(&spi_buffer[0..2])])
            .await
            .unwrap();
        */

        Timer::after_millis(100).await;
        spi_buffer[0] = 0x7D;
        spi_buffer[1] = 0xFD;
        spi_buffer[2] = 0x7F;

        /*buffer[0] = 0x7D;
        buffer[1] = reg;
        buffer[2] = 0x7F;
        spi_dev->write_then_read(buffer, 3, buffer, 1);*/
        spi_buffer_out[0..3].fill(0x7F);
        spi.transaction(&mut [
            Operation::Write(&spi_buffer[0..3]),
            Operation::TransferInPlace(&mut spi_buffer_out[0..3]),
        ])
        .await
        .unwrap();

        esp_println::println!("cap1188.rs: {:x?}", &spi_buffer_out[0..3]);

        esp_println::println!("cap1188.rs: Hello from embassy using esp-hal-async!");
        **/
        let touched = cap1188.touched().await.unwrap();
        if touched != 0 {
            esp_println::println!("cap1188.rs: Touched: {:x?}", touched);
        }
        Timer::after_millis(100).await;
    }
}

pub struct Cap1188<SPI> {
    spi: SPI,
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

impl<SPI> Cap1188<SPI>
where
    SPI: SpiDevice<u8>,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub async fn init(&mut self) -> Result<(), SPI::Error> {
        let mut prod_id = [0; 3];
        self.reset().await?;
        self.read_register(CAP1188_PRODID, &mut prod_id).await?;
        esp_println::println!("cap1188.rs: Product ID: {:x?}", &prod_id[0]);
        esp_println::println!("cap1188.rs: Manufacturer ID: {:x?}", &prod_id[1]);
        esp_println::println!("cap1188.rs: Revision: {:x?}", &prod_id[2]);
        if prod_id[0] != 0x50 {
            panic!("cap1188.rs: Invalid Product ID");
        }
        if prod_id[1] != 0x5d {
            panic!("cap1188.rs: Invalid Manufacturer");
        }
        if prod_id[2] != 0x83 {
            panic!("cap1188.rs: Revision");
        }

        /*
        allow multiple touches
        writeRegister(CAP1188_MTBLK, 0);
        Have LEDs follow touches
        writeRegister(CAP1188_LEDLINK, 0xFF);
        speed up a bit
        writeRegister(CAP1188_STANDBYCFG, 0x30);
        */

        self.write_register(CAP1188_MTBLK, &[0]).await?;
        self.write_register(CAP1188_LEDLINK, &[0xFF]).await?;
        self.write_register(CAP1188_STANDBYCFG, &[0x30]).await?;

        Ok(())
    }

    pub async fn touched(&mut self) -> Result<u8, SPI::Error> {
        let mut touched = [0; 1];
        self.read_register(CAP1188_SENINPUTSTATUS, &mut touched)
            .await?;
        /*
                  if (t) {
          writeRegister(CAP1188_MAIN, readRegister(CAP1188_MAIN) & ~CAP1188_MAIN_INT);
        }
              */
        if touched[0] != 0 {
            let mut main = [0; 1];
            self.read_register(CAP1188_MAIN, &mut main).await?;
            self.write_register(CAP1188_MAIN, &[main[0] & !CAP1188_MAIN_INT])
                .await?;
        }
        Ok(touched[0])
    }

    async fn read_register(&mut self, reg: u8, out_buf: &mut [u8]) -> Result<(), SPI::Error> {
        let init_buf = [0x7d, reg, 0x7f];

        out_buf.fill(0x7F);
        self.spi
            .transaction(&mut [
                Operation::Write(&init_buf),
                Operation::TransferInPlace(out_buf),
            ])
            .await
    }

    async fn write_register(&mut self, reg: u8, in_buf: &[u8]) -> Result<(), SPI::Error> {
        /*
                buffer[0] = 0x7D;
        buffer[1] = reg;
        buffer[2] = 0x7E;
        buffer[3] = value;
        spi_dev->write(buffer, 4);
             */
        /*let init_buf = [0x7d, reg, 0x7E];

        self.spi
            .transaction(&mut [Operation::Write(&init_buf), Operation::Write(in_buf)])
            .await*/

        let init_buf = [0x7d, reg, 0x7E, in_buf[0]];
        self.spi
            .transaction(&mut [Operation::Write(&init_buf)])
            .await
    }

    async fn reset(&mut self) -> Result<(), SPI::Error> {
        let mut buffer = [0; 2];

        buffer[0] = 0x7a;
        buffer[1] = 0x7a;

        self.spi.transaction(&mut [Operation::Write(&buffer)]).await
    }
}
