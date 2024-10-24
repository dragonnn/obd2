#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use byte::TryRead;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::{self, UARTE0},
    radio,
    uarte::{self, Uarte},
};
use embassy_time::Timer;
use ieee802154::mac::FooterMode;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Ieee802154Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});
bind_interrupts!(struct UartIrqs {
    UARTE0_UART0 => uarte::InterruptHandler<UARTE0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;

    let p = embassy_nrf::init(config);

    let mut ieee802154 = embassy_nrf::radio::ieee802154::Radio::new(p.RADIO, Ieee802154Irqs);
    ieee802154.set_channel(15);
    ieee802154.set_cca(radio::ieee802154::Cca::CarrierSense);
    info!("Radio initialized");
    //txd - p0.19 -> MCU_IF7
    //rxd - p0.22 -> MCU_IF6
    let (mut send, mut receive) = Uarte::new(
        p.UARTE0,
        UartIrqs,
        p.P0_22, //rxd
        p.P0_19, //txd
        uarte::Config::default(),
    )
    .split_with_idle(p.TIMER0, p.PPI_CH0, p.PPI_CH1);

    //receive  - p0.25 -> MCU_IF5
    //send - p1.00 -> MCU_IF4
    let mut uarte_send = Output::new(p.P1_00, Level::Low, OutputDrive::Standard);
    let mut uarte_receive = Input::new(p.P0_25, Pull::Down);

    loop {
        let mut rx_packet = radio::ieee802154::Packet::new();
        let packet = ieee802154.receive(&mut rx_packet).await;
        if packet.is_ok() {
            info!(
                "Received packet: {:?} {=[u8]:a}",
                rx_packet.lqi(),
                *rx_packet
            );
            let mut frame = ieee802154::mac::Frame::try_read(&rx_packet, FooterMode::None).unwrap();
            info!("Frame: {:?} Payload: {=[u8]:a}", frame.0, frame.0.payload);
            uarte_send.set_high();
            send.write(&frame.0.payload).await.unwrap();
            uarte_send.set_low();
        } else {
            error!("Receive failed");
        }
    }
}
