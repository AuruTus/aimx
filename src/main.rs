#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod crosshair;
mod overlay;
mod panel;
mod platform;

use log::info;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let mode = if args.iter().any(|a| a == "--overlay") {
        "overlay"
    } else {
        "panel"
    };

    info!("starting aimx in {mode} mode");

    if mode == "overlay" {
        overlay::run()
    } else {
        panel::run()
    }
}
