use defmt::*;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{DynPublisher, PubSubChannel},
};
use embassy_time::{Duration, Timer};
use nrf_modem::{DtlsSocket, Error as NrfError, LteLink, PeerVerification};
use postcard::{from_bytes, to_vec};
use types::TxFrame;

static TX_CHANNEL: PubSubChannel<CriticalSectionRawMutex, TxFrame, 128, 1, 16> = PubSubChannel::new();

#[embassy_executor::task]
pub async fn task() {
    let mut tx_channel_sub = unwrap!(TX_CHANNEL.dyn_subscriber());
    let mut socket: Option<DtlsSocket> = None;
    loop {
        let frame = tx_channel_sub.next_message_pure().await;
        defmt::info!("tx_channel_sub recv {:?}", frame);
        if socket.is_none() {
            let hostname = env!("SEND_HOST");
            let port = env!("SEND_PORT").parse().unwrap();
            info!("connecting to {}:{}", hostname, port);
            match LteLink::new().await {
                Ok(link) => {
                    if let Err(err) = link.wait_for_link().await {
                        error!("lte link wait error {:?}", err);
                    }
                    info!("lte link result: {:?}", link);
                    match DtlsSocket::connect(hostname, port, PeerVerification::Disabled, &[0xC014]).await {
                        Ok(s) => {
                            info!("connected");
                            socket = Some(s);
                        }
                        Err(e) => {
                            error!("link socket connect error {:?}", e);
                            unwrap!(link.deactivate().await);
                        }
                    }
                }
                Err(e) => {
                    error!("lte link error {:?}", e);
                }
            }
        }
        if let Some(current_socket) = &mut socket {
            let frame = unwrap!(to_vec::<_, 512>(&frame));
            info!("sending {}", frame.len());
            match current_socket.send(&frame).await {
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
}

pub fn tx_channel_pub() -> DynPublisher<'static, TxFrame> {
    unwrap!(TX_CHANNEL.dyn_publisher())
}
