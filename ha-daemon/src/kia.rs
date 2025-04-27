use crate::{config::Config, ha::device::UpdateLocation, sensor, HaStateEvent};
use openssl::ssl::{Ssl, SslContext, SslFiletype, SslMethod};
use postcard::{from_bytes, to_stdvec, to_vec};
use remoc::rch;
use serde_encrypt::{shared_key::SharedKey, traits::SerdeEncryptSharedKey as _, EncryptedMessage};
use std::{
    collections::HashMap, net::SocketAddr, path::PathBuf, pin::Pin, sync::Arc, time::Duration,
};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
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

    pub async fn run(
        self: Arc<Self>,
        mut rpc_custom_frame_receiver: UnboundedReceiver<(
            types::Obd2Frame,
            rch::oneshot::Sender<()>,
            rch::oneshot::Sender<types::Obd2Frame>,
        )>,
    ) {
        tokio::spawn(async move {
            loop {
                if let Err(err) = self.inner(&mut rpc_custom_frame_receiver).await {
                    error!("KiaHandler error: {:?}", err);
                }
            }
        });
    }

    async fn dispatch_txframe(&self, txframe: &TxFrame) {
        self.ha_sensors
            .get("last_communication")
            .unwrap()
            .update(chrono::Local::now().format("%+").to_string().into())
            .await;
        if let TxFrame::Obd2Pid(_) = txframe {
            self.ha_sensors
                .get("obd2_last_communication")
                .unwrap()
                .update(chrono::Local::now().format("%+").to_string().into())
                .await;
        }
        match txframe {
            TxFrame::State(state) => {
                let state = match state {
                    types::State::IgnitionOff => "IgnitionOff",
                    types::State::IgnitionOn => "IgnitionOn",
                    types::State::Shutdown(duration) => {
                        self.ha_sensors
                            .get("shutdown_duration")
                            .unwrap()
                            .update((duration.as_secs() / 60).into())
                            .await;
                        "Shutdown"
                    }
                    types::State::Charging => "Charging",
                    types::State::CheckCharging => "CheckCharging",
                }
                .to_string();

                self.ha_sensors
                    .get("state")
                    .unwrap()
                    .update(state.into())
                    .await;
            }
            TxFrame::Obd2Pid(types::Pid::Icu2Pid(icu2_pid)) => {
                let trunk_open = if icu2_pid.trunk_open { "on" } else { "off" };
                self.ha_sensors
                    .get("trunk_open")
                    .unwrap()
                    .update(trunk_open.into())
                    .await;

                //driver_door_open
                let driver_door_open = if icu2_pid.actuator_back_door_driver_side_unlock {
                    "on"
                } else {
                    "off"
                };
                self.ha_sensors
                    .get("driver_door_open")
                    .unwrap()
                    .update(driver_door_open.into())
                    .await;

                //passenger_door_open
                let passenger_door_open = if icu2_pid.actuator_back_door_passenger_side_unlock {
                    "on"
                } else {
                    "off"
                };

                self.ha_sensors
                    .get("passenger_door_open")
                    .unwrap()
                    .update(passenger_door_open.into())
                    .await;

                //engine_hood_open
                let engine_hood_open = if icu2_pid.engine_hood_open {
                    "on"
                } else {
                    "off"
                };
                self.ha_sensors
                    .get("engine_hood_open")
                    .unwrap()
                    .update(engine_hood_open.into())
                    .await;
            }
            TxFrame::Obd2Pid(types::Pid::Icu1Smk(icu_smk_1)) => {
                self.ha_sensors
                    .get("aux_voltage")
                    .unwrap()
                    .update(icu_smk_1.aux_battery_voltage_power_load.into())
                    .await;
            }
            TxFrame::Obd2Pid(types::Pid::BmsPid(bms_pid)) => {
                self.ha_sensors
                    .get("hv_soc")
                    .unwrap()
                    .update(bms_pid.hv_soc.into())
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
                self.ha_sensors
                    .get("hv_voltage")
                    .unwrap()
                    .update(bms_pid.hv_dc_voltage.into())
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
                self.ha_sensors
                    .get("location_last_communication")
                    .unwrap()
                    .update(chrono::Local::now().format("%+").to_string().into())
                    .await;
                self.event_sender
                    .send(HaStateEvent::UpdateLocation(UpdateLocation {
                        gps: (fix.latitude, fix.longitude),
                        gps_accuracy: fix.accuracy as i32,
                        battery: 0,
                    }))
                    .unwrap();
            }
            TxFrame::Modem(types::Modem::GnssState(state)) => {
                let state = match state {
                    types::GnssState::ContinuousFix => "ContinuousFix".to_string(),
                    types::GnssState::BackupMode => "BackupMode".to_string(),
                    types::GnssState::DisablingBackup => "DisablingBackup".to_string(),
                    types::GnssState::SingleFix => "SingleFix".to_string(),
                    types::GnssState::ErrorDisablingBackup => "ErrorDisablingBackup".to_string(),
                };
                info!("GnssState: {}", state);
                self.ha_sensors
                    .get("modem_gnss_state")
                    .unwrap()
                    .update(state.into())
                    .await;
            }
            TxFrame::Temperature(temperature) => {
                self.ha_sensors
                    .get("obd2_temperature")
                    .unwrap()
                    .update((*temperature).into())
                    .await;
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
                    .update((*soc).into())
                    .await;
                self.ha_sensors
                    .get("modem_voltage")
                    .unwrap()
                    .update((*voltage).into())
                    .await;
            }
            _ => {}
        }
    }

    async fn inner(
        &self,
        rpc_custom_frame_receiver: &mut UnboundedReceiver<(
            types::Obd2Frame,
            rch::oneshot::Sender<()>,
            rch::oneshot::Sender<types::Obd2Frame>,
        )>,
    ) -> anyhow::Result<()> {
        //let mut listener =
        //    UdpListener::bind(SocketAddr::from(([0, 0, 0, 0], self.config.kia.port))).await?;
        let shared_key_bytes = include_bytes!("../../shared_key.bin");
        let shared_key = SharedKey::new(shared_key_bytes.clone());

        let socket =
            tokio::net::UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], self.config.kia.port)))
                .await?;
        let mut peer = None;

        info!("KiaHandler listening on port: {}", self.config.kia.port);
        let mut buffer = [0u8; 1024];
        loop {
            let (n, new_peer) = socket.recv_from(&mut buffer).await?;
            let data = buffer[..n].to_vec();
            if let Some(duplicated) = &self.config.kia.duplicated {
                socket.send_to(&data, duplicated).await?;
            }
            match EncryptedMessage::deserialize(data) {
                Ok(encrypted_message) => {
                    match TxMessage::decrypt_owned(&encrypted_message, &shared_key) {
                        Ok(txmessage) => {
                            self.ha_sensors
                                .get("peer")
                                .unwrap()
                                .update(new_peer.to_string().into())
                                .await;
                            info!("Received txmessage: {:?} from: {:?}", txmessage, peer);
                            peer = Some(new_peer);
                            self.dispatch_txframe(&txmessage.frame).await;
                            if txmessage.needs_ack() || txmessage.ack {
                                if let Some(peer) = &peer {
                                    let ack = types::RxMessage::new(types::RxFrame::TxFrameAck(
                                        txmessage.id,
                                    ))
                                    .to_vec_encrypted()
                                    .unwrap();
                                    if let Err(err) = socket.send_to(ack.as_slice(), peer).await {
                                        error!("error sending ack frame: {:?} to: {:?}", err, peer);
                                    } else {
                                        info!("Sent ack frame to: {:?}", peer);
                                    }
                                } else {
                                    error!("No peer to send ack frame to");
                                }
                            } else {
                                info!("No ack needed for txmessage: {:?}", txmessage);
                            }
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
            if let Ok((frame, tx_sended, tx_response)) = rpc_custom_frame_receiver.try_recv() {
                let txmessage: types::RxMessage = types::RxFrame::Obd2Frame(frame).into();
                let encrypted_message = txmessage.to_vec_encrypted().unwrap();
                socket
                    .send_to(encrypted_message.as_slice(), new_peer)
                    .await?;

                tx_sended.send(()).unwrap();
                let rx_frame =
                    timeout(Duration::from_secs(120), socket.recv_from(&mut buffer)).await;
                match rx_frame {
                    Ok(Ok((n, _))) => {
                        let data = buffer[..n].to_vec();
                        match EncryptedMessage::deserialize(data) {
                            Ok(encrypted_message) => {
                                match TxMessage::decrypt_owned(&encrypted_message, &shared_key) {
                                    Ok(txmessage) => {
                                        info!("Received txframe: {:?}", txmessage);
                                        match txmessage.frame {
                                            types::TxFrame::Obd2Frame(obd2_frame) => {
                                                tx_response.send(obd2_frame).unwrap();
                                            }
                                            _ => {
                                                error!("Invalid txframe: {:?}", txmessage);
                                            }
                                        }
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
                    Ok(Err(err)) => {
                        error!("Error receiving ack frame: {:?}", err);
                    }
                    Err(_) => {
                        error!("Timeout waiting for ack frame");
                    }
                }
            }
        }

        Ok(())
    }
}
