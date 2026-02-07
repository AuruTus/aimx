# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

AIMX is a Windows-native transparent HUD overlay application written in Rust. It creates a borderless, always-on-top, click-through window that renders a crosshair overlay using egui/eframe with WGPU rendering.

## Build Commands

```bash
# Cross-compile for Windows from Linux (requires cargo-xwin)
make xwin

# Debug cross-compile
make xwin-debug

# Standard cargo build
cargo build --release --target x86_64-pc-windows-msvc

# With Steam integration
cargo build --release --features steam
```

## Architecture

```
src/
├── main.rs              -- entry point: arg dispatch (default=panel, overlay subcommand)
├── platform.rs          -- Win32 FFI: screen_size(), apply_overlay_style(), JobObject
├── crosshair.rs         -- crosshair drawing logic, parameterized by Config
├── config.rs            -- Config struct (serde), JSON load/save next to executable
├── panel/
│   ├── mod.rs           -- pub fn run(), icon loading, NativeOptions setup
│   ├── app.rs           -- PanelApp struct, tray minimize/restore, eframe::App impl
│   ├── tray.rs          -- create_tray_icon(), spawn_tray_poller(), menu constants
│   ├── ipc.rs           -- spawn_overlay(), send_config() over stdin pipe
│   └── style.rs         -- PanelTheme, PanelAction enum, draw_panel_ui()
└── overlay/
    ├── mod.rs           -- pub fn run(), stdin reader thread, eframe setup
    └── app.rs           -- OverlayApp struct, viewport resize/reposition, eframe::App impl
```

### Two-Process Architecture

`aimx` (no args) runs the **control panel** as the main process. It spawns `aimx --overlay` as a child process with piped stdin. Config changes are sent as newline-delimited JSON over the pipe. Closing the panel kills the overlay child.

Built on:
- **eframe/egui** -- immediate-mode GUI framework
- **raw_window_handle** -- Win32 HWND extraction for overlay window styling
- **steamworks** -- optional Steam integration behind `steam` feature flag

## Key Details

- Rust edition 2024
- Release builds use LTO, single codegen unit, and panic=abort for minimal binary size
- Cross-compiled to `x86_64-pc-windows-msvc` from WSL using `cargo xwin`
- `#![windows_subsystem = "windows"]` hides console in release builds
- Config persists as JSON (`aimx_config.json`) next to executable
- IPC: panel writes newline-delimited JSON to overlay child's stdin pipe
- Overlay is fullscreen, transparent, click-through via Win32 color-key
- Panel spawns/kills overlay child process (Show/Hide Overlay button)

## Known Issues

- Panel/overlay process grouping in Windows Task Manager is incomplete (AppUserModelID set, but overlay still shows separately)
- Panel CPU usage is higher than expected when minimized to tray (busy-loop suspected in eframe/tray event loop)

## TODO

- [x] Cross-compile test with `make xwin`
- [ ] Auto-save config on change (deferred until features stabilize; manual save is better for debugging)
- [ ] Add crosshair shape presets (dot, cross, circle+cross)
- [ ] Overlay should exit gracefully when stdin closes (parent dies unexpectedly)
