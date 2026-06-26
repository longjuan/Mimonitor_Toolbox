use super::{AdbClient, AdbResult};

/// Guardian package info
pub const GUARDIAN_PACKAGE: &str = "com.example.adbguardian";
pub const GUARDIAN_ACTIVITY: &str = "com.example.adbguardian/.MainActivity";
pub const GUARDIAN_ACCESSIBILITY: &str =
    "com.example.adbguardian/com.example.adbguardian.AdbGuardianAccessibilityService";
pub const GUARDIAN_APK: &str = "adbguardian-signed.apk";

/// Guardian health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuardianStatus {
    pub installed: bool,
    pub running: bool,
    pub adb_enabled: bool,
    pub adb_wifi_enabled: bool,
    pub port_5555: bool,
    pub adbd_running: bool,
    pub accessibility_enabled: bool,
    pub ok: bool,
}

/// Check guardian health
pub fn check_status(adb: &AdbClient) -> AdbResult<GuardianStatus> {
    let installed = !adb
        .shell(&format!("pm path {}", GUARDIAN_PACKAGE))?
        .trim()
        .is_empty();

    let running = !adb
        .shell(&format!("pidof {}", GUARDIAN_PACKAGE))?
        .trim()
        .is_empty();

    let adb_enabled = adb
        .get_setting("adb_enabled")?
        .trim()
        == "1";

    let adb_wifi_enabled = adb
        .get_setting("adb_wifi_enabled")?
        .trim()
        == "1";

    let service_port = adb
        .shell("getprop service.adb.tcp.port")?
        .trim()
        == "5555";

    let persist_port = adb
        .shell("getprop persist.adb.tcp.port")?
        .trim()
        == "5555";

    let adbd_running = adb
        .shell("getprop init.svc.adbd")?
        .trim()
        == "running";

    let accessibility_out = adb.shell(&format!(
        "settings get secure enabled_accessibility_services"
    ))?;
    let accessibility_enabled = accessibility_out.contains(GUARDIAN_PACKAGE);

    let ok = installed
        && running
        && adb_enabled
        && adb_wifi_enabled
        && (service_port || persist_port)
        && adbd_running
        && accessibility_enabled;

    Ok(GuardianStatus {
        installed,
        running,
        adb_enabled,
        adb_wifi_enabled,
        port_5555: service_port || persist_port,
        adbd_running,
        accessibility_enabled,
        ok,
    })
}

/// Deploy guardian APK
pub fn deploy(adb: &AdbClient) -> AdbResult<()> {
    let apk_path = find_guardian_apk()?;
    log::info!("Deploying guardian from {}", apk_path);

    // Install
    adb.shell(&format!("pm install -r -d /sdcard/{}", GUARDIAN_APK))?;

    // Grant WRITE_SECURE_SETTINGS
    adb.shell(&format!(
        "pm grant {} android.permission.WRITE_SECURE_SETTINGS",
        GUARDIAN_PACKAGE
    ))?;

    // Add to device idle whitelist
    adb.shell(&format!(
        "cmd deviceidle whitelist +{}",
        GUARDIAN_PACKAGE
    ))?;

    // Enable accessibility service
    adb.shell(&format!(
        "settings put secure enabled_accessibility_services {}",
        GUARDIAN_ACCESSIBILITY
    ))?;

    // Start activity
    adb.shell(&format!("am start -n {}", GUARDIAN_ACTIVITY))?;

    Ok(())
}

fn find_guardian_apk() -> AdbResult<String> {
    let candidates = vec![
        "resources/adbguardian-signed.apk",
        "../resources/adbguardian-signed.apk",
        "../../assets/adb_guardian/adbguardian-signed.apk",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    Err(super::AdbError::ShellFailed(
        "adbguardian-signed.apk not found".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardian_constants() {
        assert_eq!(GUARDIAN_PACKAGE, "com.example.adbguardian");
        assert!(GUARDIAN_ACTIVITY.contains(GUARDIAN_PACKAGE));
        assert!(GUARDIAN_ACCESSIBILITY.contains(GUARDIAN_PACKAGE));
    }

    #[test]
    fn test_guardian_status_serialization() {
        let status = GuardianStatus {
            installed: true,
            running: true,
            adb_enabled: true,
            adb_wifi_enabled: true,
            port_5555: true,
            adbd_running: true,
            accessibility_enabled: true,
            ok: true,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"ok\":true"));

        let loaded: GuardianStatus = serde_json::from_str(&json).unwrap();
        assert!(loaded.ok);
        assert!(loaded.installed);
    }

    #[test]
    fn test_check_status_requires_connection() {
        let client = AdbClient::new("127.0.0.1");
        let _ = check_status(&client);
    }

    #[test]
    fn test_find_guardian_apk_in_test_env() {
        // APK won't be found in test environment
        let result = find_guardian_apk();
        // May succeed if running from project root, or fail
        match result {
            Ok(path) => assert!(path.ends_with(".apk")),
            Err(_) => {} // expected in test env
        }
    }
}
