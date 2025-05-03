use remoc::{codec, prelude::*};
use std::{net::Ipv4Addr, sync::Arc};
use types::Pid;

use tokio::{
    net::TcpListener,
    sync::{Mutex, RwLock, broadcast::Receiver, mpsc::UnboundedSender},
};

#[derive(Debug)]
pub struct IpcServer {
    display_buffers: [Arc<Mutex<Vec<u8>>>; 2],
    buttons_rx: Receiver<(u8, bool)>,
    obd2_pids_rx: Receiver<Pid>,
}

#[rtc::async_trait]
impl ipc::Ipc for IpcServer {
    async fn display_flush(
        &mut self,
        index: ipc::DisplayIndex,
        data: Vec<u8>,
    ) -> Result<(), rtc::CallError> {
        let index = index as usize;
        self.display_buffers[index]
            .lock()
            .await
            .copy_from_slice(&data);
        Ok(())
    }

    async fn buttons(&mut self) -> Result<rch::mpsc::Receiver<(u8, bool)>, rtc::CallError> {
        let (tx, rx) = rch::mpsc::channel(1);

        let mut buttons_rx = self.buttons_rx.resubscribe();
        tokio::spawn(async move {
            loop {
                let (button, pressed) = buttons_rx.recv().await.unwrap();
                if tx.send((button, pressed)).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }

    async fn obd2_pids(&mut self) -> Result<rch::mpsc::Receiver<Pid>, rtc::CallError> {
        let (tx, rx) = rch::mpsc::channel(1);

        let mut obd2_pids_rx = self.obd2_pids_rx.resubscribe();
        tokio::spawn(async move {
            loop {
                let pid = obd2_pids_rx.recv().await.unwrap();
                if tx.send(pid).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }
}

pub fn start(
    display_buffers: [Arc<Mutex<Vec<u8>>>; 2],
    buttons_rx: Receiver<(u8, bool)>,
    obd2_pids_rx: Receiver<Pid>,
    connected_tx: UnboundedSender<()>,
) {
    std::thread::spawn(move || {
        let tokio = tokio::runtime::Runtime::new().unwrap();
        tokio.block_on(async {
            let ipc_server = Arc::new(RwLock::new(IpcServer {
                display_buffers,
                buttons_rx,
                obd2_pids_rx,
            }));

            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, ipc::TCP_PORT))
                .await
                .unwrap();
            loop {
                // Accept an incoming TCP connection.
                let (socket, addr) = listener.accept().await.unwrap();
                let (socket_rx, socket_tx) = socket.into_split();
                info!("Accepted connection from {}", addr);

                let connected_tx = connected_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    connected_tx.send(()).unwrap();
                });

                let ipc_server = ipc_server.clone();
                // Spawn a task for each incoming connection.
                tokio::spawn(async move {
                    // Create a server proxy and client for the accepted connection.
                    //
                    // The server proxy executes all incoming method calls on the shared counter_obj
                    // with a request queue length of 1.
                    //
                    // Current limitations of the Rust compiler require that we explicitly
                    // specify the codec.
                    let (server, client) =
                        ipc::IpcServerSharedMut::<_, codec::Default>::new(ipc_server, 1);

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
    });
}
