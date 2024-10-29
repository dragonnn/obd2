use defmt::{info, Format};
use embassy_futures::select::{select, Either::*};
use esp_hal::aes::{dma::AesDma, Aes, Mode};
use esp_ieee802154::{Config, Frame, Ieee802154};
use ieee802154::mac::{Address, FrameContent, FrameType, FrameVersion, Header, PanId, ShortAddress};
use serde::{Deserialize, Serialize};
use serde_encrypt::{serialize::impls::PostcardSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey};

use crate::event::event_bus_sub;

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
    loop {
        match select(send_ticker.next(), event_bus_sub.next_message_pure()).await {
            First(_) => {
                info!("send_ticker");
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
                    payload: heapless::Vec::from_slice(b"Hello World").unwrap(),
                    footer: [0u8; 2],
                });
                info!("result: {:?}", defmt::Debug2Format(&result));
                seq_number = seq_number.wrapping_add(1);
            }
            Second(event) => {
                info!("event_bus_sub: {:?}", event);
            }
        }
    }
}
