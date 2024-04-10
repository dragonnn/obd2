use defmt::{error, info, unwrap};
use embassy_time::Duration;

use crate::{
    mcp2515::{clock_8mhz, OperationMode, RxBuffer, CANINTE, RXB0CTRL, RXB1CTRL, RXM},
    types::Mcp2515,
};

#[embassy_executor::task]
pub async fn run(mut mcp2515: Mcp2515) {
    let config = crate::mcp2515::Config::default()
        .mode(OperationMode::NormalOperation)
        .bitrate(clock_8mhz::CNF_500K_BPS)
        .receive_buffer_0(
            RXB0CTRL::default()
                .with_rxm(RXM::ReceiveAny)
                .with_bukt(true),
        )
        .receive_buffer_1(RXB1CTRL::default().with_rxm(RXM::ReceiveAny));

    mcp2515.apply_config(&config).await.unwrap();
    let interputs_config = CANINTE::default().with_rx0ie(true).with_rx1ie(true);

    error!("interrupts config: {:b}", interputs_config.into_bytes());
    mcp2515
        .apply_interrupts_config(interputs_config)
        .await
        .unwrap();
    let mut timeout = Duration::from_millis(40);
    let mut ff_cf_message_buffer: heapless::Vec<u8, 4095> = heapless::Vec::new();
    let mut ff_cf_message_length = None;
    loop {
        match embassy_time::with_timeout(timeout, mcp2515.interrupt()).await {
            Ok(interrupt) => {
                error!("interrupt: {:?}", interrupt);
                let rx_status = mcp2515.rx_status().await.unwrap();
                let mut frame = None;
                if rx_status.rx0if() {
                    frame = Some(mcp2515.read_rx_buffer(RxBuffer::RXB0).await.unwrap());
                }
                if rx_status.rx1if() {
                    frame = Some(mcp2515.read_rx_buffer(RxBuffer::RXB1).await.unwrap());
                }
                if let Some(frame) = frame {
                    let frame_type = frame.data[0] & 0xF0;
                    let mut data = None;
                    match frame_type {
                        0x02 => {
                            info!("single frame: {}", frame.data);
                            data = Some(&frame.data);
                        }
                        0x10 => {
                            ff_cf_message_buffer.clear();
                            ff_cf_message_length = Some(
                                ((frame.data[0] & 0x0F) as usize) << 8 | frame.data[1] as usize,
                            );
                            info!("first frame length: {}", ff_cf_message_length);
                            unwrap!(ff_cf_message_buffer.extend_from_slice(&frame.data));
                        }
                        0x30 => {
                            info!("flow control");
                        }
                        0x20 => {
                            info!("consecutive frame");
                            if let Some(ff_cf_message_length) = ff_cf_message_length {
                                unwrap!(ff_cf_message_buffer.extend_from_slice(&frame.data[1..]));
                                if ff_cf_message_buffer.len() >= ff_cf_message_length {
                                    ff_cf_message_buffer.truncate(ff_cf_message_length);
                                    info!("message: {:?}", ff_cf_message_buffer);
                                    data = Some(&ff_cf_message_buffer);
                                }
                            } else {
                                error!("no first frame");
                            }
                        }
                        _ => {
                            error!("unknown frame: {}", frame_type);
                        }
                    }
                }
            }
            Err(_) => {
                error!("timeout");
            }
        }
    }
}
