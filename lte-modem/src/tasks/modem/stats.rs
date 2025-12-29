use defmt::*;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::Timer;

use crate::board::Modem;

static DBM_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Option<i16>, 16, 1, 16> = PubSubChannel::new();

#[embassy_executor::task]
pub async fn task(mut modem: Modem) {
    let mut dbm_pub = DBM_CHANNEL.publisher().unwrap();
    loop {
        if super::link::connected() {
            info!("refreshing dbm state");
            if let Some(dbm) = modem.dbm().await.unwrap() {
                defmt::info!("dbm: {}", dbm);
                dbm_pub.publish(Some(dbm)).await;
            } else {
                defmt::info!("dbm: --");
                dbm_pub.publish(None).await;
            }
        } else {
            dbm_pub.publish(None).await;
        };
        Timer::after_secs(5).await;
    }
}

pub fn dbm_channel_sub() -> embassy_sync::pubsub::Subscriber<'static, CriticalSectionRawMutex, Option<i16>, 16, 1, 16> {
    DBM_CHANNEL.subscriber().unwrap()
}
