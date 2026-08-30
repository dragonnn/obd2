use crate::prelude::*;
use crate::HaStateEvent;
use futures_util::{SinkExt as _, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

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
    #[error("websocket writer is closed")]
    Closed,
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
    outgoing: UnboundedSender<crate::ha::OutgoingMessage>,
    reader: tokio::task::JoinHandle<()>,
}

impl Drop for HaWs {
    fn drop(&mut self) {
        self.reader.abort();
    }
}

impl HaWs {
    pub async fn new(
        config: &crate::config::Config,
        event_sender: UnboundedSender<HaStateEvent>,
    ) -> HaWsResult<Self> {
        let (mut ws, _response) = tokio_tungstenite::connect_async(
            format!("ws://{}/api/websocket", config.ha.host).as_str(),
        )
        .await?;

        loop {
            let auth_message = tokio::time::timeout(Duration::from_secs(5), ws.next())
                .await?
                .ok_or(HaWsError::Eof)??;
            match auth_message {
                Message::Text(text) => {
                    let crate::ha::IncomingMessage::Auth(auth) = serde_json::from_str(&text)?
                    else {
                        return Err(HaWsError::UnexpectedMessage);
                    };
                    info!("auth: {:?}", auth);
                    if auth.r#type != crate::ha::auth::AuthState::Required {
                        return Err(HaWsError::UnexpectedMessage);
                    }
                    let auth = crate::ha::auth::OutgoingAuth::new(config.ha.auth.clone());
                    let text = serde_json::to_string(&crate::ha::OutgoingMessage::Auth(auth))?;
                    ws.send(Message::Text(text.into())).await?;
                    break;
                }
                Message::Ping(ping) => {
                    ws.send(Message::Pong(ping)).await?;
                }
                Message::Pong(_) => {}
                _ => return Err(HaWsError::UnexpectedMessage),
            }
        }

        let (mut sink, mut stream) = ws.split();
        let (outgoing, mut outgoing_receiver) = unbounded_channel();
        let reader = tokio::spawn(async move {
            loop {
                tokio::select! {
                    outgoing = outgoing_receiver.recv() => {
                        let Some(outgoing) = outgoing else { break };
                        let text = match serde_json::to_string(&outgoing) {
                            Ok(text) => text,
                            Err(err) => {
                                event_sender.send(HaStateEvent::WebsocketFailure(format!("serialize: {err}"))).log();
                                break;
                            }
                        };
                        trace!("send: {}", text);
                        if let Err(err) = sink.send(Message::Text(text.into())).await {
                            event_sender.send(HaStateEvent::WebsocketFailure(format!("send: {err}"))).log();
                            break;
                        }
                    }
                    message = stream.next() => {
                        match message {
                            Some(Ok(Message::Text(text))) => {
                                trace!("recv: {}", text);
                                match serde_json::from_str::<crate::ha::IncomingMessage>(&text) {
                                    Ok(crate::ha::IncomingMessage::Result {
                                        id,
                                        r#type,
                                        success,
                                    }) => {
                                        debug!("Home Assistant websocket result: id={id}, type={type}, success={success}");
                                    }
                                    Ok(_) => {}
                                    Err(err) => {
                                        event_sender
                                            .send(HaStateEvent::WebsocketFailure(format!("decode: {err}")))
                                            .log();
                                        break;
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(ping))) => {
                                if let Err(err) = sink.send(Message::Pong(ping)).await {
                                    event_sender.send(HaStateEvent::WebsocketFailure(format!("pong: {err}"))).log();
                                    break;
                                }
                            }
                            Some(Ok(Message::Pong(_))) => {}
                            Some(Ok(Message::Close(frame))) => {
                                event_sender.send(HaStateEvent::WebsocketFailure(format!("closed: {frame:?}"))).log();
                                break;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(err)) => {
                                event_sender.send(HaStateEvent::WebsocketFailure(format!("read: {err}"))).log();
                                break;
                            }
                            None => {
                                event_sender.send(HaStateEvent::WebsocketFailure("eof".to_owned())).log();
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { outgoing, reader })
    }

    pub async fn send(&mut self, msg: crate::ha::OutgoingMessage) -> HaWsResult<()> {
        self.outgoing.send(msg).map_err(|_| HaWsError::Closed)?;
        Ok(())
    }
}
