use defmt::Format;
use persistent_buff::PersistentBuff;
use serde::{Deserialize, Serialize};

use crate::tasks::gnss::Fix;

#[derive(Format, Default, Deserialize, Serialize)]
pub struct PeristentState {
    distance: f64,
    secs: f64,
    booted: bool,
    restarts: u32,
    fix: Option<Fix>,
}

pub struct PeristentManager {
    persistent_buff: &'static mut [u8],
    persistent_state: PeristentState,
}

impl PeristentManager {
    pub fn new() -> Self {
        let persistent_buff = PersistentBuff::take_managed().unwrap().take_validate(|b| {
            let state = PeristentState { booted: false, ..Default::default() };
            embedded_msgpack::encode::serde::to_array(&state, b).unwrap();
        });

        let persistent_state: PeristentState =
            embedded_msgpack::decode::from_slice(persistent_buff).unwrap_or_else(|e| {
                defmt::error!("error decoding peristante_state: {}", defmt::Debug2Format(&e));
                let state = PeristentState { booted: false, ..Default::default() };
                embedded_msgpack::encode::serde::to_array(&state, persistent_buff).unwrap();
                state
            });

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

    fn serialize(&mut self) {
        embedded_msgpack::encode::serde::to_array(&self.persistent_state, self.persistent_buff).unwrap();
    }
}
