//! Runs the actual firmware statechart with a mock clock, power latch and
//! outgoing RPC transport. No hardware is accessed.
#![allow(dead_code)]

macro_rules! info {
    ($($args:tt)*) => {};
}
macro_rules! warn {
    ($($args:tt)*) => {};
}

pub mod board {
    use embassy_time::{Duration, Timer};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub struct LyraPower(pub Arc<Mutex<Vec<bool>>>, pub Option<Duration>);

    impl LyraPower {
        pub async fn power_on(&mut self) {
            self.0.lock().unwrap().push(true);
        }

        pub async fn power_off(&mut self) {
            if let Some(delay) = self.1 {
                Timer::after(delay).await;
            }
            self.0.lock().unwrap().push(false);
        }

        pub fn is_on(&self) -> bool {
            self.0.lock().unwrap().last().copied().unwrap_or(false)
        }
    }
}

pub mod lyra {
    pub mod rpc {
        use std::sync::{Arc, Mutex};

        use lyra_protocol::ControlId;
        use postcard_rpc::{Topic, header::VarSeq};

        #[derive(Clone, Default)]
        pub struct RpcSender(pub Arc<Mutex<Vec<(&'static str, ControlId)>>>);

        impl RpcSender {
            pub async fn publish<T: Topic<Message = ControlId>>(
                &self,
                _seq: VarSeq,
                id: &ControlId,
            ) -> Result<(), core::convert::Infallible> {
                self.0.lock().unwrap().push((T::PATH, *id));
                Ok(())
            }

            pub fn last<T: Topic>(&self) -> ControlId {
                self.0
                    .lock()
                    .unwrap()
                    .iter()
                    .rev()
                    .find(|(path, _)| *path == T::PATH)
                    .unwrap()
                    .1
            }

            pub fn count<T: Topic>(&self) -> usize {
                self.0
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|(path, _)| *path == T::PATH)
                    .count()
            }
        }
    }

    pub mod state {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/lyra/state.rs"
        ));

        #[cfg(test)]
        mod tests {
            include!(concat!(env!("CARGO_MANIFEST_DIR"), "/cases.rs"));
        }
    }
}
