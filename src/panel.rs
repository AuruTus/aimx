use eframe::egui::{self, ViewportCommand};
use log::{debug, error, info, warn};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::TrayIconBuilder;

use crate::config::Config;

const MENU_SHOW_ID: &str = "show";
const MENU_QUIT_ID: &str = "quit";

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

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
            // Store the HWND so the tray poller thread can show the window directly
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
            Ok(Box::new(PanelApp::new()))
        }),
    )
}

fn create_tray_icon() -> tray_icon::TrayIcon {
    let (rgba, w, h) = load_icon_rgba();
    let icon = tray_icon::Icon::from_rgba(rgba, w, h).expect("failed to create tray icon");

    let menu = Menu::new();
    let show_item = MenuItem::with_id(MENU_SHOW_ID, "Show Panel", true, None);
    let quit_item = MenuItem::with_id(MENU_QUIT_ID, "Quit", true, None);
    menu.append(&show_item).unwrap();
    menu.append(&quit_item).unwrap();

    TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip("AIMX - Right-click for menu")
        .build()
        .expect("failed to build tray icon")
}

/// Spawns a background thread that polls tray icon and menu events
/// and sets atomic flags + wakes egui when something happens.
fn spawn_tray_poller(
    restore_flag: Arc<AtomicBool>,
    quit_flag: Arc<AtomicBool>,
    ctx: egui::Context,
) {
    let menu_rx = MenuEvent::receiver().clone();
    let tray_rx = tray_icon::TrayIconEvent::receiver().clone();

    std::thread::spawn(move || {
        info!("tray poller thread started");
        loop {
            crossbeam_channel::select! {
                recv(menu_rx) -> event => {
                    match event {
                        Ok(event) => {
                            let id = event.id().0.as_str();
                            info!("tray poller: menu event received, id={:?}", id);
                            if id == MENU_SHOW_ID {
                                info!("tray poller: setting restore flag");
                                restore_flag.store(true, Ordering::SeqCst);
                                crate::platform::show_panel_window();
                                ctx.request_repaint();
                            } else if id == MENU_QUIT_ID {
                                info!("tray poller: setting quit flag");
                                quit_flag.store(true, Ordering::SeqCst);
                                crate::platform::show_panel_window();
                                ctx.request_repaint();
                            }
                        }
                        Err(e) => {
                            warn!("tray poller: menu channel error: {e}");
                        }
                    }
                }
                recv(tray_rx) -> event => {
                    match event {
                        Ok(tray_icon::TrayIconEvent::Click {
                            button: tray_icon::MouseButton::Left,
                            button_state: tray_icon::MouseButtonState::Up,
                            ..
                        } | tray_icon::TrayIconEvent::DoubleClick {
                            button: tray_icon::MouseButton::Left,
                            ..
                        }) => {
                            info!("tray poller: left click, setting restore flag");
                            restore_flag.store(true, Ordering::SeqCst);
                            crate::platform::show_panel_window();
                            ctx.request_repaint();
                        }
                        Ok(_) => {} // ignore Move, Enter, Leave, right-clicks
                        Err(e) => {
                            warn!("tray poller: tray icon channel error: {e}");
                        }
                    }
                }
            }
        }
    });
}

struct PanelApp {
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
}

impl PanelApp {
    fn new() -> Self {
        let config = Config::load();
        let mut app = Self {
            config,
            prev_config: None,
            child: None,
            #[cfg(target_os = "windows")]
            job: crate::platform::JobObject::new(),
            show_close_dialog: false,
            remember_choice: false,
            force_quit: false,
            tray: None,
            tray_restore: Arc::new(AtomicBool::new(false)),
            tray_quit: Arc::new(AtomicBool::new(false)),
            poller_started: false,
            minimized: false,
        };
        app.spawn_overlay();
        app
    }

    fn spawn_overlay(&mut self) {
        let exe = std::env::current_exe().expect("cannot find own executable");
        let mut cmd = Command::new(exe);
        cmd.arg("overlay").stdin(Stdio::piped());

        // Hide the overlay from the taskbar / process list on Windows
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        match cmd.spawn()
        {
            Ok(child) => {
                info!("spawned overlay process (pid: {})", child.id());
                #[cfg(target_os = "windows")]
                if let Some(job) = &self.job {
                    job.assign(&child);
                }
                self.child = Some(child);
            }
            Err(e) => error!("failed to spawn overlay: {e}"),
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
        debug!("config changed, sending to overlay");
        self.prev_config = Some(json.clone());

        if let Some(child) = &mut self.child && let Some(stdin) = &mut child.stdin {
                let line = format!("{json}\n");
                if stdin.write_all(line.as_bytes()).is_err() {
                    warn!("overlay stdin write failed, process likely died");
                    self.child = None;
                }
        }
    }

    fn minimize_to_tray(&mut self, ctx: &egui::Context) {
        self.tray_restore.store(false, Ordering::SeqCst);
        self.tray_quit.store(false, Ordering::SeqCst);
        self.tray = Some(create_tray_icon());
        // Start polling thread on first minimize; it stays alive and is harmless when not minimized
        if !self.poller_started {
            self.poller_started = true;
            spawn_tray_poller(
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
        self.tray = None; // drops tray icon, removing it from system tray
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
        // Check tray menu actions (set by event handlers on the Win32 thread)
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
                "quit" => {} // let it close naturally
                "minimize" => {
                    ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                    self.minimize_to_tray(ctx);
                }
                _ => {
                    // "ask"
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
            ui.heading("Crosshair Settings");
            ui.separator();

            // Overlay control
            let overlay_alive = self.child.is_some();
            if overlay_alive {
                if ui.button("Hide Overlay").clicked() && let Some(mut child) = self.child.take() {
                        info!("killing overlay process");
                        let _ = child.kill();
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
            info!("panel exiting, killing overlay process");
            let _ = child.kill();
        }
    }
}
