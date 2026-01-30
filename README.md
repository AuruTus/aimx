# AIMX

A Windows-native crosshair overlay with a control panel, written in Rust.

AIMX renders a transparent, always-on-top, click-through crosshair on your screen. A separate control panel window lets you adjust position, size, and colors in real-time. Settings persist as JSON.

## Features

- Transparent overlay with Win32 color-key transparency
- Configurable crosshair: position offset, fill/stroke color, radius, stroke width
- Two-process architecture: panel (main) spawns overlay as a background child process
- Config saved as `aimx_config.json` next to the executable
- Overlay is hidden from the taskbar

## Usage

Run `aimx.exe` to launch the control panel, which automatically spawns the overlay.

```
aimx            # launches control panel + overlay
aimx overlay    # launches overlay only (used internally)
aimx --help     # show usage
```

Use the control panel to adjust crosshair settings. Click **Save** to persist to disk. Click **Hide Overlay** / **Show Overlay** to toggle the crosshair. Closing the control panel exits the application.

## Build

Requires Rust (edition 2024).

```bash
# Native build on Windows
make win

# Cross-compile from Linux (requires cargo-xwin)
make xwin

# Debug cross-compile
make xwin-debug

# Type check
make check
```

The release binary is at `target/x86_64-pc-windows-msvc/release/aimx.exe`.

## Architecture

```
src/
  main.rs        -- CLI entry point (clap), dispatches to panel or overlay
  panel.rs       -- control panel GUI, spawns overlay child process
  overlay.rs     -- transparent overlay window, reads config from stdin
  crosshair.rs   -- crosshair drawing logic
  config.rs      -- Config struct, JSON persistence
  platform.rs    -- Win32 FFI (transparency, click-through, screen size)
```

The panel process sends config updates to the overlay via newline-delimited JSON over stdin pipe. The overlay dynamically resizes and repositions its window to fit the crosshair at screen center + offset.

## License

All rights reserved.
