use std::net::Ipv4Addr;

use ipc::{DisplayIndex, Ipc as _};
use remoc::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;

pub fn start(mut display_rx: UnboundedReceiver<(DisplayIndex, Vec<u8>)>) {
    std::thread::spawn(move || {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(async {
            let socket = tokio::net::TcpStream::connect((Ipv4Addr::LOCALHOST, ipc::TCP_PORT))
                .await
                .unwrap();
            let (socket_rx, socket_tx) = socket.into_split();

            let mut ipc_client: ipc::IpcClient =
                remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
                    .consume()
                    .await
                    .unwrap();

            let mut buttons = ipc_client.buttons().await.unwrap();
            tokio::spawn(async move {
                loop {
                    let event = buttons.recv().await.unwrap().unwrap();
                    info!("Button event: {:?}", event);
                    crate::tasks::buttons::EVENTS.send(event).await;
                }
            });

            let mut obd2_pids = ipc_client.obd2_pids().await.unwrap();
            tokio::spawn(async move {
                loop {
                    let event = obd2_pids.recv().await.unwrap().unwrap();
                    crate::tasks::obd2::EVENTS.send(event).await;
                }
            });

            while let Some((index, data)) = display_rx.recv().await {
                ipc_client.display_flush(index, data).await.unwrap();
            }
        });
    });
}
