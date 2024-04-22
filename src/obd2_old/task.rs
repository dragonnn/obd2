use defmt::{error, info, unwrap, warn};
use embassy_time::Duration;
use embedded_can::{Frame as _, StandardId};

use crate::{
    mcp2515::{clock_8mhz, CanFrame, OperationMode, RxBuffer, TxBuffer, CANINTE, RXB0CTRL, RXB1CTRL, RXM},
    types::Mcp2515,
};

#[embassy_executor::task]
pub async fn run(mut mcp2515: Mcp2515) {
    let config = crate::mcp2515::Config::default()
        .mode(OperationMode::NormalOperation)
        .bitrate(clock_8mhz::CNF_500K_BPS)
        .receive_buffer_0(RXB0CTRL::default().with_rxm(RXM::ReceiveAny).with_bukt(true))
        .receive_buffer_1(RXB1CTRL::default().with_rxm(RXM::ReceiveAny));

    mcp2515.apply_config(&config).await.unwrap();

    let interputs_config = CANINTE::default().with_rx0ie(true).with_rx1ie(true);
    mcp2515.apply_interrupts_config(interputs_config).await.unwrap();
    let default_timeout = Duration::from_millis(4000);
    let mut timeout = Duration::from_millis(4000);
    let mut ff_cf_message_buffer: heapless::Vec<u8, 4095> = heapless::Vec::new();
    let mut ff_cf_message_length = None;
    let mut ff_cf_message_id = 0;

    info!("obd2 task started");
    loop {
        match embassy_time::with_timeout(timeout, mcp2515.interrupt()).await {
            Ok(_) => {
                //mcp2515.clear_interrupts().await.unwrap();
                let rx_status = mcp2515.rx_status().await.unwrap();
                let mut frames = [None, None];
                if rx_status.rx0if() {
                    let frame = mcp2515.read_rx_buffer(RxBuffer::RXB0).await.unwrap();
                    frames[0] = Some(frame);
                }
                if rx_status.rx1if() {
                    warn!("rx1if frame found");
                    let frame = mcp2515.read_rx_buffer(RxBuffer::RXB1).await.unwrap();
                    frames[1] = Some(frame);
                }
                let errors = mcp2515.errors().await.unwrap();
                if !errors.rx0ovr() {
                    error!("rx0ovr overflow");
                }
                if !errors.rx1ovr() {
                    error!("rx1ovr overflow");
                }

                for frame in frames.into_iter().flatten() {
                    let frame_type = frame.data[0] & 0xF0;
                    info!(
                        "frame.data[0]: {:#04x} from: {:?} type: {:#04x}",
                        frame.data,
                        defmt::Debug2Format(&frame.id()),
                        frame_type
                    );
                    let mut data = None;
                    match frame_type {
                        0x02 => {
                            info!("single frame: {}", frame.data);
                            data = Some(frame.data.as_slice());
                        }
                        0x10 => {
                            ff_cf_message_buffer.clear();
                            ff_cf_message_length =
                                Some(((frame.data[0] & 0x0F) as usize) << 8 | frame.data[1] as usize);
                            info!("first frame length: {}", ff_cf_message_length);
                            unwrap!(ff_cf_message_buffer.extend_from_slice(&frame.data));
                            ff_cf_message_id = 0;
                        }
                        0x30 => {
                            let timeout_ms = frame.data[2];
                            warn!("flow control frame with separation time: {}ms", timeout_ms);
                            if timeout_ms > 0 {
                                timeout = Duration::from_millis(timeout_ms as u64);
                            } else {
                                timeout = default_timeout;
                            }
                        }
                        0x20 => {
                            if let Some(ff_cf_message_length) = ff_cf_message_length {
                                let new_ff_cf_message_id = frame.data[0] & 0x0F;
                                info!(
                                    "consecutive frame: {} of length: {}",
                                    new_ff_cf_message_id, ff_cf_message_length
                                );
                                if new_ff_cf_message_id == ff_cf_message_id + 1 {
                                    unwrap!(ff_cf_message_buffer.extend_from_slice(&frame.data[1..]));
                                    if ff_cf_message_buffer.len() >= ff_cf_message_length {
                                        ff_cf_message_buffer.truncate(ff_cf_message_length);
                                        data = Some(ff_cf_message_buffer.as_slice());
                                    }
                                    ff_cf_message_id = new_ff_cf_message_id;
                                } else {
                                    error!(
                                        "consecutive frame id mismatch: {} != {}",
                                        new_ff_cf_message_id, ff_cf_message_id
                                    );
                                }
                            } else {
                                error!("no first frame");
                            }
                        }
                        _ => {
                            if frame.data[0] == 0x03 {
                                info!("single frame: {}", frame.data);
                                data = Some(frame.data.as_slice());
                            } else {
                                error!("unknown frame: {}", frame_type);
                            }
                        }
                    }
                    if let Some(data) = data {
                        warn!("processing obd frame: {:?} len: {:?}", data, data.len());
                    }
                }
            }
            Err(_) => {
                mcp2515.clear_interrupts().await.unwrap();
                let can_id = StandardId::new(0x7df).unwrap();
                //let data = [0x02, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00];
                let data = [0x02, 0x09, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
                let frame = CanFrame::new(can_id, &data).unwrap();
                info!("sending request frame: {:?}", frame);
                mcp2515.load_tx_buffer(TxBuffer::TXB0, &frame).await.unwrap();
                mcp2515.request_to_send(TxBuffer::TXB0).await.unwrap();
            }
        }
    }
}
