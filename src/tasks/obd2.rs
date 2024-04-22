use defmt::{error, info};
use embassy_time::{with_timeout, Duration};

use crate::{obd2::Obd2, pid};

#[embassy_executor::task]
pub async fn run(mut obd2: Obd2) {
    obd2.init().await;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
        info!("requesting bms pid");
        match with_timeout(Duration::from_millis(2500), obd2.request::<pid::BmsPid>()).await {
            Ok(Ok(bms_pid)) => {
                info!("bms pid: {:?}", bms_pid);
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
