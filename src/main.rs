#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod crosshair;
mod overlay;
mod panel;
mod platform;

use clap::{Parser, Subcommand};
use log::info;

#[derive(Parser)]
#[command(name = "aimx", about = "Crosshair overlay with control panel")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the overlay process (used internally by the panel)
    Overlay,
}

#[cfg(target_os = "windows")]
fn set_app_user_model_id() {
    unsafe extern "system" {
        fn SetCurrentProcessExplicitAppUserModelID(app_id: *const u16) -> i32;
    }
    let id: Vec<u16> = "AIMX.Panel.1\0".encode_utf16().collect();
    unsafe {
        SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
    }
}

#[cfg(not(target_os = "windows"))]
fn set_app_user_model_id() {}

fn main() -> eframe::Result<()> {
    env_logger::init();
    set_app_user_model_id();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Overlay) => {
            info!("starting aimx in overlay mode");
            overlay::run()
        }
        None => {
            info!("starting aimx in panel mode");
            panel::run()
        }
    }
}
