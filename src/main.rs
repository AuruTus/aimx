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

fn main() -> eframe::Result<()> {
    env_logger::init();

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
