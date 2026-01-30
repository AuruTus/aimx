use eframe::egui;
use std::io::Write;
use std::process::{Child, Command, Stdio};

use crate::config::Config;

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AIMX Control Panel")
            .with_inner_size([350.0, 450.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AIMX Control Panel",
        options,
        Box::new(|_cc| Ok(Box::new(PanelApp::new()))),
    )
}

struct PanelApp {
    config: Config,
    prev_config: Option<String>,
    child: Option<Child>,
}

impl PanelApp {
    fn new() -> Self {
        let config = Config::load();
        let mut app = Self {
            config,
            prev_config: None,
            child: None,
        };
        app.spawn_overlay();
        app
    }

    fn spawn_overlay(&mut self) {
        let exe = std::env::current_exe().expect("cannot find own executable");
        match Command::new(exe)
            .arg("--overlay")
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => self.child = Some(child),
            Err(e) => eprintln!("failed to spawn overlay: {e}"),
        }
    }

    fn send_config(&mut self) {
        let json = match serde_json::to_string(&self.config) {
            Ok(j) => j,
            Err(_) => return,
        };

        // Only send if config actually changed
        if self.prev_config.as_deref() == Some(&json) {
            return;
        }
        self.prev_config = Some(json.clone());

        if let Some(child) = &mut self.child {
            if let Some(stdin) = &mut child.stdin {
                let line = format!("{json}\n");
                if stdin.write_all(line.as_bytes()).is_err() {
                    // Overlay process died; drop the child handle
                    self.child = None;
                }
            }
        }
    }
}

impl eframe::App for PanelApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crosshair Settings");
            ui.separator();

            // Overlay control
            let overlay_alive = self.child.is_some();
            if overlay_alive {
                if ui.button("Hide Overlay").clicked() {
                    if let Some(mut child) = self.child.take() {
                        let _ = child.kill();
                    }
                }
            } else if ui.button("Show Overlay").clicked() {
                self.spawn_overlay();
                self.prev_config = None; // Force resend
            }

            ui.separator();
            ui.label("Position Offset");
            ui.add(egui::Slider::new(&mut self.config.offset_x, -500.0..=500.0).text("X"));
            ui.add(egui::Slider::new(&mut self.config.offset_y, -500.0..=500.0).text("Y"));

            ui.separator();
            ui.label("Size");
            ui.add(egui::Slider::new(&mut self.config.inner_radius, 0.5..=50.0).text("Inner Radius"));
            ui.add(egui::Slider::new(&mut self.config.outer_radius, 0.5..=50.0).text("Outer Radius"));
            ui.add(egui::Slider::new(&mut self.config.stroke_width, 0.1..=10.0).text("Stroke Width"));

            ui.separator();
            ui.label("Colors");

            let mut fill = egui::Color32::from_rgb(self.config.color[0], self.config.color[1], self.config.color[2]);
            ui.horizontal(|ui| {
                ui.label("Fill:");
                ui.color_edit_button_srgba(&mut fill);
            });
            self.config.color = [fill.r(), fill.g(), fill.b()];

            let mut stroke = egui::Color32::from_rgb(self.config.stroke_color[0], self.config.stroke_color[1], self.config.stroke_color[2]);
            ui.horizontal(|ui| {
                ui.label("Stroke:");
                ui.color_edit_button_srgba(&mut stroke);
            });
            self.config.stroke_color = [stroke.r(), stroke.g(), stroke.b()];

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.config.save();
                }
                if ui.button("Reset").clicked() {
                    self.config = Config::default();
                }
            });
        });

        self.send_config();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}
