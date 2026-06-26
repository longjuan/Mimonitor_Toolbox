use tauri::State;
use crate::state::AppState;

/// Get app config
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config = state.config.read().await;
    serde_json::to_value(&*config).map_err(|e| e.to_string())
}

/// Update app config
#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    updates: serde_json::Value,
) -> Result<(), String> {
    let mut config = state.config.write().await;

    // Merge updates into config
    if let Some(obj) = updates.as_object() {
        let mut config_json = serde_json::to_value(&*config).map_err(|e| e.to_string())?;
        if let Some(config_obj) = config_json.as_object_mut() {
            for (key, value) in obj {
                config_obj.insert(key.clone(), value.clone());
            }
        }
        *config = serde_json::from_value(config_json).map_err(|e| e.to_string())?;
    }

    config.save()?;
    Ok(())
}

/// Open ADB shell in external terminal
#[tauri::command]
pub async fn open_adb_shell(state: State<'_, AppState>) -> Result<(), String> {
    let ip = state.connection_ip.read().await;
    if ip.is_empty() {
        return Err("Not connected".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!("tell application \"Terminal\" to do script \"adb shell -s {}:5555\"", ip);
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/k", &format!("adb shell -s {}:5555", ip)])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let terminals = ["x-terminal-emulator", "gnome-terminal", "konsole", "xfce4-terminal"];
        for term in &terminals {
            if std::process::Command::new(term)
                .args(["-e", &format!("adb shell -s {}:5555", ip)])
                .spawn()
                .is_ok()
            {
                break;
            }
        }
    }

    Ok(())
}

/// Install APK file
#[tauri::command]
pub async fn install_apk(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let guard = state.adb.read().await;
    let adb = guard.as_ref().ok_or("Not connected")?;

    adb.push(&path, "/sdcard/temp_install.apk")
        .map_err(|e| e.to_string())?;
    let output = adb
        .shell("pm install -r -d /sdcard/temp_install.apk")
        .map_err(|e| e.to_string())?;
    Ok(output.trim().to_string())
}
