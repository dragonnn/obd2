use defmt::{error, info, unwrap, Format};
use embassy_futures::select::{select, Either::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use esp_hal::aes::{dma::AesDma, Aes, Mode};
use esp_ieee802154::{Config, Frame, Ieee802154};
use ieee802154::mac::{Address, FrameContent, FrameType, FrameVersion, Header, PanId, ShortAddress};
use serde::{Deserialize, Serialize};
use serde_encrypt::{serialize::impls::PostcardSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey};
use types::Pid;

use crate::event::{event_bus_sub, Event, KiaEvent};

#[embassy_executor::task]
pub async fn run(mut ieee802154: Ieee802154<'static>) {
    let shared_key_bytes = include_bytes!("../../../shared_key.bin");
    info!("shared_key_bytes: {:?}", shared_key_bytes);
    let shared_key: SharedKey = SharedKey::new(shared_key_bytes.clone());

    ieee802154.set_config(Config {
        channel: 15,
        promiscuous: false,
        pan_id: Some(0x4242),
        short_addr: Some(0x2222),
        cca_mode: esp_ieee802154::CcaMode::Carrier,
        txpower: 20,
        ..Default::default()
    });

    let mut seq_number = 0u8;
    let mut send_ticker = embassy_time::Ticker::every(embassy_time::Duration::from_secs(15));
    let mut event_bus_sub = event_bus_sub();

    let mut obd2_pids: heapless::FnvIndexSet<Pid, 32> = heapless::FnvIndexSet::new();
    static TX_DONE_SIGNAL: embassy_sync::signal::Signal<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        (),
    > = embassy_sync::signal::Signal::new();
    static RX_AVAILABLE_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
    ieee802154.set_tx_done_callback_fn(|| {
        info!("ieee802154.set_tx_done_callback_fn");
        TX_DONE_SIGNAL.signal(());
    });
    ieee802154.set_rx_available_callback_fn(|| {
        info!("ieee802154.set_rx_available_callback_fn");
        RX_AVAILABLE_SIGNAL.signal(());
    });
    loop {
        match select(send_ticker.next(), event_bus_sub.next_message_pure()).await {
            First(_) => {
                for pid in obd2_pids.iter() {
                    info!("pid: {:?}", pid);
                    if let Some(encrypted_pid) = types::TxFrame::Obd2Pid(pid.clone()).encrypt(&shared_key).ok() {
                        let encrypted_pid_bytes = encrypted_pid.serialize();
                        info!("encrypted_pid_bytes: {}", encrypted_pid_bytes.len());
                        let chunks = encrypted_pid_bytes.chunks(100);
                        let chunks_count = chunks.len();
                        for (c, chunk) in chunks.enumerate() {
                            let result = ieee802154.transmit(&Frame {
                                header: Header {
                                    frame_type: FrameType::Data,
                                    frame_pending: false,
                                    ack_request: true,
                                    pan_id_compress: false,
                                    seq_no_suppress: false,
                                    ie_present: false,
                                    version: FrameVersion::Ieee802154_2003,
                                    seq: seq_number,
                                    destination: Some(Address::Short(PanId(0x4242), ShortAddress(0x2222))),
                                    source: Some(Address::Short(PanId(0x4242), ShortAddress(0x2222))),
                                    auxiliary_security_header: None,
                                },
                                content: FrameContent::Data,
                                payload: unwrap!(heapless::Vec::from_slice(chunk)),
                                footer: [chunks_count as u8, c as u8],
                            });
                            info!("result: {:?}", defmt::Debug2Format(&result));
                            TX_DONE_SIGNAL.wait().await;
                            seq_number = seq_number.wrapping_add(1);
                        }
                    } else {
                        error!("types::TxFrame::Obd2Pid(pid.clone()).encrypt(&shared_key).ok() failed");
                    }
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
