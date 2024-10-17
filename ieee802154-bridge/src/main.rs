#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals, radio,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;

    let p = embassy_nrf::init(config);

    let mut ieee802154 = embassy_nrf::radio::ieee802154::Radio::new(p.RADIO, Irqs);
    ieee802154.set_channel(15);
    ieee802154.set_cca(radio::ieee802154::Cca::CarrierSense);
    info!("Radio initialized");

    loop {
        let mut rx_packet = radio::ieee802154::Packet::new();
        let packet = ieee802154.receive(&mut rx_packet).await;
        if packet.is_ok() {
            info!(
                "Received packet: {:?} {=[u8]:a}",
                rx_packet.lqi(),
                *rx_packet
            );
        } else {
            error!("Receive failed");
        }
    }
}
