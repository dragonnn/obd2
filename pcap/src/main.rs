#[macro_use]
extern crate log;

use std::fs::File;

use can::frame::Frame as CanFrame;
use pcap_parser::{traits::PcapReaderIterator, *};

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("init");

    let file = File::open("bms.pcap").unwrap();
    let mut num_blocks = 0;

    let mut reader = LegacyPcapReader::new(65536, file).expect("LegacyPcapReader");
    let mut can_frames: Vec<CanFrame> = Vec::new();
    loop {
        match reader.next() {
            Ok((offset, block)) => {
                num_blocks += 1;
                match block {
                    PcapBlockOwned::LegacyHeader(_hdr) => {
                        //info!("hdr.network: {:?}", _hdr);
                    }
                    PcapBlockOwned::Legacy(b) => {
                        info!("b.data(): {:x?}", b.data);
                    }
                    PcapBlockOwned::NG(_) => unreachable!(),
                }
                reader.consume(offset);
            }
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete(_)) => {
                reader.refill().unwrap();
            }
            Err(e) => panic!("error while reading: {:?}", e),
        }
    }
    info!("num_blocks: {}", num_blocks);
}
