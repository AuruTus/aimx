use eframe::egui::{self, Color32};

use crate::config::Config;

#[allow(dead_code)]
pub struct PanelTheme {
    pub bg_color: Color32,
    pub heading_color: Color32,
    pub accent_color: Color32,
    pub button_rounding: f32,
    pub spacing: f32,
}

impl Default for PanelTheme {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_rgb(30, 30, 30),
            heading_color: Color32::WHITE,
            accent_color: Color32::from_rgb(100, 149, 237),
            button_rounding: 4.0,
            spacing: 8.0,
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, _theme: &PanelTheme) {
    ctx.set_visuals(egui::Visuals::dark());
}

pub enum PanelAction {
    None,
    Save,
    Reset,
    ShowOverlay,
    HideOverlay,
}

pub fn draw_panel_ui(ui: &mut egui::Ui, config: &mut Config, overlay_alive: bool) -> PanelAction {
    let mut action = PanelAction::None;

    ui.heading("Crosshair Settings");
    ui.separator();

    // Overlay control
    if overlay_alive {
        if ui.button("Hide Overlay").clicked() {
            action = PanelAction::HideOverlay;
        }
    } else if ui.button("Show Overlay").clicked() {
        action = PanelAction::ShowOverlay;
    }

    ui.separator();
    ui.label("Position Offset");
    ui.add(egui::Slider::new(&mut config.offset_x, -500.0..=500.0).text("X"));
    ui.add(egui::Slider::new(&mut config.offset_y, -500.0..=500.0).text("Y"));

    ui.separator();
    ui.label("Size");
    ui.add(egui::Slider::new(&mut config.inner_radius, 0.5..=50.0).text("Inner Radius"));
    ui.add(egui::Slider::new(&mut config.outer_radius, 0.5..=50.0).text("Outer Radius"));
    ui.add(egui::Slider::new(&mut config.stroke_width, 0.1..=10.0).text("Stroke Width"));

    ui.separator();
    ui.label("Colors");

    let mut fill = Color32::from_rgb(config.color[0], config.color[1], config.color[2]);
    ui.horizontal(|ui| {
        ui.label("Fill:");
        ui.color_edit_button_srgba(&mut fill);
    });
    config.color = [fill.r(), fill.g(), fill.b()];

    let mut stroke = Color32::from_rgb(
        config.stroke_color[0],
        config.stroke_color[1],
        config.stroke_color[2],
    );
    ui.horizontal(|ui| {
        ui.label("Stroke:");
        ui.color_edit_button_srgba(&mut stroke);
    });
    config.stroke_color = [stroke.r(), stroke.g(), stroke.b()];

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            action = PanelAction::Save;
        }
        if ui.button("Reset").clicked() {
            action = PanelAction::Reset;
        }
    });

    action
}
