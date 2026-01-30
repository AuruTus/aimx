.PHONY: xwin
xwin:
	cargo xwin build --release --target x86_64-pc-windows-msvc

.PHONY: xwin-debug
xwin-debug:
	cargo xwin build --target x86_64-pc-windows-msvc

.PHONY: check
check:
	cargo check
