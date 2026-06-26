use tauri::State;
use crate::state::AppState;
use crate::adb;

/// Get guardian status
#[tauri::command]
pub async fn get_guardian_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let guard = state.adb.read().await;
    let adb = guard.as_ref().ok_or("Not connected")?;
    let status = adb::guardian::check_status(adb).map_err(|e| e.to_string())?;
    serde_json::to_value(&status).map_err(|e| e.to_string())
}

/// Deploy guardian APK
#[tauri::command]
pub async fn deploy_guardian(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.adb.read().await;
    let adb = guard.as_ref().ok_or("Not connected")?;
    adb::guardian::deploy(adb).map_err(|e| e.to_string())
}
