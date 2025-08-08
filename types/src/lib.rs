#![no_std]

extern crate alloc;

use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use crc::{Crc, CRC_32_ISCSI};
use defmt::Format;
use postcard::to_vec_crc32;
use serde::{Deserialize, Serialize};
use serde_encrypt::{
    serialize::impls::PostcardSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey,
    EncryptedMessage,
};

pub static ID_COUNTER: AtomicU32 = AtomicU32::new(0);

mod serializer;

static CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
static SHARED_KEY: &[u8; 32] = include_bytes!("../../shared_key.bin");

#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Obd2Frame {
    pub pid: u16,
    pub data: alloc::vec::Vec<u8>,
}

impl defmt::Format for Obd2Frame {
    fn format(&self, f: defmt::Formatter) {}
}

#[derive(Default, Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub struct AcPid {
    #[cfg_attr(feature = "egui", egui_probe(range = -10.0..=55.0))]
    pub vehicle_front_temp: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = -10.0..=55.0))]
    pub surround_temp: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = -10.0..=55.0))]
    pub evaporator_temp: f32,

    pub driver_mixing_air: f32,
    pub passenger_air_direction: f32,
    pub passenger_mixing_air: f32,
    pub air_direction: f32,
    pub input: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 0..=100))]
    pub humidity: u8,
    pub defrost_open: f32,

    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=100.0))]
    pub driver_vent_temp: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=100.0))]
    pub driver_floor_temp: f32,
    pub speed: u8,
    #[cfg_attr(feature = "egui", egui_probe(range = -10.0..=120.0))]
    pub ice_cooling_temp: f32,

    #[cfg_attr(feature = "egui", egui_probe(toggle_switch))]
    pub compressor_on: bool,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub struct BmsPid {
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=40.0))]
    pub hv_max_temp: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=40.0))]
    pub hv_min_temp: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 320.0..=410.0))]
    pub hv_dc_voltage: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=100.0))]
    pub hv_soc: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=200.0))]
    pub hv_cell_voltage_deviation: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 3.5..=4.1))]
    pub hv_min_cell_voltage: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 3.5..=4.1))]
    pub hv_max_cell_voltage: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = -100.0..=100.0))]
    pub hv_battery_current: f32,

    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=15.0))]
    pub aux_dc_voltage: f32,

    #[cfg_attr(feature = "egui", egui_probe(range = -0.0..=5000.0))]
    pub motor_electric_rpm: f32,
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
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub struct IceFuelRatePid {
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=15.0))]
    pub fuel_rate: f32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub struct IceTemperaturePid {
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=120.0))]
    pub temperature: f32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct IcuPid {
    pub bat_discharge_warning_first_event_milage: f32,
    pub bat_discharge_warning_first_event_soc: u8,
    pub bat_discharge_warning_final_event_milage: f32,
    pub bat_discharge_warning_final_event_soc: u8,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct Icu2Pid {
    pub back_door_driver_side_open: bool,
    pub actuator_back_door_driver_side_unlock: bool,
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

impl Icu2Pid {
    pub fn is_open(&self) -> bool {
        self.actuator_back_door_driver_side_unlock
            || self.actuator_back_door_passenger_side_unlock
            || self.trunk_open
            || self.engine_hood_open
    }
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct Icu3Pid {
    pub on_board_charger_wakeup_output: bool,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct Icu1Smk {
    pub aux_battery_voltage_power_load: f32,
    pub aux_battery_voltage_signal_cpu: f32,
    pub ground_voltage_power: f32,
    pub ground_voltage_ecu: f32,
    pub ign1_voltage: f32,
    pub ign2_voltage: f32,
    pub acc_voltage: f32,

    pub engine_rpm: u16,
    pub vehicle_speed: u8,
}

#[derive(
    Debug, Format, PartialEq, Clone, Copy, strum::IntoStaticStr, Deserialize, Serialize, Default,
)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
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

#[derive(
    Debug, Format, PartialEq, Clone, Copy, strum::IntoStaticStr, Deserialize, Serialize, Default,
)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub enum LcdEvent {
    #[default]
    Main,
    PowerOff,
    Charging,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub struct TransaxlePid {
    pub gear: Gear,
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=100.0))]
    pub clutch1_temp: f32,
    #[cfg_attr(feature = "egui", egui_probe(range = 0.0..=100.0))]
    pub clutch2_temp: f32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub struct VehicleSpeedPid {
    #[cfg_attr(feature = "egui", egui_probe(range = 0..=180))]
    pub vehicle_speed: u8,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "egui", derive(egui_probe::EguiProbe))]
pub struct OnBoardChargerPid {
    pub ac_input_voltage_instant: f32,
    pub ac_input_voltage_rms: f32,
    pub pfc_output_voltage: f32,
    pub obc_output_voltage: f32,
    pub ac_input_current: f32,
    pub obc_output_current: f32,
    pub ac_input_frequency: u8,
    pub obc_temperature_a: i8,
    pub cp_voltage: f32,
    pub cp_duty: f32,
    pub cp_frequency: f32,
    pub pd_voltage: f32,
    pub interlock_voltage: f32,
    pub aux_dc_voltage: f32,
    pub ig3_voltage: f32,
    pub pfc1_current_sensor_offset: f32,
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
    Icu3Pid(Icu3Pid),
    Icu1Smk(Icu1Smk),
    IceEnginePid(IceEnginePid),
    TransaxlePid(TransaxlePid),
    OnBoardChargerPid(OnBoardChargerPid),
}

#[derive(Debug, Format, Clone, PartialEq, Deserialize, Serialize, Hash, Eq)]
pub enum PidError {
    BmsPid,
    IceTemperaturePid,
    IceFuelRatePid,
    VehicleSpeedPid,
    AcPid,
    HybridDcDcPid,
    IcuPid,
    Icu2Pid,
    Icu3Pid,
    Icu1Smk,
    IceEnginePid,
    TransaxlePid,
    OnBoardChargerPid,
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
    Obd2PidError(PidError),
    Obd2Frame(Obd2Frame),
    Modem(Modem),
    Shutdown,
    State(State),
    Error(Error),
    Temperature(f32),
}

impl Into<TxMessage> for TxFrame {
    fn into(self) -> TxMessage {
        TxMessage::new(self)
    }
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub enum Error {}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub enum State {
    IgnitionOff,
    IgnitionOn,
    Shutdown(core::time::Duration),
    Charging,
    CheckCharging,
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
    pub id: MessageId,
    pub frame: TxFrame,
    pub timestamp: u64,
    pub ack: bool,
}

#[derive(Debug, Format, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum MessageId {
    Modem(u32),
    Obd2Dashboard(u32),
    HaDaemon(u32),
}

impl Default for MessageId {
    fn default() -> Self {
        #[cfg(feature = "id_modem")]
        let ret = MessageId::Modem(ID_COUNTER.fetch_add(1, Ordering::Relaxed));
        #[cfg(feature = "id_obd2dashboard")]
        let ret = MessageId::Obd2Dashboard(ID_COUNTER.fetch_add(1, Ordering::Relaxed));
        #[cfg(feature = "id_ha_daemon")]
        let ret = MessageId::Obd2Dashboard(ID_COUNTER.fetch_add(1, Ordering::Relaxed));

        #[cfg(all(
            not(feature = "id_modem"),
            not(feature = "id_obd2dashboard"),
            not(feature = "id_ha_daemon"),
        ))]
        let ret = MessageId::Modem(0);
        ret
    }
}

impl core::ops::Deref for MessageId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        match self {
            MessageId::Modem(id) => id,
            MessageId::Obd2Dashboard(id) => id,
            MessageId::HaDaemon(id) => id,
        }
    }
}

impl TxMessage {
    pub fn new(frame: TxFrame) -> Self {
        let mut ret = Self {
            id: MessageId::default(),
            frame,
            timestamp: 0,
            ack: false,
        };
        if *ret.id % 10 == 0 && *ret.id != 0 {
            ret.ack = true;
        }

        ret
    }

    pub fn to_vec(&self) -> Result<heapless07::Vec<u8, 512>, postcard::Error> {
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

    pub fn ack(&mut self) {
        self.ack = true;
    }

    pub fn needs_ack(&self) -> bool {
        self.ack
            || match self.frame {
                TxFrame::Shutdown => true,
                TxFrame::State(_) => true,
                TxFrame::Modem(Modem::Disconnected) => true,
                TxFrame::Modem(Modem::Connected) => true,
                TxFrame::Modem(Modem::Ping) => false,
                TxFrame::Modem(Modem::Pong) => false,
                _ => false,
            }
    }
}

impl SerdeEncryptSharedKey for TxMessage {
    type S = serializer::PostcardSerializer<Self>;
}

#[derive(Debug, Format, Clone, Deserialize, Serialize)]
pub enum Modem {
    Battery {
        voltage: f32,
        low_voltage: bool,
        soc: u8,
        charging: bool,
    },
    Connected,
    Disconnected,
    GnssFix(GnssFix),
    GnssState(GnssState),
    Reset,
    Boot,
    Ping,
    Pong,
}

#[derive(Debug, Format, Clone, Deserialize, Serialize)]
pub enum GnssState {
    BackupMode,
    DisablingBackup,
    SingleFix,
    ContinuousFix,
    ErrorDisablingBackup,
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
    TxFrameAck(MessageId),
    Modem(Modem),
    Obd2Frame(Obd2Frame),
}

impl Into<RxMessage> for RxFrame {
    fn into(self) -> RxMessage {
        RxMessage::new(self)
    }
}

impl RxFrame {
    pub fn is_ack(&self) -> bool {
        matches!(self, RxFrame::TxFrameAck(_))
    }
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct RxMessage {
    pub id: MessageId,
    pub frame: RxFrame,
    pub timestamp: u64,
    pub ack: bool,
}

impl RxMessage {
    pub fn new(frame: RxFrame) -> Self {
        let mut ret = Self {
            id: MessageId::default(),
            frame,
            timestamp: 0,
            ack: false,
        };

        if *ret.id % 10 == 0 && *ret.id != 0 {
            ret.ack = true;
        }

        ret
    }

    pub fn to_vec(&self) -> Result<heapless07::Vec<u8, 512>, postcard::Error> {
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
