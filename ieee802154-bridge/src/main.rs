#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use byte::TryRead;
use byte::TryWrite;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either::*};
use embassy_nrf::peripherals::TIMER0;
use embassy_nrf::uarte::UarteRxWithIdle;
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
use ieee802154::mac::FrameSerDesContext;
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

    let mut uarte_rx_packet: heapless::Vec<u8, 512> = heapless::Vec::new();
    let mut uarte_send_channel_pub = UARTE_SEND_CHANNEL.sender();

    let mut current_chunk_count = 0;
    let mut current_chunk = 0;

    let mut rx_packet_seq = 0;

    unwrap!(spawner.spawn(uarte_send_task(send, uarte_send, wdg1)));
    unwrap!(spawner.spawn(uarte_receive_task(receive, uarte_receive)));
    let mut last_message_time = embassy_time::Instant::now();

    loop {
        let mut rx_packet = radio::ieee802154::Packet::new();
        match select(
            ieee802154.receive(&mut rx_packet),
            UARTE_RECEIVE_CHANNEL.receive(),
        )
        .await
        {
            First(ieee802154_result) => {
                if ieee802154_result.is_ok() {
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
                            let mut ack_packet_bytes = [0; 127];
                            let ack_size = ack_frame
                                .try_write(&mut ack_packet_bytes, &mut ctx)
                                .unwrap();
                            let mut ack_packet = embassy_nrf::radio::ieee802154::Packet::new();
                            ack_packet.copy_from_slice(&ack_packet_bytes[0..ack_size]);
                            if ieee802154.try_send(&mut ack_packet).await.is_err() {
                                error!("Send failed");
                            } else {
                                info!("Sent ack");
                                last_message_time = embassy_time::Instant::now();
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
                                                info!("Got all chunks, sending to uarte_send_channel_pub");
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
                } else {
                    error!("Receive failed");
                }
            }
            Second(uarte_result) => {
                info!("Received data from uarte: {=[u8]:a}", uarte_result);
                if last_message_time.elapsed().as_millis() < 50 {
                    embassy_time::Timer::after_millis(50).await;
                }
                if let Err(err) = ieee802154
                    .try_send_buffer(&uarte_result, &mut rx_packet_seq)
                    .await
                {
                    error!("Error sending packet: {:?}", err);
                } else {
                    last_message_time = embassy_time::Instant::now();
                }
            }
        }

        wdg0.pet().await;
    }
}

pub trait TryIeee802154Send {
    async fn try_send_buffer(
        &mut self,
        packet: &[u8],
        seq_number: &mut u8,
    ) -> Result<(), embassy_nrf::radio::Error>;

    async fn try_send_raw(
        &mut self,
        header: &ieee802154::mac::Header,
        packet: &mut radio::ieee802154::Packet,
    ) -> Result<(), embassy_nrf::radio::Error>;
}

impl TryIeee802154Send for embassy_nrf::radio::ieee802154::Radio<'_, peripherals::RADIO> {
    async fn try_send_raw(
        &mut self,
        header: &ieee802154::mac::Header,
        tx_packet: &mut radio::ieee802154::Packet,
    ) -> Result<(), embassy_nrf::radio::Error> {
        for _ in 0..5 {
            self.try_send(tx_packet).await?;
            let mut rx_packet = radio::ieee802154::Packet::new();
            if let Ok(_) = self.receive(&mut rx_packet).await {
                match ieee802154::mac::Frame::try_read(&rx_packet, FooterMode::None) {
                    Ok((frame, _)) => {
                        if frame.header.frame_type == ieee802154::mac::FrameType::Acknowledgement
                            && header.destination == frame.header.destination
                        {
                            return Ok(());
                        }
                    }
                    Err(err) => {
                        error!("Error reading frame: {:?}", defmt::Debug2Format(&err));
                    }
                }
            }
        }
        Ok(())
    }
    async fn try_send_buffer(
        &mut self,
        packet: &[u8],
        seq_number: &mut u8,
    ) -> Result<(), embassy_nrf::radio::Error> {
        let chunks = packet.chunks(100);
        let chunks_count = chunks.len();
        info!("Chunks count: {}", chunks_count);
        for (c, chunk) in chunks.enumerate() {
            let frame = ieee802154::mac::Frame {
                header: ieee802154::mac::Header {
                    frame_type: ieee802154::mac::FrameType::Data,
                    frame_pending: false,
                    ack_request: true,
                    pan_id_compress: false,
                    seq_no_suppress: false,
                    ie_present: false,
                    version: ieee802154::mac::FrameVersion::Ieee802154_2003,
                    seq: *seq_number,
                    destination: Some(ieee802154::mac::Address::Short(
                        ieee802154::mac::PanId(chunks_count as u16),
                        ieee802154::mac::ShortAddress(c as u16),
                    )),
                    source: Some(ieee802154::mac::Address::Short(
                        ieee802154::mac::PanId(0x2223),
                        ieee802154::mac::ShortAddress(0x2223),
                    )),
                    auxiliary_security_header: None,
                },
                payload: chunk,
                footer: [0, 0],
                content: ieee802154::mac::FrameContent::Data,
            };
            let mut radio_tx_packet = radio::ieee802154::Packet::new();
            let mut tx_packet = [0; 256];
            match frame.try_write(
                &mut tx_packet,
                &mut FrameSerDesContext::no_security(FooterMode::Explicit),
            ) {
                Ok(res) => {
                    info!("result: {}", res);

                    radio_tx_packet.copy_from_slice(&tx_packet[0..res]);
                    info!("radio_tx_packet.len(): {}", radio_tx_packet.len());
                    self.try_send_raw(&frame.header, &mut radio_tx_packet)
                        .await?;
                }
                Err(err) => {
                    error!("Error writing frame: {:?}", defmt::Debug2Format(&err));
                }
            }
            *seq_number = seq_number.wrapping_add(1);
        }
        Ok(())
    }
}

static UARTE_SEND_CHANNEL: channel::Channel<CriticalSectionRawMutex, heapless::Vec<u8, 512>, 128> =
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

static UARTE_RECEIVE_CHANNEL: channel::Channel<
    CriticalSectionRawMutex,
    heapless::Vec<u8, 1024>,
    16,
> = channel::Channel::new();

#[embassy_executor::task]
pub async fn uarte_receive_task(
    mut uarte_receive: UarteRxWithIdle<'static, UARTE0, TIMER0>,
    mut uarte_receive_gpio: Input<'static>,
) {
    let mut buffer = [0u8; 1024];
    loop {
        uarte_receive_gpio.wait_for_high().await;

        match uarte_receive.read_until_idle(&mut buffer).await {
            Ok(size) => {
                let data = &buffer[..size];
                warn!("Received data: {=[u8]:a} with len: {}", data, size);
                UARTE_RECEIVE_CHANNEL
                    .send(unwrap!(heapless::Vec::from_slice(data)))
                    .await;
            }
            Err(err) => {
                error!("Error reading data: {:?}", err);
            }
        }
    }
}
