use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::config::Config;

pub struct OverlayApp {
    pub config: Arc<Mutex<Config>>,
    pub show_panel: bool,
}

impl OverlayApp {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            show_panel: true,
        }
    }
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
                let config = self.config.lock().unwrap().clone();
                crate::crosshair::draw(painter, center, &config);
            });

        if self.show_panel {
            let config = self.config.clone();
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("control_panel"),
                egui::ViewportBuilder::default()
                    .with_title("AIMX Control Panel")
                    .with_inner_size([350.0, 400.0]),
                move |ctx, _class| {
                    crate::panel::show(ctx, &config);
                },
            );
        }
    }
}
