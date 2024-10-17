#![no_std]

use defmt::Format;
use serde::{Deserialize, Serialize};

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct AcPid {
    pub gear: i32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct HybridDcDcPid {
    pub gear: i32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct IceEnginePid {
    pub gear: i32,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct IceFuelRatePid {
    pub fuel_rate: f64,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct IceTemperaturePid {
    pub temperature: f64,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct IcuPid {
    pub gear: i32,
}

#[derive(Debug, Format, PartialEq, Clone, Copy, strum::IntoStaticStr, Deserialize, Serialize)]
pub enum Gear {
    PN,
    R,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    U,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct TransaxlePid {
    pub gear: Gear,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub struct VehicleSpeedPid {
    pub vehicle_speed: u8,
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub enum Pid {
    BmsPid(BmsPid),
    IceTemperaturePid(IceTemperaturePid),
    IceFuelRatePid(IceFuelRatePid),
    VehicleSpeedPid(VehicleSpeedPid),
    AcPid(AcPid),
    HybridDcDcPid(HybridDcDcPid),
    Icu(IcuPid),
    IceEnginePid(IceEnginePid),
    TransaxlePid(TransaxlePid),
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub enum TxFrame {
    Obd2Pid(Pid),
}

#[derive(Debug, Format, PartialEq, Clone, Deserialize, Serialize)]
pub enum RxFrame {}
