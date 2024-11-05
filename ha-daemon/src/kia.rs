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
use types::{RxFrame, TxFrame};
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

    fn dispatch_txframe(&self, txframe: TxFrame) {
        self.ha_sensors
            .get("last_communication")
            .unwrap()
            .update(chrono::Local::now().format("%+").to_string().into());
        match txframe {
            TxFrame::Obd2Pid(types::Pid::BmsPid(bms_pid)) => {
                self.ha_sensors
                    .get("hv_soc")
                    .unwrap()
                    .update(bms_pid.hv_soc.into());
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
                self.ha_sensors.get("modem_soc").unwrap().update(soc.into());
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

        let mut socket =
            tokio::net::UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], self.config.kia.port)))
                .await?;

        info!("KiaHandler listening on port: {}", self.config.kia.port);
        let mut buffer = [0u8; 1024];
        loop {
            let (n, peer) = socket.recv_from(&mut buffer).await?;
            let data = buffer[..n].to_vec();
            match EncryptedMessage::deserialize(data) {
                Ok(encrypted_message) => {
                    match TxFrame::decrypt_owned(&encrypted_message, &shared_key) {
                        Ok(txframe) => {
                            info!("Received txframe: {:?} from: {:?}", txframe, peer);
                            self.dispatch_txframe(txframe);
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
