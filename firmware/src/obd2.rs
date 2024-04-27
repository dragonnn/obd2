use core::convert::Infallible;

use defmt::{error, info, unwrap, warn};
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embassy_time::Duration;
use embedded_can::Frame as _;
use static_cell::make_static;

use crate::{
    debug::internal_debug,
    mcp2515::{clock_8mhz, CanFrame, OperationMode, RxBuffer, TxBuffer, CANINTE, RXB0CTRL, RXB1CTRL, RXM},
    types::Mcp2515,
};

pub enum Obd2Error {
    Spi(SpiDeviceError<esp_hal::spi::Error, Infallible>),
    Parse,
    DataNotFound,
    FrameToShort,
}

impl From<SpiDeviceError<esp_hal::spi::Error, Infallible>> for Obd2Error {
    fn from(e: SpiDeviceError<esp_hal::spi::Error, Infallible>) -> Self {
        Self::Spi(e)
    }
}

pub struct Obd2 {
    mcp2515: Mcp2515,
    obd2_message_buffer: &'static mut heapless::Vec<u8, 4095>,
}

impl Obd2 {
    pub fn new(mcp2515: Mcp2515) -> Self {
        let obd2_message_buffer = make_static!(heapless::Vec::new());

        Self { mcp2515, obd2_message_buffer }
    }

    pub async fn init(&mut self) {
        let config = crate::mcp2515::Config::default()
            .mode(OperationMode::NormalOperation)
            .bitrate(clock_8mhz::CNF_500K_BPS)
            .receive_buffer_0(RXB0CTRL::default().with_rxm(RXM::ReceiveAny).with_bukt(true))
            .receive_buffer_1(RXB1CTRL::default().with_rxm(RXM::ReceiveAny));

        self.mcp2515.apply_config(&config).await.unwrap();

        let interputs_config = CANINTE::default().with_rx0ie(true).with_rx1ie(true);
        self.mcp2515.apply_interrupts_config(interputs_config).await.unwrap();
    }

    pub async fn request_pid<PID: Pid>(&mut self) -> Result<PID, Obd2Error> {
        let lock = crate::locks::SPI_BUS.lock().await;

        self.mcp2515.clear_interrupts().await?;
        let request = PID::request();

        internal_debug!("req pid: {:x?}", request.data);
        //5 ms flow control
        let flow_control = CanFrame::new(request.id(), &[0x30, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
        self.mcp2515.load_tx_buffer(TxBuffer::TXB0, &request).await?;
        error!("sending obd2 request");
        self.mcp2515.request_to_send(TxBuffer::TXB0).await?;

        let mut can_frames = [None, None];
        let obd2_data: Option<&[u8]>;
        let mut obd2_message_length = None;
        let mut obd2_message_id = 0;
        'outer: loop {
            let rx_status = self.mcp2515.rx_status().await?;
            if rx_status.rx0if() {
                let frame = self.mcp2515.read_rx_buffer(RxBuffer::RXB0).await?;
                internal_debug!("rx0if: {:x?}", frame.data);
                can_frames[0] = Some(frame);
                info!("found can frame 0: {:?}", can_frames[0]);
            }
            if rx_status.rx1if() {
                warn!("rx1if frame found");
                let frame = self.mcp2515.read_rx_buffer(RxBuffer::RXB1).await?;
                internal_debug!("rx1if: {:x?}", frame.data);
                can_frames[1] = Some(frame);
                info!("found can frame 1: {:?}", can_frames[1]);
            }
            for can_frame in can_frames.iter().flatten().filter(|frame| PID::filter_frame(frame)) {
                let obd2_frame_type = can_frame.data[0] & 0xF0;

                match obd2_frame_type {
                    0x02 => {
                        info!("single frame: {}", can_frame.data);
                        internal_debug!("single frame");
                        obd2_data = Some(can_frame.data.as_slice());
                        break 'outer;
                    }
                    0x10 => {
                        self.obd2_message_buffer.clear();
                        obd2_message_length =
                            Some(((can_frame.data[0] & 0x0F) as usize) << 8 | can_frame.data[1] as usize);
                        info!("first obd2_message_length length: {}", obd2_message_length);
                        internal_debug!("first frame");
                        self.mcp2515.load_tx_buffer(TxBuffer::TXB0, &flow_control).await?;
                        error!("sending obd2 request");
                        self.mcp2515.request_to_send(TxBuffer::TXB0).await?;

                        unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data[2..]));

                        obd2_message_id = 0;
                    }
                    0x30 => {
                        internal_debug!("flow control frame");
                        let timeout_ms = can_frame.data[2];
                        warn!("flow control frame with separation time: {}ms", timeout_ms);
                    }
                    0x20 => {
                        internal_debug!("consecutive frame");
                        if let Some(obd2_message_length) = obd2_message_length {
                            let new_obd2_message_id = can_frame.data[0] & 0x0F;
                            if new_obd2_message_id == obd2_message_id + 1 {
                                unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data[1..]));
                                //unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data));
                                if self.obd2_message_buffer.len() >= obd2_message_length {
                                    //self.obd2_message_buffer.truncate(obd2_message_length);
                                    obd2_data = Some(self.obd2_message_buffer.as_slice());
                                    info!("got last consecutive frame: {}", new_obd2_message_id);
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
                            internal_debug!("single frame in 0x03");
                            info!("single frame in _: {}", can_frame.data);
                            obd2_data = Some(can_frame.data.as_slice());
                            break 'outer;
                        } else {
                            internal_debug!("unknown frame");
                            error!("unknown frame: {}", obd2_frame_type);
                        }
                    }
                }
            }
            self.mcp2515.interrupt().await;
        }

        info!("obd2_data: {:?}", obd2_data);

        if let Some(obd2_data) = obd2_data {
            PID::parse(obd2_data)
        } else {
            error!("no obd2_data found");
            Err(Obd2Error::DataNotFound)
        }
    }
}

pub trait Pid {
    fn filter_frame(frame: &CanFrame) -> bool {
        true
    }
    fn request() -> CanFrame;
    fn parse(data: &[u8]) -> Result<Self, Obd2Error>
    where
        Self: Sized;
}
