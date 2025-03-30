use cortex_m::peripheral::NVIC;
use defmt::{info, unwrap};
use embassy_nrf::{
    interrupt,
    interrupt::{Interrupt, InterruptExt, Priority},
    pac,
};
use embassy_time::Duration;
use heapless::String;
use nrf_modem::{ConnectionPreference, LteLink, MemoryLayout, SystemMode};

use super::gnss::Gnss;
use crate::tasks::reset::ResetGuard;

#[interrupt]
fn IPC() {
    nrf_modem::ipc_irq_handler();
}

pub struct Modem {}

impl Modem {
    pub async fn new(ipc_start: u32) -> Self {
        let mut cp = unwrap!(cortex_m::Peripherals::take());

        // Enable the modem interrupts
        unsafe {
            NVIC::unmask(pac::Interrupt::IPC);
            cp.NVIC.set_priority(pac::Interrupt::IPC, 0 << 5);
        }

        info!("modem initializing with ipc_start: {:#x}", ipc_start);

        nrf_modem::init_with_custom_layout(
            SystemMode {
                lte_support: true,
                lte_psm_support: false,
                nbiot_support: false,
                gnss_support: true,
                preference: ConnectionPreference::Lte,
            },
            MemoryLayout { base_address: ipc_start, tx_area_size: 0x2000, rx_area_size: 0x2000, trace_area_size: 0 },
        )
        .await
        .unwrap();
        info!("modem initialized");
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
        //let response = nrf_modem::send_at::<64>("AT%XCOEX0").await.unwrap();
        //assert_eq!(response.as_str(), "OK\r\n");

        Ok(Gnss::new())
    }

    pub async fn send_sms(&self, numbers: &[&str], text: &str) -> Result<(), nrf_modem::Error> {
        let _guard = ResetGuard::new();
        for number in numbers {
            let mut ret = Ok(Ok(()));
            for _ in 0..5 {
                ret = embassy_time::with_timeout(
                    Duration::from_secs(30),
                    nrf_modem::Sms::new(number, text).send::<354>(),
                )
                .await
                .map_err(|_| nrf_modem::Error::NrfError(0));
                if let Ok(Ok(())) = ret {
                    break;
                }
                embassy_time::Timer::after_secs(10).await;
            }
            if let Err(err) = ret {
                return Err(err);
            }
            if let Ok(Err(err)) = ret {
                return Err(err);
            }
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

    pub async fn hw(&self) -> Result<String<32>, nrf_modem::Error> {
        let hwversion = nrf_modem::send_at::<64>("AT%HWVERSION").await.unwrap();
        if hwversion.ends_with("OK\r\n") && hwversion.len() >= 14 {
            Ok(unwrap!(String::try_from(&hwversion[8..14])))
        } else {
            Err(nrf_modem::Error::NrfError(0))
        }
    }

    pub async fn fw(&self) -> Result<String<32>, nrf_modem::Error> {
        let version = nrf_modem::send_at::<64>("AT+CGMR").await.unwrap();
        if version.ends_with("OK\r\n") && version.len() >= 17 {
            Ok(unwrap!(String::try_from(&version[12..17])))
        } else {
            Err(nrf_modem::Error::NrfError(0))
        }
    }
}
