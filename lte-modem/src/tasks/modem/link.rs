use defmt::*;
use embassy_futures::select::{select, Either::*};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{DynPublisher, PubSubChannel},
};
use embassy_time::{with_timeout, Duration, Ticker, Timer};
use nrf_modem::{CancellationToken, DtlsSocket, Error as NrfError, LteLink, PeerVerification, UdpSocket};
use postcard::{from_bytes, from_bytes_crc32, to_vec, to_vec_crc32};
use types::{Modem, TxFrame};

static TX_CHANNEL: PubSubChannel<CriticalSectionRawMutex, TxFrame, 128, 1, 16> = PubSubChannel::new();

#[embassy_executor::task]
pub async fn task() {
    //let hostname = env!("SEND_HOST");
    //let port = env!("SEND_PORT").parse().unwrap();

    let mut tx_channel_sub = unwrap!(TX_CHANNEL.dyn_subscriber());
    let mut socket: Option<UdpSocket> = None;
    let mut timeout_ticker = Ticker::every(Duration::from_secs(60));
    let mut starting_port: u16 = 10000;
    loop {
        if starting_port < 10000 {
            starting_port = 10000;
        }
        match select(tx_channel_sub.next_message_pure(), timeout_ticker.next()).await {
            First(txframe) => {
                defmt::info!("tx_channel_sub recv {:?}", txframe);
                let is_modem_battery = txframe.is_modem_battery();
                info!("is_modem_battery: {:?}", is_modem_battery);
                if socket.is_none() && !is_modem_battery {
                    match with_timeout(
                        Duration::from_secs(30),
                        UdpSocket::bind(nrf_modem::no_std_net::SocketAddr::V4(
                            nrf_modem::no_std_net::SocketAddrV4::new(
                                nrf_modem::no_std_net::Ipv4Addr::new(0, 0, 0, 0),
                                starting_port,
                            ),
                        )),
                    )
                    .await
                    {
                        Ok(Ok(s)) => {
                            info!("connected");
                            s.tx_frame_send(&TxFrame::Modem(Modem::Connected)).await.ok();
                            Timer::after_secs(1).await;
                            timeout_ticker.reset();
                            socket = Some(s);
                            starting_port = starting_port.wrapping_add(1);
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
                    if !txframe.is_modem() {
                        timeout_ticker.reset();
                    }
                    match current_socket.tx_frame_send(&txframe).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("link socket send error {:?}", e);
                            socket = None;
                        }
                    }
                }
            }
            Second(_) => {
                error!("tx_channel_sub timeout");
                if let Some(socket) = &mut socket {
                    socket.tx_frame_send(&TxFrame::Modem(Modem::Disconnected)).await.ok();
                }
                if let Some(socket) = socket.take() {
                    embassy_time::Timer::after(Duration::from_secs(20)).await;
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

impl TxFrameSend for UdpSocket {
    async fn tx_frame_send(&self, frame: &TxFrame) -> Result<(), nrf_modem::Error> {
        match with_timeout(
            Duration::from_secs(15),
            self.send_to(
                &frame.to_vec_encrypted().map_err(|_| nrf_modem::Error::Utf8Error)?,
                nrf_modem::no_std_net::SocketAddrV4::new(nrf_modem::no_std_net::Ipv4Addr::new(185, 127, 22, 95), 49671)
                    .into(),
            ),
        )
        .await
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(nrf_modem::Error::Utf8Error),
        }
    }
}

pub fn tx_channel_pub() -> DynPublisher<'static, TxFrame> {
    unwrap!(TX_CHANNEL.dyn_publisher())
}
