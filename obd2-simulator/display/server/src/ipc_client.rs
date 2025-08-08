use std::{net::Ipv4Addr, sync::atomic::Ordering};

use ipc::{DisplayIndex, Ieee802154State, Ipc as _};
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
            let mut lcd_events = ipc_client.lcd_events().await.unwrap();
            tokio::spawn(async move {
                loop {
                    let event = lcd_events.recv().await.unwrap().unwrap();
                    info!("LCD event: {:?}", event);
                    match event {
                        types::LcdEvent::Main => {
                            crate::tasks::lcd::EVENTS
                                .send(crate::lcd::LcdEvent::Main)
                                .await;
                        }
                        types::LcdEvent::PowerOff => {
                            crate::tasks::lcd::EVENTS
                                .send(crate::lcd::LcdEvent::PowerOff)
                                .await;
                        }
                        types::LcdEvent::Charging => {
                            crate::tasks::lcd::EVENTS
                                .send(crate::lcd::LcdEvent::Charging)
                                .await;
                        }
                    }
                }
            });

            let mut obd2_pids = ipc_client.obd2_pids().await.unwrap();
            tokio::spawn(async move {
                loop {
                    let event = obd2_pids.recv().await.unwrap().unwrap();
                    crate::tasks::obd2::EVENTS.send(event).await;
                }
            });

            let mut ieee802154 = ipc_client.ieee802154().await.unwrap();
            tokio::spawn(async move {
                loop {
                    let event = ieee802154.recv().await.unwrap().unwrap();
                    match event {
                        Ieee802154State::LastSend(last_send) => {
                            crate::tasks::ieee802154::LAST_SEND.store(last_send, Ordering::Relaxed);
                        }
                        Ieee802154State::LastReceive(last_receive) => {
                            crate::tasks::ieee802154::LAST_RECEIVE
                                .store(last_receive, Ordering::Relaxed);
                        }
                        Ieee802154State::LastPosition(last_position) => {
                            crate::tasks::ieee802154::LAST_POSITION
                                .store(last_position, Ordering::Relaxed);
                        }
                    }
                }
            });

            while let Some((index, data)) = display_rx.recv().await {
                ipc_client.display_flush(index, data).await.unwrap();
            }
        });
    });
}
