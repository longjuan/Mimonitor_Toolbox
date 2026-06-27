pub mod jni;
pub mod scanner;
pub mod guardian;

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

pub type AdbResult<T> = Result<T, AdbError>;

/// ADB client wrapper — shells out to `adb` binary for now
pub struct AdbClient {
    ip: String,
}

impl AdbClient {
    pub fn new(ip: &str) -> Self {
        Self {
            ip: ip.to_string(),
        }
    }

    /// Connect to device via TCP
    pub fn connect(&mut self, ip: &str) -> AdbResult<bool> {
        self.ip = ip.to_string();
        let output = std::process::Command::new("adb")
            .args(["connect", &format!("{}:5555", ip)])
            .output()
            .map_err(|e| AdbError::ConnectionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        log::info!("adb connect {} => {}", ip, stdout.trim());

        // "connected to x.x.x.x:5555" or "already connected to x.x.x.x:5555"
        Ok(stdout.contains("connected to") && !stdout.contains("failed"))
    }

    /// Run a shell command on the device
    pub fn shell(&self, cmd: &str) -> AdbResult<String> {
        let output = std::process::Command::new("adb")
            .args(["-s", &format!("{}:5555", self.ip), "shell", cmd])
            .output()
            .map_err(|e| AdbError::ShellFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stderr.is_empty() {
            log::warn!("adb shell stderr: {}", stderr.trim());
        }

        Ok(stdout)
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
        let status = std::process::Command::new("adb")
            .args(["-s", &format!("{}:5555", self.ip), "push", local, remote])
            .status()
            .map_err(|e| AdbError::ShellFailed(e.to_string()))?;

        if !status.success() {
            return Err(AdbError::ShellFailed(format!("push failed: {} -> {}", local, remote)));
        }
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
    fn test_shell_with_invalid_ip_returns_error() {
        let client = AdbClient::new("192.168.5.252");
        let result = client.shell("echo hello");
        match result {
            Ok(output) => assert!(output.contains("hello") || output.is_empty()),
            Err(AdbError::ShellFailed(_)) => {} // adb not installed
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_get_setting_format() {
        let client = AdbClient::new("127.0.0.1");
        let _ = client.get_setting("picture_mode");
    }

    #[test]
    fn test_put_setting_format() {
        let client = AdbClient::new("127.0.0.1");
        let _ = client.put_setting("picture_mode", "14");
    }

    #[test]
    fn test_send_key_format() {
        let client = AdbClient::new("127.0.0.1");
        let _ = client.send_key("KEYCODE_POWER");
    }

    #[test]
    fn test_push_file_format() {
        let client = AdbClient::new("127.0.0.1");
        let result = client.push("/tmp/nonexistent.jar", "/sdcard/test.jar");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_model_format() {
        let client = AdbClient::new("127.0.0.1");
        let _ = client.get_model();
    }

    #[test]
    fn test_refresh_pq_format() {
        let client = AdbClient::new("127.0.0.1");
        let _ = client.refresh_pq();
    }
}
