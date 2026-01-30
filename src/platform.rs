// Win32 FFI
#[cfg(target_os = "windows")]
mod win32 {
    use core::ffi::c_void;

    pub type HWND = *mut c_void;
    pub const GWL_EXSTYLE: i32 = -20;
    pub const WS_EX_LAYERED: u32 = 0x00080000;
    pub const WS_EX_TRANSPARENT: u32 = 0x00000020;
    pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    pub const LWA_COLORKEY: u32 = 0x00000001;
    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;

    unsafe extern "system" {
        pub fn GetWindowLongW(hwnd: HWND, index: i32) -> i32;
        pub fn SetWindowLongW(hwnd: HWND, index: i32, new_long: i32) -> i32;
        pub fn SetLayeredWindowAttributes(hwnd: HWND, cr_key: u32, alpha: u8, flags: u32) -> i32;
        pub fn GetSystemMetrics(index: i32) -> i32;
    }
}

/// Apply Win32 color-key transparency and click-through to the main window.
#[cfg(target_os = "windows")]
pub fn apply_overlay_style(cc: &eframe::CreationContext) {
    use raw_window_handle::HasWindowHandle;

    if let Ok(handle) = cc.window_handle()
        && let raw_window_handle::RawWindowHandle::Win32(w) = handle.as_raw()
    {
        let hwnd = w.hwnd.get() as win32::HWND;
        log::info!("applying overlay style to HWND {:?}", hwnd);
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
            win32::SetLayeredWindowAttributes(hwnd, 0x00000000, 0, win32::LWA_COLORKEY);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_overlay_style(_cc: &eframe::CreationContext) {}

/// Returns (width, height) of the primary monitor in pixels.
pub fn screen_size() -> (f32, f32) {
    #[cfg(target_os = "windows")]
    {
        let w = unsafe { win32::GetSystemMetrics(win32::SM_CXSCREEN) };
        let h = unsafe { win32::GetSystemMetrics(win32::SM_CYSCREEN) };
        if w > 0 && h > 0 {
            log::debug!("detected screen size: {w}x{h}");
            return (w as f32, h as f32);
        }
    }
    log::debug!("using fallback screen size: 1920x1080");
    (1920.0, 1080.0)
}
