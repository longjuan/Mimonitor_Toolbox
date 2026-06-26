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

/// Send a remote key event
#[tauri::command]
pub async fn send_remote_key(state: State<'_, AppState>, key: String) -> Result<(), String> {
    let keycode = match key.as_str() {
        "power" => KEY_POWER,
        "home" => KEY_HOME,
        "menu" => KEY_MENU,
        "back" => KEY_BACK,
        "up" => KEY_DPAD_UP,
        "down" => KEY_DPAD_DOWN,
        "left" => KEY_DPAD_LEFT,
        "right" => KEY_DPAD_RIGHT,
        "ok" => KEY_DPAD_CENTER,
        "volume_up" => KEY_VOLUME_UP,
        "volume_down" => KEY_VOLUME_DOWN,
        "mute" => KEY_MUTE,
        _ => return Err(format!("Unknown key: {}", key)),
    };
    with_adb(&state, |adb| {
        adb.send_key(keycode).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}
