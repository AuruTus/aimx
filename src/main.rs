#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod crosshair;
mod overlay;
mod panel;
mod platform;

use eframe::egui;
use std::sync::{Arc, Mutex};

fn main() -> eframe::Result<()> {
    let config = Arc::new(Mutex::new(config::Config::load()));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_inner_size([100.0, 100.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PrecisionHUD_Overlay",
        options,
        Box::new(move |cc| {
            platform::apply_overlay_style(cc);
            Ok(Box::new(overlay::OverlayApp::new(config)))
        }),
    )
}
