use crc::{Crc, CRC_32_ISCSI};
use defmt::*;
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

    icu2_pid: Option<types::Icu2Pid>,
}

pub struct PeristentManager {
    persistent_buff: &'static mut [u8],
    persistent_state: PeristentState,
}

impl PeristentManager {
    pub fn new() -> Self {
        let persistent_buff = PersistentBuff::take_managed().unwrap().take_validate(|_buff| {});
        let persistent_buff_len = persistent_buff.len();
        let used_persistent_buff = persistent_buff[0];
        let (first_persistent_buff, second_persistent_buf) =
            &mut persistent_buff[2..persistent_buff_len].split_at_mut((persistent_buff_len - 2) / 2);

        let persistent_state: PeristentState = postcard::from_bytes_crc32(
            if used_persistent_buff == 0 { first_persistent_buff } else { second_persistent_buf },
            CRC.digest(),
        )
        .unwrap_or_else(|e| {
            postcard::from_bytes_crc32(
                if used_persistent_buff == 0 { second_persistent_buf } else { first_persistent_buff },
                CRC.digest(),
            )
            .unwrap_or_else(|e| {
                defmt::error!("Failed to deserialize persistent state: {:?}", e);
                Default::default()
            })
        });

        info!("Used persistent buff: {}", used_persistent_buff);
        info!("Persistent state: {:?}", persistent_state);

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

    pub fn update_icu2_pid(&mut self, icu2_pid: Option<types::Icu2Pid>) {
        self.persistent_state.icu2_pid = icu2_pid;
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

    pub fn get_icu2_pid(&self) -> Option<types::Icu2Pid> {
        self.persistent_state.icu2_pid.clone()
    }

    fn serialize(&mut self) {
        let _guard = ResetGuard::new();
        let persistent_buff_len = self.persistent_buff.len();
        let used_persistent_buff = self.persistent_buff[0];
        let (first_persistent_buff, second_persistent_buf) =
            &mut self.persistent_buff[2..persistent_buff_len].split_at_mut((persistent_buff_len - 2) / 2);

        let write_perisent_buff = if used_persistent_buff == 0 { second_persistent_buf } else { first_persistent_buff };

        match postcard::to_slice_crc32(&self.persistent_state, write_perisent_buff, CRC.digest()) {
            Ok(_) => {
                self.persistent_buff[0] = if used_persistent_buff == 0 { 1 } else { 0 };
            }
            Err(e) => defmt::error!("Failed to serialize persistent state: {:?}", e),
        }
    }
}
