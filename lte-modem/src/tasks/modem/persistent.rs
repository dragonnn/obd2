use crc::{Crc, CRC_32_ISCSI};
use defmt::Format;
use persistent_buff::PersistentBuff;
use serde::{Deserialize, Serialize};

use crate::tasks::{gnss::Fix, reset::ResetGuard};

static CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

#[derive(Format, Default, Deserialize, Serialize)]
pub struct PeristentState {
    distance: f64,
    secs: f64,
    booted: bool,
    restarts: u32,
    fix: Option<Fix>,
    state: Option<types::State>,
}

pub struct PeristentManager {
    persistent_buff: &'static mut [u8],
    persistent_state: PeristentState,
}

impl PeristentManager {
    pub fn new() -> Self {
        let persistent_buff = PersistentBuff::take_managed().unwrap().take_validate(|_buff| {});

        let persistent_state: PeristentState =
            postcard::from_bytes_crc32(&persistent_buff, CRC.digest()).unwrap_or_default();

        Self { persistent_buff, persistent_state }
    }

    pub fn update_booted(&mut self, booted: bool) {
        self.persistent_state.booted = booted;
        self.serialize();
    }

    pub fn update_distance(&mut self, distance: f64) {
        self.persistent_state.distance = distance;
        self.serialize();
    }

    pub fn update_secs(&mut self, secs: f64) {
        self.persistent_state.secs = secs;
        self.serialize();
    }

    pub fn add_restarts(&mut self) {
        self.persistent_state.restarts += 1;
        self.serialize();
    }

    pub fn update_fix(&mut self, fix: Option<Fix>) {
        self.persistent_state.fix = fix;
        self.serialize();
    }

    pub fn update_state(&mut self, state: Option<types::State>) {
        self.persistent_state.state = state;
        self.serialize();
    }

    pub fn get_booted(&self) -> bool {
        self.persistent_state.booted
    }

    pub fn get_distance(&self) -> f64 {
        self.persistent_state.distance
    }

    pub fn get_secs(&self) -> f64 {
        self.persistent_state.secs
    }

    pub fn get_restarts(&self) -> u32 {
        self.persistent_state.restarts
    }

    pub fn get_fix(&self) -> Option<Fix> {
        self.persistent_state.fix
    }

    pub fn get_state(&self) -> Option<types::State> {
        self.persistent_state.state.clone()
    }

    fn serialize(&mut self) {
        let _guard = ResetGuard::new();
        match postcard::to_slice_crc32(&self.persistent_state, &mut self.persistent_buff, CRC.digest()) {
            Ok(_) => {}
            Err(e) => defmt::error!("Failed to serialize persistent state: {:?}", e),
        }
    }
}
