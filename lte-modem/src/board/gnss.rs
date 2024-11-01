use embassy_time::{with_timeout, Duration, Ticker};
use futures::StreamExt;
use nrf_modem::{
    nrfxlib_sys, Error as ModemError, Gnss as ModemGnss, GnssConfig as ModemGnssConfig, GnssData as ModemGnssData,
    GnssPowerSaveMode as ModemGnssPowerSaveMode, GnssStream as ModemGnssStream, GnssUsecase as ModemGnssUsecase,
};

pub struct Gnss {
    //handler: Option<ModemGnss>,
    stream: Option<ModemGnssStream>,

    duration: Duration,
    timeout: Duration,
    ticker: Ticker,
    low_accuracy: bool,
}

impl Gnss {
    pub fn new() -> Self {
        let duration = Duration::from_secs(20);
        let timeout = Duration::from_secs(60);

        Self { stream: None, duration, timeout, ticker: Ticker::every(duration), low_accuracy: false }
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
            self.stream =
                Some(self.handler().await?.start_periodic_fix(self.get_config(), self.duration.as_secs() as u16)?);
        } else {
            self.stream = None;
            self.ticker = Ticker::every(self.duration);
        }
        Ok(())
    }

    fn get_config(&self) -> ModemGnssConfig {
        ModemGnssConfig {
            use_case: ModemGnssUsecase { low_accuracy: self.low_accuracy, ..Default::default() },
            power_mode: ModemGnssPowerSaveMode::DutyCycling,
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
        }
    }

    pub async fn next(&mut self) -> Result<Option<nrfxlib_sys::nrf_modem_gnss_pvt_data_frame>, ModemError> {
        if self.stream.is_none() {
            self.ticker.next().await;
        }

        with_timeout(self.timeout, self.get_fix()).await.map_err(|_| {
            defmt::error!("gnss timeout");
            ModemError::NrfError(0)
        })?
    }
}
