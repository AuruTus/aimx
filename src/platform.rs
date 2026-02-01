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

/// Shared HWND storage so the tray poller thread can show the window directly via Win32.
static PANEL_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Store the panel window HWND for later use by `show_panel_window`.
pub fn store_panel_hwnd(hwnd: isize) {
    PANEL_HWND.store(hwnd, std::sync::atomic::Ordering::SeqCst);
    log::info!("stored panel HWND: {:#x}", hwnd);
}

/// Force-show the panel window via Win32 ShowWindow/SetForegroundWindow.
/// Works even when eframe considers the window hidden.
#[cfg(target_os = "windows")]
pub fn show_panel_window() {
    let hwnd = PANEL_HWND.load(std::sync::atomic::Ordering::SeqCst);
    if hwnd != 0 {
        const SW_SHOW: i32 = 5;
        unsafe extern "system" {
            fn ShowWindow(hwnd: *mut core::ffi::c_void, cmd: i32) -> i32;
            fn SetForegroundWindow(hwnd: *mut core::ffi::c_void) -> i32;
        }
        unsafe {
            ShowWindow(hwnd as *mut core::ffi::c_void, SW_SHOW);
            SetForegroundWindow(hwnd as *mut core::ffi::c_void);
        }
        log::info!("show_panel_window: showed HWND {:#x}", hwnd);
    } else {
        log::warn!("show_panel_window: no HWND stored");
    }
}

#[cfg(not(target_os = "windows"))]
pub fn show_panel_window() {}

/// A Win32 Job Object that kills all assigned processes when dropped.
/// Ensures child processes (e.g. overlay) are terminated when the panel exits.
#[cfg(target_os = "windows")]
pub struct JobObject {
    handle: *mut core::ffi::c_void,
}

#[cfg(target_os = "windows")]
unsafe impl Send for JobObject {}

#[cfg(target_os = "windows")]
impl JobObject {
    pub fn new() -> Option<Self> {
        use core::ffi::c_void;

        #[repr(C)]
        struct JobObjectExtendedLimitInformation {
            basic: [u8; 48], // JOBOBJECT_BASIC_LIMIT_INFORMATION
            io_info: [u8; 48],
            process_memory_limit: usize,
            job_memory_limit: usize,
            peak_process_memory_used: usize,
            peak_job_memory_used: usize,
        }

        const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x2000;

        unsafe extern "system" {
            fn CreateJobObjectW(attrs: *mut c_void, name: *const u16) -> *mut c_void;
            fn SetInformationJobObject(
                job: *mut c_void,
                class: u32,
                info: *const c_void,
                len: u32,
            ) -> i32;
        }

        unsafe {
            let handle = CreateJobObjectW(core::ptr::null_mut(), core::ptr::null());
            if handle.is_null() {
                log::warn!("failed to create job object");
                return None;
            }

            let mut info = core::mem::zeroed::<JobObjectExtendedLimitInformation>();
            // LimitFlags is at offset 20 in the basic struct (5th u32)
            let limit_flags_ptr = info.basic.as_mut_ptr().add(20) as *mut u32;
            *limit_flags_ptr = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: u32 = 9;
            let ret = SetInformationJobObject(
                handle,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                &info as *const _ as *const c_void,
                core::mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
            );
            if ret == 0 {
                log::warn!("failed to set job object limits");
            }

            log::info!("created job object: {:?}", handle);
            Some(Self { handle })
        }
    }

    pub fn assign(&self, child: &std::process::Child) {
        use core::ffi::c_void;

        unsafe extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
            fn AssignProcessToJobObject(job: *mut c_void, process: *mut c_void) -> i32;
            fn CloseHandle(handle: *mut c_void) -> i32;
        }

        const PROCESS_ALL_ACCESS: u32 = 0x1FFFFF;

        unsafe {
            let proc_handle = OpenProcess(PROCESS_ALL_ACCESS, 0, child.id());
            if proc_handle.is_null() {
                log::warn!("failed to open child process for job assignment");
                return;
            }
            let ret = AssignProcessToJobObject(self.handle, proc_handle);
            if ret == 0 {
                log::warn!("failed to assign child to job object");
            } else {
                log::info!("assigned child pid {} to job object", child.id());
            }
            CloseHandle(proc_handle);
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for JobObject {
    fn drop(&mut self) {
        unsafe extern "system" {
            fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
        }
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

/// Set the Application User Model ID so both panel and overlay
/// are grouped together in the Windows taskbar / task manager.
#[cfg(target_os = "windows")]
pub fn set_app_user_model_id() {
    unsafe extern "system" {
        fn SetCurrentProcessExplicitAppUserModelID(app_id: *const u16) -> i32;
    }
    // "AIMX" encoded as null-terminated UTF-16
    let id: Vec<u16> = "AIMX".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
    }
    log::info!("set AppUserModelID to AIMX");
}

#[cfg(not(target_os = "windows"))]
pub fn set_app_user_model_id() {}

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
