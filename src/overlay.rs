use eframe::egui;
use std::io::BufRead;
use std::sync::{Arc, Mutex};

use crate::config::Config;

pub fn run() -> eframe::Result<()> {
    let config = Arc::new(Mutex::new(Config::load()));
    let (sw, sh) = crate::platform::screen_size();

    // Background thread reads config updates from stdin
    let config_reader = config.clone();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) if !line.is_empty() => {
                    if let Ok(cfg) = serde_json::from_str::<Config>(&line) {
                        *config_reader.lock().unwrap() = cfg;
                    }
                }
                Err(_) => break, // stdin closed, parent died
                _ => {}
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_position([0.0, 0.0])
            .with_inner_size([sw, sh]),
        ..Default::default()
    };

    eframe::run_native(
        "AIMX Overlay",
        options,
        Box::new(move |cc| {
            crate::platform::apply_overlay_style(cc);
            Ok(Box::new(OverlayApp { config }))
        }),
    )
}

struct OverlayApp {
    config: Arc<Mutex<Config>>,
}

impl eframe::App for OverlayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let center = ui.max_rect().center();
                let cfg = self.config.lock().unwrap().clone();
                crate::crosshair::draw(painter, center, &cfg);
            });

        // Request repaint so we pick up config changes from stdin
        ctx.request_repaint();
    }
}
