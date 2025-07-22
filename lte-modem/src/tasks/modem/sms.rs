use core::{
    fmt::Write,
    hash::{Hash, Hasher},
    sync::atomic::Ordering,
    write,
};

use defmt::*;
use derivative::Derivative;
use embassy_futures::select::{select, Either::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use heapless::{String, Vec};

use crate::{board::Modem, tasks};

static SMS_STATE_CHANNEL: Channel<CriticalSectionRawMutex, SmsData, 16> = Channel::new();

pub async fn send_state_delayed(
    event: SmsEvent,
    force_new_fix: bool,
    new_fix_if_missing: bool,
    restarts: u32,
    all_numbers: bool,
) -> Result<(), nrf_modem::Error> {
    //let mut event_string = String::new();
    //write!(&mut event_string, "{}", event).ok();
    let sms_data = SmsData { event, force_new_fix, new_fix_if_missing, restarts, all_numbers };
    if SMS_STATE_CHANNEL.try_send(sms_data).is_err() {
        defmt::error!("sms channel full, dropping message");
    }
    Ok(())
}

#[derive(Format, Clone, Default)]
pub struct SmsData {
    pub event: SmsEvent,
    pub force_new_fix: bool,
    pub new_fix_if_missing: bool,
    pub restarts: u32,
    pub all_numbers: bool,
}

#[derive(Format, Clone, Derivative)]
#[derivative(Hash)]
pub enum SmsEvent {
    Driving(#[derivative(Hash = "ignore")] bool),
    Closed {
        #[derivative(Hash = "ignore")]
        trunk_open: bool,
        #[derivative(Hash = "ignore")]
        engine_hood_open: bool,
        #[derivative(Hash = "ignore")]
        actuator_back_door_passenger_side_unlock: bool,
        #[derivative(Hash = "ignore")]
        actuator_back_door_driver_side_unlock: bool,
    },
    Fix(#[derivative(Hash = "ignore")] bool),
    MovementOnBattery(#[derivative(Hash = "ignore")] f64),
    Custom(&'static str),
}

impl PartialEq for SmsEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SmsEvent::Driving(_), SmsEvent::Driving(_)) => true,
            (SmsEvent::Closed { .. }, SmsEvent::Closed { .. }) => true,
            (SmsEvent::Fix(_), SmsEvent::Fix(_)) => true,
            (SmsEvent::MovementOnBattery(_), SmsEvent::MovementOnBattery(_)) => true,
            (SmsEvent::Custom(a), SmsEvent::Custom(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for SmsEvent {}

impl Default for SmsEvent {
    fn default() -> Self {
        SmsEvent::Custom("default")
    }
}

impl SmsEvent {
    pub fn to_string(&self) -> String<64> {
        let mut s = String::new();
        match self {
            SmsEvent::Driving(driving) => {
                if *driving {
                    write!(&mut s, "driving...").ok();
                } else {
                    write!(&mut s, "parked...").ok();
                }
            }
            SmsEvent::Closed {
                trunk_open,
                engine_hood_open,
                actuator_back_door_passenger_side_unlock,
                actuator_back_door_driver_side_unlock,
            } => {
                if *trunk_open {
                    write!(&mut s, "trunk open...\n\n").ok();
                } else if *actuator_back_door_driver_side_unlock || *actuator_back_door_passenger_side_unlock {
                    write!(&mut s, "unlock...\n\n").ok();
                } else {
                    write!(&mut s, "closed...\n").ok();
                }

                write!(&mut s, "t:{},", if *trunk_open { "o" } else { "c" }).ok();
                write!(&mut s, "d:{},", if *actuator_back_door_driver_side_unlock { "o" } else { "c" }).ok();
                write!(&mut s, "p:{},", if *actuator_back_door_passenger_side_unlock { "o" } else { "c" }).ok();
                write!(&mut s, "e:{}\n", if *engine_hood_open { "o" } else { "c" }).ok();
            }
            SmsEvent::Fix(fix) => {
                if *fix {
                    write!(&mut s, "found fix...").ok();
                } else {
                    write!(&mut s, "lost fix...").ok();
                }
            }
            SmsEvent::MovementOnBattery(distance) => {
                write!(&mut s, "movement on battery: {:.2}m", distance).ok();
            }
            SmsEvent::Custom(msg) => {
                write!(&mut s, "{}", msg).ok();
            }
        }
        s
    }

    fn is_driving(&self) -> bool {
        matches!(self, SmsEvent::Driving(true))
    }

    fn is_parked(&self) -> bool {
        matches!(self, SmsEvent::Driving(false))
    }

    fn is_closed(&self) -> bool {
        if let SmsEvent::Closed {
            trunk_open,
            engine_hood_open,
            actuator_back_door_passenger_side_unlock,
            actuator_back_door_driver_side_unlock,
        } = self
        {
            !trunk_open
                && !engine_hood_open
                && !actuator_back_door_passenger_side_unlock
                && !actuator_back_door_driver_side_unlock
        } else {
            false
        }
    }
}

#[embassy_executor::task]
pub async fn task(modem: Modem) {
    let sms_channel_sub = SMS_STATE_CHANNEL.receiver();
    let mut events = heapless::FnvIndexSet::<SmsEvent, 8>::new();
    let mut default_sms_data = SmsData::default();
    let mut has_parked = false;
    let mut has_driving = false;
    let mut has_closed = false;

    loop {
        match select(sms_channel_sub.receive(), async {
            if events.is_empty() {
                core::future::pending::<()>().await;
            } else if events.len() == events.capacity() || (has_closed && (has_driving || has_parked)) {
                ()
            } else {
                Timer::after_secs(15 * 60).await;
            }
        })
        .await
        {
            First(msg) => {
                defmt::info!("sms task got message: {:?}", msg);
                default_sms_data.force_new_fix |= msg.force_new_fix;
                default_sms_data.new_fix_if_missing |= msg.new_fix_if_missing;
                default_sms_data.restarts = msg.restarts;
                default_sms_data.all_numbers |= msg.all_numbers;
                if msg.event.is_driving() {
                    has_driving = true;
                } else if msg.event.is_parked() {
                    has_parked = true;
                } else if msg.event.is_closed() {
                    has_closed = true;
                }
                events.remove(&msg.event);
                events.insert(msg.event).ok();
            }
            Second(_) => {
                if !events.is_empty() {
                    info!("sms task timeout, sending state");
                    if let Err(err) = send_state(
                        &modem,
                        &events,
                        default_sms_data.force_new_fix,
                        default_sms_data.new_fix_if_missing,
                        default_sms_data.restarts,
                        default_sms_data.all_numbers,
                    )
                    .await
                    {
                        defmt::error!("error sending sms: {:?}", err);
                    }
                    info!("sms sended");
                    default_sms_data = SmsData::default();
                    events.clear();

                    has_parked = false;
                    has_driving = false;
                    has_closed = false;
                }
            }
        }
    }
}

pub async fn send_state(
    modem: &Modem,
    events: &heapless::FnvIndexSet<SmsEvent, 8>,
    force_new_fix: bool,
    new_fix_if_missing: bool,
    restarts: u32,
    all_numbers: bool,
) -> Result<(), nrf_modem::Error> {
    defmt::trace!("sending sms");
    let battery = tasks::battery::State::get().await;
    let mut fix = tasks::gnss::State::get_current_fix().await;
    if (fix.is_none() && new_fix_if_missing) || force_new_fix {
        if let Some(new_fix) = tasks::gnss::State::wait_for_fix(Duration::from_secs(120)).await {
            fix = Some(new_fix);
        }
    }
    let mut sms: String<300> = String::new();
    for event in events {
        write!(&mut sms, "{}\n", event.to_string()).ok();
    }
    write!(
        &mut sms,
        "\n\nbat: {}{}%\nv: {:.2}V\res: {}\n",
        if battery.charging { "+" } else { "-" },
        battery.capacity,
        battery.voltage as f32 / 1000.0,
        restarts
    )
    .map_err(|_| nrf_modem::Error::OutOfMemory)?;
    if let Some(fix) = fix {
        write!(
            &mut sms,
            "lat: {:.5}\nlon: {:.5}\nalt: {:.1}m\nacc: {:.1}m\n{:02}:{:02}:{:02} {:02}-{:02}-{:04}\n",
            fix.latitude,
            fix.longitude,
            fix.altitude,
            fix.accuracy,
            fix.hour,
            fix.minute,
            fix.seconds,
            fix.day,
            fix.month,
            fix.year
        )
        .ok();
    } else {
        writeln!(&mut sms, "fix: none").ok();
    }
    defmt::info!("starting sms send");
    let mut link = Err(nrf_modem::Error::NrfError(0));
    for _ in 0..5 {
        link = modem.link(Duration::from_secs(120)).await;
        if link.is_ok() {
            break;
        }
        if link.is_err() {
            warn!("link error");
            embassy_time::Timer::after(Duration::from_secs(10)).await;
        }
    }
    let link = link?;

    if let Some(dbm) = modem.dbm().await.unwrap() {
        writeln!(&mut sms, "dbm: {}", dbm).ok();
    } else {
        writeln!(&mut sms, "dbm: --").ok();
    }

    //write!(&mut sms, "twi2_resets: {}", crate::board::TWI2_RESETS.load(Ordering::SeqCst))
    //    .map_err(|_| nrf_modem::Error::OutOfMemory)?;
    if sms.capacity() == sms.len() {
        sms.pop();
        sms.pop();
        sms.pop();
        sms.push('.').ok();
        sms.push('.').ok();
        sms.push('.').ok();
    }

    if all_numbers {
        modem.send_sms(crate::config::SMS_NUMBERS, &sms).await?;
    } else {
        modem.send_sms(crate::config::PANIC_SMS_NUMBERS, &sms).await?;
    }
    defmt::info!("sms send ok");
    link.deactivate().await?;
    Ok(())
}
