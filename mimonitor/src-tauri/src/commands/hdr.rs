use tauri::State;
use crate::state::AppState;

/// Get HDR status
#[tauri::command]
pub async fn get_hdr_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let hdr = state.hdr_memory.read().await;
    let hdr_enabled = crate::hdr::is_hdr_enabled();
    Ok(serde_json::json!({
        "hdr_on": hdr_enabled,
        "memory_enabled": hdr.enabled,
        "sdr_memory": hdr.sdr_memory,
        "hdr_memory": hdr.hdr_memory,
        "last_state": hdr.last_state,
    }))
}

/// Toggle HDR memory feature
#[tauri::command]
pub async fn set_hdr_memory_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    {
        let mut hdr = state.hdr_memory.write().await;
        hdr.enabled = enabled;
    }
    {
        let mut config = state.config.write().await;
        config.hdr_sdr_local_dimming_enabled = enabled;
        let _ = config.save();
    }
    Ok(())
}
