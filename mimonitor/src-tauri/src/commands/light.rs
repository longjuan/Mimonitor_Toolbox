use tauri::State;
use crate::state::AppState;
use crate::state::constants::*;
use crate::adb::{self, AdbClient};

async fn with_adb<F, R>(state: &State<'_, AppState>, f: F) -> Result<R, String>
where
    F: FnOnce(&AdbClient) -> Result<R, String>,
{
    let guard = state.adb.read().await;
    let adb = guard.as_ref().ok_or("Not connected")?;
    f(adb)
}

/// Get all light settings (single batch ADB call)
#[tauri::command]
pub async fn get_light_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_adb(&state, |adb| {
        let keys = vec![
            SET_LED_MODE, SET_LED_BRIGHTNESS, SET_LED_COLOR_TEMP, SET_LED_COLOR_VALUE,
        ];
        let mut result = serde_json::Map::new();
        for key in &keys {
            let val = adb.get_setting(key).unwrap_or_default();
            result.insert(key.to_string(), serde_json::Value::String(val));
        }
        log::info!("get_light_settings: {:?}", result);
        Ok(serde_json::Value::Object(result))
    })
    .await
}

/// Set LED mode — sends HIDL command AND updates Android setting
#[tauri::command]
pub async fn set_led_mode(state: State<'_, AppState>, mode: String) -> Result<(), String> {
    log::info!("set_led_mode: {}", mode);
    with_adb(&state, |adb| {
        adb::jni::led_command(adb, &mode, &[]).map_err(|e| e.to_string())?;
        let setting_val = match mode.as_str() {
            "off" => "4",
            "lighting" => "0",
            "solid" => "2",
            "ambient" => "1",
            "cycle" => "3",
            _ => return Ok(()),
        };
        let _ = adb.put_setting(SET_LED_MODE, setting_val);
        Ok(())
    })
    .await
}

/// Set LED lighting mode — sends HIDL command AND updates Android settings
#[tauri::command]
pub async fn set_led_lighting(
    state: State<'_, AppState>,
    brightness: i32,
    color_temp: i32,
) -> Result<(), String> {
    log::info!("set_led_lighting: brightness={}, color_temp={}", brightness, color_temp);
    with_adb(&state, |adb| {
        adb::jni::led_command(adb, "lighting", &[&brightness.to_string(), &color_temp.to_string()])
            .map_err(|e| e.to_string())?;
        let _ = adb.put_setting(SET_LED_MODE, "0");
        let _ = adb.put_setting(SET_LED_BRIGHTNESS, &brightness.to_string());
        let _ = adb.put_setting(SET_LED_COLOR_TEMP, &color_temp.to_string());
        Ok(())
    })
    .await
}

/// Set LED solid mode — sends HIDL command AND updates Android settings
#[tauri::command]
pub async fn set_led_solid(
    state: State<'_, AppState>,
    brightness: i32,
    color: i32,
) -> Result<(), String> {
    log::info!("set_led_solid: brightness={}, color={}", brightness, color);
    with_adb(&state, |adb| {
        adb::jni::led_command(adb, "solid", &[&brightness.to_string(), &color.to_string()])
            .map_err(|e| e.to_string())?;
        let _ = adb.put_setting(SET_LED_MODE, "2");
        let _ = adb.put_setting(SET_LED_BRIGHTNESS, &brightness.to_string());
        let _ = adb.put_setting(SET_LED_COLOR_VALUE, &color.to_string());
        Ok(())
    })
    .await
}
