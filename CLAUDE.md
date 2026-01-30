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
├── main.rs        -- entry point: arg dispatch (default=panel, --overlay=overlay)
├── platform.rs    -- Win32 FFI: screen_size(), apply_overlay_style()
├── overlay.rs     -- overlay process: fullscreen transparent window, reads config from stdin
├── panel.rs       -- panel process: control panel GUI, spawns overlay as child, writes config to stdin
├── crosshair.rs   -- crosshair drawing logic, parameterized by Config
├── config.rs      -- Config struct (serde), JSON load/save next to executable
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

## TODO

- [ ] Cross-compile test with `make xwin`
- [ ] Add hotkey (e.g. F1) to toggle overlay visibility
- [ ] Consider auto-save on change vs current manual save button
- [ ] Add crosshair shape presets (dot, cross, circle+cross)
- [ ] Overlay should exit gracefully when stdin closes (parent dies unexpectedly)
