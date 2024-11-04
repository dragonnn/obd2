use crate::config::Config;
use openssl::ssl::{Ssl, SslContext, SslFiletype, SslMethod};
use postcard::{from_bytes, to_stdvec, to_vec};
use serde_encrypt::{shared_key::SharedKey, traits::SerdeEncryptSharedKey as _, EncryptedMessage};
use std::{net::SocketAddr, path::PathBuf, pin::Pin, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    time::timeout,
};
use tokio_openssl::SslStream;
use types::{RxFrame, TxFrame};
use udp_stream::{UdpListener, UdpStream};

#[derive(Debug, Clone)]
pub struct KiaHandler {
    config: Arc<Config>,
}

impl KiaHandler {
    pub fn new(config: Arc<Config>) -> Arc<Self> {
        Arc::new(KiaHandler { config })
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

pub struct KiaSessionHandler {
    //session: tokio_dtls_stream_sink::Session,
    session: Option<SslStream<UdpStream>>,
    handler: KiaHandler,
    ssl_context: SslContext,
}

impl KiaSessionHandler {
    pub fn new(handler: KiaHandler, ssl_context: SslContext) -> Self {
        KiaSessionHandler {
            session: None,
            handler,
            ssl_context,
        }
    }

    pub async fn run(mut self, session: UdpStream) {
        tokio::spawn(async move {
            timeout(Duration::from_secs(5), async {
                let mut dtls = SslStream::new(Ssl::new(&self.ssl_context)?, session)?;
                Pin::new(&mut dtls).accept().await?;
                self.session = Some(dtls);
                Ok::<(), anyhow::Error>(())
            })
            .await
            .unwrap()
            .unwrap();
            loop {
                if let Err(err) = self.inner().await {
                    error!("KiaSessionHandler error: {:?}", err);
                } else {
                    break;
                }
            }
            self.session.unwrap().shutdown().await.unwrap();
            info!("Session closed");
        });
    }

    async fn inner(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];
        let session = self.session.as_mut().unwrap();
        loop {
            match timeout(
                Duration::from_secs(self.handler.config.kia.timeout),
                session.read(&mut buf),
            )
            .await
            {
                Ok(Ok(n)) => {
                    let data = &buf[..n];
                    info!("Received {} bytes: {:?}", n, data);
                    match from_bytes::<types::TxFrame>(data) {
                        Ok(tx_frame) => {
                            session
                                .write(&to_stdvec(&RxFrame::TxFrameAck).unwrap())
                                .await?;
                            info!("Received TxFrame: {:?}", tx_frame);
                            if tx_frame.is_disconnect() {
                                info!("Disconnecting...");
                                break;
                            }
                        }
                        Err(err) => {
                            error!("Error parsing TxFrame: {:?}", err);
                        }
                    }
                }
                Ok(Err(err)) => {
                    error!("Error reading data: {:?}", err);
                    break;
                }
                Err(_err) => {
                    error!("Connection timeout");
                    break;
                }
            }
        }
        Ok(())
    }
}
