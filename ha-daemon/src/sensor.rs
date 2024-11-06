use std::sync::Arc;

use crate::config::Config;
use crate::config::HaSensor;
use crate::db::DbHandle;
use crate::ha::device::RegisterSensor;
use crate::ha::device::UpdateSensor;
use crate::HaStateEvent;
//use jaq_interpret::{Ctx, Error, FilterT, ParseCtx, RcIter, Val};
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct HaSensorHandler {
    config: Arc<Config>,
    unique_id: String,
    ha: HaSensor,
    db: DbHandle,
    event_sender: Arc<UnboundedSender<HaStateEvent>>,
}

impl HaSensorHandler {
    pub async fn new(
        config: Arc<Config>,
        unique_id: &str,
        mut ha: HaSensor,
        db: DbHandle,
        event_sender: Arc<UnboundedSender<HaStateEvent>>,
    ) -> Arc<Self> {
        ha.ha.unique_id = unique_id.to_string();
        ha.ha.state = db.get(&unique_id).await;
        ha.ha.disabled = false;
        ha.ha.r#type = "sensor".to_string();
        Arc::new(HaSensorHandler {
            config,
            unique_id: unique_id.to_string(),
            ha,
            event_sender,
            db,
        })
    }

    pub fn register(&self) -> RegisterSensor {
        self.ha.ha.clone()
    }

    pub async fn update(&self, value: serde_json::Value) {
        self.db.set(&self.unique_id, value.clone()).await;
        self.event_sender
            .send(HaStateEvent::UpdateSensor(UpdateSensor {
                unique_id: self.unique_id.clone(),
                state: value,
                r#type: self.ha.ha.r#type.clone(),
                icon: self.ha.ha.icon.clone(),
            }))
            .unwrap();
    }
}
