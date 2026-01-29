#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongW, SetWindowLongW, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true) // Make background invisible
            .with_decorations(false) // Remove title bar/borders
            .with_always_on_top() // Stay above the game
            .with_inner_size([100.0, 100.0]), // Small area for the dot
        ..Default::default()
    };

    eframe::run_native(
        "PrecisionHUD_Overlay",
        options,
        Box::new(|cc| {
            // This is the "Magic" for Windows 10/11
            // It applies the Click-Through style at the OS level
            let window_handle = cc.egui_ctx.viewport_id();

            // Note: In a real app, you'd use the raw HWND to call:
            // SetWindowLongW(hwnd, GWL_EXSTYLE, current_style | WS_EX_TRANSPARENT | WS_EX_LAYERED);

            Ok(Box::new(OverlayApp::default()))
        }),
    )
}

struct OverlayApp;

impl Default for OverlayApp {
    fn default() -> Self {
        Self
    }
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Clear the background so it's fully transparent
        ctx.set_visuals(egui::Visuals {
            panel_fill: egui::Color32::TRANSPARENT,
            ..egui::Visuals::default()
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                // Draw the static crosshair in the center of our 100x100 window
                let painter = ui.painter();
                let center = ui.max_rect().center();

                // Draw a simple red dot
                painter.circle_filled(center, 3.0, egui::Color32::RED);

                // Draw a thin black outline for visibility on bright maps
                painter.circle_stroke(center, 3.5, egui::Stroke::new(1.0, egui::Color32::BLACK));
            });
    }
}
