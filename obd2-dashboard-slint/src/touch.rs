use std::cell::Cell;
use std::rc::Rc;

use input::event::{DeviceEvent, EventTrait};

// The software renderer maps landscape (x, y) to portrait (y, 927 - x).
// Apply its inverse to normalized libinput coordinates: (u, v) -> (1-v, u).
const CALIBRATION: [f32; 6] = [0., -1., 1., 1., 0., 0.];

#[derive(Default)]
struct TouchInput {
    calibrated: Cell<bool>,
}

impl TouchInput {
    fn event(&self, event: &input::Event) -> bool {
        let mut device = event.device();
        if device.name() != "Hynitron CST3530 Touchscreen" {
            return false;
        }
        match event {
            input::Event::Device(DeviceEvent::Added(_)) => {
                match device.config_calibration_set_matrix(CALIBRATION) {
                    Ok(()) => {
                        self.calibrated.set(true);
                        eprintln!("touch: CST3530 connected, mapped to 270-degree dashboard");
                    }
                    Err(error) => {
                        self.calibrated.set(false);
                        eprintln!("touch: cannot calibrate CST3530: {error:?}");
                    }
                }
            }
            input::Event::Device(DeviceEvent::Removed(_)) => {
                self.calibrated.set(false);
            }
            input::Event::Touch(_) => {
                // Do not deliver incorrectly oriented input if calibration failed.
                if !self.calibrated.get() {
                    return true;
                }
            }
            _ => {}
        }
        // Observe only: LinuxKMS still delivers these calibrated events to Slint.
        false
    }
}

pub fn install() -> Result<(), Box<dyn std::error::Error>> {
    let touch = Rc::new(TouchInput::default());
    let hook = touch.clone();
    let backend = i_slint_backend_linuxkms::BackendBuilder::default()
        .with_renderer_name("software".into())
        .with_libinput_event_hook(Box::new(move |event| hook.event(event)))
        .build()?;
    slint::platform::set_platform(Box::new(backend))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_corners_map_to_rotated_dashboard() {
        for ((u, v), expected) in [
            ((0., 0.), (1., 0.)),
            ((1., 0.), (1., 1.)),
            ((0., 1.), (0., 0.)),
            ((1., 1.), (0., 1.)),
            ((0.5, 0.5), (0.5, 0.5)),
        ] {
            let [a, b, c, d, e, f] = CALIBRATION;
            assert_eq!((a * u + b * v + c, d * u + e * v + f), expected);
        }
    }
}
