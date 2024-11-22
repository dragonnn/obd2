use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either::*};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{DynPublisher, DynSubscriber, PubSubChannel},
    signal::Signal,
};
use embassy_time::{with_timeout, Duration, Instant, Ticker, Timer};
use nrf_modem::{
    CancellationToken, DtlsSocket, Error as NrfError, LteLink, OwnedUdpReceiveSocket, OwnedUdpSendSocket,
    PeerVerification, UdpSocket,
};
use postcard::{from_bytes, from_bytes_crc32, to_vec, to_vec_crc32};
use types::{Modem, RxFrame, TxFrame, TxMessage};

use crate::board::Gnss;

static TX_CHANNEL: PubSubChannel<CriticalSectionRawMutex, TxFrame, 256, 1, 16> = PubSubChannel::new();
static RX_CHANNEL: PubSubChannel<CriticalSectionRawMutex, RxFrame, 256, 2, 16> = PubSubChannel::new();
static ACK_TIMEOUT: AtomicUsize = AtomicUsize::new(0);

static CONNECTED: AtomicBool = AtomicBool::new(false);
static DISCONNECT_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
#[embassy_executor::task]
pub async fn send_task(spawner: Spawner) {
    let ip: core::net::Ipv4Addr = env!("SEND_HOST").parse().unwrap();
    let port: u16 = env!("SEND_PORT").parse().unwrap();

    let mut tx_channel_sub = unwrap!(TX_CHANNEL.dyn_subscriber());
    let mut socket: Option<OwnedUdpSendSocket> = None;
    let mut timeout_ticker: Option<Ticker> = None;
    let mut starting_port: u16 = 10000;
    let mut rx_channel_sub = rx_channel_sub();
    loop {
        if starting_port < 10000 {
            starting_port = 10000;
        }
        let mut txframe_shutdown = false;

        match select(tx_channel_sub.next_message_pure(), async {
            if txframe_shutdown {
                txframe_shutdown = false;
            } else {
                if let Some(timeout_ticker) = &mut timeout_ticker {
                    timeout_ticker.next().await;
                } else {
                    futures::pending!()
                }
            }
        })
        .await
        {
            First(txframe) => {
                if let types::TxFrame::Modem(Modem::Reset) = txframe {
                    crate::tasks::reset::request_reset();
                }
                if let types::TxFrame::Shutdown = txframe {
                    txframe_shutdown = true;
                }
                info!("tx_channel_sub recv {:?}", txframe);
                let is_modem_battery = txframe.is_modem_battery();
                let txmessage = TxMessage::new(txframe);
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
                            let (socket_rx, socket_tx) = s.split_owned().await.unwrap();
                            info!("connected");
                            spawner.spawn(recv_task(socket_rx)).ok();
                            timeout_ticker = Some(Ticker::every(Duration::from_secs(120)));
                            socket_tx
                                .tx_frame_send(
                                    &TxMessage::new(TxFrame::Modem(Modem::Connected)),
                                    ip,
                                    port,
                                    &mut rx_channel_sub,
                                )
                                .await
                                .ok();
                            let battery = crate::tasks::battery::State::get().await;
                            socket_tx
                                .tx_frame_send(
                                    &TxMessage::new(TxFrame::Modem(battery.into())),
                                    ip,
                                    port,
                                    &mut rx_channel_sub,
                                )
                                .await
                                .ok();
                            Timer::after_millis(100).await;
                            socket = Some(socket_tx);
                            starting_port = starting_port.wrapping_add(1);
                            CONNECTED.store(true, Ordering::Relaxed);
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
                    if !txmessage.frame.is_modem() {
                        timeout_ticker.as_mut().map(|t| t.reset());
                    }
                    match current_socket.tx_frame_send(&txmessage, ip, port, &mut rx_channel_sub).await {
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
                    if let Some(gnss) = crate::tasks::gnss::State::get_current_fix().await {
                        socket
                            .tx_frame_send(
                                &TxMessage::new(TxFrame::Modem(Modem::GnssFix(gnss))),
                                ip,
                                port,
                                &mut rx_channel_sub,
                            )
                            .await
                            .ok();
                    }
                    socket
                        .tx_frame_send(
                            &TxMessage::new(TxFrame::Modem(Modem::Disconnected)),
                            ip,
                            port,
                            &mut rx_channel_sub,
                        )
                        .await
                        .ok();
                }
                if let Some(socket) = socket.take() {
                    embassy_time::Timer::after(Duration::from_secs(1)).await;
                    match with_timeout(Duration::from_secs(5), socket.deactivate()).await {
                        Ok(_) => {
                            info!("socket closed");
                        }
                        Err(e) => {
                            error!("link socket close error {:?}", e);
                        }
                    }
                    CONNECTED.store(false, Ordering::Relaxed);
                    DISCONNECT_SIGNAL.signal(());
                    timeout_ticker = None;
                }
            }
        }
    }
}

#[embassy_executor::task]
pub async fn recv_task(socket_rx: OwnedUdpReceiveSocket) {
    let mut rx_buf = [0; 512];
    let rx_pub = unwrap!(RX_CHANNEL.dyn_publisher());
    loop {
        match select(DISCONNECT_SIGNAL.wait(), socket_rx.receive_from(&mut rx_buf)).await {
            First(_) => break,
            Second(Ok((readed, _peer))) => {
                info!("got data: {:?} from peer", readed);
                match types::RxMessage::from_bytes_encrypted(&readed) {
                    Ok(rx_message) => {
                        info!("rx_message: {:?}", rx_message);
                        rx_pub.publish_immediate(rx_message.frame);
                    }
                    Err(_err) => {
                        error!("error decoding rx message");
                    }
                }
            }
            Second(Err(err)) => {
                error!("got socket_rx error: {:?}", err);
            }
        }
    }
    with_timeout(Duration::from_secs(5), socket_rx.deactivate()).await.ok();
    warn!("recv task exit");
}

trait TxMessageSend {
    async fn tx_frame_send(
        &self,
        message: &TxMessage,
        ip: core::net::Ipv4Addr,
        port: u16,
        rx: &mut RxChannelSub,
    ) -> Result<(), NrfError>;
}

impl TxMessageSend for OwnedUdpSendSocket {
    async fn tx_frame_send(
        &self,
        message: &TxMessage,
        ip: core::net::Ipv4Addr,
        port: u16,
        rx: &mut RxChannelSub,
    ) -> Result<(), nrf_modem::Error> {
        if ACK_TIMEOUT.load(Ordering::Relaxed) > 60 {
            crate::tasks::reset::request_reset();
        }

        if message.needs_ack() {
            rx.clear();
        }
        let ack_wait = Instant::now();
        loop {
            match with_timeout(
                Duration::from_secs(15),
                self.send_to(
                    &message.to_vec_encrypted().map_err(|_| nrf_modem::Error::Utf8Error)?,
                    (ip.octets(), port).into(),
                ),
            )
            .await
            {
                Ok(Ok(_)) => {
                    if message.needs_ack() {
                        loop {
                            match with_timeout(Duration::from_secs(15), rx.next_message_pure()).await {
                                Ok(rx_frame) => {
                                    if let types::RxFrame::TxFrameAck(ack_id) = rx_frame {
                                        if ack_id == message.id {
                                            info!("got ack id: {:?}", ack_id);
                                            return Ok(());
                                        }
                                    }
                                }
                                Err(_) => {
                                    ACK_TIMEOUT.fetch_add(1, Ordering::Relaxed);
                                    error!("ack timeout");
                                }
                            }
                            if ack_wait.elapsed() > Duration::from_secs(60) {
                                error!("ack timeout inside loop");
                                return Err(nrf_modem::Error::Utf8Error);
                            }
                        }
                    } else {
                        return Ok(());
                    }
                }
                Ok(Err(e)) => {
                    error!("send error {:?}", e);
                    return Err(nrf_modem::Error::Utf8Error);
                }
                Err(_) => {
                    error!("send timeout");
                    return Err(nrf_modem::Error::Utf8Error);
                }
            }
        }
    }
}

pub type TxChannelPub = DynPublisher<'static, TxFrame>;
pub type RxChannelSub = DynSubscriber<'static, RxFrame>;

pub fn tx_channel_pub() -> TxChannelPub {
    unwrap!(TX_CHANNEL.dyn_publisher())
}

pub fn rx_channel_sub() -> RxChannelSub {
    unwrap!(RX_CHANNEL.dyn_subscriber())
}

pub fn connected() -> bool {
    CONNECTED.load(Ordering::Relaxed)
}
