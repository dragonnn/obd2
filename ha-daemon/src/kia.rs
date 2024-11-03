use crate::config::Config;
use openssl::ssl::{SslContext, SslFiletype, SslMethod};
use postcard::{from_bytes, to_stdvec, to_vec};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::time::timeout;
use types::RxFrame;

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
        let key: PathBuf = ["certs", "server-key.pem"].iter().collect();
        let cert: PathBuf = ["certs", "server-cert.pem"].iter().collect();
        let ca: PathBuf = ["certs", "ca-cert.pem"].iter().collect();

        let mut ctx = SslContext::builder(SslMethod::dtls()).unwrap();
        ctx.set_private_key_file(key, SslFiletype::PEM).unwrap();
        ctx.set_certificate_chain_file(cert).unwrap();
        ctx.set_ca_file(ca).unwrap();
        ctx.check_private_key().unwrap();
        let ctx = ctx.build();

        let udp_socket = tokio::net::UdpSocket::bind(("0.0.0.0", self.config.kia.port)).await?;

        let mut dtls_socket = tokio_dtls_stream_sink::Server::new(udp_socket);

        info!("KiaHandler listening on port: {}", self.config.kia.port);
        loop {
            match dtls_socket.accept(Some(&ctx)).await {
                Ok(connection) => {
                    info!("New connection from: {:?}", connection.peer());
                    let handler = KiaSessionHandler::new(connection, self.clone());
                    handler.run().await;
                }
                Err(err) => {
                    error!("Error accepting connection: {:?}", err);
                }
            }
        }

        Ok(())
    }
}

pub struct KiaSessionHandler {
    session: tokio_dtls_stream_sink::Session,
    handler: KiaHandler,
}

impl KiaSessionHandler {
    pub fn new(session: tokio_dtls_stream_sink::Session, handler: KiaHandler) -> Self {
        KiaSessionHandler { session, handler }
    }

    pub async fn run(mut self) {
        tokio::spawn(async move {
            loop {
                if let Err(err) = self.inner().await {
                    error!("KiaSessionHandler error: {:?}", err);
                }
            }
        });
    }

    async fn inner(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];
        loop {
            match timeout(
                Duration::from_secs(self.handler.config.kia.timeout),
                self.session.read(&mut buf),
            )
            .await
            {
                Ok(Ok(n)) => {
                    let data = &buf[..n];
                    info!("Received {} bytes: {:?}", n, data);
                    match from_bytes::<types::TxFrame>(data) {
                        Ok(tx_frame) => {
                            self.session
                                .write(&to_stdvec(&RxFrame::TxFrameAck).unwrap())
                                .await?;
                            info!("Received TxFrame: {:?}", tx_frame);
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
