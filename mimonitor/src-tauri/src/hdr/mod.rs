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
    pub suppress_until: Option<std::time::Instant>,
}

impl HdrMemory {
    pub fn new() -> Self {
        Self {
            enabled: false,
            last_state: None,
            sdr_memory: None,
            hdr_memory: None,
            suppress_until: None,
        }
    }

    /// Called when HDR state is polled
    pub fn on_poll(&mut self, hdr_on: bool) -> Option<HdrAction> {
        if !self.enabled {
            return None;
        }

        // Check suppress window
        if let Some(until) = self.suppress_until {
            if std::time::Instant::now() < until {
                return None;
            }
            self.suppress_until = None;
        }

        if self.last_state != Some(hdr_on) {
            self.last_state = Some(hdr_on);
            let memory_value = if hdr_on { self.hdr_memory } else { self.sdr_memory };
            Some(HdrAction::Apply { hdr_on, memory_value })
        } else {
            None
        }
    }

    /// Remember current local dimming value
    pub fn remember(&mut self, hdr_on: bool, value: i32) {
        if hdr_on {
            self.hdr_memory = Some(value);
        } else {
            self.sdr_memory = Some(value);
        }
    }

    /// Suppress memory writes for a duration (e.g., after toggle-off hotkey)
    pub fn suppress(&mut self, duration: std::time::Duration) {
        self.suppress_until = Some(std::time::Instant::now() + duration);
    }
}

#[derive(Debug)]
pub enum HdrAction {
    Apply { hdr_on: bool, memory_value: Option<i32> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdr_memory_initial_state() {
        let hdr = HdrMemory::new();
        assert!(!hdr.enabled);
        assert!(hdr.last_state.is_none());
        assert!(hdr.sdr_memory.is_none());
        assert!(hdr.hdr_memory.is_none());
        assert!(hdr.suppress_until.is_none());
    }

    #[test]
    fn test_hdr_memory_disabled_returns_none() {
        let mut hdr = HdrMemory::new();
        hdr.enabled = false;
        assert!(hdr.on_poll(true).is_none());
        assert!(hdr.on_poll(false).is_none());
    }

    #[test]
    fn test_hdr_memory_detect_sdr_to_hdr() {
        let mut hdr = HdrMemory::new();
        hdr.enabled = true;
        hdr.hdr_memory = Some(3);

        let action = hdr.on_poll(true);
        assert!(action.is_some());
        if let Some(HdrAction::Apply { hdr_on, memory_value }) = action {
            assert!(hdr_on);
            assert_eq!(memory_value, Some(3));
        }
    }

    #[test]
    fn test_hdr_memory_detect_hdr_to_sdr() {
        let mut hdr = HdrMemory::new();
        hdr.enabled = true;
        hdr.last_state = Some(true);
        hdr.sdr_memory = Some(1);

        let action = hdr.on_poll(false);
        assert!(action.is_some());
        if let Some(HdrAction::Apply { hdr_on, memory_value }) = action {
            assert!(!hdr_on);
            assert_eq!(memory_value, Some(1));
        }
    }

    #[test]
    fn test_hdr_memory_no_change_returns_none() {
        let mut hdr = HdrMemory::new();
        hdr.enabled = true;
        hdr.last_state = Some(true);

        // Same state as last — no action
        assert!(hdr.on_poll(true).is_none());
    }

    #[test]
    fn test_hdr_memory_remember() {
        let mut hdr = HdrMemory::new();
        hdr.remember(true, 3);
        assert_eq!(hdr.hdr_memory, Some(3));
        assert!(hdr.sdr_memory.is_none());

        hdr.remember(false, 1);
        assert_eq!(hdr.sdr_memory, Some(1));
        assert_eq!(hdr.hdr_memory, Some(3));
    }

    #[test]
    fn test_hdr_memory_suppress() {
        let mut hdr = HdrMemory::new();
        hdr.enabled = true;

        // Suppress for 10 seconds
        hdr.suppress(std::time::Duration::from_secs(10));

        // During suppress, on_poll should return None
        assert!(hdr.on_poll(true).is_none());
    }

    #[test]
    fn test_hdr_memory_suppress_expires() {
        let mut hdr = HdrMemory::new();
        hdr.enabled = true;

        // Suppress for 0 seconds (expires immediately)
        hdr.suppress(std::time::Duration::from_secs(0));

        // After suppress expires, should work again
        std::thread::sleep(std::time::Duration::from_millis(10));
        let action = hdr.on_poll(true);
        assert!(action.is_some());
    }

    #[test]
    fn test_hdr_memory_multiple_transitions() {
        let mut hdr = HdrMemory::new();
        hdr.enabled = true;

        // SDR → HDR
        let action = hdr.on_poll(true);
        assert!(action.is_some());

        // HDR → HDR (no change)
        assert!(hdr.on_poll(true).is_none());

        // HDR → SDR
        let action = hdr.on_poll(false);
        assert!(action.is_some());

        // SDR → SDR (no change)
        assert!(hdr.on_poll(false).is_none());
    }

    #[test]
    fn test_hdr_memory_remember_overwrites() {
        let mut hdr = HdrMemory::new();
        hdr.remember(true, 3);
        assert_eq!(hdr.hdr_memory, Some(3));

        hdr.remember(true, 1);
        assert_eq!(hdr.hdr_memory, Some(1)); // overwritten
    }
}
