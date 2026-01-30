use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::config::Config;

pub fn show(ctx: &egui::Context, config: &Arc<Mutex<Config>>) {
    let mut cfg = config.lock().unwrap().clone();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Crosshair Settings");
        ui.separator();

        ui.label("Position Offset");
        ui.add(egui::Slider::new(&mut cfg.offset_x, -500.0..=500.0).text("X"));
        ui.add(egui::Slider::new(&mut cfg.offset_y, -500.0..=500.0).text("Y"));

        ui.separator();
        ui.label("Size");
        ui.add(egui::Slider::new(&mut cfg.inner_radius, 0.5..=50.0).text("Inner Radius"));
        ui.add(egui::Slider::new(&mut cfg.outer_radius, 0.5..=50.0).text("Outer Radius"));
        ui.add(egui::Slider::new(&mut cfg.stroke_width, 0.1..=10.0).text("Stroke Width"));

        ui.separator();
        ui.label("Colors");

        let mut fill = egui::Color32::from_rgb(cfg.color[0], cfg.color[1], cfg.color[2]);
        ui.horizontal(|ui| {
            ui.label("Fill:");
            ui.color_edit_button_srgba(&mut fill);
        });
        cfg.color = [fill.r(), fill.g(), fill.b()];

        let mut stroke = egui::Color32::from_rgb(cfg.stroke_color[0], cfg.stroke_color[1], cfg.stroke_color[2]);
        ui.horizontal(|ui| {
            ui.label("Stroke:");
            ui.color_edit_button_srgba(&mut stroke);
        });
        cfg.stroke_color = [stroke.r(), stroke.g(), stroke.b()];

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                cfg.save();
            }
            if ui.button("Reset").clicked() {
                cfg = Config::default();
            }
        });
    });

    *config.lock().unwrap() = cfg;
}
