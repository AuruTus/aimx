#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod crosshair;
mod overlay;
mod platform;

use eframe::egui;

fn main() -> eframe::Result<()> {
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
        Box::new(|cc| {
            platform::apply_overlay_style(cc);
            Ok(Box::new(overlay::OverlayApp))
        }),
    )
}
