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

/// Get all picture settings (single batch ADB call)
#[tauri::command]
pub async fn get_picture_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_adb(&state, |adb| {
        let keys = vec![
            SET_PICTURE_MODE, SET_BACKLIGHT, SET_BRIGHTNESS, SET_CONTRAST,
            SET_SATURATION, SET_HUE, SET_SHARPNESS, SET_COLOR_TEMP,
            SET_LOCAL_DIMMING, SET_DYNAMIC_DEF, SET_RESPONSE_TIME, SET_COLOR_SPACE,
        ];
        let values = adb.get_settings_batch(&keys).map_err(|e| e.to_string())?;
        let mut result = serde_json::Map::new();
        for (key, val) in keys.iter().zip(values.iter()) {
            result.insert(key.to_string(), serde_json::Value::String(val.clone()));
        }
        Ok(serde_json::Value::Object(result))
    })
    .await
}

/// Set a picture slider value (backlight, brightness, contrast, etc.)
#[tauri::command]
pub async fn set_picture_slider(
    state: State<'_, AppState>,
    key: String,
    value: i32,
) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(&key, &value.to_string()).map_err(|e| e.to_string())?;
        // Dual write for backlight
        if key == SET_BACKLIGHT {
            let _ = adb.put_setting(SET_BACKLIGHT_XIAOMI, &value.to_string());
        }
        Ok(())
    })
    .await
}

/// Set picture mode (standard/game/movie)
#[tauri::command]
pub async fn set_picture_mode(state: State<'_, AppState>, mode: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_PICTURE_MODE, &mode.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Set color temperature (with Xiaomi-to-MTK mapping)
#[tauri::command]
pub async fn set_color_temperature(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_COLOR_TEMP, &value.to_string())
            .map_err(|e| e.to_string())?;
        // JNI write with MTK mapping
        if let Some(&(_, mtk_val)) = XIAOMI_TO_MTK_COLOR_TEMP.iter().find(|(k, _)| *k == value) {
            let _ = adb::jni::jni_set(adb, JNI_CLR_TEMP, mtk_val, 3);
        }
        Ok(())
    })
    .await
}

/// Set local dimming level
#[tauri::command]
pub async fn set_local_dimming(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_LOCAL_DIMMING, &value.to_string())
            .map_err(|e| e.to_string())?;
        let _ = adb.put_setting(SET_LOCAL_DIMMING_TV, &value.to_string());
        let _ = adb::jni::jni_set(adb, JNI_LOCAL_DIMMING, value, 3);
        Ok(())
    })
    .await
}

/// Set color space/gamut
#[tauri::command]
pub async fn set_color_space(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_COLOR_SPACE, &value.to_string())
            .map_err(|e| e.to_string())?;
        let _ = adb.put_setting(SET_COLOR_SPACE_TV, &value.to_string());
        let _ = adb::jni::jni_set(adb, JNI_GAMUT, value, 3);
        Ok(())
    })
    .await
}

/// Set response time
#[tauri::command]
pub async fn set_response_time(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_RESPONSE_TIME, &value.to_string())
            .map_err(|e| e.to_string())?;
        let _ = adb::jni::jni_set(adb, JNI_RESPONSE_TIME, value, 3);
        Ok(())
    })
    .await
}

/// Set dynamic definition
#[tauri::command]
pub async fn set_dynamic_definition(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_DYNAMIC_DEF, &value.to_string())
            .map_err(|e| e.to_string())?;
        let _ = adb::jni::jni_set(adb, JNI_INSERT_BLACK, value, 3);
        Ok(())
    })
    .await
}

/// Set RGB color gains
#[tauri::command]
pub async fn set_color_gains(
    state: State<'_, AppState>,
    red: i32,
    green: i32,
    blue: i32,
) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_RED_GAIN, &red.to_string())
            .map_err(|e| e.to_string())?;
        adb.put_setting(SET_GREEN_GAIN, &green.to_string())
            .map_err(|e| e.to_string())?;
        adb.put_setting(SET_BLUE_GAIN, &blue.to_string())
            .map_err(|e| e.to_string())?;
        let _ = adb::jni::jni_set_color_gains(adb, red, green, blue);
        Ok(())
    })
    .await
}

/// Reset current picture mode to defaults
#[tauri::command]
pub async fn reset_picture_mode(state: State<'_, AppState>) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb::jni::jni_set(adb, JNI_RESET, 0, 3).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Refresh picture quality
#[tauri::command]
pub async fn refresh_pq(state: State<'_, AppState>) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.refresh_pq().map_err(|e| e.to_string())
    })
    .await
}
