//! Lyra-side handler for nRF manager control topics over UART.

use std::io;
use std::thread;
use std::time::Duration;

use lyra_protocol::{
    ControlId, EchoProbeTopic, EchoReplyTopic, ShutdownAckTopic, ShutdownRequestTopic,
};
use postcard_rpc::accumulator::raw::{CobsAccumulator, FeedResult};
use postcard_rpc::header::{VarHeader, VarKey, VarSeq};
use postcard_rpc::{Key, Topic};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::runtime::Builder;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tracing::{debug, error, info, trace, warn};

mod logging;

// U9 connects the nRF UART to Lyra UART1 (TX pin 15, RX pin 16).
const UART: &str = "/dev/ttyS1";
const BAUD: u32 = 115_200;
const FRAME_BYTES: usize = 1024;

pub fn start() -> io::Result<()> {
    let runtime = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    thread::Builder::new()
        .name("lyra-rpc".into())
        .spawn(move || {
            tracing::subscriber::with_default(logging::subscriber(), || runtime.block_on(run()));
        })?;
    Ok(())
}

async fn run() {
    let mut shutdown_started = false;
    let mut request_shutdown = || {
        if shutdown_started {
            debug!("Poweroff already requested");
            return;
        }
        match Command::new("/sbin/poweroff").spawn() {
            Ok(mut child) => {
                shutdown_started = true;
                info!("Poweroff started");
                // BusyBox syncs storage before notifying init. Keep answering probes.
                tokio::spawn(async move {
                    match child.wait().await {
                        Ok(status) if status.success() => {}
                        Ok(status) => error!(%status, "Poweroff failed"),
                        Err(error) => error!(%error, "Could not wait for poweroff"),
                    }
                });
            }
            Err(error) => error!(%error, "Could not start poweroff"),
        }
    };

    loop {
        match tokio_serial::new(UART, BAUD).open_native_async() {
            Ok(mut port) => {
                info!(port = UART, baud = BAUD, "UART RPC listening");
                if let Err(error) = serve(&mut port, &mut request_shutdown).await {
                    warn!(port = UART, %error, "UART RPC disconnected");
                }
            }
            Err(error) => warn!(port = UART, %error, "Could not open RPC UART"),
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn serve(port: &mut SerialStream, request_shutdown: &mut impl FnMut()) -> io::Result<()> {
    let mut accumulator = CobsAccumulator::<FRAME_BYTES>::new();
    let mut input = [0; 256];
    let mut discarding = false;
    loop {
        let count = port.read(&mut input).await?;
        if count == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "UART closed"));
        }

        for chunk in input[..count].split_inclusive(|byte| *byte == 0) {
            let ends_frame = chunk.last() == Some(&0);
            if discarding {
                discarding = !ends_frame;
                continue;
            }
            match accumulator.feed(chunk) {
                FeedResult::Success { data, .. } => {
                    handle_frame(port, data, request_shutdown).await?;
                }
                FeedResult::OverFull(_) => {
                    trace!("Discarding oversized UART frame");
                    discarding = !ends_frame;
                }
                FeedResult::DeserError(_) => trace!("Discarding invalid COBS frame"),
                FeedResult::Consumed => {}
            }
        }
    }
}

async fn handle_frame(
    port: &mut SerialStream,
    frame: &[u8],
    request_shutdown: &mut impl FnMut(),
) -> io::Result<()> {
    let Some((header, body)) = VarHeader::take_from_slice(frame) else {
        trace!("Discarding frame with invalid RPC header");
        return Ok(());
    };
    let (reply_key, shutdown) = if header.key == VarKey::Key8(EchoProbeTopic::TOPIC_KEY) {
        (EchoReplyTopic::TOPIC_KEY, false)
    } else if header.key == VarKey::Key8(ShutdownRequestTopic::TOPIC_KEY) {
        (ShutdownAckTopic::TOPIC_KEY, true)
    } else {
        trace!(key = ?header.key, "Ignoring unhandled RPC key");
        return Ok(());
    };
    let seq = i32::from(header.seq_no);
    let Ok((id, [])) = postcard::take_from_bytes::<ControlId>(body) else {
        trace!(seq, "Discarding frame with invalid control ID");
        return Ok(());
    };

    if shutdown {
        info!(id, seq, "Shutdown request received");
    } else {
        debug!(id, seq, "Echo probe received");
    }
    send_reply(port, reply_key, header.seq_no, id).await?;
    if shutdown {
        // Wait for the UART to drain before poweroff can stop the dashboard.
        io::Write::flush(port)?;
        info!(id, seq, "Shutdown acknowledgement sent");
        request_shutdown();
    } else {
        debug!(id, seq, "Echo reply sent");
    }
    Ok(())
}

async fn send_reply(
    port: &mut SerialStream,
    key: Key,
    seq_no: VarSeq,
    id: ControlId,
) -> io::Result<()> {
    // A full RPC header takes at most 13 bytes and a postcard u64 at most 10.
    let mut frame = [0; 23];
    let header = VarHeader {
        key: VarKey::Key8(key),
        seq_no,
    };
    let (header_bytes, body) = header.write_to_slice(&mut frame).expect("header fits");
    let frame_len = header_bytes.len() + postcard::to_slice(&id, body).expect("ID fits").len();
    let mut encoded = [0; cobs::max_encoding_length(23) + 1];
    let len = cobs::encode(&frame[..frame_len], &mut encoded);
    port.write_all(&encoded[..=len]).await
}
