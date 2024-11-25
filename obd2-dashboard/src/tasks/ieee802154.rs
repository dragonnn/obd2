use defmt::{error, info, unwrap, warn, Format};
use embassy_futures::select::{select, select4, Either4::*};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
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
use types::Pid;

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
pub async fn run(mut ieee802154: Ieee802154<'static>) {
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

    let mut send_ticker = embassy_time::Ticker::every(embassy_time::Duration::from_secs(15));

    let mut ieee802154 = AsyncIeee802154::new(ieee802154);

    let _shutdown_guard = ShutdownGuard::new();
    let mut shutdown_signal = get_shutdown_signal();
    let mut extra_send_sub = unwrap!(EXTRA_SEND.subscriber());

    {
        //let reset_message =
        //    types::TxMessage::new(types::TxFrame::Modem(types::Modem::Reset)).to_vec_encrypted().unwrap();
        let ping_message = types::TxMessage::new(types::TxFrame::Modem(types::Modem::Ping)).to_vec_encrypted().unwrap();
        let pong_message = types::TxMessage::new(types::TxFrame::Modem(types::Modem::Pong)).to_vec_encrypted().unwrap();
        //ieee802154.transmit_buffer(&reset_message, 2, Duration::from_secs(10), true).await.ok();
        let bootup_start = Instant::now();
        loop {
            /*loop {
                if ieee802154.transmit_buffer(&pong_message, 2, Duration::from_secs(2), true).await.is_err() {
                    error!("pong send failed");
                }
            }*/
            if ieee802154.transmit_buffer(&ping_message, 2, Duration::from_secs(2), true).await.is_err() {
                error!("ping send failed");
            }
            info!("ping sent");
            match with_timeout(Duration::from_secs(5), ieee802154.receive()).await {
                Ok(Ok(types::RxFrame::Modem(types::Modem::Boot))) => {
                    info!("modem boot received");
                    break;
                }
                Ok(Ok(types::RxFrame::Modem(types::Modem::Pong))) => {
                    info!("modem pong received");
                    break;
                }
                Ok(Ok(rxmessage)) => {
                    info!("no modem bootup message received, got: {:?}", rxmessage);
                }
                Ok(Err(err)) => {
                    error!("ieee802154.receive() failed, err: {:?}", defmt::Debug2Format(&err));
                }
                Err(err) => {
                    error!("ieee802154.receive() timeout");
                }
            }
            if bootup_start.elapsed() > Duration::from_secs(500 * 120) {
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
            ieee802154.receive(),
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
                    //info!("pid: {:?}", pid);
                    if let Ok(encrypted_pid) =
                        types::TxMessage::new(types::TxFrame::Obd2Pid(pid.clone())).to_vec_encrypted()
                    {
                        if let Err(err) =
                            ieee802154.transmit_buffer(&encrypted_pid, 2, Duration::from_secs(2), true).await
                        {
                            error!("ieee802154.transmit_buffer(&encrypted_pid_bytes, 2, Duration::from_secs(5)) failed: {:?} {:?}", err, pid);
                        }
                    } else {
                        error!("types::TxFrame::Obd2Pid(pid.clone()).encrypt(&shared_key).ok() failed");
                    }
                    embassy_time::Timer::after(embassy_time::Duration::from_millis(25)).await;
                }
                if let Some(state) = &state {
                    warn!("sending extra state: {:?}", state);
                    if let Ok(encrypted_state) =
                        types::TxMessage::new(types::TxFrame::State(state.clone())).to_vec_encrypted()
                    {
                        if let Err(err) =
                            ieee802154.transmit_buffer(&encrypted_state, 2, Duration::from_secs(2), true).await
                        {
                            error!(
                                "ieee802154.transmit_buffer(&encrypted_state, 2, Duration::from_secs(5)) failed: {:?}",
                                err
                            );
                        }
                    }
                }
                info!("send_ticker elapsed: {:?}ms", now.elapsed().as_millis());
                send_ticker.reset();
            }
            Second(_) => {
                info!("ieee802154 shutdown");
                if let Ok(shutdown) = types::TxMessage::new(types::TxFrame::Shutdown).to_vec_encrypted() {
                    ieee802154.transmit_buffer(&shutdown, 2, Duration::from_secs(2), true).await.ok();
                }
                break;
            }
            Third(extra_txframe) => {
                match &extra_txframe {
                    types::TxFrame::State(new_state) => {
                        state = Some(new_state.clone());
                    }
                    _ => {}
                }

                if let Ok(encrypted_extra) = types::TxMessage::new(extra_txframe).to_vec_encrypted() {
                    if let Err(err) =
                        ieee802154.transmit_buffer(&encrypted_extra, 2, Duration::from_secs(1), true).await
                    {
                        error!("ieee802154.transmit_buffer(&encrypted_extra_bytes, 2, Duration::from_secs(5)) failed: {:?}", err);
                    }
                } else {
                    error!("types::TxFrame::Extra(extra_txframe).encrypt(&shared_key).ok() failed");
                }
            }
            Fourth(received_frame) => match received_frame {
                Ok(received_frame) => {
                    if let types::RxFrame::Modem(modem) = received_frame {
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
                Err(err) => {
                    error!("ieee802154.receive_raw() failed");
                }
            },
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

    received_frame_buffer: heapless::Vec<ReceivedFrame, 16>,

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
            info!("TX_DONE_SIGNAL.signal()");
            TX_DONE_SIGNAL.signal(());
        });
        ieee802154.set_tx_done_callback_fn(|| {
            RX_AVAILABLE_SIGNAL.signal(());
        });
        ieee802154.start_receive();

        Self {
            ieee802154,
            tx_done_signal: &TX_DONE_SIGNAL,
            rx_available_signal: &RX_AVAILABLE_SIGNAL,
            seq_number: 0,
            received_frame_buffer: heapless::Vec::new(),
        }
    }

    pub async fn transmit_buffer(
        &mut self,
        buffer: &[u8],
        retry: u8,
        timeout: Duration,
        ack: bool,
    ) -> Result<(), AsyncIeee802154Error> {
        warn!("transmit_buffer buffer.len(): {}", buffer.len());
        let elepsed = Instant::now();
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
            let elepsed_chunk = Instant::now();
            self.transmit_raw(&frame, retry, timeout, ack).await?;
            info!("transmit_raw chunk {} elapsed: {}ms", c, elepsed_chunk.elapsed().as_millis());
            self.seq_number = self.seq_number.wrapping_add(1);
        }
        info!("transmit_buffer elapsed: {}ms", elepsed.elapsed().as_millis());
        Ok(())
    }

    pub async fn transmit_raw(
        &mut self,
        frame: &Frame,
        total_retry: u8,
        timeout: Duration,
        ack: bool,
    ) -> Result<(), AsyncIeee802154Error> {
        for retry in 0..total_retry {
            self.tx_done_signal.reset();
            self.rx_available_signal.reset();
            self.ieee802154.transmit(frame)?;
            if with_timeout(timeout, self.tx_done_signal.wait()).await.is_err() {
                error!("timeout waiting for tx_done_signal on retry: {}", retry);
                continue;
            } else if retry > 0 {
                error!("tx_done_signal received on retry: {}", retry);
            }
            if !ack {
                return Ok(());
            }

            //for _ in 0..retry {
            match with_timeout(timeout / total_retry as u32, self.receive_raw(false)).await {
                Ok(Ok(response)) => {
                    if response.frame.header.frame_type == FrameType::Acknowledgement {
                        if response.frame.header.destination == frame.header.destination {
                            info!("acknowledgement received");
                            return Ok(());
                        } else {
                            warn!(
                                "unexpected ack frame {:?} expected: {:?}",
                                defmt::Debug2Format(&response.frame.header.destination),
                                defmt::Debug2Format(&frame.header.destination)
                            );
                        }
                    } else {
                        warn!("unexpected response expected, storing into buffer {:?}", defmt::Debug2Format(&response));
                        self.received_frame_buffer.push(response).ok();
                    }
                }
                Err(_err) => {
                    error!("timeout reciving ack frame");
                }
                Ok(Err(err)) => {
                    error!("error receiving frame: {:?}", defmt::Debug2Format(&err));
                }
            }
            //info!("retrying recive");
            //}
        }
        Err(AsyncIeee802154Error::Timeout)
    }

    pub async fn receive_raw(&mut self, use_buffer: bool) -> Result<ReceivedFrame, esp_ieee802154::Error> {
        if use_buffer {
            if let Some(frame) = self.received_frame_buffer.pop() {
                info!("frame from buffer");
                return Ok(frame);
            }
        }

        //let msg = self.ieee802154.get_received();
        //if let Some(Ok(frame)) = msg {
        //    info!("frame from get_received early on: {:?}", defmt::Debug2Format(&frame));
        //    return Ok(frame);
        //}
        loop {
            self.rx_available_signal.wait().await;
            if let Some(frame) = self.ieee802154.get_received() {
                let frame = frame?;
                return Ok(frame);
            } else {
                error!("get_received failed");
            }
        }
    }

    pub async fn receive(&mut self) -> Result<types::RxFrame, esp_ieee802154::Error> {
        loop {
            let received_frame = self.receive_raw(true).await?;
            info!(
                "received_frame: {:?} received_frame.frame.payload.len(): {}",
                defmt::Debug2Format(&received_frame),
                received_frame.frame.payload.len()
            );
            if received_frame.frame.payload.len() < 2 {
                error!("received_frame.frame.payload.len() < 2");
            } else {
                let decrypted_frame = types::RxMessage::from_bytes_encrypted(
                    &received_frame.frame.payload[0..received_frame.frame.payload.len() - 2],
                )
                .map_err(|e| {
                    error!("types::RxMessage::from_bytes_encrypted failed: {:?}", defmt::Debug2Format(&e));
                    esp_ieee802154::Error::BadInput
                })?;
                return Ok(decrypted_frame.frame);
            }
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
