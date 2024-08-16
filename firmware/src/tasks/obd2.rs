use defmt::{error, info, Format};
use embassy_time::{with_timeout, Duration};

use crate::{
    debug::internal_debug,
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
    info!("obd2 task started");
    obd2.init().await;
    info!("obd2 init done");
    loop {
        info!("requesting bms pid");
        match with_timeout(Duration::from_millis(2500), obd2.request_pid::<pid::BmsPid>()).await {
            Ok(Ok(bms_pid)) => {
                info!("bms pid: {:?}", bms_pid);
                internal_debug!("bms pid: {:?}", bms_pid);
                KIA_EVENTS.send(KiaEvent::Obd2Event(Obd2Event::BmsPid(bms_pid))).await;
            }
            Ok(Err(e)) => {
                internal_debug!("error requesting bms pid");
                error!("error requesting bms pid");
            }
            Err(_) => {
                internal_debug!("timeout requesting bms pid");
                error!("timeout requesting bms pid");
            }
        }
        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
    }
}
