use tauri::State;
use crate::state::AppState;
use crate::adb::AdbClient;

/// Connect to a monitor by IP
#[tauri::command]
pub async fn connect(ip: String, state: State<'_, AppState>) -> Result<bool, String> {
    let mut adb_guard = state.adb.write().await;
    let mut client = AdbClient::new(&ip);
    match client.connect(&ip) {
        Ok(_) => {
            let model = client.get_model().unwrap_or_else(|_| "unknown".to_string());
            log::info!("Connected to {} ({})", ip, model);
            *state.connected.write().await = true;
            *state.connection_ip.write().await = ip.clone();
            // Save IP
            {
                let mut config = state.config.write().await;
                config.saved_ip = ip;
                let _ = config.save();
            }
            *adb_guard = Some(client);
            Ok(true)
        }
        Err(e) => {
            log::error!("Connection failed: {}", e);
            Err(e.to_string())
        }
    }
}

/// Disconnect from the monitor
#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    *state.adb.write().await = None;
    *state.connected.write().await = false;
    *state.connection_ip.write().await = String::new();
    log::info!("Disconnected");
    Ok(())
}

/// Get connection status
#[tauri::command]
pub async fn get_connection_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let connected = *state.connected.read().await;
    let ip = state.connection_ip.read().await.clone();
    let model = if connected {
        let adb = state.adb.read().await;
        if let Some(ref adb) = *adb {
            adb.get_model().unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        }
    } else {
        String::new()
    };
    Ok(serde_json::json!({
        "connected": connected,
        "ip": ip,
        "model": model,
    }))
}

/// Scan local network for devices
#[tauri::command]
pub async fn scan_network(_state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    use crate::adb::scanner;
    let (devices, subnets) = scanner::scan_network("").await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "devices": devices.iter().map(|d| serde_json::json!({"ip": d.ip, "model": d.model})).collect::<Vec<_>>(),
        "subnets": subnets,
    }))
}
