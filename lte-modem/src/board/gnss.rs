use defmt::*;
use embassy_time::{with_timeout, Duration, Ticker};
use futures::StreamExt;
use nrf_modem::{
    nrfxlib_sys, Error as ModemError, Gnss as ModemGnss, GnssConfig as ModemGnssConfig, GnssData as ModemGnssData,
    GnssPowerSaveMode as ModemGnssPowerSaveMode, GnssStream as ModemGnssStream, GnssUsecase as ModemGnssUsecase,
};
use types::{GnssState, Modem, TxFrame, TxMessage};

use crate::tasks::modem::link::{tx_channel_pub, TxChannelPub};

pub struct Gnss {
    //handler: Option<ModemGnss>,
    stream: Option<ModemGnssStream>,

    duration: Duration,
    timeout: Duration,
    ticker: Ticker,
    low_accuracy: bool,
    tx_channel_pub: TxChannelPub,

    first_fix: bool,
}

impl Gnss {
    pub fn new() -> Self {
        let duration = Duration::from_secs(20);
        let timeout = Duration::from_secs(5 * 60);
        let tx_channel_pub = tx_channel_pub();
        unsafe {
            nrfxlib_sys::nrf_modem_gnss_prio_mode_enable();
        }

        Self {
            stream: None,
            duration,
            timeout,
            ticker: Ticker::every(duration),
            low_accuracy: false,
            tx_channel_pub,
            first_fix: true,
        }
    }

    async fn handler(&mut self) -> Result<ModemGnss, ModemError> {
        //if let Some(handler) = self.handler.take() {
        //    Ok(handler)
        //} else {
        ModemGnss::new().await
        //}
    }

    async fn start(&mut self) -> Result<(), ModemError> {
        if self.duration.as_millis() < 20000 {
            info!("start periodic fix");
            self.stream = Some(self.handler().await?.start_continuous_fix(self.get_config())?);
            self.tx_channel_pub
                .publish_immediate(TxMessage::new(TxFrame::Modem(Modem::GnssState(GnssState::PeriodicFix))));
        } else {
            self.stream = None;
            self.tx_channel_pub.publish_immediate(TxMessage::new(TxFrame::Modem(Modem::GnssState(
                GnssState::TickerFix(self.duration.as_secs() as u32),
            ))));
            self.ticker = Ticker::every(self.duration);
        }
        Ok(())
    }

    fn get_config(&self) -> ModemGnssConfig {
        ModemGnssConfig {
            use_case: ModemGnssUsecase { low_accuracy: self.low_accuracy, ..Default::default() },
            power_mode: if self.low_accuracy {
                ModemGnssPowerSaveMode::DutyCycling
            } else {
                ModemGnssPowerSaveMode::Disabled
            },
            ..Default::default()
        }
    }

    pub async fn conf(&mut self, duration: Duration, low_accuracy: bool) {
        if self.duration != duration || self.low_accuracy != low_accuracy {
            self.duration = duration;
            self.low_accuracy = low_accuracy;
            self.stream = None;
            self.start().await.ok();
        }
    }

    async fn get_fix(&mut self) -> Result<Option<nrfxlib_sys::nrf_modem_gnss_pvt_data_frame>, ModemError> {
        if let Some(stream) = &mut self.stream {
            loop {
                let gnss_frame = stream.next().await.map_or(Ok(None), |v| v.map(Some))?;
                if let Some(ModemGnssData::PositionVelocityTime(postion_gnss_frame)) = gnss_frame {
                    if postion_gnss_frame.accuracy != 0.0
                        && postion_gnss_frame.accuracy < 250.0
                        && postion_gnss_frame.altitude > -50.0
                        && postion_gnss_frame.altitude < 8000.0
                    {
                        return Ok(Some(postion_gnss_frame));
                    }
                }
            }
        } else {
            let handler = self.handler().await?;
            let mut stream = handler.start_continuous_fix(self.get_config())?;
            info!("start continuous fix");

            loop {
                let gnss_frame = stream.next().await.map_or(Ok(None), |v| v.map(Some))?;
                match gnss_frame {
                    Some(ModemGnssData::PositionVelocityTime(postion_gnss_frame)) => {
                        info!(
                            "got fix with accuracy: {} on {}:{}",
                            postion_gnss_frame.accuracy, postion_gnss_frame.latitude, postion_gnss_frame.longitude
                        );
                        if postion_gnss_frame.accuracy != 0.0
                            && postion_gnss_frame.accuracy < 250.0
                            && postion_gnss_frame.altitude > -50.0
                            && postion_gnss_frame.altitude < 8000.0
                        {
                            return Ok(Some(postion_gnss_frame));
                        }
                    }
                    Some(ModemGnssData::Nmea(nmea)) => {
                        info!("got nmea: {:?}", nmea.as_str());
                    }
                    Some(ModemGnssData::Agps(agps)) => {
                        info!(
                            "got agps: data_flags: {} system_count: {} system: {:?}",
                            agps.data_flags,
                            agps.system_count,
                            defmt::Debug2Format(&agps.system)
                        );
                    }
                    None => {
                        info!("got none");
                    }
                }
                if let Some(ModemGnssData::PositionVelocityTime(postion_gnss_frame)) = gnss_frame {
                    if postion_gnss_frame.accuracy != 0.0
                        && postion_gnss_frame.accuracy < 250.0
                        && postion_gnss_frame.altitude > -50.0
                        && postion_gnss_frame.altitude < 8000.0
                    {
                        return Ok(Some(postion_gnss_frame));
                    }
                }
            }
        }
    }

    pub async fn next(&mut self) -> Result<Option<nrfxlib_sys::nrf_modem_gnss_pvt_data_frame>, ModemError> {
        if self.stream.is_none() {
            self.ticker.next().await;
        }

        self.tx_channel_pub
            .publish_immediate(TxMessage::new(TxFrame::Modem(Modem::GnssState(GnssState::WaitingForFix))));
        let mut _link_lock = None;
        if self.first_fix {
            warn!("force link disconnect and lock");
            _link_lock = crate::tasks::modem::link::force_disconnect_and_lock().await;
        }
        let ret = with_timeout(self.timeout, self.get_fix()).await.map_err(|_| {
            defmt::error!("gnss timeout");
            self.tx_channel_pub
                .publish_immediate(TxMessage::new(TxFrame::Modem(Modem::GnssState(GnssState::TimeoutFix))));
            ModemError::NrfError(0)
        })?;

        if ret.as_ref().map(|f| f.is_some()).unwrap_or_default() {
            self.first_fix = false;
        }
        ret
    }

    pub fn ticker_reset(&mut self) {
        self.ticker.reset();
    }
}
