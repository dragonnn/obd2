use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::broadcast::{Receiver, Sender, channel};

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
    ipc_server::start(display_buffers.clone(), buttons_rx);

    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([256.0 * 2.0, 80.0]),
        ..Default::default()
    };
    options.window_builder = Some(Box::new(|w| {
        w.with_app_id("com.example.obd2simulator")
            .with_title("OBD2 Simulator")
            .with_resizable(false)
            .with_inner_size(egui::vec2(256.0 * 2.0 + 25.0, 100.0))
            .with_maximized(false)
            .with_decorations(false)
            .with_visible(true)
    }));
    eframe::run_native(
        "OBD2 Simulator",
        options,
        Box::new(|cc| Ok(Box::<Ui>::new(Ui::new(display_buffers.clone(), buttons_tx)))),
    )
}

struct Ui {
    display_buffers: [Arc<Mutex<Vec<u8>>>; 2],
    buttons_tx: Sender<(u8, bool)>,
}

impl Ui {
    fn new(display_buffers: [Arc<Mutex<Vec<u8>>>; 2], buttons_tx: Sender<(u8, bool)>) -> Self {
        Self {
            display_buffers,
            buttons_tx,
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
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

                    ui.image(egui::ImageSource::Texture(texture1));
                    ui.image(egui::ImageSource::Texture(texture2));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(86.0, 16.0);
                    for i in 0..6 {
                        if ui.button(format!("{}", i)).clicked() {
                            info!("Button {} clicked", i);
                            self.buttons_tx.send((i, true)).ok();
                        }
                    }
                });
                ctx.request_repaint_after(std::time::Duration::from_millis(10));
            });
        });
    }
}
