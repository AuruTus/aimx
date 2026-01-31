mod app;
mod ipc;
mod style;
mod tray;

use eframe::egui;

const ICON_BYTES: &[u8] = include_bytes!("../../assets/icon.png");

fn load_icon_rgba() -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory(ICON_BYTES)
        .expect("failed to decode embedded icon")
        .into_rgba8();
    let (w, h) = img.dimensions();
    (img.into_raw(), w, h)
}

pub fn run() -> eframe::Result<()> {
    let (rgba, w, h) = load_icon_rgba();
    let viewport_icon = egui::IconData {
        rgba: rgba.clone(),
        width: w,
        height: h,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AIMX Control Panel")
            .with_inner_size([350.0, 450.0])
            .with_icon(viewport_icon),
        ..Default::default()
    };

    eframe::run_native(
        "AIMX Control Panel",
        options,
        Box::new(|cc| {
            #[cfg(target_os = "windows")]
            {
                use raw_window_handle::HasWindowHandle;
                if let Ok(handle) = cc.window_handle()
                    && let raw_window_handle::RawWindowHandle::Win32(w) = handle.as_raw()
                {
                    crate::platform::store_panel_hwnd(w.hwnd.get());
                }
            }
            #[cfg(not(target_os = "windows"))]
            let _ = cc;
            Ok(Box::new(app::PanelApp::new()))
        }),
    )
}
