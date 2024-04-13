//! embassy hello world
//!
//! This is an example of running the embassy executor with multiple tasks
//! concurrently.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]

use defmt::info;
use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Executor;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_backtrace as _;
use esp_hal::spi::master::prelude::_esp_hal_spi_master_dma_WithDmaSpi2;
//use esp_hal::spi::master::SpiBusController;
use esp_hal::{
    clock::ClockControl,
    dma::Dma,
    dma::DmaDescriptor,
    dma::DmaPriority,
    embassy,
    peripherals::Peripherals,
    prelude::*,
    riscv::singleton,
    spi::{
        master::{dma::SpiDma, Spi},
        FullDuplexMode, SpiMode,
    },
    Delay, IO,
};
use sh1122::{
    async_display::buffered_graphics::AsyncBufferedGraphicsMode, display::DisplayRotation,
    AsyncDisplay, PixelCoord,
};
use static_cell::{make_static, StaticCell};

use crate::cap1188::Cap1188;
use crate::mcp2515::Mcp2515;

mod cap1188;
mod display;
mod hal;
mod mcp2515;
mod obd2;
mod types;

#[embassy_executor::task]
async fn run1() {
    loop {
        esp_println::println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

pub type SpiType<'d> =
    SpiDma<'d, esp_hal::peripherals::SPI2, esp_hal::dma::Channel0, FullDuplexMode>;

#[entry]
fn main() -> ! {
    let hal = hal::init();

    let executor = make_static!(Executor::new());
    executor.run(|spawner| {
        spawner
            .spawn(display::task::run4(hal.display1, hal.display2, hal.buttons))
            .ok();
        spawner.spawn(obd2::run(hal.obd2)).ok();
    })
}
