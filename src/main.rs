#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]

extern crate alloc;

use core::mem::MaybeUninit;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::entry;
use esp_hal_procmacros::main;

mod cap1188;
mod display;
mod hal;
mod mcp2515;
mod obd2;
mod state;
mod types;

#[global_allocator]
pub(crate) static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[main]
async fn main(spawner: Spawner) {
    init_heap();

    let hal = hal::init();

    spawner
        .spawn(display::task::run4(hal.display1, hal.display2, hal.buttons))
        .ok();
    spawner.spawn(obd2::run(hal.obd2)).ok();

    state::run().await;
}
