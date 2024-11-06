#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use byte::TryRead;
use byte::TryWrite;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::uarte::UarteTx;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::{self, UARTE0},
    radio,
    uarte::{self, Uarte},
    wdt::{Config, Watchdog, WatchdogHandle},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;
use embassy_time::Duration;
use embassy_time::Timer;
use ieee802154::mac::FooterMode;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Ieee802154Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});
bind_interrupts!(struct UartIrqs {
    UARTE0_UART0 => uarte::InterruptHandler<UARTE0>;
});

pub struct Wdg(WatchdogHandle);

impl Wdg {
    pub async fn new(wdt: peripherals::WDT) -> (Self, Self) {
        let mut config = Config::default();

        #[cfg(not(debug_assertions))]
        {
            config.timeout_ticks = 32768 * 120;
            config.run_during_debug_halt = false;
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

    let p = embassy_nrf::init(config);
    let (mut wdg0, mut wdg1) = Wdg::new(p.WDT).await;

    let mut ieee802154 = embassy_nrf::radio::ieee802154::Radio::new(p.RADIO, Ieee802154Irqs);
    ieee802154.set_channel(15);
    ieee802154.set_cca(radio::ieee802154::Cca::CarrierSense);
    info!("Radio initialized");
    //txd - p0.19 -> MCU_IF7
    //rxd - p0.22 -> MCU_IF6
    let mut uart_config = uarte::Config::default();
    uart_config.baudrate = uarte::Baudrate::BAUD1M;
    uart_config.parity = uarte::Parity::INCLUDED;
    let (mut send, mut receive) = Uarte::new(
        p.UARTE0,
        UartIrqs,
        p.P0_22, //rxd
        p.P0_19, //txd
        uart_config,
    )
    .split_with_idle(p.TIMER0, p.PPI_CH0, p.PPI_CH1);

    //receive  - p0.25 -> MCU_IF5
    //send - p1.00 -> MCU_IF4
    let mut uarte_send = Output::new(p.P1_00, Level::Low, OutputDrive::Standard);
    let mut uarte_receive = Input::new(p.P0_25, Pull::Down);
    let mut ctx = ieee802154::mac::frame::FrameSerDesContext::no_security(FooterMode::None);

    let mut uarte_rx_packet: heapless::Vec<u8, 1024> = heapless::Vec::new();
    let mut uarte_send_channel_pub = UARTE_SEND_CHANNEL.sender();

    let mut current_chunk_count = 0;
    let mut current_chunk = 0;

    unwrap!(spawner.spawn(uarte_send_task(send, uarte_send, wdg1)));

    loop {
        let mut rx_packet = radio::ieee802154::Packet::new();
        let packet = ieee802154.receive(&mut rx_packet).await;
        wdg0.pet().await;
        match ieee802154.receive(&mut rx_packet).await {
            Ok(_) => {
                info!("Received packet: {:?}", rx_packet.lqi(),);
                match ieee802154::mac::Frame::try_read(&rx_packet, FooterMode::None) {
                    Ok((mut frame, size)) => {
                        info!("Frame: {:?} Payload: {:x}", frame, frame.payload);
                        frame.header.frame_type = ieee802154::mac::FrameType::Acknowledgement;
                        let ack_frame = ieee802154::mac::Frame {
                            header: frame.header,
                            payload: &[],
                            footer: frame.footer,
                            content: ieee802154::mac::FrameContent::Acknowledgement,
                        };
                        let mut ack_packet_bytes = [0; 256];
                        match ack_frame.try_write(&mut ack_packet_bytes, &mut ctx) {
                            Ok(ack_size) => {
                                let mut ack_packet = embassy_nrf::radio::ieee802154::Packet::new();
                                ack_packet.copy_from_slice(&ack_packet_bytes[0..ack_size]);
                                if ieee802154.try_send(&mut ack_packet).await.is_err() {
                                    error!("Send failed");
                                }
                            }
                            Err(err) => {
                                error!("Error writing ack frame: {:?}", defmt::Debug2Format(&err));
                            }
                        }

                        match frame.header.destination {
                            Some(ieee802154::mac::Address::Short(chunk_count, chunk)) => {
                                let chunk_count = chunk_count.0 as u8;
                                let chunk = chunk.0 as u8;
                                info!(
                                    "New Chunk count: {} Chunk: {} with payload: {}",
                                    chunk_count,
                                    chunk,
                                    frame.payload.len()
                                );
                                if current_chunk_count == 0 {
                                    current_chunk_count = chunk_count;
                                    current_chunk = chunk;
                                }

                                if current_chunk_count == chunk_count
                                    && (current_chunk == chunk || current_chunk + 1 == chunk)
                                {
                                    if let Err(err) =
                                        uarte_rx_packet.extend_from_slice(&frame.payload)
                                    {
                                        error!("Error extending uarte_rx_packet: {:?}", err);
                                        uarte_rx_packet.clear();
                                        current_chunk_count = 0;
                                    } else {
                                        current_chunk = chunk;
                                        if current_chunk_count == chunk + 1 {
                                            info!(
                                                "Got all chunks, sending to uarte_send_channel_pub"
                                            );
                                            uarte_send_channel_pub
                                                .send(uarte_rx_packet.clone())
                                                .await;
                                            uarte_rx_packet.clear();
                                            current_chunk_count = 0;
                                        }
                                    }
                                } else {
                                    error!(
                                        "Invalid chunk count or chunk, current: {} {} got: {} {}",
                                        current_chunk_count, current_chunk, chunk_count, chunk
                                    );
                                    uarte_rx_packet.clear();
                                    current_chunk_count = 0;
                                }
                            }
                            _ => {
                                error!("Invalid destination");
                            }
                        }
                    }
                    Err(err) => {
                        error!("Error reading frame: {:?}", defmt::Debug2Format(&err));
                    }
                }
            }
            Err(err) => {
                error!("Error receiving packet: {:?}", err);
            }
        }
    }
}

static UARTE_SEND_CHANNEL: channel::Channel<CriticalSectionRawMutex, heapless::Vec<u8, 1024>, 16> =
    channel::Channel::new();

#[embassy_executor::task]
pub async fn uarte_send_task(
    mut uarte_send: UarteTx<'static, UARTE0>,
    mut uarte_send_gpio: Output<'static>,
    mut wdg1: Wdg,
) {
    let uarte_send_channel_sub = UARTE_SEND_CHANNEL.receiver();
    loop {
        let data = uarte_send_channel_sub.receive().await;
        wdg1.pet().await;
        info!("Sending data: {=[u8]:a} with len: {}", data, data.len());
        uarte_send_gpio.set_high();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(2)).await;
        if let Err(err) = uarte_send.write(&data).await {
            error!("Error sending data: {:?}", err);
        }
        uarte_send_gpio.set_low();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(2)).await;
    }
}
