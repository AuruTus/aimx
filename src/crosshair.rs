use eframe::egui;

pub fn draw(painter: &egui::Painter, center: egui::Pos2) {
    painter.circle_filled(center, 3.0, egui::Color32::RED);
    painter.circle_stroke(center, 3.5, egui::Stroke::new(1.0, egui::Color32::BLACK));
}
