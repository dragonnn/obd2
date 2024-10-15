#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![allow(clippy::uninlined_format_args)]
extern crate tinyrlibc;

//use core::panic::PanicInfo;

/*#[cfg(not(debug_assertions))]
#[inline(never)]
#[crate::panic_handler]
fn panic() -> ! {
    cortex_m::peripheral::SCB::sys_reset();
}*/
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
//use panic_probe as _;
use panic_persist as _;
use panic_persist::get_panic_message_utf8;
use tinyrlibc as _;

mod board;
mod config;
mod led;
mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let panic_message = get_panic_message_utf8();
    if let Some(panic) = panic_message {
        defmt::error!("{}", panic);
    }
    Timer::after(Duration::from_millis(500)).await;
    defmt::info!("starting");

    let mut board = board::Board::new().await;
    board.buzzer.on();
    Timer::after(Duration::from_millis(200)).await;
    board.buzzer.off();
    defmt::info!("board initialized");

    Timer::after(Duration::from_secs(1)).await;

    let gnss = board.modem.gnss().await.unwrap();

    let sense = board.sense.take().unwrap();
    let lightwell = board.lightwell.take().unwrap();
    let battery = board.battery.take().unwrap();
    let low_power_accelerometer = board.low_power_accelerometer.take().unwrap();
    let button = board.button.take().unwrap();
    let wdg = board.wdg.take().unwrap();
    let light_sensor = board.light_sensor.take().unwrap();

    if let Some(panic) = panic_message {
        if !panic.contains("twi reset") {
            board.modem.send_sms(crate::config::SMS_NUMBERS, panic).await.ok();
        }
    }

    defmt::info!("starting tasks");
    unwrap!(spawner.spawn(tasks::battery::task(battery)));
    Timer::after(Duration::from_millis(100)).await;
    unwrap!(spawner.spawn(tasks::gnss::task(gnss)));
    unwrap!(spawner.spawn(tasks::state::task(sense, lightwell, wdg, light_sensor)));
    unwrap!(spawner.spawn(tasks::montion_detection::task(low_power_accelerometer)));
    unwrap!(spawner.spawn(tasks::button::task(button)));

    defmt::info!("entering main loop");

    tasks::modem::task(board.modem).await;
}

//#[link_section = ".spm"]
//#[used]
//static SPM: [u8; 24052] = *include_bytes!("zephyr.bin");
