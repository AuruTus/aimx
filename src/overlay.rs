use eframe::egui;
use log::{debug, info, warn};
use std::io::BufRead;
use std::sync::{Arc, Mutex};

use crate::config::Config;

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
            // Store the egui context so the stdin reader thread can wake us
            *repaint_ctx.lock().unwrap() = Some(cc.egui_ctx.clone());
            Ok(Box::new(OverlayApp {
                config,
                config_changed,
                screen_size: (sw, sh),
            }))
        }),
    )
}

struct OverlayApp {
    config: Arc<Mutex<Config>>,
    config_changed: Arc<std::sync::atomic::AtomicBool>,
    screen_size: (f32, f32),
}

impl eframe::App for OverlayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let cfg = self.config.lock().unwrap().clone();

        if self.config_changed.swap(false, std::sync::atomic::Ordering::SeqCst) {
            let win_size = cfg.window_size();

            // Resize window to fit crosshair
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                egui::vec2(win_size, win_size),
            ));

            // Position window so crosshair is at screen center + offset
            let x = (self.screen_size.0 - win_size) / 2.0 + cfg.offset_x;
            let y = (self.screen_size.1 - win_size) / 2.0 + cfg.offset_y;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                egui::pos2(x, y).to_vec2().to_pos2(),
            ));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let center = ui.max_rect().center();
                crate::crosshair::draw(painter, center, &cfg);
            });
    }
}
