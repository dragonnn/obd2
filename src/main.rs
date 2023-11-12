//! embassy hello world
//!
//! This is an example of running the embassy executor with multiple tasks
//! concurrently.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]

use display_interface_spi::SPIInterface;
use embassy_executor::Executor;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp32c3_hal::spi::master::prelude::_esp_hal_spi_master_dma_WithDmaSpi2;
use esp32c3_hal::spi::master::SpiBusController;
use esp32c3_hal::{
    clock::ClockControl,
    dma::DmaPriority,
    embassy,
    gdma::Gdma,
    peripherals::Peripherals,
    prelude::*,
    riscv::singleton,
    spi::{
        master::{dma::SpiDma, Spi},
        FullDuplexMode, SpiMode,
    },
    Delay, IO,
};
use esp_backtrace as _;
use sh1122::{
    async_display::buffered_graphics::AsyncBufferedGraphicsMode, display::DisplayRotation,
    AsyncDisplay, PixelCoord,
};
use static_cell::make_static;

mod display;

#[embassy_executor::task]
async fn run1() {
    loop {
        esp_println::println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

pub type SpiType<'d> =
    SpiDma<'d, esp32c3_hal::peripherals::SPI2, esp32c3_hal::gdma::Channel0, FullDuplexMode>;

#[embassy_executor::task]
async fn run2(spi: &'static mut SpiType<'static>) {
    let send_buffer = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut buffer = [0; 8];
    esp_println::println!("Sending bytes");
    embedded_hal_async::spi::SpiBus::transfer(spi, &mut buffer, &send_buffer)
        .await
        .unwrap();

    loop {
        esp_println::println!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[entry]
fn main() -> ! {
    esp_println::println!("Init!");
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    //#[cfg(feature = "embassy-time-systick")]
    embassy::init(
        &clocks,
        esp32c3_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    esp32c3_hal::interrupt::enable(
        esp32c3_hal::peripherals::Interrupt::DMA_CH0,
        esp32c3_hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let sclk = io.pins.gpio6;
    let mosi = io.pins.gpio7;

    let descriptors = make_static!([0u32; 8 * 3]);
    let rx_descriptors = make_static!([0u32; 8 * 3]);

    let spi = Spi::new(peripherals.SPI2, 12u32.MHz(), SpiMode::Mode0, &clocks)
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_dma(dma_channel.configure(
            false,
            descriptors,
            rx_descriptors,
            DmaPriority::Priority0,
        ));

    let mut dc = io.pins.gpio9.into_push_pull_output();
    let mut cs1 = io.pins.gpio10.into_push_pull_output();
    let mut cs2 = io.pins.gpio1.into_push_pull_output();
    let mut rs = io.pins.gpio4.into_push_pull_output();

    let mut delay = Delay::new(&clocks);
    dc.set_high().unwrap();
    rs.set_low().unwrap();
    cs1.set_high().unwrap();
    cs2.set_high().unwrap();
    delay.delay_ms(5u32);
    rs.set_high().unwrap();

    delay.delay_ms(5u32);

    //let (interface1, interface2) = unsafe {
    //    let spi2 = core::ptr::read(&spi);
    //    let dc2 = core::ptr::read(&dc);
    //
    //    (SPIInterface::new(spi, dc), SPIInterface::new(spi2, dc2))
    //};
    //let spibus = SpiBusController::from_spi(spi);

    let spi2 = unsafe { core::ptr::read(&spi) };
    let dc2 = unsafe { core::ptr::read(&dc) };

    let spi_device = ExclusiveDevice::new(spi, cs1, NoDelay);
    let spi2_device = ExclusiveDevice::new(spi2, cs2, NoDelay);
    let interface1 = SPIInterface::new(spi_device, dc);
    let interface2 = SPIInterface::new(spi2_device, dc2);

    let display1 = AsyncDisplay::new(
        interface1,
        PixelCoord(256, 64),
        PixelCoord(0, 0),
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    let display2 = AsyncDisplay::new(
        interface2,
        PixelCoord(256, 64),
        PixelCoord(0, 0),
        DisplayRotation::Rotate180,
    )
    .into_buffered_graphics_mode();

    let executor = make_static!(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(display::task::run1(display1, display2)).ok();
        //spawner.spawn(run2(spi)).ok();
    })
}

/*impl<
        'a,
        A: embedded_hal_async::spi::ErrorType,
        B: esp32c3_hal::dma::ChannelTypes,
        C: esp32c3_hal::spi::IsFullDuplex,
    > embedded_hal_async::spi::SpiDevice for SpiDma<'a, A, B, C>
{
    async fn transaction(
        &mut self,
        operations: &mut [embedded_hal_async::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        todo!()
    }
}*/
