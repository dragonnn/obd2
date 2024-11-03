#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{future, pin_mut, SinkExt as _, StreamExt};
use ha::device::UpdateSensor;
use ha::ws::HaWs;
use serde::{Deserialize, Serialize};
use statig::prelude::*;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

mod config;
mod ha;
mod kia;
mod prelude;
mod sensor;

use prelude::*;

#[derive(Debug)]
pub struct HaState {
    config: Arc<config::Config>,

    rest: reqwest::Client,
    ws: Option<HaWs>,

    webhook: Option<ha::device::WebhookInfo>,

    event_sender: Arc<UnboundedSender<HaStateEvent>>,

    ha_sensors: HashMap<String, Arc<sensor::HaSensorHandler>>,

    sensor_register: bool,
}

#[derive(Debug)]
pub enum HaStateEvent {
    Step,
    UpdateSensor(UpdateSensor),
}

#[state_machine(
    initial = "State::load()",
    state(derive(Debug)),
    superstate(derive(Debug)),
    on_dispatch = "Self::on_dispatch",
    on_transition = "Self::on_transition"
)]
impl HaState {
    #[action]
    async fn entry_load(&mut self) {
        self.ws = None;
        if let Ok(webhookinfo) = tokio::fs::read_to_string("WebHookInfo.json")
            .await
            .let_log()
        {
            self.webhook = serde_json::from_str(&webhookinfo).let_log().ok();
        }
        self.event_sender.send(HaStateEvent::Step).unwrap();
    }

    #[state(entry_action = "entry_load")]
    async fn load(&mut self, event: &HaStateEvent) -> Response<State> {
        info!("webhook: {:?}", self.webhook);
        if self.webhook.is_some() {
            Transition(State::connect())
        } else {
            warn!("webhook not found");
            Transition(State::register())
        }
    }

    #[action]
    async fn entry_register(&mut self) {
        info!("registering device");
        if let Ok(response) = self
            .rest
            .post(format!(
                "http://{}/api/mobile_app/registrations",
                self.config.ha.host
            ))
            .json(&ha::device::DeviceInfo {
                device_id: self.config.ha.device_id.to_owned(),
                app_id: self.config.ha.device_id.to_owned(),
                app_name: self.config.ha.device_id.to_owned(),
                app_version: "0.1.0".to_owned(),
                device_name: self.config.ha.device_id.to_owned(),
                manufacturer: self.config.ha.device_id.to_owned(),
                model: self.config.ha.device_id.to_owned(),
                os_name: "Linux".to_owned(),
                os_version: "Arch".to_owned(),
                supports_encryption: false,
                app_data: ha::device::AppData {
                    push_notification_key: self.config.ha.push_notification_key.to_owned(),
                },
            })
            .header("Authorization", format!("Bearer {}", self.config.ha.auth))
            .send()
            .await
            .let_log()
        {
            info!("response: {:?}", response);
            if let Ok(webhookinfo) = response.json::<ha::device::WebhookInfo>().await {
                tokio::fs::write(
                    "WebHookInfo.json",
                    serde_json::to_string(&webhookinfo).unwrap(),
                )
                .await
                .log();
                debug!("setting sensor_register to true");
                self.sensor_register = true;
            }
        }
        self.event_sender.send(HaStateEvent::Step).unwrap();
    }

    #[state(entry_action = "entry_register")]
    async fn register(&mut self, event: &HaStateEvent) -> Response<State> {
        Transition(State::load())
    }

    #[action]
    async fn entry_connect(&mut self) {
        if let Ok(ws) = HaWs::new(&self.config).await.let_log() {
            self.ws = Some(ws);
        }

        self.event_sender.send(HaStateEvent::Step).unwrap();
    }

    #[state(entry_action = "entry_connect")]
    async fn connect(&mut self, event: &HaStateEvent) -> Response<State> {
        if self.ws.is_some() {
            if self.sensor_register {
                Transition(State::sensors_register())
            } else {
                Transition(State::connected())
            }
        } else {
            Transition(State::load())
        }
    }

    #[action]
    async fn entry_connected(&mut self) {
        self.event_sender.send(HaStateEvent::Step).unwrap();
        for (_, sensor) in self.ha_sensors.iter() {
            sensor.clone().run().await;
        }
    }

    #[state(entry_action = "entry_connected")]
    async fn connected(&mut self, event: &HaStateEvent) -> Response<State> {
        if let Some(ws) = &mut self.ws {
            match event {
                HaStateEvent::UpdateSensor(update_sensor) => {
                    info!("update sensor: {:?}", update_sensor);
                    if ws
                        .send(ha::OutgoingMessage::DeviceWebHookHandle(
                            ha::device::WebHookHandle::update(
                                self.webhook.as_ref().unwrap().webhook_id.to_owned(),
                                update_sensor.clone(),
                            ),
                        ))
                        .await
                        .let_log()
                        .is_err()
                    {
                        return Transition(State::load());
                    }
                }
                _ => {}
            }
            ws.next().await.log();
            Handled
        } else {
            Transition(State::load())
        }
    }

    #[action]
    async fn entry_sensors_register(&mut self) {
        self.event_sender.send(HaStateEvent::Step).unwrap();
        if self.sensor_register {
            warn!("registering sensors");
            for (_, sensor) in self.ha_sensors.iter() {
                warn!("registering sensor: {:?}", sensor);
                let webhook_id = self.webhook.as_ref().unwrap().webhook_id.to_owned();
                let register_sensor = sensor.register();
                if let Some(ws) = &mut self.ws {
                    ws.send(ha::OutgoingMessage::DeviceWebHookHandle(
                        ha::device::WebHookHandle::register(webhook_id, register_sensor),
                    ))
                    .await
                    .log();
                }
            }
            self.sensor_register = false;
        }
    }

    #[state(entry_action = "entry_sensors_register")]
    async fn sensors_register(&mut self, event: &HaStateEvent) -> Response<State> {
        Transition(State::connected())
    }
}

impl HaState {
    fn on_transition(&mut self, source: &State, target: &State) {
        info!("transitioned from `{:?}` to `{:?}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<HaState>, event: &HaStateEvent) {
        info!("dispatched `{:?}` to `{:?}`", event, state);
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let config = Arc::new(config::Config::load());
    info!("found config: {:?}", config);
    let (event_sender, mut event_receiver) = unbounded_channel::<HaStateEvent>();
    let event_sender = Arc::new(event_sender);

    let ha_sensors = config
        .sensors
        .iter()
        .map(|(unique_id, ha)| {
            (
                unique_id.clone(),
                sensor::HaSensorHandler::new(
                    config.clone(),
                    unique_id,
                    ha.clone(),
                    event_sender.clone(),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut ha_state_machine = HaState {
        config: config.clone(),
        rest: reqwest::Client::new(),
        ws: None,
        webhook: None,
        event_sender: event_sender.clone(),
        ha_sensors,
        sensor_register: false,
    }
    .state_machine();
    info!("initial state: {:?}", ha_state_machine.state());

    event_sender.send(HaStateEvent::Step).unwrap();

    let kia = kia::KiaHandler::new(config);
    kia.run().await;

    loop {
        let event = event_receiver.recv().await.unwrap();
        info!("event: {:?}", event);
        ha_state_machine.handle(&event).await;
    }
}
