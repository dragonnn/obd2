#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;

use byte::TryWrite;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    Peri, bind_interrupts,
    gpio::Output,
    peripherals, radio,
    spim::{self, Spim},
    uarte::Uarte,
    wdt::{Config, Watchdog, WatchdogHandle},
};
use embassy_time::{Duration, Ticker, Timer, with_timeout};
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

async fn lyra_uart_echo_test(uart: &mut Uarte<'_>) {
    let (tx, rx) = uart.split_by_ref();
    let mut ticker = Ticker::every(Duration::from_secs(1));

    for attempt in 1..=5 {
        let mut sent = *b"echo 0\r\n";
        sent[5] = b'0' + attempt;
        let mut received = [0; 8];
        info!("lyra_uart echo {}: TX {=[u8]:a}", attempt, sent);

        // Poll RX first so an immediate echo is captured during transmission.
        let (read_result, write_result) = join(
            with_timeout(Duration::from_millis(500), rx.read(&mut received)),
            with_timeout(Duration::from_millis(500), tx.write(&sent)),
        )
        .await;

        match (write_result, read_result) {
            (Ok(Ok(())), Ok(Ok(()))) if received == sent => {
                info!("lyra_uart echo {}: OK, RX {=[u8]:a}", attempt, received);
            }
            (Ok(Ok(())), Ok(Ok(()))) => {
                warn!(
                    "lyra_uart echo {}: mismatch, RX {=[u8]:a}",
                    attempt, received
                );
            }
            (tx, rx) => {
                warn!("lyra_uart echo {}: TX {:?}, RX {:?}", attempt, tx, rx);
            }
        }

        if attempt < 5 {
            ticker.next().await;
        }
    }
}

async fn mcp2518fd_read_register(
    spi: &mut Spim<'_>,
    cs: &mut Output<'_>,
    address: u16,
) -> Result<u32, spim::Error> {
    // READ instruction: 4-bit opcode 0b0011, then the 12-bit register address.
    let mut data = [
        0x30 | ((address >> 8) as u8 & 0x0f),
        address as u8,
        0,
        0,
        0,
        0,
    ];
    cs.set_low();
    Timer::after_micros(1).await;
    let result = spi.transfer_in_place(&mut data).await;
    cs.set_high();
    result?;
    Timer::after_micros(1).await;

    // Register bytes are read in increasing address order, low byte first.
    Ok(u32::from_le_bytes([data[2], data[3], data[4], data[5]]))
}

async fn mcp2518fd_spi_test<'d>(
    spi: &mut Spim<'_>,
    can1_cs: &mut Output<'d>,
    can2_cs: &mut Output<'d>,
) {
    // Both CS lines must be inactive before accessing either device.
    can1_cs.set_high();
    can2_cs.set_high();
    info!("MCP2518FD SPI test: 1 MHz, mode 0");

    for (can, cs) in [(1, can1_cs), (2, can2_cs)] {
        let devid = mcp2518fd_read_register(spi, cs, 0xE14).await;
        let osc = mcp2518fd_read_register(spi, cs, 0xE00).await;

        match (devid, osc) {
            (Ok(devid), Ok(osc)) => {
                info!(
                    "CAN{} MCP2518FD DEVID=0x{:08x}, ID=0x{:x}, REV=0x{:x}",
                    can,
                    devid,
                    (devid >> 4) & 0xf,
                    devid & 0xf
                );
                info!(
                    "CAN{} OSC=0x{:08x}, OSCRDY={}",
                    can,
                    osc,
                    osc & (1 << 10) != 0
                );

                // DEVID can legitimately be zero; use OSC to check the reply too.
                if devid & !0xff != 0 || osc & !0x157d != 0 {
                    warn!(
                        "CAN{} invalid reserved bits; check SPI wiring, CS, and power",
                        can
                    );
                } else if osc & (1 << 10) == 0 {
                    warn!(
                        "CAN{} oscillator not ready; check controller power, clock, and SPI",
                        can
                    );
                }
            }
            (devid, osc) => {
                warn!(
                    "CAN{} SPI read failed: DEVID {:?}, OSC {:?}",
                    can, devid, osc
                );
            }
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;

    let p = embassy_nrf::init(config);
    let board::Board {
        mut pins,
        mut spi,
        mut lyra_uart,
        ext_uart: _ext_uart,
        radio: _radio,
        wdt,
    } = board::setup(p);
    let (mut wdg0, mut wdg1) = Wdg::new(wdt).await;
    // U13 (TPS3839) holds U11's CLR low for up to 350 ms after power-on.
    Timer::after_millis(400).await;
    mcp2518fd_spi_test(&mut spi, &mut pins.cs1_can1, &mut pins.cs2_can2).await;
    pins.lyra_power.power_up().await;
    join(Timer::after_secs(5), lyra_uart_echo_test(&mut lyra_uart)).await;
    pins.lyra_power.power_down().await;

    /*let mut ieee802154 = embassy_nrf::radio::ieee802154::Radio::new(_radio, Ieee802154Irqs);
    ieee802154.set_channel(15);
    ieee802154.set_cca(radio::ieee802154::Cca::CarrierSense);*/
    info!("Main loop");

    loop {
        wdg0.pet().await;
        wdg1.pet().await;
        embassy_time::Timer::after_secs(5).await;
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
