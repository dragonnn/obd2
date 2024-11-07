#![no_std]

extern crate alloc;

use core::sync::atomic::{AtomicUsize, Ordering};
use crc::{Crc, CRC_32_ISCSI};
use defmt::Format;
use postcard::to_vec_crc32;
use serde::{Deserialize, Serialize};
use serde_encrypt::{
    serialize::impls::PostcardSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey,
    EncryptedMessage,
};

pub static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

mod serializer;

static CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
static SHARED_KEY: &[u8; 32] = include_bytes!("../../shared_key.bin");

#[derive(Default, Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct AcPid {
    pub gear: i32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct BmsPid {
    pub hv_max_temp: f64,
    pub hv_min_temp: f64,
    pub hv_dc_voltage: f64,
    pub hv_soc: f64,
    pub hv_cell_voltage_deviation: f64,
    pub hv_min_cell_voltage: f64,
    pub hv_max_cell_voltage: f64,
    pub hv_battery_current: f64,

    pub aux_dc_voltage: f64,

    pub motor_electric_rpm: f64,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct HybridDcDcPid {
    pub gear: i32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct IceEnginePid {
    pub gear: i32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct IceFuelRatePid {
    pub fuel_rate: f64,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct IceTemperaturePid {
    pub temperature: f64,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct IcuPid {
    pub bat_discharge_warning_first_event_milage: f64,
    pub bat_discharge_warning_first_event_soc: u8,
    pub bat_discharge_warning_final_event_milage: f64,
    pub bat_discharge_warning_final_event_soc: u8,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct Icu2Pid {
    pub back_door_driver_side_open: bool,
    pub actuator_back_dor_driver_side_unlock: bool,
    pub back_door_passenger_side_open: bool,
    pub actuator_back_door_passenger_side_unlock: bool,
    pub front_door_passenger_side_open: bool,
    pub front_door_driver_side_open: bool,
    pub trunk_open: bool,

    pub engine_hood_open: bool,
    pub driver_buckled: bool,
    pub passenger_buckled: bool,
    pub breaking_fluid: bool,
    pub ignition_1_on: bool,
    pub ignition_2_on: bool,

    pub signal_back_av: bool,
}

#[derive(
    Debug, Format, PartialEq, Clone, Copy, strum::IntoStaticStr, Deserialize, Serialize, Default,
)]
pub enum Gear {
    PN,
    R,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    #[default]
    U,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct TransaxlePid {
    pub gear: Gear,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct VehicleSpeedPid {
    pub vehicle_speed: u8,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct OnBoardChargerPid {
    pub ac_input_voltage_instant: f64,
    pub ac_input_voltage_rms: f64,
    pub pfc_output_voltage: f64,
    pub obc_output_voltage: f64,
    pub ac_input_current: f64,
    pub obc_output_current: f64,
    pub ac_input_frequency: u8,
    pub obc_temperature_a: u8,
    pub cp_voltage: f64,
    pub cp_duty: f64,
    pub cp_frequency: f64,
    pub pd_voltage: f64,
    pub interlock_voltage: f64,
    pub aux_dc_voltage: f64,
    pub ig3_voltage: f64,
    pub pfc1_current_sensor_offset: f64,
}

#[derive(Debug, Format, Clone, Deserialize, Serialize)]
pub enum Pid {
    BmsPid(BmsPid),
    IceTemperaturePid(IceTemperaturePid),
    IceFuelRatePid(IceFuelRatePid),
    VehicleSpeedPid(VehicleSpeedPid),
    AcPid(AcPid),
    HybridDcDcPid(HybridDcDcPid),
    IcuPid(IcuPid),
    Icu2Pid(Icu2Pid),
    IceEnginePid(IceEnginePid),
    TransaxlePid(TransaxlePid),
    OnBoardChargerPid(OnBoardChargerPid),
}

impl core::hash::Hash for Pid {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for Pid {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Eq for Pid {}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub enum TxFrame {
    Obd2Pid(Pid),
    Modem(Modem),
    Shutdown,
}

impl TxFrame {
    pub fn is_modem(&self) -> bool {
        matches!(self, TxFrame::Modem(_))
    }

    pub fn is_modem_battery(&self) -> bool {
        matches!(self, TxFrame::Modem(Modem::Battery { .. }))
    }

    pub fn is_disconnect(&self) -> bool {
        matches!(self, TxFrame::Modem(Modem::Disconnected))
    }

    pub fn is_connect(&self) -> bool {
        matches!(self, TxFrame::Modem(Modem::Connected))
    }
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct TxMessage {
    pub id: u64,
    pub frame: TxFrame,
    pub timestamp: u64,
}

impl TxMessage {
    pub fn new(frame: TxFrame) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed) as u64,
            frame,
            timestamp: 0,
        }
    }

    pub fn to_vec(&self) -> Result<heapless::Vec<u8, 512>, postcard::Error> {
        to_vec_crc32::<_, 512>(self, CRC.digest())
    }

    pub fn to_vec_encrypted(&self) -> Result<alloc::vec::Vec<u8>, serde_encrypt::Error> {
        let shared_key = SharedKey::new(SHARED_KEY.clone());
        Ok(self.encrypt(&shared_key)?.serialize())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes_crc32(bytes, CRC.digest())
    }

    pub fn from_bytes_encrypted(bytes: &[u8]) -> Result<Self, serde_encrypt::Error> {
        let bytes = bytes.to_vec();
        let shared_key = SharedKey::new(SHARED_KEY.clone());
        Self::decrypt_owned(&EncryptedMessage::deserialize(bytes)?, &shared_key)
    }
}

impl SerdeEncryptSharedKey for TxMessage {
    type S = serializer::PostcardSerializer<Self>;
}

#[derive(Debug, Format, Clone, Deserialize, Serialize)]
pub enum Modem {
    Battery {
        voltage: f64,
        low_voltage: bool,
        soc: u8,
        charging: bool,
    },
    Connected,
    Disconnected,
    GnssFix(GnssFix),
}

#[derive(Debug, PartialEq, Format, Clone, Copy, Deserialize, Serialize)]
pub struct GnssFix {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
    pub accuracy: f32,

    pub year: u16,
    pub month: u8,
    pub day: u8,

    pub hour: u8,
    pub minute: u8,
    pub seconds: u8,
    pub ms: u16,
    pub elapsed: u64,
}
use num_traits::real::Real;

impl core::ops::Sub for GnssFix {
    type Output = f64;

    fn sub(self, other: Self) -> Self::Output {
        let r = 6378.137;
        let d_lat = (other.latitude * core::f64::consts::PI / 180.0)
            - (self.latitude * core::f64::consts::PI / 180.0);
        let d_lon = (other.longitude * core::f64::consts::PI / 180.0)
            - (self.longitude * core::f64::consts::PI / 180.0);
        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + (self.latitude * core::f64::consts::PI / 180.0).cos()
                * (other.latitude * core::f64::consts::PI / 180.0).cos()
                * (d_lon / 2.0).sin()
                * (d_lon / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let d = r * c;
        d * 1000.0
    }
}

impl core::hash::Hash for Modem {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for Modem {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub enum RxFrame {
    TxFrameAck(u64),
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct RxMessage {
    pub id: u64,
    pub frame: RxFrame,
    pub timestamp: u64,
}

impl RxMessage {
    pub fn new(frame: RxFrame) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed) as u64,
            frame,
            timestamp: 0,
        }
    }

    pub fn to_vec(&self) -> Result<heapless::Vec<u8, 512>, postcard::Error> {
        to_vec_crc32::<_, 512>(self, CRC.digest())
    }

    pub fn to_vec_encrypted(&self) -> Result<alloc::vec::Vec<u8>, serde_encrypt::Error> {
        let shared_key = SharedKey::new(SHARED_KEY.clone());
        Ok(self.encrypt(&shared_key)?.serialize())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes_crc32(bytes, CRC.digest())
    }

    pub fn from_bytes_encrypted(bytes: &[u8]) -> Result<Self, serde_encrypt::Error> {
        let bytes = bytes.to_vec();
        let shared_key = SharedKey::new(SHARED_KEY.clone());
        Self::decrypt_owned(&EncryptedMessage::deserialize(bytes)?, &shared_key)
    }
}

impl SerdeEncryptSharedKey for RxMessage {
    type S = serializer::PostcardSerializer<Self>;
}
