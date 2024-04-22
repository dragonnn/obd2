use defmt::{error, info, Format};
use embassy_time::{with_timeout, Duration};

use crate::{
    event::{KiaEvent, KIA_EVENTS},
    obd2::Obd2,
    pid,
};

#[derive(Format, PartialEq, Clone)]
pub enum Obd2Event {
    BmsPid(pid::BmsPid),
}

#[embassy_executor::task]
pub async fn run(mut obd2: Obd2) {
    obd2.init().await;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
        info!("requesting bms pid");
        match with_timeout(Duration::from_millis(2500), obd2.request::<pid::BmsPid>()).await {
            Ok(Ok(bms_pid)) => {
                info!("bms pid: {:?}", bms_pid);
                KIA_EVENTS.send(KiaEvent::Obd2Event(Obd2Event::BmsPid(bms_pid))).await;
            }
            Ok(Err(e)) => {
                error!("error requesting bms pid");
            }
            Err(_) => {
                error!("timeout requesting bms pid");
            }
        }
    }
}
