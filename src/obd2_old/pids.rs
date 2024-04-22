use defmt::{info, Format};
use enum_dispatch::enum_dispatch;
use strum::EnumIter;

#[derive(Format, EnumIter)]
pub enum Ids {
    Ecu = 0x7E8,
}

#[derive(Format, EnumIter)]
pub enum EcuPids {
    EngineSpeed = 0x0C,
    HybridBatteryPackSoL = 0x5B,
    EngineOilTemperature = 0x5C,
    EngineFuelRate = 0x5E,
    EngineCoolantTemperature = 0x05,
    TransmissionGear = 0xA4,
    Extended = 0x21,
}

#[derive(Format, EnumIter)]
pub enum ExtendedPids {
    Bms = 0x01,
}

#[derive(Format, EnumIter)]
pub enum Requests {
    EngineSpeed,
    HybridBatteryPackSoL,
    EngineOilTemperature,
    EngineFuelRate,
    EngineCoolantTemperature,
    TransmissionGear,
    Bms,
}

impl Requests {
    pub fn request(&self) -> &'static [u8; 8] {
        match self {
            Self::EngineSpeed => &[0x02, 0x01, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00],
            Self::HybridBatteryPackSoL => &[0x02, 0x01, 0x5B, 0x00, 0x00, 0x00, 0x00, 0x00],
            Self::EngineOilTemperature => &[0x02, 0x01, 0x5C, 0x00, 0x00, 0x00, 0x00, 0x00],
            Self::EngineFuelRate => &[0x02, 0x01, 0x5E, 0x00, 0x00, 0x00, 0x00, 0x00],
            Self::EngineCoolantTemperature => &[0x02, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00],
            Self::TransmissionGear => &[0x02, 0x01, 0xA4, 0x00, 0x00, 0x00, 0x00, 0x00],
            Self::Bms => &[0x02, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
        }
    }
}

pub trait Pid {
    const ID: Option<u8>;
    const PID: u8;

    fn parse(data: &[u8]) -> Option<Self>
    where
        Self: Sized;
}
#[derive(Format, Default)]
pub struct EngineSpeed(pub f64);

impl Pid for EngineSpeed {
    const ID: Option<u8> = None;
    const PID: u8 = 0x0C;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let rpm = (data[2] as f64 * 256.0 + data[3] as f64) / 4.0;
        Some(Self(rpm))
    }
}
#[derive(Format, Default)]
pub struct HybridBatteryPackSoL(pub f64);

impl Pid for HybridBatteryPackSoL {
    const ID: Option<u8> = None;
    const PID: u8 = 0x5B;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        let soc = 100.0 / 255.0 * data[2] as f64;
        Some(Self(soc))
    }
}
#[derive(Format, Default)]
pub struct EngineOilTemperature(pub i32);

impl Pid for EngineOilTemperature {
    const ID: Option<u8> = None;
    const PID: u8 = 0x5C;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        let temp = data[2] as i32 - 40;
        Some(Self(temp))
    }
}
#[derive(Format, Default)]
pub struct EngineFuelRate(pub f64);

impl Pid for EngineFuelRate {
    const ID: Option<u8> = None;
    const PID: u8 = 0x5E;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let rate = (data[2] as f64 * 256.0 + data[3] as f64) / 20.0;
        Some(Self(rate))
    }
}
#[derive(Format, Default)]
pub struct EngineCoolantTemperature(pub i32);

impl Pid for EngineCoolantTemperature {
    const ID: Option<u8> = None;
    const PID: u8 = 0x05;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        let temp = data[2] as i32 - 40;
        Some(Self(temp))
    }
}
#[derive(Format, Default)]
pub enum TransmissionGear {
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    #[default]
    Park,
    Reverse,
    Neutral,
}

impl Pid for TransmissionGear {
    const ID: Option<u8> = None;
    const PID: u8 = 0xA4;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        info!("TrasmisionGear: {:?}", data[2]);

        match data[2] {
            0x01 => Some(Self::D1),
            0x02 => Some(Self::D2),
            0x03 => Some(Self::D3),
            0x04 => Some(Self::D4),
            0x05 => Some(Self::D5),
            0x06 => Some(Self::D6),
            0x07 => Some(Self::D7),
            0x08 => Some(Self::Park),
            0x09 => Some(Self::Reverse),
            0x0A => Some(Self::Neutral),
            _ => None,
        }
    }
}

pub enum Values {
    EngineSpeed(EngineSpeed),
    HybridBatteryPackSoL(HybridBatteryPackSoL),
    EngineOilTemperature(EngineOilTemperature),
    EngineFuelRate(EngineFuelRate),
    EngineCoolantTemperature(EngineCoolantTemperature),
    TransmissionGear(TransmissionGear),
}

//pub fn parse<const N: usize>(out: &mut heapless::IndexSet) {}
