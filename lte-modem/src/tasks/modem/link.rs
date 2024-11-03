use defmt::*;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{DynPublisher, PubSubChannel},
};
use embassy_time::{with_timeout, Duration, Timer};
use nrf_modem::{DtlsSocket, Error as NrfError, LteLink, PeerVerification};
use postcard::{from_bytes, from_bytes_crc32, to_vec, to_vec_crc32};
use types::{Modem, TxFrame};

static TX_CHANNEL: PubSubChannel<CriticalSectionRawMutex, TxFrame, 128, 1, 16> = PubSubChannel::new();

#[embassy_executor::task]
pub async fn task() {
    let hostname = env!("SEND_HOST");
    let port = env!("SEND_PORT").parse().unwrap();

    let mut tx_channel_sub = unwrap!(TX_CHANNEL.dyn_subscriber());
    let mut socket: Option<DtlsSocket> = None;
    loop {
        match with_timeout(Duration::from_secs(5 * 60), tx_channel_sub.next_message_pure()).await {
            Ok(txframe) => {
                defmt::info!("tx_channel_sub recv {:?}", txframe);
                if socket.is_none() {
                    info!("connecting to {}:{}", hostname, port);

                    match with_timeout(
                        Duration::from_secs(120),
                        DtlsSocket::connect(hostname, port, PeerVerification::Disabled, &[0xC014]),
                    )
                    .await
                    {
                        Ok(Ok(s)) => {
                            info!("connected");
                            s.tx_frame_send(&TxFrame::Modem(Modem::Connected)).await.ok();
                            socket = Some(s);
                        }
                        Ok(Err(e)) => {
                            error!("link socket connect error {:?}", e);
                        }
                        Err(_) => {
                            error!("link socket connect timeout");
                        }
                    }
                }
                if let Some(current_socket) = &mut socket {
                    match current_socket.tx_frame_send(&txframe).await {
                        Ok(_) => {
                            info!("sent");
                        }
                        Err(e) => {
                            error!("link socket send error {:?}", e);
                            socket = None;
                        }
                    }
                }
            }
            Err(_) => {
                error!("tx_channel_sub timeout");
                if let Some(socket) = socket.take() {
                    socket.tx_frame_send(&TxFrame::Modem(Modem::Disconnected)).await.ok();
                    match socket.deactivate().await {
                        Ok(_) => {
                            info!("socket closed");
                        }
                        Err(e) => {
                            error!("link socket close error {:?}", e);
                        }
                    }
                }
            }
        }
    }
}

trait TxFrameSend {
    async fn tx_frame_send(&self, frame: &TxFrame) -> Result<(), NrfError>;
}

impl TxFrameSend for DtlsSocket {
    async fn tx_frame_send(&self, frame: &TxFrame) -> Result<(), NrfError> {
        let frame = unwrap!(to_vec::<_, 512>(&frame));
        info!("sending {}", frame.len());
        self.send(&frame).await
    }
}

pub fn tx_channel_pub() -> DynPublisher<'static, TxFrame> {
    unwrap!(TX_CHANNEL.dyn_publisher())
}
