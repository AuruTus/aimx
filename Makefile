# Native Windows build (run on Windows or CI)
.PHONY: win
win:
	cargo build --release --target x86_64-pc-windows-msvc

# Cross-compile for Windows from Linux (requires cargo-xwin)
.PHONY: xwin
xwin:
	cargo xwin build --release --target x86_64-pc-windows-msvc

# Cross-compile debug build from Linux
.PHONY: xwin-debug
xwin-debug:
	cargo xwin build --target x86_64-pc-windows-msvc

.PHONY: check
check:
	cargo check
