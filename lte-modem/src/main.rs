#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![feature(async_closure)]
#![allow(clippy::uninlined_format_args)]
#![warn(clippy::large_futures)]
#![feature(impl_trait_in_assoc_type)]
extern crate alloc;
extern crate tinyrlibc;

//use core::panic::PanicInfo;

/*#[cfg(not(debug_assertions))]
#[inline(never)]
#[crate::panic_handler]
fn panic() -> ! {
    cortex_m::peripheral::SCB::sys_reset();
}*/
use defmt::*;
//use defmt_rtt as _;
use defmt_brtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap as Heap;
//use panic_probe as _;
use panic_persist as _;
use panic_persist::get_panic_message_utf8;
use tinyrlibc as _;

mod board;
mod config;
mod led;
mod tasks;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[derive(Debug, Format)]
pub enum ResetReason {
    Dog,
    Off,
    Dif,
    Sreq,
    LockUp,
    CtrlAp,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let logger = unwrap!(defmt_brtt::init());
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 16 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let mut reset_reasons: heapless::Vec<ResetReason, 6> = heapless::Vec::new();
    unsafe {
        let pac = nrf9160_pac::Peripherals::steal();
        let reset_reason = pac.POWER_NS.resetreas.read();
        if reset_reason.dog().bit_is_set() {
            reset_reasons.push(ResetReason::Dog).ok();
        }
        if reset_reason.off().bit_is_set() {
            reset_reasons.push(ResetReason::Off).ok();
        }
        if reset_reason.dif().bit_is_set() {
            reset_reasons.push(ResetReason::Dif).ok();
        }
        if reset_reason.sreq().bit_is_set() {
            reset_reasons.push(ResetReason::Sreq).ok();
        }
        if reset_reason.lockup().bit_is_set() {
            reset_reasons.push(ResetReason::LockUp).ok();
        }
        if reset_reason.ctrlap().bit_is_set() {
            reset_reasons.push(ResetReason::CtrlAp).ok();
        }
    }
    let panic_message = get_panic_message_utf8();
    if let Some(panic) = panic_message {
        defmt::error!("{}", panic);
        Timer::after(Duration::from_millis(500)).await;
    }
    defmt::info!("starting");

    let mut board = board::Board::new().await;
    board.buzzer.on();
    Timer::after(Duration::from_millis(200)).await;
    board.buzzer.off();
    defmt::info!("board initialized");

    Timer::after(Duration::from_secs(1)).await;

    let gnss = unwrap!(board.modem.gnss().await);

    let sense = unwrap!(board.sense.take());
    let lightwell = unwrap!(board.lightwell.take());
    let battery = unwrap!(board.battery.take());
    let low_power_accelerometer = unwrap!(board.low_power_accelerometer.take());
    let button = unwrap!(board.button.take());
    let wdg = unwrap!(board.wdg.take());
    let light_sensor = unwrap!(board.light_sensor.take());
    let uarte = unwrap!(board.uarte.take());
    let uarte_send = unwrap!(board.uarte_send.take());
    let uarte_receive = unwrap!(board.uarte_receive.take());
    let uarte_reset = unwrap!(board.uarte_reset.take());
    let charging_control = unwrap!(board.charging_control.take());
    //let uarte_tx_debug = unwrap!(board.uarte_tx_debug.take());
    let uarte_gnss = unwrap!(board.uarte_tx_gnss.take());
    let gnss_pss = unwrap!(board.gnss_pss.take());
    let gnss_force_on = unwrap!(board.gnss_force_on.take());

    //unwrap!(spawner.spawn(tasks::logger::task(logger, uarte_tx_debug, panic_message)));

    if let Some(panic) = panic_message {
        //if !panic.contains("twi reset") {
        board.modem.send_sms(crate::config::PANIC_SMS_NUMBERS, panic).await.ok();
        //}
    } else {
        if reset_reasons.len() > 0 {
            use core::fmt::Write;
            info!("reset reasons: {:?}", reset_reasons);
            let mut reset_reasons_str = heapless::String::<256>::new();
            core::write!(reset_reasons_str, "{:?}", reset_reasons).ok();
            info!("{}", reset_reasons_str);
            reset_reasons_str.pop();
            let reset_reasons_str = reset_reasons_str.trim_start_matches("[");
            board.modem.send_sms(crate::config::PANIC_SMS_NUMBERS, &reset_reasons_str).await.ok();
        }
    }

    defmt::info!("starting tasks");
    unwrap!(spawner.spawn(tasks::battery::task(battery, charging_control)));
    Timer::after(Duration::from_millis(100)).await;
    //unwrap!(spawner.spawn(tasks::gnss::task(gnss)));
    unwrap!(spawner.spawn(tasks::external_gnss::task(uarte_gnss, gnss_pss, gnss_force_on)));
    unwrap!(spawner.spawn(tasks::state::task(sense, lightwell, wdg, light_sensor)));
    unwrap!(spawner.spawn(tasks::montion_detection::task(low_power_accelerometer)));
    unwrap!(spawner.spawn(tasks::button::task(button)));
    unwrap!(spawner.spawn(tasks::reset::task()));
    tasks::uarte::run(&spawner, uarte, uarte_send, uarte_receive, uarte_reset);

    defmt::info!("entering main loop");

    tasks::modem::task(board.modem, &spawner).await;
}

//#[link_section = ".spm"]
//#[used]
//static SPM: [u8; 24052] = *include_bytes!("zephyr.bin");
