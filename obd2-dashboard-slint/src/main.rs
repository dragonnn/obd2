// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::thread;

use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;

slint::include_modules!();

#[cfg(feature = "board-kms")]
fn configure_board_backend() {
    // Keep the board executable self-contained: the LinuxKMS backend must be
    // directed to the legacy framebuffer and use the panel's 90-degree
    // orientation before Slint creates its window/backend.
    unsafe {
        std::env::set_var("SLINT_BACKEND", "linuxkms-software");
        std::env::set_var("SLINT_BACKEND_LINUXFB", "1");
        std::env::set_var("SLINT_KMS_ROTATION", "90");
    }
}

#[cfg(not(feature = "board-kms"))]
fn configure_board_backend() {}

fn main() -> Result<(), Box<dyn Error>> {
    configure_board_backend();
    let ui = AppWindow::new()?;

    // SIGTERM does not unwind Rust stack frames. Ask Slint's event loop to
    // quit instead, so the KMS/framebuffer backend is dropped normally and
    // releases the display for the next application.
    let mut signals = Signals::new([SIGINT, SIGQUIT, SIGTERM])?;
    thread::spawn(move || {
        if signals.forever().next().is_some() {
            let _ = slint::invoke_from_event_loop(|| {
                let _ = slint::quit_event_loop();
            });
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });

    ui.run()?;

    Ok(())
}
