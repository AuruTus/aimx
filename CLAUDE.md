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
├── main.rs        -- entry point: NativeOptions setup, eframe::run_native
├── platform.rs    -- Win32 FFI, apply_overlay_style() (HWND extraction, click-through)
├── overlay.rs     -- OverlayApp struct, eframe::App impl (clear_color, update)
├── crosshair.rs   -- crosshair drawing logic (circle_filled + circle_stroke)
```

Built on:
- **eframe/egui** — immediate-mode GUI framework for rendering the overlay
- **raw_window_handle** — Win32 HWND extraction for overlay window styling
- **steamworks** — optional Steam integration behind `steam` feature flag

The overlay window is configured as transparent, undecorated, and click-through at the OS level. The `OverlayApp` struct implements `eframe::App` and draws the HUD elements in `update()`.

## Key Details

- Rust edition 2024
- Release builds use LTO, single codegen unit, and panic=abort for minimal binary size
- Cross-compiled to `x86_64-pc-windows-msvc` from WSL using `cargo xwin`
- `#![windows_subsystem = "windows"]` hides console in release builds
