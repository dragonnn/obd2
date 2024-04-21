use core::convert::Infallible;

use defmt::{error, info, unwrap, warn};
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embassy_time::Duration;
use embedded_can::Frame as _;
use static_cell::make_static;

use crate::{
    mcp2515::{CanFrame, RxBuffer, TxBuffer},
    types::Mcp2515,
};

pub enum Obd2Error {
    Spi(SpiDeviceError<esp_hal::spi::Error, Infallible>),
    Parse,
    DataNotFound,
}

impl From<SpiDeviceError<esp_hal::spi::Error, Infallible>> for Obd2Error {
    fn from(e: SpiDeviceError<esp_hal::spi::Error, Infallible>) -> Self {
        Self::Spi(e)
    }
}

pub struct Obd2 {
    mcp2515: Mcp2515,
    obd2_message_buffer: &'static mut heapless::Vec<u8, 4095>,
    obd2_timeout: Option<Duration>,
}

impl Obd2 {
    pub fn new(mcp2515: Mcp2515) -> Self {
        let obd2_message_buffer = make_static!(heapless::Vec::new());
        let obd2_timeout = None;

        Self { mcp2515, obd2_message_buffer, obd2_timeout }
    }

    pub async fn request<PID: Pid>(&mut self) -> Result<PID, Obd2Error> {
        self.mcp2515.load_tx_buffer(TxBuffer::TXB0, &PID::request()).await?;
        self.mcp2515.request_to_send(TxBuffer::TXB0).await?;

        let mut can_frames = [None, None];
        let obd2_data: Option<&[u8]>;
        'outer: loop {
            self.mcp2515.interrupt().await;
            let rx_status = self.mcp2515.rx_status().await?;
            if rx_status.rx0if() {
                can_frames[0] = Some(self.mcp2515.read_rx_buffer(RxBuffer::RXB0).await?);
            }
            if rx_status.rx1if() {
                warn!("rx1if frame found");
                can_frames[1] = Some(self.mcp2515.read_rx_buffer(RxBuffer::RXB1).await?);
            }
            for can_frame in can_frames.iter().flatten() {
                let mut obd2_message_length = None;
                let mut obd2_message_id = 0;
                let obd2_frame_type = can_frame.data[0] & 0xF0;
                info!(
                    "can_frame.data[0]: {:#04x} from: {:?} type: {:#04x}",
                    can_frame.data,
                    defmt::Debug2Format(&can_frame.id()),
                    obd2_frame_type
                );

                match obd2_frame_type {
                    0x02 => {
                        info!("single frame: {}", can_frame.data);
                        obd2_data = Some(can_frame.data.as_slice());
                        break 'outer;
                    }
                    0x10 => {
                        self.obd2_message_buffer.clear();
                        obd2_message_length =
                            Some(((can_frame.data[0] & 0x0F) as usize) << 8 | can_frame.data[1] as usize);
                        info!("first obd2_message_length length: {}", obd2_message_length);
                        unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data));
                        obd2_message_id = 0;
                    }
                    0x30 => {
                        let timeout_ms = can_frame.data[2];
                        warn!("flow control frame with separation time: {}ms", timeout_ms);
                        if timeout_ms > 0 {
                            self.obd2_timeout = Some(Duration::from_millis(timeout_ms as u64));
                        }
                    }
                    0x20 => {
                        if let Some(obd2_message_length) = obd2_message_length {
                            let new_obd2_message_id = can_frame.data[0] & 0x0F;
                            info!("consecutive frame: {} of length: {}", new_obd2_message_id, obd2_message_length);
                            if new_obd2_message_id == obd2_message_id + 1 {
                                unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data[1..]));
                                if self.obd2_message_buffer.len() >= obd2_message_length {
                                    self.obd2_message_buffer.truncate(obd2_message_length);
                                    obd2_data = Some(self.obd2_message_buffer.as_slice());
                                    break 'outer;
                                }
                                obd2_message_id = new_obd2_message_id;
                            } else {
                                error!("consecutive frame id mismatch: {} != {}", new_obd2_message_id, obd2_message_id);
                            }
                        } else {
                            error!("no first frame");
                        }
                    }
                    _ => {
                        if can_frame.data[0] == 0x03 {
                            info!("single frame in _: {}", can_frame.data);
                            obd2_data = Some(can_frame.data.as_slice());
                            break 'outer;
                        } else {
                            error!("unknown frame: {}", obd2_frame_type);
                        }
                    }
                }
            }
        }

        if let Some(obd2_data) = obd2_data {
            PID::parse(obd2_data)
        } else {
            Err(Obd2Error::DataNotFound)
        }
    }
}

pub trait Pid {
    fn request() -> CanFrame;
    fn parse(data: &[u8]) -> Result<Self, Obd2Error>
    where
        Self: Sized;
}
