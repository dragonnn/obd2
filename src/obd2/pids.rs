use defmt::info;

pub trait Pid {
    const ID: Option<u8>;
    const PID: u8;

    fn parse(data: &[u8]) -> Option<Self>
    where
        Self: Sized;
}

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

pub enum TransmissionGear {
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
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

pub enum Pids {
    EngineSpeed(EngineSpeed),
    HybridBatteryPackSoL(HybridBatteryPackSoL),
    EngineOilTemperature(EngineOilTemperature),
    EngineFuelRate(EngineFuelRate),
    EngineCoolantTemperature(EngineCoolantTemperature),
    TransmissionGear(TransmissionGear),
}

impl Pids {
    pub fn parse_frame(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        let pid = data[2];

        match pid {
            EngineSpeed::PID => EngineSpeed::parse(data).map(Self::EngineSpeed),
            HybridBatteryPackSoL::PID => {
                HybridBatteryPackSoL::parse(data).map(Self::HybridBatteryPackSoL)
            }
            EngineOilTemperature::PID => {
                EngineOilTemperature::parse(data).map(Self::EngineOilTemperature)
            }
            EngineFuelRate::PID => EngineFuelRate::parse(data).map(Self::EngineFuelRate),
            EngineCoolantTemperature::PID => {
                EngineCoolantTemperature::parse(data).map(Self::EngineCoolantTemperature)
            }
            TransmissionGear::PID => TransmissionGear::parse(data).map(Self::TransmissionGear),
            _ => None,
        }
    }
}
