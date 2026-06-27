/// HDR detection — platform-specific implementations

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;

/// Query whether HDR is currently enabled on the primary display
pub fn is_hdr_enabled() -> Option<bool> {
    #[cfg(target_os = "windows")]
    {
        windows::query_hdr()
    }
    #[cfg(target_os = "macos")]
    {
        macos::query_hdr()
    }
    #[cfg(target_os = "linux")]
    {
        linux::query_hdr()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// HDR memory state machine
#[derive(Debug, Clone)]
pub struct HdrMemory {
    pub enabled: bool,
    pub last_state: Option<bool>,  // true = HDR, false = SDR
    pub sdr_memory: Option<i32>,
    pub hdr_memory: Option<i32>,
}
