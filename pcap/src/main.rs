use std::fs::File;

use pcap_parser::{traits::PcapReaderIterator, *};

fn main() {
    let file = File::open("bms.pcap").unwrap();
    let mut num_blocks = 0;

    let mut reader = LegacyPcapReader::new(65536, file).expect("LegacyPcapReader");
    loop {
        match reader.next() {
            Ok((offset, block)) => {
                println!("got new block");
                num_blocks += 1;
                match block {
                    PcapBlockOwned::LegacyHeader(_hdr) => {
                        // save hdr.network (linktype)
                        println!("hdr.network: {:?}", _hdr);
                    }
                    PcapBlockOwned::Legacy(_b) => {
                        // use linktype to parse b.data()
                        println!("b.data(): {:?}", _b);
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
    println!("num_blocks: {}", num_blocks);
    println!("num_blocks: {}", num_blocks);
}
