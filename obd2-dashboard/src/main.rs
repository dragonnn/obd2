#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;

use core::{mem::MaybeUninit, panic::PanicInfo};

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use esp_hal_embassy::main;
use panic_persist::{self as _, get_panic_message_utf8};

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
    let panic = get_panic_message_utf8();
    if let Some(msg) = panic {
        error!("Panic: {:?}", msg);
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;

    info!("init");
    hal.led.set_low();
    hal.can_listen.shutdown().await;
    spawner.spawn(tasks::lcd::run(hal.display1, hal.display2, panic)).ok();
    spawner.spawn(tasks::led::run(hal.led)).ok();
    spawner.spawn(tasks::buttons::run(hal.buttons)).ok();
    spawner.spawn(tasks::obd2::run(hal.obd2)).ok();
    spawner.spawn(tasks::power::run(hal.power)).ok();
    #[cfg(feature = "usb_serial")]
    spawner.spawn(tasks::usb::run(hal.usb_serial)).ok();
    spawner.spawn(tasks::ieee802154::run(hal.ieee802154, spawner)).ok();

    tasks::state::run(hal.rtc).await;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    riscv::interrupt::machine::disable();
    panic_persist::report_panic_info(info);
    unsafe { riscv::interrupt::machine::enable() };

    esp_hal::reset::software_reset();
    loop {}
}

#[no_mangle]
pub extern "Rust" fn custom_halt() -> ! {
    esp_hal::reset::software_reset();

    riscv::interrupt::machine::disable();
    loop {}
}
