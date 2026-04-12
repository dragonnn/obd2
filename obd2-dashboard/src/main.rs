#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trivial_bounds)]

extern crate alloc;

use core::{mem::MaybeUninit, panic::PanicInfo};

use defmt::{error, expect, info, unwrap};
#[cfg(feature = "defmt-brtt")]
use defmt_brtt as _;
#[cfg(not(feature = "defmt-brtt"))]
use defmt_rtt as _;
use embassy_executor::Spawner;
use esp_rtos::main;
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
mod prelude;
mod tasks;
mod types;

fn init_heap() {
    esp_alloc::heap_allocator!(size: 8 * 1024);
}

esp_bootloader_esp_idf::esp_app_desc!();

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

    #[cfg(not(feature = "xiao"))]
    {
        info!("running default config");
        spawner.spawn(tasks::obd2::run(hal.obd2).unwrap());
        spawner.spawn(tasks::temperature::run(hal.temperature).unwrap());
        spawner.spawn(tasks::lcd::run(hal.display1, hal.display2, panic).unwrap());
        spawner.spawn(tasks::led::run(hal.led).unwrap());
        spawner.spawn(tasks::buttons::run(hal.buttons).unwrap());
        spawner.spawn(tasks::can_listen::run(hal.can_listen).unwrap());
        spawner.spawn(tasks::power::run(hal.power).unwrap());
        spawner.spawn(tasks::ieee802154::run(hal.ieee802154, spawner).unwrap());
    }

    #[cfg(feature = "xiao")]
    {
        warn!("running xia config");
        spawner.spawn(tasks::obd2::run(hal.obd2).unwrap());
    }

    tasks::state::run(hal.rtc).await;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    riscv::interrupt::machine::disable();
    panic_persist::report_panic_info(info);
    unsafe { riscv::interrupt::machine::enable() };

    esp_hal::system::software_reset();
    loop {}
}

#[unsafe(no_mangle)]
pub extern "Rust" fn custom_halt() -> ! {
    esp_hal::system::software_reset();

    riscv::interrupt::machine::disable();
    loop {}
}
