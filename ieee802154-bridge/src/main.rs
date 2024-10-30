#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use byte::TryRead;
use byte::TryWrite;
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
    let mut ctx = ieee802154::mac::frame::FrameSerDesContext::no_security(FooterMode::None);

    loop {
        let mut rx_packet = radio::ieee802154::Packet::new();
        let packet = ieee802154.receive(&mut rx_packet).await;
        if packet.is_ok() {
            info!(
                "Received packet: {:?} {=[u8]:a}",
                rx_packet.lqi(),
                *rx_packet
            );
            let (mut frame, size) =
                ieee802154::mac::Frame::try_read(&rx_packet, FooterMode::Explicit).unwrap();
            info!("Frame: {:?} Payload: {=[u8]:a}", frame, frame.payload);
            frame.header.frame_type = ieee802154::mac::FrameType::Acknowledgement;
            let ack_frame = ieee802154::mac::Frame {
                header: frame.header,
                payload: &[],
                footer: frame.footer,
                content: ieee802154::mac::FrameContent::Acknowledgement,
            };
            let mut ack_packet_bytes = [0; 127];
            let ack_size = ack_frame
                .try_write(&mut ack_packet_bytes, &mut ctx)
                .unwrap();
            let mut ack_packet = embassy_nrf::radio::ieee802154::Packet::new();
            ack_packet.copy_from_slice(&ack_packet_bytes[0..ack_size]);
            if ieee802154.try_send(&mut ack_packet).await.is_err() {
                error!("Send failed");
            }

            uarte_send.set_high();
            send.write(&frame.payload).await.unwrap();
            uarte_send.set_low();
        } else {
            error!("Receive failed");
        }
    }
}
