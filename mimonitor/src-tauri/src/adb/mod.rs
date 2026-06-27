pub mod jni;
pub mod scanner;
pub mod guardian;

use std::net::SocketAddr;
use std::sync::Mutex;

use adb_client::tcp::ADBTcpDevice;
use adb_client::ADBDeviceExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdbError {
    #[error("ADB connection failed: {0}")]
    ConnectionFailed(String),
    #[error("ADB shell command failed: {0}")]
    ShellFailed(String),
    #[error("JNI call failed: {0}")]
    JniFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<adb_client::RustADBError> for AdbError {
    fn from(e: adb_client::RustADBError) -> Self {
        AdbError::ShellFailed(e.to_string())
    }
}

pub type AdbResult<T> = Result<T, AdbError>;

/// ADB client wrapper — uses adb_client crate (pure Rust, no external adb binary)
pub struct AdbClient {
    device: Mutex<Option<ADBTcpDevice>>,
    ip: String,
}

impl AdbClient {
    pub fn new(ip: &str) -> Self {
        Self {
            device: Mutex::new(None),
            ip: ip.to_string(),
        }
    }

    /// Connect to device via TCP (pure Rust, no adb binary needed)
    pub fn connect(&mut self, ip: &str) -> AdbResult<bool> {
        self.ip = ip.to_string();
        let addr: SocketAddr = format!("{}:5555", ip).parse()
            .map_err(|e: std::net::AddrParseError| AdbError::ConnectionFailed(e.to_string()))?;

        match ADBTcpDevice::new(addr) {
            Ok(device) => {
                log::info!("adb connected to {}", ip);
                self.device = Mutex::new(Some(device));
                Ok(true)
            }
            Err(e) => {
                log::warn!("adb connect {} failed: {}", ip, e);
                Err(AdbError::ConnectionFailed(e.to_string()))
            }
        }
    }

    /// Run a shell command on the device
    pub fn shell(&self, cmd: &str) -> AdbResult<String> {
        let mut device = self.device.lock()
            .map_err(|e| AdbError::ShellFailed(e.to_string()))?;
        let device = device.as_mut()
            .ok_or_else(|| AdbError::ConnectionFailed("Not connected".into()))?;

        let mut stdout_buf = Vec::new();
        device.shell_command(&cmd.to_string(), Some(&mut stdout_buf), None)?;

        let output = String::from_utf8_lossy(&stdout_buf).to_string();
        Ok(output)
    }

    /// Get an Android global setting
    pub fn get_setting(&self, key: &str) -> AdbResult<String> {
        let result = self.shell(&format!("settings get global {}", key))?;
        let trimmed = result.trim().to_string();
        log::debug!("settings get {} => {}", key, trimmed);
        Ok(trimmed)
    }

    /// Get multiple Android global settings in one shell call (fast batch read)
    pub fn get_settings_batch(&self, keys: &[&str]) -> AdbResult<Vec<String>> {
        let cmd: String = keys
            .iter()
            .map(|k| format!("settings get global {}", k))
            .collect::<Vec<_>>()
            .join(" && echo __SEP__ && ");
        let output = self.shell(&cmd)?;
        log::debug!("Batch raw output: {:?}", output);
        let values: Vec<String> = output
            .split("__SEP__")
            .map(|s| s.trim().to_string())
            .collect();
        log::debug!("Batch parsed values: {:?}", values);
        let mut result = values;
        result.truncate(keys.len());
        while result.len() < keys.len() {
            result.push(String::new());
        }
        Ok(result)
    }

    /// Set an Android global setting
    pub fn put_setting(&self, key: &str, value: &str) -> AdbResult<String> {
        let result = self.shell(&format!("settings put global {} {}", key, value))?;
        log::debug!("settings put global {} {} => {}", key, value, result.trim());
        Ok(result)
    }

    /// Send a key event
    pub fn send_key(&self, keycode: &str) -> AdbResult<String> {
        let result = self.shell(&format!("input keyevent {}", keycode))?;
        log::debug!("input keyevent {} => {}", keycode, result.trim());
        Ok(result)
    }

    /// Push a file to the device
    pub fn push(&self, local: &str, remote: &str) -> AdbResult<()> {
        let mut device = self.device.lock()
            .map_err(|e| AdbError::ShellFailed(e.to_string()))?;
        let device = device.as_mut()
            .ok_or_else(|| AdbError::ConnectionFailed("Not connected".into()))?;

        let mut file = std::fs::File::open(local)
            .map_err(|e| AdbError::ShellFailed(format!("Cannot open {}: {}", local, e)))?;
        device.push(&mut file, &remote.to_string())?;
        log::info!("push {} -> {}", local, remote);
        Ok(())
    }

    /// Get device model
    pub fn get_model(&self) -> AdbResult<String> {
        let result = self.shell("getprop ro.product.model")?;
        Ok(result.trim().to_string())
    }

    /// Refresh picture quality (trigger PQ re-read)
    pub fn refresh_pq(&self) -> AdbResult<()> {
        self.shell("am broadcast -a com.xiaomi.mitv.action.PIC_MODE_CHANGED --ei picmode 7")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client_has_no_device() {
        let client = AdbClient::new("192.168.5.252");
        assert!(client.device.lock().unwrap().is_none());
    }

    #[test]
    fn test_shell_without_connection_returns_error() {
        let client = AdbClient::new("192.168.5.252");
        let result = client.shell("echo hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_to_invalid_ip_fails() {
        let mut client = AdbClient::new("0.0.0.0");
        // Connect to loopback port that's not an ADB server - should fail fast
        let result = client.connect("127.0.0.1");
        assert!(result.is_err());
    }
}
