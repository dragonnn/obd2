#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;

use byte::TryWrite;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    Peri, bind_interrupts,
    peripherals::{self, UARTE0},
    radio, uarte,
    wdt::{Config, Watchdog, WatchdogHandle},
};
use embassy_time::Duration;
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use ieee802154::mac::FooterMode;
use ieee802154::mac::FrameSerDesContext;
use {defmt_rtt as _, panic_probe as _};

mod board;

#[global_allocator]
static HEAP: Heap = Heap::empty();

bind_interrupts!(struct Ieee802154Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});
bind_interrupts!(struct UartIrqs {
    UARTE0 => uarte::InterruptHandler<UARTE0>;
});

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
async fn main(_spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;

    let p = embassy_nrf::init(config);
    let board::Board {
        mut pins,
        radio,
        wdt,
    } = board::setup(p);
    let (mut wdg0, mut wdg1) = Wdg::new(wdt).await;
    // U13 (TPS3839) holds U11's CLR low for up to 350 ms after power-on.
    Timer::after_millis(400).await;
    pins.power.power_up().await;
    Timer::after_secs(20).await;
    pins.power.power_down().await;

    let mut ieee802154 = embassy_nrf::radio::ieee802154::Radio::new(radio, Ieee802154Irqs);
    ieee802154.set_channel(15);
    ieee802154.set_cca(radio::ieee802154::Cca::CarrierSense);
    info!("Radio initialized");

    loop {
        wdg0.pet().await;
        wdg1.pet().await;
        embassy_time::Timer::after_secs(1).await;
    }
}

pub trait TryIeee802154Send {
    fn try_send_buffer(
        &mut self,
        packet: &[u8],
        seq_number: &mut u8,
    ) -> impl core::future::Future<Output = Result<(), embassy_nrf::radio::Error>>;

    fn try_send_raw(
        &mut self,
        packet: &mut radio::ieee802154::Packet,
    ) -> impl core::future::Future<Output = Result<(), embassy_nrf::radio::Error>>;
}

impl TryIeee802154Send for embassy_nrf::radio::ieee802154::Radio<'_> {
    async fn try_send_raw(
        &mut self,
        tx_packet: &mut radio::ieee802154::Packet,
    ) -> Result<(), embassy_nrf::radio::Error> {
        self.try_send(tx_packet).await?;
        Ok(())
    }
    async fn try_send_buffer(
        &mut self,
        packet: &[u8],
        seq_number: &mut u8,
    ) -> Result<(), embassy_nrf::radio::Error> {
        let chunks = packet.chunks(100);
        let chunks_count = chunks.len();
        warn!("Chunks count: {}", chunks_count);
        for (c, chunk) in chunks.enumerate() {
            let frame = ieee802154::mac::Frame {
                header: ieee802154::mac::Header {
                    frame_type: ieee802154::mac::FrameType::Data,
                    frame_pending: false,
                    ack_request: false,
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
                    radio_tx_packet.copy_from_slice(&tx_packet[0..res]);
                    info!("radio_tx_packet.len(): {}", radio_tx_packet.len());
                    self.try_send_raw(&mut radio_tx_packet).await?;
                }
                Err(err) => {
                    error!("error writing frame: {:?}", defmt::Debug2Format(&err));
                }
            }
            *seq_number = seq_number.wrapping_add(1);
        }
        Ok(())
    }
}
