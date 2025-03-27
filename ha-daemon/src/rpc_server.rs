//! This crate implements the server of the remote counting service.

use remoc::{codec, prelude::*};
use std::{net::Ipv4Addr, sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot::{Receiver as OneshotReceiver, Sender as OneshotSender},
        RwLock,
    },
    time::sleep,
};

use crate::rpc::{Rpc, RpcServerSharedMut, TCP_PORT};

/// Server object for the counting service, keeping the state.
#[derive()]
pub struct RpcServer {
    rpc_custom_frame_sender: UnboundedSender<(
        types::Obd2Frame,
        rch::oneshot::Sender<()>,
        rch::oneshot::Sender<types::Obd2Frame>,
    )>,
}

/// Implementation of remote counting service.
#[rtc::async_trait]
impl Rpc for RpcServer {
    async fn send_custom_frame(
        &self,
        frame: types::Obd2Frame,
    ) -> Result<
        (
            rch::oneshot::Receiver<()>,
            rch::oneshot::Receiver<types::Obd2Frame>,
        ),
        rtc::CallError,
    > {
        // Create a channel to send the custom frame
        let (tx_sended, rx_sended) = rch::oneshot::channel();
        let (tx_response, rx_response) = rch::oneshot::channel();

        // Send the custom frame to the server
        self.rpc_custom_frame_sender
            .send((frame, tx_sended, tx_response))
            .unwrap();
        Ok((rx_sended, rx_response))
    }
}

pub async fn start(
    rpc_custom_frame_sender: UnboundedSender<(
        types::Obd2Frame,
        rch::oneshot::Sender<()>,
        rch::oneshot::Sender<types::Obd2Frame>,
    )>,
) {
    tokio::spawn(async {
        // Create a counter object that will be shared between all clients.
        // You could also create one counter object per connection.
        let rpc_server = Arc::new(RwLock::new(RpcServer {
            rpc_custom_frame_sender,
        }));

        // Listen to TCP connections using Tokio.
        // In reality you would probably use TLS or WebSockets over HTTPS.
        info!("Listening on port {}. Press Ctrl+C to exit.", TCP_PORT);
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, TCP_PORT))
            .await
            .unwrap();

        loop {
            // Accept an incoming TCP connection.
            let (socket, addr) = listener.accept().await.unwrap();
            let (socket_rx, socket_tx) = socket.into_split();
            info!("Accepted connection from {}", addr);

            // Create a new shared reference to the counter object.
            let rpc_server = rpc_server.clone();

            // Spawn a task for each incoming connection.
            tokio::spawn(async move {
                // Create a server proxy and client for the accepted connection.
                //
                // The server proxy executes all incoming method calls on the shared counter_obj
                // with a request queue length of 1.
                //
                // Current limitations of the Rust compiler require that we explicitly
                // specify the codec.
                let (server, client) = RpcServerSharedMut::<_, codec::Default>::new(rpc_server, 1);

                // Establish a Remoc connection with default configuration over the TCP connection and
                // provide (i.e. send) the counter client to the client.
                remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
                    .provide(client)
                    .await
                    .unwrap();

                // Serve incoming requests from the client on this task.
                // `true` indicates that requests are handled in parallel.
                server.serve(true).await.unwrap();
            });
        }
    });
}
