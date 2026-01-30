// Win32 FFI - avoids dependency version conflicts with the `windows` crate
mod win32 {
    use core::ffi::c_void;

    pub type HWND = *mut c_void;
    pub const GWL_EXSTYLE: i32 = -20;
    pub const WS_EX_LAYERED: u32 = 0x00080000;
    pub const WS_EX_TRANSPARENT: u32 = 0x00000020;
    pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    pub const LWA_COLORKEY: u32 = 0x00000001;

    unsafe extern "system" {
        // user32.dll
        pub fn GetWindowLongW(hwnd: HWND, index: i32) -> i32;
        pub fn SetWindowLongW(hwnd: HWND, index: i32, new_long: i32) -> i32;
        pub fn SetLayeredWindowAttributes(hwnd: HWND, cr_key: u32, alpha: u8, flags: u32) -> i32;
    }
}

fn make_click_through(hwnd: win32::HWND) {
    unsafe {
        let style = win32::GetWindowLongW(hwnd, win32::GWL_EXSTYLE);
        win32::SetWindowLongW(
            hwnd,
            win32::GWL_EXSTYLE,
            style
                | win32::WS_EX_LAYERED as i32
                | win32::WS_EX_TRANSPARENT as i32
                | win32::WS_EX_TOOLWINDOW as i32,
        );
        // Use color key: make black (0x00000000) fully transparent
        win32::SetLayeredWindowAttributes(hwnd, 0x00000000, 0, win32::LWA_COLORKEY);
    }
}

pub fn apply_overlay_style(cc: &eframe::CreationContext) {
    use raw_window_handle::HasWindowHandle;

    if let Ok(handle) = cc.window_handle()
        && let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw()
    {
        make_click_through(win32.hwnd.get() as *mut core::ffi::c_void);
    }
}
