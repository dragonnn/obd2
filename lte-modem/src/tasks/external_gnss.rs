use defmt::*;
use embassy_futures::select::{select, Either};
use embassy_nrf::gpio::{Input, Output};
use embassy_sync::pubsub::DynPublisher;
use embassy_time::{with_timeout, Duration, Instant, Timer};
use nmea0183::{ParseResult, Parser};
use statig::prelude::*;
use types::{GnssFix, Modem, TxFrame};

use super::{
    modem::link::{RxChannelPub, TxChannelPub},
    uarte::state_channel_sub,
};
use crate::board::{BoardGnssUarteRx, BoardGnssUarteTx};

const SET_BAUDRATE: &[u8] = b"$PMTK251,9600*17\r\n";
const SET_POS_FIX_400MS: &[u8] = b"$PMTK220,400*2A\r\n";
const SET_NMEA_OUTPUT: &[u8] = b"$PMTK314,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,1,0*2\r\n";
const SET_BACKUP_MODE: &[u8] = b"$PMTK225,4*2F\r\n";

#[embassy_executor::task]
pub async fn task(
    uarte_gnss: (BoardGnssUarteTx, BoardGnssUarteRx),
    gnss_pss: Input<'static>,
    mut gnss_force_on: Output<'static>,
) {
    let fix_pub = unwrap!(crate::tasks::gnss::CHANNEL.dyn_publisher());
    let rx_channel_pub = crate::tasks::modem::link::rx_channel_pub();
    let tx_channel_pub = crate::tasks::modem::link::tx_channel_pub();

    let (uarte_tx, uarte_rx) = uarte_gnss;
    let mut state_channel_sub = state_channel_sub();
    let mut gnss_state_context = GnssContext { uarte_rx, fix: None };
    let mut gnss_state_machine =
        GnssState { uarte_tx, gnss_force_on, gnss_pss, fix_pub, rx_channel_pub, tx_channel_pub }.state_machine();

    gnss_state_machine.init_with_context(&mut gnss_state_context).await;

    gnss_state_machine.handle_with_context(&GnssStateEvent::Step, &mut gnss_state_context).await;
    let mut current_state: Option<types::State> = None;
    let mut buffer = [0; 1024];
    loop {
        if !(*gnss_state_machine.state() == State::backup()) {
            match select(
                state_channel_sub.next_message_pure(),
                with_timeout(Duration::from_secs(30), gnss_state_context.uarte_rx.read_until_idle(&mut buffer)),
            )
            .await
            {
                Either::First(state) => {
                    if current_state != Some(state.clone()) {
                        gnss_state_machine
                            .handle_with_context(&GnssStateEvent::State(state.clone()), &mut gnss_state_context)
                            .await
                    }

                    current_state = Some(state);
                }
                Either::Second(Ok(rx_result)) => {
                    if let Ok(readed) = rx_result {
                        let buffer = &buffer[..readed];
                        let mut parser = Parser::new();
                        for result in parser.parse_from_bytes(buffer) {
                            match result {
                                Ok(nmea) => {
                                    gnss_state_machine
                                        .handle_with_context(&GnssStateEvent::Nmea(nmea), &mut gnss_state_context)
                                        .await;
                                }
                                Err(err) => error!("nmea parse error: {} on: {=[u8]:a}", err, buffer),
                            }
                        }
                    }
                }
                Either::Second(Err(_)) => {
                    error!("gnss read timeout");
                    gnss_state_machine.handle_with_context(&GnssStateEvent::Step, &mut gnss_state_context).await;
                }
            }
        } else {
            match select(state_channel_sub.next_message_pure(), super::gnss::REQUEST.wait()).await {
                Either::First(state) => {
                    if current_state != Some(state.clone()) {
                        gnss_state_machine
                            .handle_with_context(&GnssStateEvent::State(state.clone()), &mut gnss_state_context)
                            .await
                    }
                    current_state = Some(state);
                }
                Either::Second(_) => {
                    gnss_state_machine.handle_with_context(&GnssStateEvent::SingleFix, &mut gnss_state_context).await
                }
            }
        }
    }
}
pub enum GnssStateEvent {
    Step,
    SingleFix,
    State(types::State),
    Nmea(nmea0183::ParseResult),
}

impl defmt::Format for GnssStateEvent {
    fn format(&self, f: Formatter) {
        match self {
            Self::Step => defmt::write!(f, "Step",),
            Self::SingleFix => defmt::write!(f, "SingleFix",),
            Self::State(state) => defmt::write!(f, "State({:?})", state),
            Self::Nmea(nmea) => defmt::write!(f, "Nmea({:?})", defmt::Debug2Format(nmea)),
        }
    }
}

pub struct GnssContext {
    uarte_rx: BoardGnssUarteRx,
    fix: Option<GnssFix>,
}

unsafe impl Sync for GnssContext {}

#[derive()]
pub struct GnssState {
    uarte_tx: BoardGnssUarteTx,
    gnss_pss: Input<'static>,
    gnss_force_on: Output<'static>,

    fix_pub: DynPublisher<'static, GnssFix>,
    rx_channel_pub: RxChannelPub,
    tx_channel_pub: TxChannelPub,
}

#[state_machine(
    // This sets the initial state to `led_on`.
    initial = "State::backup()",
    // Derive the Debug trait on the `State` enum.
    state(derive(Format, Debug, PartialEq, Eq)),
    // Derive the Debug trait on the `Superstate` enum.
    superstate(derive(Format, Debug)),
    // Set the `on_transition` callback.
    on_transition = "Self::on_transition",
    // Set the `on_dispatch` callback.
    on_dispatch = "Self::on_dispatch"
)]
impl GnssState {
    #[action]
    async fn enable_backup(&mut self) {
        warn!("gnss enter backup");
        self.gnss_force_on.set_high();
        self.uarte_tx.write(SET_BACKUP_MODE).await.ok();
        Timer::after_millis(100).await;
        self.tx_channel_pub.publish_immediate(TxFrame::Modem(Modem::GnssState(types::GnssState::BackupMode)).into());
    }

    #[state(entry_action = "enable_backup")]
    async fn backup(&mut self, context: &mut GnssContext, event: &GnssStateEvent) -> Response<State> {
        warn!("backup mode: {:?}", event);
        match event {
            GnssStateEvent::State(state) => match state {
                types::State::IgnitionOn => Transition(State::continuous_fix(Instant::now(), None)),
                types::State::IgnitionOff => Transition(State::single_fix(Instant::now())),
                _ => Handled,
            },
            GnssStateEvent::SingleFix => Transition(State::single_fix(Instant::now())),
            _ => Handled,
        }
    }

    #[action]
    async fn disable_backup(&mut self, context: &mut GnssContext) {
        context.fix = None;
        warn!("gnss disable backup");
        for _i in 0..20 {
            self.gnss_force_on.set_low();
            Timer::after_millis(100).await;
            for cmd in [SET_BAUDRATE, SET_POS_FIX_400MS, SET_NMEA_OUTPUT].iter() {
                self.uarte_tx.write(*cmd).await.ok();
                Timer::after_millis(100).await;
            }
            let mut buffer = [0; 128];
            match with_timeout(Duration::from_secs(15), context.uarte_rx.read_until_idle(&mut buffer)).await {
                Ok(Ok(readed)) => {
                    let buffer = &buffer[..readed];
                    info!("gnss: {=[u8]:a}", buffer);
                    if buffer.len() > 0 {
                        return;
                    }
                }
                Ok(Err(e)) => {
                    error!("gnss disable backup read error: {}", e);
                }
                Err(_) => {
                    error!("gnss disable backup timeout");
                    self.gnss_force_on.set_high();
                    Timer::after_millis(100).await;
                }
            }
        }
        self.tx_channel_pub
            .publish_immediate(TxFrame::Modem(Modem::GnssState(types::GnssState::ErrorDisablingBackup)).into());
        error!("gnss disable backup failed");
    }

    #[state(entry_action = "disable_backup")]
    async fn continuous_fix(
        &mut self,
        context: &mut GnssContext,
        event: &GnssStateEvent,
        timeout: &mut Instant,
        last_fix_send: &mut Option<Instant>,
    ) -> Response<State> {
        if timeout.elapsed().as_secs() > 20 {
            warn!("no nmea message in 20s, trying to renable");
            self.disable_backup(context).await;
            Timer::after_millis(100).await;
        }
        match event {
            GnssStateEvent::Nmea(nmea) => {
                *timeout = Instant::now();
                if let ParseResult::RMC(Some(rmc)) = nmea {
                    let fix: super::gnss::FromFix = rmc.into();
                    let mut distance = f64::MAX;
                    if let Some(last_fix) = context.fix {
                        distance = fix.0 - last_fix;
                    }

                    let mut fix_send = distance > 10.0;
                    if let Some(last_fix_send) = last_fix_send {
                        if last_fix_send.elapsed().as_secs() > 60 {
                            fix_send = true;
                        }
                    }

                    if fix_send {
                        self.rx_channel_pub
                            .publish_immediate(types::RxFrame::Modem(types::Modem::GnssFix(fix.0)).into());
                        self.fix_pub.publish_immediate(fix.0);
                        context.fix = Some(fix.0);
                        last_fix_send.replace(Instant::now());
                    }
                    super::gnss::STATE.lock().await.fix = Some(fix.0);
                }
                Handled
            }
            GnssStateEvent::State(state) => match state {
                types::State::IgnitionOn => Handled,
                _ => Transition(State::backup()),
            },
            _ => Handled,
        }
    }
    #[state(entry_action = "disable_backup")]
    async fn single_fix(
        &mut self,
        context: &mut GnssContext,
        event: &GnssStateEvent,
        timeout: &mut Instant,
    ) -> Response<State> {
        if timeout.elapsed().as_secs() > 5 * 60 {
            error!("single fix timeout");
            return Transition(State::backup());
        }

        match event {
            GnssStateEvent::Nmea(nmea) => {
                if let ParseResult::RMC(Some(rmc)) = nmea {
                    let fix: super::gnss::FromFix = rmc.into();
                    self.rx_channel_pub.publish_immediate(types::RxFrame::Modem(types::Modem::GnssFix(fix.0)).into());
                    self.fix_pub.publish_immediate(fix.0);
                    super::gnss::STATE.lock().await.fix = Some(fix.0);
                    Transition(State::backup())
                } else {
                    Handled
                }
            }
            GnssStateEvent::State(state) => match state {
                types::State::IgnitionOn => Transition(State::continuous_fix(Instant::now(), None)),
                _ => Handled,
            },
            _ => Handled,
        }
    }
}

impl GnssState {
    fn on_transition(&mut self, source: &State, target: &State) {
        trace!("transitioned from `{:?}` to `{:?}`", source, target);
        match target {
            State::Backup {} => self
                .tx_channel_pub
                .publish_immediate(TxFrame::Modem(Modem::GnssState(types::GnssState::BackupMode)).into()),
            State::SingleFix { timeout: _ } => self
                .tx_channel_pub
                .publish_immediate(TxFrame::Modem(Modem::GnssState(types::GnssState::SingleFix)).into()),
            State::ContinuousFix { timeout: _, last_fix_send: _ } => self
                .tx_channel_pub
                .publish_immediate(TxFrame::Modem(Modem::GnssState(types::GnssState::ContinuousFix)).into()),
        }
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<GnssState>, event: &GnssStateEvent) {
        trace!("dispatched `{:?}` to `{:?}`", event, state);
    }
}
