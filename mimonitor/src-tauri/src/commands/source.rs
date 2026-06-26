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

/// Get current input source
#[tauri::command]
pub async fn get_input_source(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_adb(&state, |adb| {
        let val = adb.get_setting(SET_SOURCE).unwrap_or_default();
        let source_id: i32 = val.trim().parse().unwrap_or(0);
        let name = match source_id {
            23 => "HDMI 1",
            24 => "HDMI 2",
            29 => "DP",
            30 => "USBC",
            _ => "Unknown",
        };
        Ok(serde_json::json!({
            "id": source_id,
            "name": name,
        }))
    })
    .await
}

/// Switch input source
#[tauri::command]
pub async fn set_input_source(state: State<'_, AppState>, source_id: i32) -> Result<(), String> {
    with_adb(&state, |adb| {
        // Force stop tvplayer first
        let _ = adb.shell("am force-stop com.xiaomi.mitv.tvplayer");
        // Set the source
        adb.put_setting(SET_SOURCE, &source_id.to_string())
            .map_err(|e| e.to_string())?;
        // Launch external source activity
        let _ = adb.shell(&format!(
            "am start -n com.xiaomi.mitv.tvplayer/.ExternalSourceActivity --ei input_source {}",
            source_id
        ));
        Ok(())
    })
    .await
}
