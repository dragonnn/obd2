// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::thread;
use std::time::Duration;

#[cfg(feature = "board-kms")]
use std::fs;
#[cfg(feature = "board-kms")]
use std::process::Command;

use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;

slint::include_modules!();

#[cfg(feature = "board-kms")]
mod touch;

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

fn start_debug_wifi_in_background() {
    thread::spawn(|| {
        eprintln!("debug Wi-Fi: delaying setup for 5 seconds");
        thread::sleep(Duration::from_secs(5));
        start_debug_wifi();
    });
}

#[cfg(feature = "board-kms")]
fn spawn_system_action(name: &str, program: &str, args: &[&str]) {
    match Command::new(program).args(args).spawn() {
        Ok(_) => eprintln!("power menu: requested {name}"),
        Err(error) => eprintln!("power menu: failed to request {name}: {error}"),
    }
}

#[cfg(not(feature = "board-kms"))]
fn spawn_system_action(name: &str, _program: &str, _args: &[&str]) {
    eprintln!("power menu: {name} is disabled outside the board-kms build");
}

#[cfg(feature = "board-kms")]
const BACKLIGHT_BRIGHTNESS_PATH: &str = "/sys/class/backlight/co6300/brightness";

#[cfg(feature = "board-kms")]
fn read_brightness() -> i32 {
    match fs::read_to_string(BACKLIGHT_BRIGHTNESS_PATH) {
        Ok(value) => match value.trim().parse::<i32>() {
            Ok(value) => value.clamp(0, 255),
            Err(error) => {
                eprintln!("settings menu: invalid brightness value: {error}");
                255
            }
        },
        Err(error) => {
            eprintln!("settings menu: failed to read brightness: {error}");
            255
        }
    }
}

#[cfg(not(feature = "board-kms"))]
fn read_brightness() -> i32 {
    255
}

#[cfg(feature = "board-kms")]
fn write_brightness(value: i32) {
    let value = value.clamp(0, 255);
    if let Err(error) = fs::write(BACKLIGHT_BRIGHTNESS_PATH, value.to_string()) {
        eprintln!("settings menu: failed to write brightness: {error}");
    }
}

#[cfg(not(feature = "board-kms"))]
fn write_brightness(value: i32) {
    eprintln!("settings menu: brightness {value} is disabled outside the board-kms build");
}

fn main() -> Result<(), Box<dyn Error>> {
    configure_board_backend();
    #[cfg(feature = "board-kms")]
    touch::install()?;
    start_debug_wifi_in_background();
    let ui = AppWindow::new()?;
    ui.set_brightness(read_brightness());

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

    ui.on_restart_requested(|| {
        spawn_system_action(
            "application restart",
            "/etc/init.d/S15obd2-dashboard",
            &["restart"],
        );
    });
    ui.on_reboot_requested(|| {
        spawn_system_action("system reboot", "/sbin/reboot", &["-f"]);
    });
    ui.on_shutdown_requested(|| {
        spawn_system_action("system shutdown", "/sbin/poweroff", &[]);
    });
    ui.on_brightness_changed(write_brightness);

    ui.run()?;

    Ok(())
}
