use eframe::egui;

use crate::config::Config;

pub fn draw(painter: &egui::Painter, center: egui::Pos2, config: &Config) {
    let fill = egui::Color32::from_rgb(config.color[0], config.color[1], config.color[2]);
    let stroke_color = egui::Color32::from_rgb(config.stroke_color[0], config.stroke_color[1], config.stroke_color[2]);

    painter.circle_filled(center, config.inner_radius, fill);
    painter.circle_stroke(center, config.outer_radius, egui::Stroke::new(config.stroke_width, stroke_color));
}
