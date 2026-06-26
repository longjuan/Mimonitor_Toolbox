use tauri::State;
use crate::state::AppState;
use crate::state::constants::*;
use crate::adb::AdbClient;

async fn with_adb<F, R>(state: &State<'_, AppState>, f: F) -> Result<R, String>
where
    F: FnOnce(&AdbClient) -> Result<R, String>,
{
    let guard = state.adb.read().await;
    let adb = guard.as_ref().ok_or("Not connected")?;
    f(adb)
}

/// Get all game settings (single batch ADB call)
#[tauri::command]
pub async fn get_game_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_adb(&state, |adb| {
        let keys = vec![
            SET_CROSSHAIR, SET_DYNAMIC_CROSSHAIR, SET_SNIPER_SCOPE,
            SET_SCOPE_NIGHT, SET_FPS_COUNTER, SET_STOPWATCH, SET_TIMER,
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

/// Set crosshair style (0=off, 1-5=styles)
#[tauri::command]
pub async fn set_crosshair(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_CROSSHAIR, &value.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Set dynamic crosshair (0/1)
#[tauri::command]
pub async fn set_dynamic_crosshair(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_DYNAMIC_CROSSHAIR, &value.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Set sniper scope (0=off, 1=1.1x, 3=1.3x, 5=1.5x, 7=1.7x, 10=2.0x)
#[tauri::command]
pub async fn set_sniper_scope(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_SNIPER_SCOPE, &value.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Set scope night vision (0/1)
#[tauri::command]
pub async fn set_scope_night_vision(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_SCOPE_NIGHT, &value.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Set 320Hz competitive mode (0/1)
#[tauri::command]
pub async fn set_320hz_mode(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        // Need to determine current input source to choose HDMI vs DP EDID
        let source = adb.get_setting(SET_SOURCE).unwrap_or_default();
        let source_val: i32 = source.trim().parse().unwrap_or(29);
        let jni_key = if source_val == SOURCE_DP || source_val == SOURCE_USBC {
            JNI_EDID_DP
        } else {
            JNI_EDID_HDMI
        };
        let _ = crate::adb::jni::jni_set(adb, jni_key, value, 3);
        Ok(())
    })
    .await
}

/// Set FreeSync (0/1)
#[tauri::command]
pub async fn set_freesync(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        let source = adb.get_setting(SET_SOURCE).unwrap_or_default();
        let source_val: i32 = source.trim().parse().unwrap_or(29);
        let jni_key = if source_val == SOURCE_DP || source_val == SOURCE_USBC {
            JNI_FREESYNC_DP
        } else {
            JNI_FREESYNC_HDMI
        };
        let _ = crate::adb::jni::jni_set(adb, jni_key, value, 3);
        Ok(())
    })
    .await
}

/// Set FPS counter (0=off, 1=refresh rate, 2=histogram)
#[tauri::command]
pub async fn set_fps_counter(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_FPS_COUNTER, &value.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Set stopwatch (0/1)
#[tauri::command]
pub async fn set_stopwatch(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_STOPWATCH, &value.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// Set timer (0=off, 60=1min, 300=5min, 1800=30min, 3600=60min)
#[tauri::command]
pub async fn set_timer(state: State<'_, AppState>, value: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        adb.put_setting(SET_TIMER, &value.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}
