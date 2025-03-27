#[macro_use]
extern crate log;

mod rpc;

use clap::Parser;
use remoc::prelude::*;
use rpc::{Rpc as _, RpcClient, TCP_PORT};
use std::net::Ipv4Addr;
use tokio::net::TcpStream;

/// Simple program to greet a person
#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    host: Option<String>,

    /// Number of times to greet
    #[command(subcommand)]
    command: ArgsCommand,
}

#[derive(Debug, clap::Subcommand)]
enum ArgsCommand {
    /// Send a custom frame
    SendCustomFrame(ArgsCommandSendCustomFrame),
}

fn parse_hex_u16(s: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(s.trim_start_matches("0x"), 16)
}

fn parse_hex_vec(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    info!("Parsing hex string: {}", s);
    // Remove "0x" prefix if present
    let s = s.trim_start_matches("0x");
    // Ensure the string length is even (each byte is 2 hex digits)
    if s.len() % 2 != 0 {
        panic!("Hex string must have an even length");
    }
    // Parse each pair of hex digits into a u8
    let parsed = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect();

    info!("Parsed hex string to bytes: {:?}", parsed);
    parsed
}

#[derive(Debug, clap::Parser)]
pub struct ArgsCommandSendCustomFrame {
    #[arg(short, long, value_parser = parse_hex_u16)]
    pub pid: u16,
    #[arg(short, long, value_parser = parse_hex_vec)]
    pub data: std::vec::Vec<u8>,
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_LOG_STYLE", "always");

    env_logger::init();

    let args = Args::parse();

    info!("Parsed arguments: {:x?}", args);

    let host = args
        .host
        .unwrap_or("127.0.0.1".to_string())
        .parse::<Ipv4Addr>()
        .unwrap_or_else(|e| panic!("Invalid host address: {}", e));

    let socket = TcpStream::connect((host, TCP_PORT)).await.unwrap();
    info!("Connected to server at {}:{}", host, TCP_PORT);
    let (socket_rx, socket_tx) = socket.into_split();

    // Establish a Remoc connection with default configuration over the TCP connection and
    // consume (i.e. receive) the counter client from the server.
    let client: RpcClient = remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
        .consume()
        .await
        .unwrap();

    match args.command {
        ArgsCommand::SendCustomFrame(args) => {
            info!("Sending custom frame with PID: 0x{:x}", args.pid);
            let (sended, response) = client
                .send_custom_frame(types::Obd2Frame {
                    pid: args.pid,
                    data: args.data,
                })
                .await
                .unwrap();
            info!("Custom frame sended to daemon");
            // Wait for the response
            let response_sended = sended.await.unwrap();
            info!(
                "Custom frame sended to obd2 dashboard: {:?}",
                response_sended
            );
            // Process the response
            let response = response.await.unwrap();
            info!(
                "Custom frame response processed successfully with: {:x?}",
                response
            );
        }
    }
}
