use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration as TimeDuration, Instant, Timer, with_timeout};
use hsmc::{Duration, statechart};
use lyra_protocol::{ControlId, EchoProbeTopic, ShutdownRequestTopic};
use postcard_rpc::header::VarSeq;

use crate::board::LyraPower;

use super::rpc::RpcSender;

const PROBE_INTERVAL: Duration = Duration::from_secs(1);
const BOOT_TIMEOUT: Duration = Duration::from_secs(60);
const RETRY_DELAY: Duration = Duration::from_secs(5);
const MISSED_ECHO_LIMIT: u8 = 3;
const TX_TIMEOUT: TimeDuration = TimeDuration::from_millis(100);
const SHUTDOWN_TIMEOUT: TimeDuration = TimeDuration::from_secs(30);
// The power latch sequence takes at least 15 ms; reserve time for it.
const POWER_CUT_MARGIN: TimeDuration = TimeDuration::from_millis(20);

#[derive(Debug, Clone)]
pub enum LyraEvent {
    PowerOn,
    PowerOff,
    EchoReply { id: ControlId },
    ShutdownAck { id: ControlId },
    Responsive,
    Unresponsive,
    ShutdownAccepted,
    ForceOff,
}

pub struct LyraContext {
    lyra_power: LyraPower,
    rpc_sender: RpcSender,
    next_id: ControlId,
    pending_echo: Option<ControlId>,
    missed_echoes: u8,
    shutdown_id: Option<ControlId>,
    shutdown_deadline: Instant,
    pending_power_on: bool,
}

impl LyraContext {
    fn new(lyra_power: LyraPower, rpc_sender: RpcSender) -> Self {
        Self {
            lyra_power,
            rpc_sender,
            next_id: 0,
            pending_echo: None,
            missed_echoes: 0,
            shutdown_id: None,
            shutdown_deadline: Instant::MIN,
            pending_power_on: false,
        }
    }

    fn fresh_id(&mut self) -> ControlId {
        self.next_id = self.next_id.wrapping_add(1);
        self.next_id
    }

    fn missed_echo(&mut self) -> bool {
        if self.pending_echo.take().is_some() {
            self.missed_echoes = self.missed_echoes.saturating_add(1);
        }
        self.missed_echoes >= MISSED_ECHO_LIMIT
    }

    fn accept_echo(&mut self, id: ControlId) -> bool {
        if self.pending_echo != Some(id) {
            return false;
        }
        self.pending_echo = None;
        self.missed_echoes = 0;
        true
    }

    async fn probe(&mut self, timeout: TimeDuration) {
        let id = self.fresh_id();
        self.pending_echo = Some(id);
        if !matches!(
            with_timeout(
                timeout,
                self.rpc_sender
                    .publish::<EchoProbeTopic>(VarSeq::Seq4(id as i32), &id),
            )
            .await,
            Ok(Ok(()))
        ) {
            warn!("Lyra echo probe could not be sent");
        }
    }

    async fn request_shutdown(&mut self, timeout: TimeDuration) {
        let id = self.shutdown_id.unwrap();
        if !matches!(
            with_timeout(
                timeout,
                self.rpc_sender
                    .publish::<ShutdownRequestTopic>(VarSeq::Seq4(id as i32), &id),
            )
            .await,
            Ok(Ok(()))
        ) {
            warn!("Lyra shutdown request could not be sent");
        }
    }

    fn shutdown_tx_budget(&self) -> Option<TimeDuration> {
        let now = Instant::now();
        (now < self.shutdown_deadline)
            .then(|| core::cmp::min(TX_TIMEOUT, self.shutdown_deadline - now))
    }
}

statechart! {
    Lyra {
        context: LyraContext;
        events: LyraEvent;
        default(PowerOff);

        state PowerOff {
            entry: power_off;
            on(PowerOn) => PowerOn;
        }

        state PowerOn {
            default(Starting);
            on(PowerOff) => Stopping;

            state Starting {
                entry: power_on;
                on(every PROBE_INTERVAL) => probe_boot;
                on(after BOOT_TIMEOUT) => Retry;
                on(EchoReply { id: ControlId }) => check_boot_echo;
                on(Responsive) => Online;
            }

            state Online {
                entry: send_first_probe;
                on(every PROBE_INTERVAL) => check_health;
                on(EchoReply { id: ControlId }) => check_echo;
                on(Unresponsive) => Retry;
            }

            state Retry {
                entry: power_off;
                on(after RETRY_DELAY) => Starting;
                on(PowerOff) => PowerOff;
            }
        }

        state Stopping {
            entry: begin_shutdown;
            exit: replay_power_on;
            during: shutdown_timeout(shutdown_deadline);
            default(AwaitShutdownAck);
            on(ForceOff) => PowerOff;
            on(PowerOn) => defer_power_on;
            on(PowerOff) => cancel_power_on;

            state AwaitShutdownAck {
                on(every PROBE_INTERVAL) => resend_shutdown;
                on(ShutdownAck { id: ControlId }) => check_shutdown_ack;
                on(ShutdownAccepted) => PingUntilSilent;
            }

            state PingUntilSilent {
                entry: send_first_probe;
                on(every PROBE_INTERVAL) => check_shutdown_health;
                on(EchoReply { id: ControlId }) => check_echo;
            }
        }
    }
}

impl LyraPowerOffActions for LyraActionContext<'_> {
    async fn power_off(&mut self) {
        self.pending_echo = None;
        self.shutdown_id = None;
        self.lyra_power.power_off().await;
        info!("Lyra power off");
    }
}

impl LyraStartingActions for LyraActionContext<'_> {
    async fn power_on(&mut self) {
        self.pending_echo = None;
        self.missed_echoes = 0;
        self.lyra_power.power_on().await;
        info!("Lyra power on; waiting for echo replies");
        self.probe(TX_TIMEOUT).await;
    }

    async fn probe_boot(&mut self) {
        self.missed_echo();
        self.probe(TX_TIMEOUT).await;
    }

    async fn check_boot_echo(&mut self, id: ControlId) {
        if self.accept_echo(id) {
            info!("Lyra echo confirmed");
            self.emit(LyraEvent::Responsive).unwrap();
        }
    }
}

impl LyraOnlineActions for LyraActionContext<'_> {
    async fn send_first_probe(&mut self) {
        self.probe(TX_TIMEOUT).await;
    }

    async fn check_health(&mut self) {
        if self.missed_echo() {
            warn!("Lyra echo timed out; cycling power");
            self.emit(LyraEvent::Unresponsive).unwrap();
        } else {
            self.probe(TX_TIMEOUT).await;
        }
    }

    async fn check_echo(&mut self, id: ControlId) {
        self.accept_echo(id);
    }
}

impl LyraRetryActions for LyraActionContext<'_> {
    async fn power_off(&mut self) {
        self.pending_echo = None;
        self.lyra_power.power_off().await;
        warn!("Lyra off; retrying after 5 seconds");
    }
}

impl LyraStoppingActions for LyraActionContext<'_> {
    async fn begin_shutdown(&mut self) {
        self.shutdown_deadline = Instant::now() + SHUTDOWN_TIMEOUT - POWER_CUT_MARGIN;
        self.pending_echo = None;
        let id = self.fresh_id();
        self.shutdown_id = Some(id);
        info!("Requesting Lyra shutdown");
        self.request_shutdown(TX_TIMEOUT).await;
    }

    async fn defer_power_on(&mut self) {
        self.pending_power_on = true;
    }

    async fn cancel_power_on(&mut self) {
        self.pending_power_on = false;
    }

    async fn replay_power_on(&mut self) {
        if core::mem::take(&mut self.pending_power_on) {
            // hsmc processes this only after PowerOff's power_off entry completes.
            self.emit(LyraEvent::PowerOn).unwrap();
        }
    }
}

impl LyraAwaitShutdownAckActions for LyraActionContext<'_> {
    async fn resend_shutdown(&mut self) {
        if let Some(budget) = self.shutdown_tx_budget() {
            self.request_shutdown(budget).await;
        }
    }

    async fn check_shutdown_ack(&mut self, id: ControlId) {
        if self.shutdown_id == Some(id) {
            info!("Lyra shutdown acknowledged; probing until silent");
            self.emit(LyraEvent::ShutdownAccepted).unwrap();
        }
    }
}

impl LyraPingUntilSilentActions for LyraActionContext<'_> {
    async fn send_first_probe(&mut self) {
        self.pending_echo = None;
        self.missed_echoes = 0;
        if let Some(budget) = self.shutdown_tx_budget() {
            self.probe(budget).await;
        }
    }

    async fn check_shutdown_health(&mut self) {
        if self.missed_echo() {
            info!("Lyra stopped replying; cutting power");
            self.emit(LyraEvent::ForceOff).unwrap();
        } else if let Some(budget) = self.shutdown_tx_budget() {
            self.probe(budget).await;
        }
    }

    async fn check_echo(&mut self, id: ControlId) {
        self.accept_echo(id);
    }
}

async fn shutdown_timeout(deadline: &mut Instant) -> LyraEvent {
    // An absolute deadline survives during-future cancellation on other events.
    Timer::at(*deadline).await;
    LyraEvent::ForceOff
}

pub const QN: usize = 8;
pub static EVENT_CHAN: Channel<CriticalSectionRawMutex, LyraEvent, QN> = Channel::new();

pub fn sender() -> LyraSender {
    LyraSender::from_channel(&EVENT_CHAN)
}

#[embassy_executor::task]
pub async fn run(lyra_power: LyraPower, rpc_sender: RpcSender) {
    let mut machine = Lyra::new(LyraContext::new(lyra_power, rpc_sender), &EVENT_CHAN);
    machine.run().await.unwrap();
}
