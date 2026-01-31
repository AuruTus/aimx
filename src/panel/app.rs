use eframe::egui::{self, ViewportCommand};
use log::info;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::Config;
use super::{ipc, style, tray};

pub struct PanelApp {
    config: Config,
    prev_config: Option<String>,
    child: Option<Child>,
    #[cfg(target_os = "windows")]
    job: Option<crate::platform::JobObject>,
    show_close_dialog: bool,
    remember_choice: bool,
    force_quit: bool,
    tray: Option<tray_icon::TrayIcon>,
    tray_restore: Arc<AtomicBool>,
    tray_quit: Arc<AtomicBool>,
    poller_started: bool,
    minimized: bool,
    theme: style::PanelTheme,
}

impl PanelApp {
    pub fn new() -> Self {
        let config = Config::load();
        #[cfg(target_os = "windows")]
        let job = crate::platform::JobObject::new();

        let child = ipc::spawn_overlay(
            #[cfg(target_os = "windows")]
            &job,
        );

        Self {
            config,
            prev_config: None,
            child,
            #[cfg(target_os = "windows")]
            job,
            show_close_dialog: false,
            remember_choice: false,
            force_quit: false,
            tray: None,
            tray_restore: Arc::new(AtomicBool::new(false)),
            tray_quit: Arc::new(AtomicBool::new(false)),
            poller_started: false,
            minimized: false,
            theme: style::PanelTheme::default(),
        }
    }

    fn minimize_to_tray(&mut self, ctx: &egui::Context) {
        self.tray_restore.store(false, Ordering::SeqCst);
        self.tray_quit.store(false, Ordering::SeqCst);
        self.tray = Some(tray::create_tray_icon());
        if !self.poller_started {
            self.poller_started = true;
            tray::spawn_tray_poller(
                self.tray_restore.clone(),
                self.tray_quit.clone(),
                ctx.clone(),
            );
        }
        self.minimized = true;
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        info!("minimized to tray");
    }

    fn restore_from_tray(&mut self, ctx: &egui::Context) {
        self.tray = None;
        self.minimized = false;
        self.tray_restore.store(false, Ordering::SeqCst);
        self.tray_quit.store(false, Ordering::SeqCst);
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        info!("restored from tray");
    }
}

impl eframe::App for PanelApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        style::apply_theme(ctx, &self.theme);

        // Check tray menu actions
        if self.minimized {
            if self.tray_quit.load(Ordering::SeqCst) {
                self.restore_from_tray(ctx);
                self.force_quit = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
            if self.tray_restore.load(Ordering::SeqCst) {
                self.restore_from_tray(ctx);
            }
            return;
        }

        // Handle close request
        if ctx.input(|i| i.viewport().close_requested()) && !self.force_quit {
            match self.config.close_action.as_str() {
                "quit" => {}
                "minimize" => {
                    ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                    self.minimize_to_tray(ctx);
                }
                _ => {
                    ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                    self.show_close_dialog = true;
                }
            }
        }

        // Close dialog
        if self.show_close_dialog {
            egui::Window::new("Close AIMX")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("What would you like to do?");
                    ui.add_space(8.0);

                    ui.checkbox(&mut self.remember_choice, "Remember my choice");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Quit").clicked() {
                            if self.remember_choice {
                                self.config.close_action = "quit".to_string();
                                self.config.save();
                            }
                            self.show_close_dialog = false;
                            self.force_quit = true;
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                        if ui.button("Minimize to Tray").clicked() {
                            if self.remember_choice {
                                self.config.close_action = "minimize".to_string();
                                self.config.save();
                            }
                            self.show_close_dialog = false;
                            self.minimize_to_tray(ctx);
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let overlay_alive = self.child.is_some();
            let action = style::draw_panel_ui(ui, &mut self.config, overlay_alive);

            match action {
                style::PanelAction::Save => self.config.save(),
                style::PanelAction::Reset => self.config = Config::default(),
                style::PanelAction::ShowOverlay => {
                    self.child = ipc::spawn_overlay(
                        #[cfg(target_os = "windows")]
                        &self.job,
                    );
                    self.prev_config = None;
                }
                style::PanelAction::HideOverlay => {
                    if let Some(mut child) = self.child.take() {
                        info!("killing overlay process");
                        let _ = child.kill();
                    }
                }
                style::PanelAction::None => {}
            }
        });

        ipc::send_config(&mut self.child, &self.config, &mut self.prev_config);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(mut child) = self.child.take() {
            info!("panel exiting, killing overlay process");
            let _ = child.kill();
        }
    }
}
