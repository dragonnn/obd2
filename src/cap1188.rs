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
    loop {
        let mut spi_buffer_read = [0x7F; 256];
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
        Timer::after_millis(5000).await;
    }
}
