use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::Log as _;

#[derive(Debug, Clone)]
pub struct DbHandle {
    db: Arc<Mutex<Db>>,
    cancel: CancellationToken,
}

impl DbHandle {
    pub async fn new() -> Self {
        DbHandle {
            db: Arc::new(Mutex::new(Db::new().await)),
            cancel: CancellationToken::new(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let db = self.db.lock().await;
        db.get(key).cloned()
    }

    pub async fn set(&self, key: &str, value: serde_json::Value) {
        let mut db = self.db.lock().await;
        db.set(key, value);
    }

    pub async fn run(&self) -> tokio::task::JoinHandle<()> {
        let handle = self.clone();
        tokio::spawn(async move {
            handle.inner().await;
        })
    }

    async fn inner(&self) {
        let mut current_db = self.db.lock().await.clone();
        loop {
            let mut exit = false;
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {

                },
                _ = self.cancel.cancelled() => {
                    exit = true;
                }
            }

            let new_db = self.db.lock().await.clone();
            if new_db != current_db {
                current_db = new_db;
                if let Ok(json) = serde_json::to_string(&current_db) {
                    if let Err(e) = tokio::fs::write("Db.json", json).await {
                        error!("Failed to write Db.json: {}", e);
                    }
                }
            }
            if exit {
                break;
            }
        }
        info!("DbHandle exiting");
    }

    pub async fn stop(&self) {
        self.cancel.cancel();
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Db {
    storage: HashMap<String, serde_json::Value>,
}

impl Db {
    pub async fn new() -> Self {
        if let Ok(webhookinfo) = tokio::fs::read_to_string("Db.json").await.let_log() {
            if let Ok(db) = serde_json::from_str(&webhookinfo).let_log() {
                return db;
            }
        }
        Db {
            storage: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.storage.get(key)
    }

    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.storage.insert(key.to_string(), value);
    }
}
