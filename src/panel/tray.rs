use eframe::egui;
use log::{info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::TrayIconBuilder;

pub const MENU_SHOW_ID: &str = "show";
pub const MENU_QUIT_ID: &str = "quit";

pub fn create_tray_icon() -> tray_icon::TrayIcon {
    let (rgba, w, h) = super::load_icon_rgba();
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
pub fn spawn_tray_poller(
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
