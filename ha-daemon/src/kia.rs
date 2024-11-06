use crate::{config::Config, ha::device::UpdateLocation, sensor, HaStateEvent};
use openssl::ssl::{Ssl, SslContext, SslFiletype, SslMethod};
use postcard::{from_bytes, to_stdvec, to_vec};
use serde_encrypt::{shared_key::SharedKey, traits::SerdeEncryptSharedKey as _, EncryptedMessage};
use std::{
    collections::HashMap, net::SocketAddr, path::PathBuf, pin::Pin, sync::Arc, time::Duration,
};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    sync::mpsc::UnboundedSender,
    time::timeout,
};
use tokio_openssl::SslStream;
use types::{RxFrame, TxFrame, TxMessage};
use udp_stream::{UdpListener, UdpStream};

#[derive(Debug, Clone)]
pub struct KiaHandler {
    config: Arc<Config>,
    ha_sensors: Arc<HashMap<String, Arc<sensor::HaSensorHandler>>>,
    event_sender: Arc<UnboundedSender<HaStateEvent>>,
}

impl KiaHandler {
    pub fn new(
        config: Arc<Config>,
        ha_sensors: Arc<HashMap<String, Arc<sensor::HaSensorHandler>>>,
        event_sender: Arc<UnboundedSender<HaStateEvent>>,
    ) -> Arc<Self> {
        Arc::new(KiaHandler {
            config,
            ha_sensors,
            event_sender,
        })
    }

    pub async fn run(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                if let Err(err) = self.inner().await {
                    error!("KiaHandler error: {:?}", err);
                }
            }
        });
    }

    async fn dispatch_txframe(&self, txframe: TxFrame) {
        self.ha_sensors
            .get("last_communication")
            .unwrap()
            .update(chrono::Local::now().format("%+").to_string().into())
            .await;
        match txframe {
            TxFrame::Obd2Pid(types::Pid::BmsPid(bms_pid)) => {
                self.ha_sensors
                    .get("hv_soc")
                    .unwrap()
                    .update(bms_pid.hv_soc.into())
                    .await;
                self.ha_sensors
                    .get("aux_voltage")
                    .unwrap()
                    .update(bms_pid.aux_dc_voltage.into())
                    .await;
                self.ha_sensors
                    .get("hv_temperature_max")
                    .unwrap()
                    .update(bms_pid.hv_max_temp.into())
                    .await;
                self.ha_sensors
                    .get("hv_power")
                    .unwrap()
                    .update((bms_pid.hv_dc_voltage * bms_pid.hv_battery_current).into())
                    .await;
            }
            TxFrame::Obd2Pid(types::Pid::OnBoardChargerPid(on_board_charger_pid)) => {
                self.ha_sensors
                    .get("obc_temperature")
                    .unwrap()
                    .update(on_board_charger_pid.obc_temperature_a.into())
                    .await;
            }
            TxFrame::Modem(types::Modem::GnssFix(fix)) => {
                self.event_sender
                    .send(HaStateEvent::UpdateLocation(UpdateLocation {
                        gps: (fix.latitude, fix.longitude),
                        gps_accuracy: fix.accuracy as i32,
                        battery: 0,
                    }))
                    .unwrap();
            }
            TxFrame::Modem(types::Modem::Battery {
                voltage,
                low_voltage,
                soc,
                charging,
            }) => {
                self.ha_sensors
                    .get("modem_soc")
                    .unwrap()
                    .update(soc.into())
                    .await;
                self.ha_sensors
                    .get("modem_voltage")
                    .unwrap()
                    .update(voltage.into())
                    .await;
            }
            _ => {}
        }
    }

    async fn inner(&self) -> anyhow::Result<()> {
        //let mut listener =
        //    UdpListener::bind(SocketAddr::from(([0, 0, 0, 0], self.config.kia.port))).await?;
        let shared_key_bytes = include_bytes!("../../shared_key.bin");
        info!("Shared key: {:?}", shared_key_bytes);
        let shared_key = SharedKey::new(shared_key_bytes.clone());

        let socket =
            tokio::net::UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], self.config.kia.port)))
                .await?;

        info!("KiaHandler listening on port: {}", self.config.kia.port);
        let mut buffer = [0u8; 1024];
        loop {
            let (n, peer) = socket.recv_from(&mut buffer).await?;
            let data = buffer[..n].to_vec();
            if let Some(duplicated) = &self.config.kia.duplicated {
                socket.send_to(&data, duplicated).await?;
            }
            match EncryptedMessage::deserialize(data) {
                Ok(encrypted_message) => {
                    match TxMessage::decrypt_owned(&encrypted_message, &shared_key) {
                        Ok(txmessage) => {
                            info!("Received txmessage: {:?} from: {:?}", txmessage, peer);
                            self.dispatch_txframe(txmessage.frame).await;
                        }
                        Err(err) => {
                            error!("Error decrypting message: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    error!("Error deserializing message: {:?}", err);
                }
            }
        }

        Ok(())
    }
}
