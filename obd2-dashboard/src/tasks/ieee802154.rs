use defmt::{error, info, unwrap, warn, Format};
use embassy_executor::Spawner;
use embassy_futures::select::{select, select4, Either4::*};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::Channel,
    mutex::Mutex,
    pubsub::{DynPublisher, PubSubChannel},
    signal::Signal,
};
use embassy_time::{with_timeout, Duration, Instant};
use esp_hal::aes::{dma::AesDma, Aes, Mode};
use esp_ieee802154::{Config, Frame, Ieee802154, ReceivedFrame};
use ieee802154::mac::{Address, FrameContent, FrameType, FrameVersion, Header, PanId, ShortAddress};
use serde::{Deserialize, Serialize};
use serde_encrypt::{serialize::impls::PostcardSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey};
use static_cell::StaticCell;
use types::{MessageId, Pid, RxFrame, RxMessage, TxFrame, TxMessage};

use super::power::ShutdownGuard;
use crate::{
    event::{event_bus_sub, Event, KiaEvent},
    tasks::power::get_shutdown_signal,
};

static SEND_NOW_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static EXTRA_SEND: PubSubChannel<CriticalSectionRawMutex, types::TxFrame, 64, 1, 32> = PubSubChannel::new();
static PIDS_SEND: Mutex<CriticalSectionRawMutex, heapless::FnvIndexSet<Pid, 64>> =
    Mutex::new(heapless::FnvIndexSet::new());

#[embassy_executor::task]
pub async fn run(ieee802154: Ieee802154<'static>, spawner: Spawner) {
    spawner.must_spawn(ieee802154_run(ieee802154));

    let mut send_ticker = embassy_time::Ticker::every(embassy_time::Duration::from_secs(15));

    let _shutdown_guard = ShutdownGuard::new();
    let mut shutdown_signal = get_shutdown_signal();
    let mut extra_send_sub = unwrap!(EXTRA_SEND.subscriber());

    let txmessage_pub = IEEE802154_SEND.sender();
    let rxmessage_sub = IEEE802154_RECEIVE.receiver();

    {
        let bootup_start = Instant::now();
        loop {
            txmessage_pub.send(TxFrame::Modem(types::Modem::Ping).into()).await;
            info!("ping sent");
            match with_timeout(Duration::from_secs(10), rxmessage_sub.receive()).await {
                Ok(rxmessage) => match rxmessage.frame {
                    RxFrame::Modem(types::Modem::Boot) => {
                        info!("modem boot received");
                        break;
                    }
                    RxFrame::Modem(types::Modem::Pong) => {
                        info!("modem pong received");
                        break;
                    }
                    rxmessage => {
                        info!("no modem bootup message received, got: {:?}", rxmessage);
                    }
                },
                Err(_err) => {
                    error!("ieee802154.receive() timeout");
                }
            }
            if bootup_start.elapsed() > Duration::from_secs(60) {
                error!("modem bootup timeout");
                break;
            }
        }
    }

    let mut state: Option<types::State> = None;

    loop {
        match select4(
            async {
                select(send_ticker.next(), SEND_NOW_SIGNAL.wait()).await;
            },
            shutdown_signal.next_message_pure(),
            extra_send_sub.next_message_pure(),
            rxmessage_sub.receive(),
        )
        .await
        {
            First(_) => {
                let now = embassy_time::Instant::now();
                let obd2_pids;
                {
                    let mut obd2_pids_lock = PIDS_SEND.lock().await;
                    obd2_pids = obd2_pids_lock.clone();
                    obd2_pids_lock.clear();
                }

                for pid in obd2_pids.iter() {
                    txmessage_pub.send(TxFrame::Obd2Pid(pid.clone()).into()).await;
                    embassy_time::Timer::after(embassy_time::Duration::from_millis(25)).await;
                }
                if let Some(state) = &state {
                    warn!("sending extra state: {:?}", state);
                    txmessage_pub.send(TxFrame::State(state.clone()).into()).await;
                }
                info!("send_ticker elapsed: {:?}ms", now.elapsed().as_millis());
                send_ticker.reset();
            }
            Second(_) => {
                info!("ieee802154 shutdown");
                txmessage_pub.send(types::TxFrame::Shutdown.into()).await;
                break;
            }
            Third(extra_txframe) => {
                match &extra_txframe {
                    types::TxFrame::State(new_state) => {
                        state = Some(new_state.clone());
                    }
                    _ => {}
                }
                txmessage_pub.send(extra_txframe.into()).await;
            }
            Fourth(received_frame) => {
                if let types::RxFrame::Modem(modem) = received_frame.frame {
                    match modem {
                        types::Modem::Reset => info!("modem reset"),
                        types::Modem::GnssState(gnss_state) => info!("gnss_state: {:?}", gnss_state),
                        types::Modem::GnssFix(gnss_fix) => info!("gnss_fix: {:?}", gnss_fix),
                        types::Modem::Connected => info!("modem connected"),
                        types::Modem::Disconnected => info!("modem disconnected"),
                        types::Modem::Battery { voltage, low_voltage, soc, charging } => info!(
                            "battery: voltage: {:?} low_voltage: {:?} soc: {:?} charging: {:?}",
                            voltage, low_voltage, soc, charging
                        ),
                        types::Modem::Boot => info!("modem boot"),
                        types::Modem::Ping => info!("modem ping"),
                        types::Modem::Pong => info!("modem pong"),
                    }
                }
            }
        }
    }
}

static IEEE802154_SEND: Channel<CriticalSectionRawMutex, TxMessage, 64> = Channel::new();
static IEEE802154_RECEIVE: Channel<CriticalSectionRawMutex, RxMessage, 64> = Channel::new();

#[embassy_executor::task]
async fn ieee802154_run(mut ieee802154: Ieee802154<'static>) {
    use embassy_futures::select::{select3, Either3::*};
    ieee802154.set_config(Config {
        channel: 15,
        promiscuous: true,
        pan_id: Some(0x4242),
        short_addr: Some(0x2222),
        cca_mode: esp_ieee802154::CcaMode::Carrier,
        txpower: 20,
        rx_when_idle: true,
        auto_ack_tx: false,
        auto_ack_rx: false,
        ..Default::default()
    });

    let mut ieee802154 = AsyncIeee802154::new(ieee802154);
    let _shutdown_guard = ShutdownGuard::new();
    let mut shutdown_signal = get_shutdown_signal();

    let ieee802154_send_sub = IEEE802154_SEND.receiver();
    let ieee802154_receive_pub = IEEE802154_RECEIVE.sender();
    let local_timeout = Duration::from_secs(1);
    let remote_timeout = Duration::from_secs(10);
    loop {
        match select3(ieee802154.receive(), ieee802154_send_sub.receive(), shutdown_signal.next_message_pure()).await {
            First(rxmessage) => {
                info!("got rx message: {:?}", rxmessage);
                if ieee802154_receive_pub.is_full() {
                    warn!("ieee802154_receive_pub is full");
                    ieee802154_receive_pub.clear();
                }
                ieee802154_receive_pub.send(rxmessage).await;
            }
            Second(txmessage) => {
                let needs_ack = txmessage.needs_ack();
                if let Err(err) = ieee802154
                    .transmit_txmessage(txmessage, 5, if needs_ack { remote_timeout } else { local_timeout })
                    .await
                {
                    error!("ieee802154.transmit_txmessage failed: {:?}", err);
                }
            }
            Third(_) => {
                info!("ieee802154 shutdown");
                while let Ok(txmessage) = ieee802154_send_sub.try_receive() {
                    let needs_ack = txmessage.needs_ack();
                    if let Err(err) = ieee802154
                        .transmit_txmessage(txmessage, 5, if needs_ack { remote_timeout } else { local_timeout })
                        .await
                    {
                        error!("ieee802154.transmit_txmessage failed: {:?}", err);
                    }
                }
                break;
            }
        }
    }
}

#[derive(Debug)]
pub enum AsyncIeee802154Error {
    Timeout,
    Ieee802154(esp_ieee802154::Error),
    SerdeEncrypt,
}

impl defmt::Format for AsyncIeee802154Error {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Self::Timeout => defmt::write!(f, "Timeout",),
            Self::Ieee802154(err) => defmt::write!(f, "Ieee802154({:?})", defmt::Debug2Format(err)),
            Self::SerdeEncrypt => defmt::write!(f, "SerdeEncrypt",),
        }
    }
}

impl From<esp_ieee802154::Error> for AsyncIeee802154Error {
    fn from(err: esp_ieee802154::Error) -> Self {
        Self::Ieee802154(err)
    }
}

impl From<serde_encrypt::Error> for AsyncIeee802154Error {
    fn from(_err: serde_encrypt::Error) -> Self {
        Self::SerdeEncrypt
    }
}

pub struct AsyncIeee802154 {
    ieee802154: Ieee802154<'static>,
    tx_done_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    rx_available_signal: &'static Signal<CriticalSectionRawMutex, ()>,

    rxmessage_buffer: heapless::Vec<RxMessage, 16>,

    tx_seq_number: u8,
    rx_seq_number: u8,
}

impl AsyncIeee802154 {
    pub fn new(mut ieee802154: Ieee802154<'static>) -> Self {
        static TX_DONE_SIGNAL: embassy_sync::signal::Signal<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            (),
        > = embassy_sync::signal::Signal::new();
        static RX_AVAILABLE_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

        ieee802154.set_rx_available_callback_fn(|| {
            RX_AVAILABLE_SIGNAL.signal(());
        });
        ieee802154.set_tx_done_callback_fn(|| {
            TX_DONE_SIGNAL.signal(());
        });
        ieee802154.start_receive();

        Self {
            ieee802154,
            tx_done_signal: &TX_DONE_SIGNAL,
            rx_available_signal: &RX_AVAILABLE_SIGNAL,
            tx_seq_number: 0,
            rx_seq_number: 0,
            rxmessage_buffer: heapless::Vec::new(),
        }
    }

    pub async fn transmit_txmessage(
        &mut self,
        txmessage: TxMessage,
        retry: u8,
        timeout: Duration,
    ) -> Result<(), AsyncIeee802154Error> {
        let elapsed = Instant::now();
        let txmessage_id = txmessage.id;
        let buffer = txmessage.to_vec_encrypted()?;
        for current_retry in 0..retry {
            let chunks = buffer.chunks(100);
            let chunks_count = chunks.len();
            for (c, chunk) in chunks.enumerate() {
                let frame = Frame {
                    header: Header {
                        frame_type: FrameType::Data,
                        frame_pending: c == 0,
                        ack_request: true,
                        pan_id_compress: false,
                        seq_no_suppress: false,
                        ie_present: false,
                        version: FrameVersion::Ieee802154_2003,
                        seq: self.tx_seq_number,
                        destination: Some(Address::Short(PanId(chunks_count as u16), ShortAddress(c as u16))),
                        source: Some(Address::Short(PanId(0x2222), ShortAddress(0x2222))),
                        auxiliary_security_header: None,
                    },
                    content: FrameContent::Data,
                    payload: unwrap!(heapless::Vec::from_slice(chunk)),
                    footer: [0, 0],
                };
                if self.transmit_raw(&frame, Duration::from_millis(100)).await.is_err() {
                    error!("transmit_raw failed");
                }

                self.tx_seq_number = self.tx_seq_number.wrapping_add(1);
            }

            match with_timeout(timeout, async {
                loop {
                    let ack = self.receive_ack(txmessage_id).await;
                    if ack == txmessage_id {
                        if current_retry > 0 {
                            warn!("ack received after retry: {}", current_retry);
                        }
                        break;
                    } else {
                        error!("ack != txmessage_id");
                    }
                }
            })
            .await
            {
                Ok(ack) => {
                    info!("transmit_tx message ok, elapsed: {}ms", elapsed.elapsed().as_millis());
                    return Ok(());
                }
                Err(_) => {
                    error!("receive_ack timeout for txmessage: {:?}", txmessage);
                }
            }
        }
        error!("transmit_buffer elapsed: {}ms", elapsed.elapsed().as_millis());
        Err(AsyncIeee802154Error::Timeout)
    }

    async fn transmit_raw(&mut self, frame: &Frame, timeout: Duration) -> Result<(), AsyncIeee802154Error> {
        self.tx_done_signal.reset();
        //self.rx_available_signal.reset();
        self.ieee802154.transmit(frame)?;
        if with_timeout(timeout, self.tx_done_signal.wait()).await.is_err() {
            warn!("timeout waiting for tx_done_signal, timeout was: {}ms", timeout.as_millis());
        }

        Ok(())
    }

    fn frame_seq_number_check(&mut self, frame: &ReceivedFrame) -> bool {
        let new_rx_seq_number = frame.frame.header.seq;
        if new_rx_seq_number == self.rx_seq_number {
            warn!("frame with same seq number received");
            false
        } else if new_rx_seq_number == self.rx_seq_number.wrapping_add(1) {
            self.rx_seq_number = new_rx_seq_number;
            true
        } else if new_rx_seq_number == 0 && self.rx_seq_number == 0 {
            warn!("frame seq number both 0");
            true
        } else {
            warn!("frame seq number out of order, expected: {}, got: {}", self.rx_seq_number, new_rx_seq_number);
            self.rx_seq_number = new_rx_seq_number;
            true
        }
    }

    pub async fn receive_raw(&mut self) -> ReceivedFrame {
        let msg = self.ieee802154.received();
        if let Some(Ok(frame)) = msg {
            if self.frame_seq_number_check(&frame) {
                warn!("early frame return");
                return frame;
            }
        }
        loop {
            self.rx_available_signal.wait().await;
            /*let mut previous_frame = None;
            while let Some(Ok(frame)) = self.ieee802154.get_received() {
                previous_frame = Some(frame);
            }

            if let Some(frame) = previous_frame {
                if self.frame_seq_number_check(&frame) {
                    return frame;
                }
            }*/

            if let Some(Ok(frame)) = self.ieee802154.received() {
                if self.frame_seq_number_check(&frame) {
                    return frame;
                }
            } else {
                //error!("get_received failed");
            }
        }
    }

    pub async fn receive_ack(&mut self, txmessage_id: MessageId) -> types::MessageId {
        loop {
            let rxmessage = self.internal_receive().await;
            if let types::RxFrame::TxFrameAck(ack) = rxmessage.frame {
                if ack == txmessage_id {
                    return ack;
                } else {
                    warn!("ack != txmessage_id");
                    self.rxmessage_buffer.push(rxmessage).ok();
                }
            } else {
                warn!("no ack received_frame: {:?}", rxmessage);
                self.rxmessage_buffer.push(rxmessage).ok();
            }
        }
    }

    async fn internal_receive(&mut self) -> RxMessage {
        loop {
            let received_frame = self.receive_raw().await;
            if received_frame.frame.payload.len() < 2 {
                error!("received_frame.frame.payload.len() < 2");
            } else {
                match types::RxMessage::from_bytes_encrypted(
                    &received_frame.frame.payload[0..received_frame.frame.payload.len() - 2],
                ) {
                    Ok(rxmessage) => return rxmessage,
                    Err(err) => error!("RxMessage::from_bytes_encrypted failed: {:?}", defmt::Debug2Format(&err)),
                }
            }
        }
    }

    pub async fn receive(&mut self) -> RxMessage {
        loop {
            if let Some(rxmessage) = self.rxmessage_buffer.pop() {
                warn!("message from buffer: {:?}", rxmessage);
                return rxmessage;
            }
            return self.internal_receive().await;
        }
    }
}

pub fn send_now() {
    SEND_NOW_SIGNAL.signal(());
}

pub type TxFramePub = DynPublisher<'static, types::TxFrame>;

pub fn extra_txframes_pub() -> TxFramePub {
    unwrap!(EXTRA_SEND.dyn_publisher())
}

pub async fn insert_send_pid(pid: &Pid) {
    let mut pids_send = PIDS_SEND.lock().await;
    pids_send.remove(pid);
    pids_send.insert(pid.clone()).ok();
}

pub async fn clear_pids(pid: &Pid) {
    PIDS_SEND.lock().await.clear();
}
