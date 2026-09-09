use std::cell::RefCell;
use std::rc::Rc;

use input::event::touch::{TouchEventPosition, TouchEventSlot};
use input::event::{DeviceEvent, EventTrait, TouchEvent};
use slint::{ComponentHandle, VecModel};

use crate::{AppWindow, TouchMarker};

#[path = "touch_state.rs"]
mod state;
use state::{CALIBRATION, TouchState};

#[derive(Default)]
pub struct TouchInput {
    ui: RefCell<Option<slint::Weak<AppWindow>>>,
    state: RefCell<TouchState>,
    calibrated: std::cell::Cell<bool>,
}

impl TouchInput {
    pub fn attach(&self, ui: &AppWindow) {
        *self.ui.borrow_mut() = Some(ui.as_weak());
    }

    fn publish(&self) {
        let Some(ui) = self.ui.borrow().as_ref().and_then(slint::Weak::upgrade) else {
            return;
        };
        let markers = self
            .state
            .borrow()
            .positions
            .iter()
            .map(|position| {
                let (x, y) = position.unwrap_or_default();
                TouchMarker {
                    active: position.is_some(),
                    x,
                    y,
                }
            })
            .collect::<Vec<_>>();
        ui.set_touch_markers(Rc::new(VecModel::from(markers)).into());
    }

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
                self.state.borrow_mut().clear();
                self.publish();
            }
            input::Event::Touch(touch) => {
                // Do not deliver incorrectly oriented input if calibration failed.
                if !self.calibrated.get() {
                    return true;
                }
                match touch {
                    TouchEvent::Down(e) => self.state.borrow_mut().update(
                        e.slot(),
                        Some((e.x_transformed(1) as f32, e.y_transformed(1) as f32)),
                    ),
                    TouchEvent::Motion(e) => self.state.borrow_mut().update(
                        e.slot(),
                        Some((e.x_transformed(1) as f32, e.y_transformed(1) as f32)),
                    ),
                    TouchEvent::Up(e) => self.state.borrow_mut().update(e.slot(), None),
                    TouchEvent::Cancel(e) => {
                        if e.slot().is_some() {
                            self.state.borrow_mut().update(e.slot(), None);
                        } else {
                            self.state.borrow_mut().clear();
                        }
                        self.publish();
                    }
                    // Publish complete frames, avoiding partially updated multitouch dots.
                    TouchEvent::Frame(_) => self.publish(),
                    _ => {}
                }
            }
            _ => {}
        }
        // Observe only: LinuxKMS still delivers these calibrated events to Slint.
        false
    }
}

pub fn install() -> Result<Rc<TouchInput>, Box<dyn std::error::Error>> {
    let touch = Rc::new(TouchInput::default());
    let hook = touch.clone();
    let backend = i_slint_backend_linuxkms::BackendBuilder::default()
        .with_renderer_name("software".into())
        .with_libinput_event_hook(Box::new(move |event| hook.event(event)))
        .build()?;
    slint::platform::set_platform(Box::new(backend))?;
    Ok(touch)
}
