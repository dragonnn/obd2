use defmt::{error, info, unwrap, Format};
use embassy_futures::select::{select, Either::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{with_timeout, Duration};
use esp_hal::aes::{dma::AesDma, Aes, Mode};
use esp_ieee802154::{Config, Frame, Ieee802154, ReceivedFrame};
use ieee802154::mac::{Address, FrameContent, FrameType, FrameVersion, Header, PanId, ShortAddress};
use serde::{Deserialize, Serialize};
use serde_encrypt::{serialize::impls::PostcardSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey};
use static_cell::StaticCell;
use types::Pid;

use crate::event::{event_bus_sub, Event, KiaEvent};

#[embassy_executor::task]
pub async fn run(mut ieee802154: Ieee802154<'static>) {
    let shared_key_bytes = include_bytes!("../../../shared_key.bin");
    info!("shared_key_bytes: {:?}", shared_key_bytes);
    let shared_key: SharedKey = SharedKey::new(shared_key_bytes.clone());

    ieee802154.set_config(Config {
        channel: 15,
        promiscuous: true,
        pan_id: Some(0x4242),
        short_addr: Some(0x2222),
        cca_mode: esp_ieee802154::CcaMode::CarrierOrEd,
        txpower: 20,
        rx_when_idle: true,
        ..Default::default()
    });

    let mut send_ticker = embassy_time::Ticker::every(embassy_time::Duration::from_secs(60));
    let mut event_bus_sub = event_bus_sub();

    let mut obd2_pids: heapless::FnvIndexSet<Pid, 32> = heapless::FnvIndexSet::new();

    let mut ieee802154 = AsyncIeee802154::new(ieee802154);

    loop {
        match select(send_ticker.next(), event_bus_sub.next_message_pure()).await {
            First(_) => {
                for pid in obd2_pids.iter() {
                    info!("pid: {:?}", pid);
                    if let Some(encrypted_pid) = types::TxFrame::Obd2Pid(pid.clone()).encrypt(&shared_key).ok() {
                        let encrypted_pid_bytes = encrypted_pid.serialize();
                        if let Err(err) =
                            ieee802154.transmit_buffer(&encrypted_pid_bytes, 2, Duration::from_secs(5)).await
                        {
                            error!("ieee802154.transmit_buffer(&encrypted_pid_bytes, 2, Duration::from_secs(5)) failed: {:?}", err);
                        }
                    } else {
                        error!("types::TxFrame::Obd2Pid(pid.clone()).encrypt(&shared_key).ok() failed");
                    }
                    embassy_time::Timer::after(Duration::from_millis(5000)).await;
                }
            }
            Second(event) => {
                //info!("event_bus_sub: {:?}", event);
                match event {
                    Event::Kia(KiaEvent::Obd2Event(pid)) => {
                        if obd2_pids.insert(pid).is_err() {
                            info!("obd2_pids.insert(pid) failed");
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum AsyncIeee802154Error {
    Timeout,
    Ieee802154(esp_ieee802154::Error),
}

impl defmt::Format for AsyncIeee802154Error {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Self::Timeout => defmt::write!(f, "Timeout",),
            Self::Ieee802154(err) => defmt::write!(f, "Ieee802154({:?})", defmt::Debug2Format(err)),
        }
    }
}

impl From<esp_ieee802154::Error> for AsyncIeee802154Error {
    fn from(err: esp_ieee802154::Error) -> Self {
        Self::Ieee802154(err)
    }
}

pub struct AsyncIeee802154 {
    ieee802154: Ieee802154<'static>,
    tx_done_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    rx_available_signal: &'static Signal<CriticalSectionRawMutex, ()>,

    seq_number: u8,
}

impl AsyncIeee802154 {
    pub fn new(mut ieee802154: Ieee802154<'static>) -> Self {
        static TX_DONE_SIGNAL: embassy_sync::signal::Signal<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            (),
        > = embassy_sync::signal::Signal::new();
        static RX_AVAILABLE_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

        ieee802154.set_rx_available_callback_fn(|| {
            info!("ieee802154.set_rx_available_callback_fn");
            TX_DONE_SIGNAL.signal(());
        });
        ieee802154.set_tx_done_callback_fn(|| {
            info!("ieee802154.set_tx_done_callback_fn");
            RX_AVAILABLE_SIGNAL.signal(());
        });
        ieee802154.start_receive();

        Self { ieee802154, tx_done_signal: &TX_DONE_SIGNAL, rx_available_signal: &RX_AVAILABLE_SIGNAL, seq_number: 0 }
    }

    pub async fn transmit_buffer(
        &mut self,
        buffer: &[u8],
        retry: u8,
        timeout: Duration,
    ) -> Result<(), AsyncIeee802154Error> {
        let chunks = buffer.chunks(100);
        let chunks_count = chunks.len();
        for (c, chunk) in chunks.enumerate() {
            let frame = Frame {
                header: Header {
                    frame_type: FrameType::Data,
                    frame_pending: false,
                    ack_request: true,
                    pan_id_compress: false,
                    seq_no_suppress: false,
                    ie_present: false,
                    version: FrameVersion::Ieee802154_2003,
                    seq: self.seq_number,
                    destination: Some(Address::Short(PanId(chunks_count as u16), ShortAddress(c as u16))),
                    source: Some(Address::Short(PanId(0x2222), ShortAddress(0x2222))),
                    auxiliary_security_header: None,
                },
                content: FrameContent::Data,
                payload: unwrap!(heapless::Vec::from_slice(chunk)),
                footer: [0, 0],
            };
            self.transmit_raw(&frame, retry, timeout).await?;
            info!("transmit raw end");
            self.seq_number = self.seq_number.wrapping_add(1);
            if chunks_count > 1 {
                embassy_time::Timer::after(Duration::from_millis(10)).await;
            }
        }
        info!("transmit_buffer end");
        Ok(())
    }

    pub async fn transmit_raw(
        &mut self,
        frame: &Frame,
        retry: u8,
        timeout: Duration,
    ) -> Result<(), AsyncIeee802154Error> {
        for _ in 0..retry {
            self.tx_done_signal.reset();
            self.rx_available_signal.reset();
            self.ieee802154.transmit(frame)?;

            match with_timeout(timeout, async {
                self.tx_done_signal.wait().await;
                self.receive_raw().await
            })
            .await
            {
                Ok(Ok(response)) => {
                    if response.frame.header.frame_type == FrameType::Acknowledgement
                        && response.frame.header.destination == frame.header.destination
                    {
                        info!("raw transmit success");
                        return Ok(());
                    } else {
                        error!(
                            "unexpected response: {:?} expected: {:?}",
                            defmt::Debug2Format(&response),
                            frame.footer
                        );
                    }
                }
                Err(_) => {
                    error!("timeout transmitting frame: {:?}", defmt::Debug2Format(&frame));
                }
                Ok(Err(err)) => {
                    error!("error transmitting frame: {:?}", defmt::Debug2Format(&err));
                }
            }
        }
        Err(AsyncIeee802154Error::Timeout)
    }

    pub async fn receive_raw(&mut self) -> Result<ReceivedFrame, esp_ieee802154::Error> {
        self.rx_available_signal.wait().await;
        if let Some(frame) = self.ieee802154.get_received() {
            let frame = frame?;
            Ok(frame)
        } else {
            Err(esp_ieee802154::Error::Incomplete)
        }
    }
}
