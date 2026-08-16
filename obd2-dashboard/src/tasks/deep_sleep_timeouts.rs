use embassy_time::Duration;

/// Time to wait after ignition-off when the car is closed.
pub const CAR_CLOSED: Duration = Duration::from_secs(1 * 60);

/// Time to wait after ignition-off when the car is open.
pub const CAR_OPEN: Duration = Duration::from_secs(2 * 60);

/// Time to wait after an ignition-off timeout reset has been received.
pub const AFTER_TIMEOUT_RESET: Duration = Duration::from_secs(30 * 60);

/// Time to wait when the latest OBD2 polling loop did not complete.
pub const INCOMPLETE_OBD2_LOOP: Duration = Duration::from_secs(2 * 60);

/// Time to wait when OBD2 initialization failed.
pub const OBD2_NOT_INITIALIZED: Duration = Duration::from_secs(30 * 60);

/// Safety timeout used while ignition is off when no OBD2 loop-end event arrives.
pub const FALLBACK: Duration = Duration::from_secs(2 * 60);
