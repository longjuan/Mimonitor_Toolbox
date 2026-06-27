use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod constants;

/// Application state shared across all Tauri commands
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub adb: Arc<RwLock<Option<super::adb::AdbClient>>>,
    pub connected: Arc<RwLock<bool>>,
    pub connection_ip: Arc<RwLock<String>>,
    pub hdr_memory: Arc<RwLock<super::hdr::HdrMemory>>,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let hdr_memory = super::hdr::HdrMemory {
            enabled: config.hdr_sdr_local_dimming_enabled,
            last_state: None,
            sdr_memory: config.local_dimming_memory.sdr,
            hdr_memory: config.local_dimming_memory.hdr,
        };
        Self {
            config: Arc::new(RwLock::new(config)),
            adb: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
            connection_ip: Arc::new(RwLock::new(String::new())),
            hdr_memory: Arc::new(RwLock::new(hdr_memory)),
        }
    }
}

/// Persistent application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub saved_ip: String,
    pub close_behavior: String, // "tray" | "exit"
    pub never_ask_close: bool,
    pub theme: String, // "auto" | "dark" | "light"
    pub autostart: bool,
    pub hdr_sdr_local_dimming_enabled: bool,
    pub local_dimming_memory: LocalDimmingMemory,
    pub local_dimming_toggle_last_value: i32,
    pub hotkeys: std::collections::HashMap<String, HotkeyBinding>,
    pub adjust_hotkeys: Vec<AdjustHotkey>,
    pub log_to_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDimmingMemory {
    pub sdr: Option<i32>,
    pub hdr: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub modifier: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustHotkey {
    pub param: String,
    pub direction: String, // "increase" | "decrease"
    pub step: i32,
    pub modifier: String,
    pub key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            saved_ip: String::new(),
            close_behavior: "tray".to_string(),
            never_ask_close: false,
            theme: "auto".to_string(),
            autostart: false,
            hdr_sdr_local_dimming_enabled: false,
            local_dimming_memory: LocalDimmingMemory {
                sdr: None,
                hdr: None,
            },
            local_dimming_toggle_last_value: 3,
            hotkeys: std::collections::HashMap::new(),
            adjust_hotkeys: Vec::new(),
            log_to_file: false,
        }
    }
}

impl AppConfig {
    fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mimonitor")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    /// Load config from disk, or return defaults
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(config) => return config,
                    Err(e) => log::warn!("Failed to parse config: {}", e),
                },
                Err(e) => log::warn!("Failed to read config: {}", e),
            }
        }
        Self::default()
    }

    /// Save config to disk atomically
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;

        // Atomic write: temp file + rename
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = AppConfig::default();
        assert_eq!(config.saved_ip, "");
        assert_eq!(config.close_behavior, "tray");
        assert!(!config.never_ask_close);
        assert_eq!(config.theme, "auto");
        assert!(!config.autostart);
        assert!(!config.hdr_sdr_local_dimming_enabled);
        assert_eq!(config.local_dimming_toggle_last_value, 3);
        assert!(config.local_dimming_memory.sdr.is_none());
        assert!(config.local_dimming_memory.hdr.is_none());
        assert!(config.hotkeys.is_empty());
        assert!(config.adjust_hotkeys.is_empty());
        assert!(!config.log_to_file);
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("mimonitor_test_config");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // We can't easily test save/load since they use hardcoded paths,
        // but we can test serialization roundtrip
        let mut config = AppConfig::default();
        config.saved_ip = "192.168.5.252".to_string();
        config.close_behavior = "exit".to_string();
        config.theme = "dark".to_string();

        let json = serde_json::to_string_pretty(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.saved_ip, "192.168.5.252");
        assert_eq!(loaded.close_behavior, "exit");
        assert_eq!(loaded.theme, "dark");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_json_format() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"saved_ip\""));
        assert!(json.contains("\"close_behavior\":\"tray\""));
    }
}
