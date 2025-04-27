#![feature(impl_trait_in_assoc_type)]
#![feature(trivial_bounds)]
extern crate alloc;

pub mod lcd {
    include!("../../../obd2-dashboard/src/tasks/lcd/mod.rs");
}

mod display {
    include!("../../../obd2-dashboard/src/display/mod.rs");
}

mod locks {
    include!("../../../obd2-dashboard/src/locks.rs");
}

mod debug {
    include!("../../../obd2-dashboard/src/debug.rs");
}

mod dummy_display;
mod hal;
mod tasks;
mod types;

fn main() {
    println!("Hello, world!");
}
