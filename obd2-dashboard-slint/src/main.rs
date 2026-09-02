// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;

slint::include_modules!();

#[cfg(feature = "board-kms")]
fn configure_board_backend() {
    // Keep the board executable self-contained: the LinuxKMS backend must be
    // directed to the legacy framebuffer and use the panel's 270-degree
    // orientation before Slint creates its window/backend.
    unsafe {
        std::env::set_var("SLINT_BACKEND", "linuxkms-software");
        std::env::set_var("SLINT_BACKEND_LINUXFB", "1");
        std::env::set_var("SLINT_KMS_ROTATION", "270");
    }
}

#[cfg(not(feature = "board-kms"))]
fn configure_board_backend() {}

#[cfg(feature = "board-kms")]
fn start_debug_wifi() {
    eprintln!("debug Wi-Fi: starting setup before Slint");

    let run = |program: &str, args: &[&str]| {
        eprintln!("debug Wi-Fi: running {} {:?}", program, args);
        match Command::new(program).args(args).status() {
            Ok(status) => {
                eprintln!("debug Wi-Fi: {} exited with {}", program, status);
                status.success()
            }
            Err(error) => {
                eprintln!("debug Wi-Fi: {} failed: {}", program, error);
                false
            }
        }
    };

    // The AIC driver is built as an out-of-tree module and depends on the
    // mac80211/cfg80211 stack. Because this debug path uses direct insmod,
    // load that dependency chain explicitly instead of relying on modprobe.
    // The rfkill write is harmless when this board exposes no rfkill entry.
    let _ = fs::write("/sys/class/rfkill/rfkill0/state", "1");
    run(
        "/sbin/insmod",
        &["/usr/lib/modules/6.1.99/kernel/lib/crypto/libarc4.ko"],
    );
    run(
        "/sbin/insmod",
        &["/usr/lib/modules/6.1.99/kernel/net/wireless/cfg80211.ko"],
    );
    run(
        "/sbin/insmod",
        &["/usr/lib/modules/6.1.99/kernel/net/mac80211/mac80211.ko"],
    );
    run("/sbin/insmod", &["/usr/lib/modules/aic_load_fw.ko"]);
    run("/sbin/insmod", &["/usr/lib/modules/aic8800_fdrv.ko"]);

    let config = r#"ctrl_interface=/run/wpa_supplicant
update_config=0
country=PL

network={
    ssid="dragonn2"
    psk="Twb3MRYd"
    key_mgmt=WPA-PSK
}
"#;
    if fs::write("/run/wpa_supplicant-debug.conf", config).is_err() {
        eprintln!("debug Wi-Fi: cannot write wpa_supplicant configuration");
        return;
    }

    let mut wlan_ready = false;
    for _ in 0..60 {
        if run("/sbin/ip", &["link", "show", "wlan0"]) {
            wlan_ready = true;
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }
    if !wlan_ready {
        eprintln!("debug Wi-Fi: wlan0 did not appear; continuing to Slint");
        return;
    }

    run("/sbin/ip", &["link", "set", "wlan0", "up"]);
    run(
        "/usr/sbin/wpa_supplicant",
        &["-B", "-i", "wlan0", "-c", "/run/wpa_supplicant-debug.conf"],
    );
    thread::sleep(Duration::from_secs(3));
    run("/sbin/ip", &["addr", "flush", "dev", "wlan0"]);
    run(
        "/sbin/ip",
        &["addr", "add", "192.168.89.107/21", "dev", "wlan0"],
    );
    run("/sbin/ip", &["link", "set", "wlan0", "up"]);

    // S50sshd is retained in the image by the Buildroot OpenSSH package.
    run("/etc/init.d/S50sshd", &["start"]);
    eprintln!("debug Wi-Fi: setup finished; continuing to Slint");
}

#[cfg(not(feature = "board-kms"))]
fn start_debug_wifi() {}

fn main() -> Result<(), Box<dyn Error>> {
    configure_board_backend();
    start_debug_wifi();
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
