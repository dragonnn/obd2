#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]

extern crate alloc;

use core::mem::MaybeUninit;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use esp_hal::entry;
use esp_hal_procmacros::main;

mod cap1188;
//mod defmt_serial;
mod debug;
mod display;
mod event;
mod hal;
mod locks;
mod mcp2515;
mod obd2;
mod pid;
mod power;
mod tasks;
mod types;

#[global_allocator]
pub(crate) static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 8 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[main]
async fn main(spawner: Spawner) {
    info!("heap init");
    init_heap();
    info!("hal init");
    let hal = hal::init().await;
    //spawner.spawn(tasks::usb::run(hal.usb_serial)).ok();

    info!("init");
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    info!("init");

    //spawner.spawn(display::task::run4(hal.display1, hal.display2)).ok();
    //spawner.spawn(obd2::run(hal.obd2)).ok();

    spawner.spawn(tasks::led::run(hal.led)).ok();
    spawner.spawn(tasks::buttons::run(hal.buttons)).ok();
    spawner.spawn(tasks::lcd::run(hal.display1, hal.display2)).ok();
    //spawner.spawn(tasks::obd2::run(hal.obd2)).ok();
    //spawner.spawn(tasks::power::run(hal.power)).ok();

    tasks::state::run().await;
}
