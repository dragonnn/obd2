use core::{fmt::Write, sync::atomic::Ordering, write};

use defmt::warn;
use embassy_time::Duration;
use heapless::String;

use crate::{board::Modem, tasks};

pub async fn send_state(
    modem: &Modem,
    event: &str,
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
    let montion_detect = tasks::montion_detection::State::get().await;

    let mut sms: String<300> = String::new();
    write!(
        &mut sms,
        "{}\n\nbat: {}{}%\nv: {:.2}V\nmontions: {}\nrestarts: {}\n",
        event,
        if battery.charging { "+" } else { "-" },
        battery.capacity,
        battery.voltage as f32 / 1000.0,
        montion_detect.0,
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
        .map_err(|_| nrf_modem::Error::OutOfMemory)?;
    } else {
        writeln!(&mut sms, "fix: none").map_err(|_| nrf_modem::Error::OutOfMemory)?;
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
        writeln!(&mut sms, "dbm: {}", dbm).map_err(|_| nrf_modem::Error::OutOfMemory)?;
    } else {
        writeln!(&mut sms, "dbm: --").map_err(|_| nrf_modem::Error::OutOfMemory)?;
    }

    //write!(&mut sms, "twi2_resets: {}", crate::board::TWI2_RESETS.load(Ordering::SeqCst))
    //    .map_err(|_| nrf_modem::Error::OutOfMemory)?;
    if all_numbers {
        modem.send_sms(crate::config::SMS_NUMBERS, &sms).await?;
    } else {
        modem.send_sms(crate::config::PANIC_SMS_NUMBERS, &sms).await?;
    }
    defmt::info!("sms send ok");
    link.deactivate().await?;
    Ok(())
}
