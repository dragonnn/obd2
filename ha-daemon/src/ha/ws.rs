use crate::prelude::*;
use futures_util::{future, pin_mut, SinkExt as _, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HaWsError {
    #[error("ws error")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("unknow message")]
    UnknownMessage,
    #[error("unexpected message")]
    UnexpectedMessage,
    #[error("eof")]
    Eof,
    #[error("timeout")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

impl HaWsError {
    pub fn is_ws(&self) -> bool {
        matches!(self, Self::Ws(_))
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
}

pub type HaWsResult<T> = Result<T, HaWsError>;

#[derive(Debug)]
pub struct HaWs {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl HaWs {
    pub async fn new(config: &crate::config::Config) -> HaWsResult<Self> {
        let (ws, response) = tokio_tungstenite::connect_async(
            format!("ws://{}/api/websocket", config.ha.host).as_str(),
        )
        .await?;

        let mut ret = Self { ws };

        if let crate::ha::IncomingMessage::Auth(auth) = ret.next().await? {
            info!("auth: {:?}", auth);
            if auth.r#type != crate::ha::auth::AuthState::Required {
                return Err(HaWsError::UnexpectedMessage);
            }
            let auth = crate::ha::auth::OutgoingAuth::new(config.ha.auth.clone());
            ret.send(crate::ha::OutgoingMessage::Auth(auth)).await?;
        }

        Ok(ret)
    }

    pub async fn next(&mut self) -> HaWsResult<crate::ha::IncomingMessage> {
        loop {
            let msg = tokio::time::timeout(std::time::Duration::from_secs(5), self.ws.next())
                .await?
                .ok_or(HaWsError::Eof)??;
            match msg {
                Message::Text(text) => {
                    let ret = serde_json::from_str(&text)?;
                    trace!("recv: {}", text);
                    return Ok(ret);
                }
                Message::Ping(ping) => {
                    self.ws.send(Message::Pong(ping)).await?;
                }
                _ => return Err(HaWsError::UnknownMessage),
            }
        }
    }

    pub async fn send(&mut self, msg: crate::ha::OutgoingMessage) -> HaWsResult<()> {
        let text = serde_json::to_string(&msg)?;
        trace!("send: {}", text);
        self.ws.send(Message::Text(text.into())).await?;
        Ok(())
    }
}
