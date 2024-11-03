use crate::prelude::*;
use futures_util::{future, pin_mut, SinkExt as _, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub struct HaWs {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl HaWs {
    pub async fn new(config: &crate::config::Config) -> anyhow::Result<Self> {
        let (ws, response) = tokio_tungstenite::connect_async(
            format!("ws://{}/api/websocket", config.ha.host).as_str(),
        )
        .await?;

        let mut ret = Self { ws };

        if let crate::ha::IncomingMessage::Auth(auth) = ret.next().await? {
            info!("auth: {:?}", auth);
            if auth.r#type != crate::ha::auth::AuthState::Required {
                return Err(anyhow::anyhow!("unexpected message"));
            }
            let auth = crate::ha::auth::OutgoingAuth::new(config.ha.auth.clone());
            ret.send(crate::ha::OutgoingMessage::Auth(auth)).await?;
        }

        Ok(ret)
    }

    pub async fn next(&mut self) -> anyhow::Result<crate::ha::IncomingMessage> {
        loop {
            let msg = tokio::time::timeout(std::time::Duration::from_secs(5), self.ws.next())
                .await?
                .ok_or(anyhow::anyhow!("EOF"))??;
            match msg {
                Message::Text(text) => {
                    info!("text: {}", text);
                    let ret = serde_json::from_str(&text)?;
                    return Ok(ret);
                }
                Message::Ping(ping) => {
                    self.ws.send(Message::Pong(ping)).await?;
                }
                _ => return Err(anyhow::anyhow!("unexpected message")),
            }
        }
    }

    pub async fn send(&mut self, msg: crate::ha::OutgoingMessage) -> anyhow::Result<()> {
        let text = serde_json::to_string(&msg)?;
        info!("send: {}", text);
        self.ws.send(Message::Text(text)).await?;
        Ok(())
    }
}
