use core::{any::TypeId, convert::Infallible};

use defmt::{error, info, unwrap, warn};
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embassy_time::{with_timeout, Duration, Instant};
use embedded_can::{Frame as _, StandardId};
use heapless::Entry;
use static_cell::make_static;
use types::Obd2Frame;

use crate::{
    debug::internal_debug,
    event::{Event, EventBusPub, KiaEvent, Obd2Event, KIA_EVENTS},
    mcp2515::{
        clock_16mhz, clock_8mhz, CanFrame, OperationMode, RxBuffer, TxBuffer, CANINTE, CLKPRE, RXB0CTRL, RXB1CTRL, RXM,
    },
    prelude::*,
    tasks::{
        ieee802154::{insert_send_pid, insert_send_pid_error},
        lcd::obd2_debug_pids_enabled,
        obd2::Obd2Debug,
    },
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
    obd2_pid_errors: heapless::FnvIndexMap<TypeId, usize, 32>,
    obd2_pid_errors_periods: heapless::FnvIndexMap<TypeId, Instant, 32>,
    obd2_pid_periods: heapless::FnvIndexMap<TypeId, Instant, 32>,
    obd2_pid_periods_disable: bool,
}

impl Obd2 {
    pub fn new(mcp2515: Mcp2515) -> Self {
        static OBD2_MESSAGE_BUFFER_STATIC: static_cell::StaticCell<heapless::Vec<u8, 4095>> =
            static_cell::StaticCell::new();

        let obd2_message_buffer = OBD2_MESSAGE_BUFFER_STATIC.init_with(|| heapless::Vec::new());
        let obd2_pid_errors = heapless::FnvIndexMap::new();
        let obd2_pid_errors_periods = heapless::FnvIndexMap::new();
        let obd2_pid_periods = heapless::FnvIndexMap::new();

        Self {
            mcp2515,
            obd2_message_buffer,
            obd2_pid_errors_periods,
            obd2_pid_errors,
            obd2_pid_periods,
            obd2_pid_periods_disable: false,
        }
    }

    pub fn enable_obd2_pid_periods(&mut self) {
        self.obd2_pid_periods_disable = false;
    }

    pub fn disable_obd2_pid_periods(&mut self) {
        self.obd2_pid_periods_disable = true;
    }

    pub async fn init(&mut self) -> Result<(), SpiDeviceError<esp_hal::spi::Error, Infallible>> {
        let config = crate::mcp2515::Config::default()
            .mode(OperationMode::NormalOperation)
            .bitrate(clock_16mhz::CNF_500K_BPS)
            .set_clk_prescaler(CLKPRE::SystemClockDiv2)
            .receive_buffer_0(RXB0CTRL::default().with_rxm(RXM::ReceiveAny).with_bukt(true))
            .receive_buffer_1(RXB1CTRL::default().with_rxm(RXM::ReceiveAny));

        self.mcp2515.apply_config(&config, true).await?;

        let interputs_config = CANINTE::default().with_rx0ie(true).with_rx1ie(true);
        self.mcp2515.apply_interrupts_config(interputs_config).await?;

        info!("obd2 init done");

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.mcp2515.shutdown().await;
    }

    async fn request<'a>(&'a mut self, request: &CanFrame) -> Result<&'a [u8], Obd2Error> {
        let mut _lock = Some(crate::locks::SPI_BUS.lock().await);
        self.mcp2515.clear_interrupts().await?;
        let flow_control = unwrap!(CanFrame::new(request.id(), &[0x30, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]));
        self.mcp2515.load_tx_buffer(TxBuffer::TXB0, &request).await?;
        self.mcp2515.request_to_send(TxBuffer::TXB0).await?;

        let mut can_frames = [None, None];
        let obd2_data: Option<&[u8]>;
        let mut obd2_message_length = None;
        let mut obd2_message_id = 0;
        'outer: loop {
            let rx_status = self.mcp2515.rx_status().await?;
            if rx_status.rx0if() {
                let frame = self.mcp2515.read_rx_buffer(RxBuffer::RXB0).await?;
                //info!("rx0if: {=[u8]:x} {=i32:x}", frame.data, frame.id_header.get_i32());
                can_frames[0] = Some(frame);
            }
            if rx_status.rx1if() {
                let frame = self.mcp2515.read_rx_buffer(RxBuffer::RXB1).await?;
                //info!("rx1if: {=[u8]:x} {=i32:x}", frame.data, frame.id_header.get_i32());
                can_frames[1] = Some(frame);
            }
            for can_frame in can_frames.iter().flatten() {
                let obd2_frame_type = can_frame.data[0] & 0xF0;

                match obd2_frame_type {
                    0x02 | 0x04 | 0x00 => {
                        //internal_debug!("single frame {:x?}", can_frame.data);
                        self.obd2_message_buffer.clear();
                        self.obd2_message_buffer.extend_from_slice(&can_frame.data);
                        obd2_data = Some(self.obd2_message_buffer.as_slice());
                        break 'outer;
                    }
                    0x10 => {
                        self.obd2_message_buffer.clear();
                        obd2_message_length =
                            Some(((can_frame.data[0] & 0x0F) as usize) << 8 | can_frame.data[1] as usize);
                        //internal_debug!("first frame {:x?}", can_frame.data);
                        self.mcp2515.load_tx_buffer(TxBuffer::TXB0, &flow_control).await?;
                        self.mcp2515.request_to_send(TxBuffer::TXB0).await?;

                        unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data[2..]));

                        obd2_message_id = 0;
                    }
                    0x30 => {
                        let timeout_ms = can_frame.data[2];
                    }
                    0x20 => {
                        //internal_debug!("consecutive frame {:x?}", can_frame.data);
                        if let Some(obd2_message_length) = obd2_message_length {
                            let new_obd2_message_id = can_frame.data[0] & 0x0F;
                            if new_obd2_message_id == obd2_message_id + 1 {
                                unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data[1..]));
                                //unwrap!(self.obd2_message_buffer.extend_from_slice(&can_frame.data));
                                if self.obd2_message_buffer.len() >= obd2_message_length {
                                    //self.obd2_message_buffer.truncate(obd2_message_length);
                                    obd2_data = Some(self.obd2_message_buffer.as_slice());
                                    //info!("got last consecutive frame: {}", new_obd2_message_id);
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
                            //internal_debug!("single frame in 0x03 {:x?}", can_frame.data);
                            self.obd2_message_buffer.clear();
                            self.obd2_message_buffer.extend_from_slice(&can_frame.data);
                            obd2_data = Some(self.obd2_message_buffer.as_slice());
                            break 'outer;
                        } else {
                            internal_debug!("unknown frame {:x?}", can_frame.data);
                            error!("unknown frame: {} {=[u8]:#04x}", obd2_frame_type, can_frame.data);
                        }
                    }
                }
            }
            while embassy_time::with_timeout(embassy_time::Duration::from_millis(50), self.mcp2515.interrupt())
                .await
                .is_err()
            {
                if _lock.is_some() {
                    //error!("timeout waiting for interrupt, drooping SPI lock");
                    _lock = None;
                }
            }
        }

        if let Some(obd2_data) = obd2_data {
            Ok(obd2_data)
        } else {
            error!("no obd2_data found");
            Err(Obd2Error::DataNotFound)
        }
    }

    pub async fn request_pid<PID: Pid>(&mut self) -> Result<(PID, alloc::vec::Vec<u8>), Obd2Error> {
        let request = PID::request();

        let obd2_data = self.request(&request).await?;

        let pid = PID::parse(&obd2_data)?;
        Ok((pid, if obd2_debug_pids_enabled() { obd2_data.to_vec() } else { alloc::vec::Vec::new() }))
    }

    pub async fn reset(&mut self) {
        self.init().await.ok();
    }

    pub async fn handle_pid<PID: Pid + core::any::Any>(&mut self) -> bool {
        if !self.obd2_pid_errors.is_empty() && self.obd2_pid_errors.iter().all(|(_, errors)| *errors >= 10) {
            warn!("too many errors, clearing errors");
            self.init().await.ok();
            self.obd2_pid_errors.clear();
        }

        let type_id = TypeId::of::<PID>();
        if let Some(period) = PID::period() {
            if !self.obd2_pid_periods_disable {
                if let Some(last_time) = self.obd2_pid_periods.get(&type_id).map(|time| *time) {
                    if Instant::now() - last_time < period {
                        return true;
                    }
                }
                self.obd2_pid_periods.insert(type_id, Instant::now()).ok();
            }
        }
        let mut errors = self.obd2_pid_errors.get(&type_id).map(|errors| *errors).unwrap_or(0);
        let obd2_debug_pids_enabled = obd2_debug_pids_enabled();
        let mut ret = false;
        if errors < 10 {
            match with_timeout(Duration::from_millis(350), self.request_pid::<PID>()).await {
                Ok(Ok((pid_result, buffer))) => {
                    let pid_result = pid_result.into_event();
                    insert_send_pid(&pid_result).await;
                    let event = KiaEvent::Obd2Event(pid_result);
                    KIA_EVENTS.send(event.clone()).timeout_millis(10).await.ok();
                    if obd2_debug_pids_enabled {
                        KIA_EVENTS.try_send(KiaEvent::Obd2Debug(Obd2Debug::new::<PID>(Some(buffer)))).ok();
                    }
                    embassy_time::Timer::after(embassy_time::Duration::from_millis(25)).await;
                    errors = 0;
                    ret = true;
                }
                Ok(Err(_e)) => {
                    insert_send_pid_error(&PID::into_error()).timeout_millis(10).await.ok();
                    error!("error requesting pid");
                    internal_debug!("error requesting pid");
                    if obd2_debug_pids_enabled {
                        KIA_EVENTS.try_send(KiaEvent::Obd2Debug(Obd2Debug::new::<PID>(None))).ok();
                    }
                    errors += 1;
                }
                Err(_) => {
                    insert_send_pid_error(&PID::into_error()).await;
                    error!("timeout requesting pid: {}", core::any::type_name::<PID>());
                    internal_debug!("timeout requesting pid");
                    if obd2_debug_pids_enabled {
                        KIA_EVENTS.try_send(KiaEvent::Obd2Debug(Obd2Debug::new::<PID>(None))).ok();
                    }
                    errors += 1;
                }
            }
        } else {
            //ignore errors
            ret = true;
            if obd2_debug_pids_enabled {
                KIA_EVENTS.try_send(KiaEvent::Obd2Debug(Obd2Debug::new::<PID>(None))).ok();
            }
            let last_time =
                self.obd2_pid_errors_periods.get(&type_id).map(|time| *time).unwrap_or(Instant::from_millis(0));
            if last_time.elapsed() > Duration::from_secs(5 * 60) {
                error!("last error was more then 5*60s ago, clearing errors");
                errors = 0;
            }
            self.obd2_pid_errors_periods.insert(type_id, Instant::now()).ok();
        }

        self.obd2_pid_errors.insert(type_id, errors).ok();
        return ret;
    }

    pub fn clear_pids_cache(&mut self) {
        self.obd2_pid_errors.clear();
        self.obd2_pid_errors_periods.clear();
        self.obd2_pid_periods.clear();
    }

    pub async fn send_custom_frame(&mut self, frame: Obd2Frame) -> Result<Obd2Frame, Obd2Error> {
        let can_id = unwrap!(StandardId::new(frame.pid));
        let can_frame = unwrap!(CanFrame::new(can_id, &frame.data));

        let response = self.request(&can_frame).await?;
        Ok(Obd2Frame { pid: frame.pid, data: response.to_vec() })
    }
}

pub trait Pid {
    fn request() -> CanFrame;
    fn parse(data: &[u8]) -> Result<Self, Obd2Error>
    where
        Self: Sized;
    fn into_event(self) -> Obd2Event;
    fn into_error() -> types::PidError;
    fn period() -> Option<Duration> {
        None
    }
}
