mod app;

use eframe::egui;
use log::info;
use std::sync::{Arc, Mutex};

use crate::config::Config;
use app::OverlayApp;

pub fn run() -> eframe::Result<()> {
    let config = Arc::new(Mutex::new(Config::load()));
    let config_changed = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let (sw, sh) = crate::platform::screen_size();
    info!("screen size: {sw}x{sh}");

    let cfg = config.lock().unwrap().clone();
    let win_size = cfg.window_size();
    let start_x = (sw - win_size) / 2.0 + cfg.offset_x;
    let start_y = (sh - win_size) / 2.0 + cfg.offset_y;
    info!("overlay window: pos=({start_x}, {start_y}), size={win_size}x{win_size}");

    // Shared egui context so the stdin reader can wake the overlay
    let repaint_ctx: Arc<Mutex<Option<egui::Context>>> = Arc::new(Mutex::new(None));

    // Background thread reads config updates from stdin
    let config_reader = config.clone();
    let changed_writer = config_changed.clone();
    let repaint_ctx_reader = repaint_ctx.clone();
    std::thread::spawn(move || {
        use log::{debug, warn};
        use std::io::BufRead;

        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) if !line.is_empty() => {
                    match serde_json::from_str::<Config>(&line) {
                        Ok(cfg) => {
                            debug!("received config update via stdin");
                            *config_reader.lock().unwrap() = cfg;
                            changed_writer.store(true, std::sync::atomic::Ordering::SeqCst);
                            if let Some(ctx) = repaint_ctx_reader.lock().unwrap().as_ref() {
                                ctx.request_repaint();
                            }
                        }
                        Err(e) => warn!("bad config from stdin: {e}"),
                    }
                }
                Err(e) => {
                    info!("stdin closed ({e}), overlay will stop receiving updates");
                    break;
                }
                _ => {}
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_taskbar(false)
            .with_position([start_x, start_y])
            .with_inner_size([win_size, win_size]),
        ..Default::default()
    };

    eframe::run_native(
        "AIMX Overlay",
        options,
        Box::new(move |cc| {
            crate::platform::apply_overlay_style(cc);
            *repaint_ctx.lock().unwrap() = Some(cc.egui_ctx.clone());
            Ok(Box::new(OverlayApp::new(config, config_changed, (sw, sh))))
        }),
    )
}
