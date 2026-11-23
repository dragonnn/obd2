//! Postcard-RPC server on the dedicated Lyra UART (COBS framing, 115200 8N1).

use embassy_executor::Spawner;
use embassy_nrf::buffered_uarte::{BufferedUarte, BufferedUarteRx, BufferedUarteTx};
use embassy_sync_0_7::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use lyra_protocol::{ControlId, EchoEndpoint, EchoPayload, EchoReplyTopic, ShutdownAckTopic};
use postcard_rpc::{
    define_dispatch,
    header::VarHeader,
    server::{
        Dispatch, Sender, Server,
        impls::embedded_io_async_v0_7::{
            WireStorage,
            dispatch_impl::{WireRxImpl, WireSpawnImpl, WireTxImpl},
        },
    },
};
use static_cell::ConstStaticCell;

// define_dispatch only uses the spawn function for spawned handlers.
#[allow(unused_imports)]
use postcard_rpc::server::impls::embedded_io_async_v0_7::dispatch_impl::spawn_fn;

type UartTx = WireTxImpl<CriticalSectionRawMutex, BufferedUarteTx<'static>>;
type UartRx = WireRxImpl<BufferedUarteRx<'static>>;
pub type RpcSender = Sender<UartTx>;
pub type RpcServer = Server<UartTx, UartRx, &'static mut [u8], EchoDispatcher>;

const FRAME_BYTES: usize = 1024;
static WIRE: WireStorage<
    BufferedUarteRx<'static>,
    BufferedUarteTx<'static>,
    CriticalSectionRawMutex,
    FRAME_BYTES,
    FRAME_BYTES,
> = WireStorage::new();
static RX_FRAME: ConstStaticCell<[u8; FRAME_BYTES]> = ConstStaticCell::new([0; FRAME_BYTES]);

// The dispatcher uses these macros in const expressions. Override defmt's
// versions only in this module, including the dispatcher's generated modules.
macro_rules! assert {
    ($($args:tt)*) => { core::assert!($($args)*) };
}

macro_rules! unreachable {
    ($($args:tt)*) => { core::unreachable!($($args)*) };
}

define_dispatch! {
    app: EchoDispatcher;
    spawn_fn: spawn_fn;
    tx_impl: UartTx;
    spawn_impl: WireSpawnImpl;
    context: ();

    endpoints: {
        list: lyra_protocol::ENDPOINTS;
        | EndpointTy   | kind     | handler |
        | ----------   | ----     | ------- |
        | EchoEndpoint | blocking | echo    |
    };
    topics_in: {
        list: lyra_protocol::TOPICS_IN;
        | TopicTy          | kind  | handler      |
        | -------          | ----  | -------      |
        | EchoReplyTopic   | async | echo_reply   |
        | ShutdownAckTopic | async | shutdown_ack |
    };
    topics_out: {
        list: lyra_protocol::TOPICS_OUT;
    };
}

fn echo(_context: &mut (), _header: VarHeader, payload: EchoPayload) -> EchoPayload {
    payload
}

async fn echo_reply(_context: &mut (), _header: VarHeader, id: ControlId, _tx: &RpcSender) {
    super::state::sender()
        .send(super::state::LyraEvent::EchoReply { id })
        .await
        .unwrap();
}

async fn shutdown_ack(_context: &mut (), _header: VarHeader, id: ControlId, _tx: &RpcSender) {
    super::state::sender()
        .send(super::state::LyraEvent::ShutdownAck { id })
        .await
        .unwrap();
}

/// Initialize before spawning the controller so it never has to wait for UART
/// setup while handling an ignition event.
pub fn init(uart: BufferedUarte<'static>, spawner: Spawner) -> (RpcServer, RpcSender) {
    let (uart_rx, uart_tx) = uart.split();
    let (rx, tx) = WIRE.init(uart_rx, uart_tx).unwrap();
    let dispatcher = EchoDispatcher::new((), spawner.into());
    let key_kind = dispatcher.min_key_len();
    let server: RpcServer =
        Server::new(tx, rx, RX_FRAME.take().as_mut_slice(), dispatcher, key_kind);
    let sender = server.sender();
    (server, sender)
}

#[embassy_executor::task]
pub async fn run(mut server: RpcServer) {
    info!("Lyra UART RPC server ready");
    loop {
        let error = server.run().await;
        warn!("Lyra UART RPC transport error: {:?}", error);
        Timer::after_millis(100).await;
    }
}
