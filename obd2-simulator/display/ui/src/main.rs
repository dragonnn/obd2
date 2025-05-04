use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use types::Pid;

#[macro_use]
extern crate tracing;

mod ipc_server;

fn main() -> eframe::Result<()> {
    tracing_subscriber::FmtSubscriber::builder().init();

    let display_buffers = [
        Arc::new(Mutex::new(vec![0x00u8; 256 * 64])),
        Arc::new(Mutex::new(vec![0x00u8; 256 * 64])),
    ];

    let (buttons_tx, buttons_rx): (Sender<(u8, bool)>, Receiver<(u8, bool)>) = channel(10);
    let (obd2_pids_tx, obd2_pids_rx): (Sender<types::Pid>, Receiver<types::Pid>) = channel(10);
    let (connected_tx, connected_rx): (UnboundedSender<()>, UnboundedReceiver<()>) =
        unbounded_channel();

    ipc_server::start(
        display_buffers.clone(),
        buttons_rx,
        obd2_pids_rx,
        connected_tx,
    );

    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([256.0 * 2.0, 80.0]),
        ..Default::default()
    };
    options.window_builder = Some(Box::new(|w| {
        w.with_app_id("com.example.obd2simulator")
            .with_title("OBD2 Simulator")
            .with_resizable(false)
            .with_inner_size(egui::vec2(256.0 * 2.0 + 25.0, 600.0))
            .with_maximized(false)
            .with_decorations(false)
            .with_visible(true)
    }));
    eframe::run_native(
        "OBD2 Simulator",
        options,
        Box::new(|cc| {
            Ok(Box::<Ui>::new(Ui::new(
                display_buffers.clone(),
                buttons_tx,
                obd2_pids_tx,
                connected_rx,
            )))
        }),
    )
}

struct Ui {
    display_buffers: [Arc<Mutex<Vec<u8>>>; 2],
    buttons_tx: Sender<(u8, bool)>,
    obd2_pids_tx: Sender<Pid>,
    connected_rx: UnboundedReceiver<()>,

    ac_pid: types::AcPid,
    bms_pid: types::BmsPid,
    ice_fuel_rate_pid: types::IceFuelRatePid,
    vehicle_speed_pid: types::VehicleSpeedPid,
    transaxle_pid: types::TransaxlePid,
}

impl Ui {
    fn new(
        display_buffers: [Arc<Mutex<Vec<u8>>>; 2],
        buttons_tx: Sender<(u8, bool)>,
        obd2_pids_tx: Sender<Pid>,
        connected_rx: UnboundedReceiver<()>,
    ) -> Self {
        Self {
            display_buffers,
            buttons_tx,
            obd2_pids_tx,
            connected_rx,

            ac_pid: Default::default(),
            bms_pid: Default::default(),
            ice_fuel_rate_pid: Default::default(),
            vehicle_speed_pid: Default::default(),
            transaxle_pid: Default::default(),
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                let mut menu_button_pressed = false;

                ui.horizontal(|ui| {
                    let display1 = egui::ColorImage::from_gray(
                        [256, 64],
                        &self.display_buffers[0].blocking_lock(),
                    );
                    let handle1 =
                        ctx.load_texture("display1", display1, egui::TextureOptions::default());
                    let texture1 =
                        egui::load::SizedTexture::new(handle1.id(), egui::vec2(256.0, 64.0));

                    let display2 = egui::ColorImage::from_gray(
                        [256, 64],
                        &self.display_buffers[1].blocking_lock(),
                    );
                    let handle2 =
                        ctx.load_texture("display2", display2, egui::TextureOptions::default());
                    let texture2 =
                        egui::load::SizedTexture::new(handle2.id(), egui::vec2(256.0, 64.0));
                    ui.vertical(|ui| {
                        ui.image(egui::ImageSource::Texture(texture1));
                        ui.horizontal_centered(|ui| {
                            ui.add_space(15.0);
                            ui.style_mut().spacing.item_spacing = egui::vec2(38.0, 16.0);
                            for i in 0..4 {
                                if ui.button(format!("{}", i)).clicked() {
                                    info!("Button {} clicked", i);
                                    self.buttons_tx.send((i, true)).ok();
                                    menu_button_pressed = true;
                                }
                            }
                        });
                    });

                    ui.vertical(|ui| {
                        ui.image(egui::ImageSource::Texture(texture2));
                        ui.horizontal_centered(|ui| {
                            ui.add_space(15.0 + 53.0);
                            ui.style_mut().spacing.item_spacing = egui::vec2(38.0, 16.0);
                            for i in 4..8 {
                                if ui.button(format!("{}", i)).clicked() {
                                    info!("Button {} clicked", i);
                                    self.buttons_tx.send((i, true)).ok();
                                    menu_button_pressed = true;
                                }
                            }
                        });
                    });
                });
                /*ui.horizontal_wrapped(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(86.0, 16.0);
                    for i in 0..6 {
                        if ui.button(format!("{}", i)).clicked() {
                            info!("Button {} clicked", i);
                            self.buttons_tx.send((i, true)).ok();
                        }
                    }
                });*/

                let connected = self.connected_rx.try_recv().is_ok();
                if menu_button_pressed {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Base PID");
                        if egui_probe::Probe::new(&mut self.ice_fuel_rate_pid)
                            .show(ui)
                            .changed()
                            || connected
                            || menu_button_pressed
                        {
                            self.obd2_pids_tx
                                .send(Pid::IceFuelRatePid(self.ice_fuel_rate_pid.clone()))
                                .ok();
                        }
                        if egui_probe::Probe::new(&mut self.vehicle_speed_pid)
                            .show(ui)
                            .changed()
                            || connected
                            || menu_button_pressed
                        {
                            self.obd2_pids_tx
                                .send(Pid::VehicleSpeedPid(self.vehicle_speed_pid.clone()))
                                .ok();
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("AC PID");
                        if egui_probe::Probe::new(&mut self.ac_pid).show(ui).changed()
                            || connected
                            || menu_button_pressed
                        {
                            self.obd2_pids_tx.send(Pid::AcPid(self.ac_pid.clone())).ok();
                        }
                    });

                    ui.vertical(|ui| {
                        ui.label("BMS PID");
                        if egui_probe::Probe::new(&mut self.bms_pid).show(ui).changed()
                            || connected
                            || menu_button_pressed
                        {
                            self.obd2_pids_tx
                                .send(Pid::BmsPid(self.bms_pid.clone()))
                                .ok();
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("TRANSAXLE PID");
                        if egui_probe::Probe::new(&mut self.transaxle_pid)
                            .show(ui)
                            .changed()
                            || connected
                            || menu_button_pressed
                        {
                            info!("Transaxle PID changed: {:?}", self.transaxle_pid);
                            self.obd2_pids_tx
                                .send(Pid::TransaxlePid(self.transaxle_pid.clone()))
                                .ok();
                        }
                    });
                });

                ctx.request_repaint_after(std::time::Duration::from_millis(10));
            });
        });
    }
}
