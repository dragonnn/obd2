use cortex_m::peripheral::NVIC;
use defmt::unwrap;
use embassy_nrf::{
    interrupt,
    interrupt::{Interrupt, InterruptExt, Priority},
    pac,
};
use embassy_time::Duration;
use heapless::String;
use nrf_modem::{ConnectionPreference, LteLink, SystemMode};

use super::gnss::Gnss;

#[interrupt]
fn IPC() {
    nrf_modem::ipc_irq_handler();
}

pub struct Modem {}

impl Modem {
    pub async fn new() -> Self {
        let mut cp = unwrap!(cortex_m::Peripherals::take());

        // Enable the modem interrupts
        unsafe {
            NVIC::unmask(pac::Interrupt::IPC);
            cp.NVIC.set_priority(pac::Interrupt::IPC, 0 << 5);
        }

        nrf_modem::init(SystemMode {
            lte_support: true,
            lte_psm_support: false,
            nbiot_support: false,
            gnss_support: true,
            preference: ConnectionPreference::Lte,
        })
        .await
        .unwrap();
        Self {}
    }

    pub async fn link(&self, timeout: Duration) -> Result<LteLink, nrf_modem::Error> {
        //TODO add proper error type
        let link = embassy_time::with_timeout(timeout, LteLink::new()).await.map_err(|_| {
            defmt::error!("link timeout");
            nrf_modem::Error::NrfError(0)
        })??;

        embassy_time::with_timeout(timeout, link.wait_for_link()).await.map_err(|_| {
            defmt::error!("link timeout");
            nrf_modem::Error::NrfError(0)
        })??;
        Ok(link)
    }

    pub async fn gnss(&self) -> Result<Gnss, nrf_modem::Error> {
        let response = nrf_modem::send_at::<64>("AT%XCOEX0").await.unwrap();
        assert_eq!(response.as_str(), "OK\r\n");

        Ok(Gnss::new())
    }

    pub async fn send_sms(&self, numbers: &[&str], text: &str) -> Result<(), nrf_modem::Error> {
        for number in numbers {
            embassy_time::with_timeout(Duration::from_secs(120), nrf_modem::Sms::new(number, text).send::<354>())
                .await
                .map_err(|_| nrf_modem::Error::NrfError(0))??;
        }
        Ok(())
    }

    pub async fn imei(&self) -> Result<String<15>, nrf_modem::Error> {
        let imei = nrf_modem::send_at::<64>("AT+CGSN=1").await.unwrap();
        if imei.ends_with("OK\r\n") && imei.len() >= 23 {
            Ok(unwrap!(String::try_from(&imei[8..23])))
        } else {
            Err(nrf_modem::Error::NrfError(0))
        }
    }

    pub async fn dbm(&self) -> Result<Option<i16>, nrf_modem::Error> {
        let dbm = nrf_modem::send_at::<64>("AT+CESQ").await.unwrap();
        defmt::info!("got dbm: {:?} {}", dbm.as_str(), dbm.len());
        if dbm.ends_with("OK\r\n") && dbm.len() >= 26 {
            dbm.split(',')
                .nth(5)
                .unwrap()
                .split('\r')
                .next()
                .unwrap()
                .parse()
                .map_err(|_| nrf_modem::Error::NrfError(0))
                .map(|dbm: i16| if dbm == 255 { None } else { Some(dbm - 140) })
        } else {
            defmt::error!("error getting dbm");
            Err(nrf_modem::Error::NrfError(0))
        }
    }
}