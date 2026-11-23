#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#[macro_use]
extern crate defmt;

extern crate alloc;

use byte::TryWrite;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    Peri, bind_interrupts, peripherals, radio,
    wdt::{Config, Watchdog, WatchdogHandle},
};
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap as Heap;
use ieee802154::mac::FooterMode;
use ieee802154::mac::FrameSerDesContext;
use {defmt_rtt as _, panic_probe as _};

mod board;
mod can;
mod state;

mod car;
mod lyra;

mod prelude;

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub struct Wdg(WatchdogHandle);

impl Wdg {
    pub async fn new(wdt: Peri<'static, peripherals::WDT>) -> (Self, Self) {
        let mut config = Config::default();

        #[cfg(not(debug_assertions))]
        {
            config.timeout_ticks = 32768 * 120;
            config.action_during_debug_halt = embassy_nrf::wdt::HaltConfig::Pause;
        }

        #[cfg(debug_assertions)]
        {
            config.timeout_ticks = 32768 * 120;
        }

        let (_wdt, [handle, handle2]) = match Watchdog::try_new(wdt, config) {
            Ok(x) => x,
            Err(_) => {
                defmt::error!(
                    "watchdog already active with wrong config, waiting for it to timeout..."
                );
                loop {
                    Timer::after(Duration::from_millis(100)).await;
                }
            }
        };

        (Self(handle), Self(handle2))
    }

    pub async fn pet(&mut self) {
        self.0.pet();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    config.dcdc.reg0 = false;
    config.dcdc.reg0_voltage = None;
    config.dcdc.reg1 = true;

    let p = embassy_nrf::init(config);
    info!("start board setup");
    let board::Board {
        mut pins,
        spi: _spi,
        mut can_listen,
        mut can_obd2,
        lyra_uart,
        ext_uart: _ext_uart,
        radio: _radio,
        wdt,
    } = board::setup(p).await;
    let (mut wdg0, mut wdg1) = Wdg::new(wdt).await;
    spawner.spawn(lyra::rpc::run(lyra_uart, spawner).unwrap());
    pins.lyra_power.power_up().await;
    Timer::after_secs(5).await;
    pins.lyra_power.power_down().await;

    /*let mut ieee802154 = embassy_nrf::radio::ieee802154::Radio::new(_radio, Ieee802154Irqs);
    ieee802154.set_channel(15);
    ieee802154.set_cca(radio::ieee802154::Cca::CarrierSense);*/
    info!("Main loop");
    //state::radio_task().await;

    spawner.spawn(car::ignition::run(pins.ignition).unwrap());
    spawner.spawn(car::state::run().unwrap());

    loop {
        wdg0.pet().await;
        wdg1.pet().await;
        embassy_time::Timer::after_secs(10).await;
        can_listen.idle().await;
        can_obd2.idle().await;
    }
}
