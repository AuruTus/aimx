#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod crosshair;
mod overlay;
mod panel;
mod platform;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--overlay") {
        overlay::run()
    } else {
        panel::run()
    }
}
