#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]
#![feature(impl_trait_in_assoc_type)]

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

fn init_heap() {
    const HEAP_SIZE: usize = 8 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            HEAP.as_mut_ptr() as *mut u8,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }
}

#[main]
async fn main(spawner: Spawner) {
    info!("heap init");
    init_heap();
    info!("hal init");
    let mut hal = hal::init();
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;

    info!("init");
    hal.led.set_low();

    spawner.spawn(tasks::led::run(hal.led)).ok();
    spawner.spawn(tasks::buttons::run(hal.buttons)).ok();
    spawner.spawn(tasks::lcd::run(hal.display1, hal.display2)).ok();
    spawner.spawn(tasks::obd2::run(hal.obd2)).ok();
    spawner.spawn(tasks::power::run(hal.power)).ok();
    #[cfg(feature = "usb_serial")]
    spawner.spawn(tasks::usb::run(hal.usb_serial)).ok();
    spawner.spawn(tasks::ieee802154::run(hal.ieee802154)).ok();

    tasks::state::run().await;
}
