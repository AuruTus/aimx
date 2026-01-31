use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::config::Config;

pub struct OverlayApp {
    config: Arc<Mutex<Config>>,
    config_changed: Arc<std::sync::atomic::AtomicBool>,
    screen_size: (f32, f32),
}

impl OverlayApp {
    pub fn new(
        config: Arc<Mutex<Config>>,
        config_changed: Arc<std::sync::atomic::AtomicBool>,
        screen_size: (f32, f32),
    ) -> Self {
        Self {
            config,
            config_changed,
            screen_size,
        }
    }
}

impl eframe::App for OverlayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let cfg = self.config.lock().unwrap().clone();

        if self.config_changed.swap(false, std::sync::atomic::Ordering::SeqCst) {
            let win_size = cfg.window_size();

            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                egui::vec2(win_size, win_size),
            ));

            let x = (self.screen_size.0 - win_size) / 2.0 + cfg.offset_x;
            let y = (self.screen_size.1 - win_size) / 2.0 + cfg.offset_y;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                egui::pos2(x, y).to_vec2().to_pos2(),
            ));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let center = ui.max_rect().center();
                crate::crosshair::draw(painter, center, &cfg);
            });
    }
}
