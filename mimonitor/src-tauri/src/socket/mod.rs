use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use crate::state::AppState;
use crate::adb::{self, AdbClient};

#[derive(serde::Deserialize)]
struct RpcRequest {
    method: String,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct RpcResponse {
    result: Option<serde_json::Value>,
    error: Option<String>,
    id: Option<serde_json::Value>,
}

pub async fn start_server(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    let port_file = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".mimonitor")
        .join("port");
    let _ = std::fs::create_dir_all(port_file.parent().unwrap());
    let _ = std::fs::write(&port_file, port.to_string());

    log::info!("Socket server listening on 127.0.0.1:{}", port);

    loop {
        let (stream, _addr) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, state).await {
                log::warn!("Client error: {}", e);
            }
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 { break; }

        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => RpcResponse {
                result: Some(handle_rpc(&req.method, req.params, &state).await),
                error: None,
                id: req.id,
            },
            Err(e) => RpcResponse {
                result: None,
                error: Some(e.to_string()),
                id: None,
            },
        };

        let mut resp = serde_json::to_string(&response)?;
        resp.push('\n');
        writer.write_all(resp.as_bytes()).await?;
    }
    Ok(())
}

/// Helper: get ADB client from state
async fn with_adb<F>(state: &Arc<AppState>, f: F) -> serde_json::Value
where
    F: FnOnce(&AdbClient) -> Result<serde_json::Value, String>,
{
    let guard = state.adb.read().await;
    match guard.as_ref() {
        Some(adb) => match f(adb) {
            Ok(v) => v,
            Err(e) => serde_json::json!({"error": e}),
        },
        None => serde_json::json!({"error": "Not connected"}),
    }
}

async fn handle_rpc(method: &str, params: Option<serde_json::Value>, state: &Arc<AppState>) -> serde_json::Value {
    match method {
        // === Connection ===
        "get_status" => {
            let connected = *state.connected.read().await;
            let ip = state.connection_ip.read().await.clone();
            let model = if connected {
                let guard = state.adb.read().await;
                guard.as_ref().map(|a| a.get_model().unwrap_or_default()).unwrap_or_default()
            } else { String::new() };
            serde_json::json!({"connected": connected, "ip": ip, "model": model})
        }
        "connect" => {
            let ip = params.as_ref().and_then(|p| p["ip"].as_str()).unwrap_or("");
            if ip.is_empty() { return serde_json::json!({"error": "Missing ip"}); }
            let mut guard = state.adb.write().await;
            let mut client = AdbClient::new(ip);
            match client.connect(ip) {
                Ok(true) => {
                    let model = client.get_model().unwrap_or_default();
                    *state.connected.write().await = true;
                    *state.connection_ip.write().await = ip.to_string();
                    let mut config = state.config.write().await;
                    config.saved_ip = ip.to_string();
                    let _ = config.save();
                    *guard = Some(client);
                    serde_json::json!({"success": true, "model": model})
                }
                Ok(false) => serde_json::json!({"error": "Connection failed"}),
                Err(e) => serde_json::json!({"error": e.to_string()}),
            }
        }
        "disconnect" => {
            *state.adb.write().await = None;
            *state.connected.write().await = false;
            *state.connection_ip.write().await = String::new();
            serde_json::json!({"success": true})
        }
        "scan" => {
            let subnets = adb::scanner::get_local_subnets();
            let mut all_devices = Vec::new();
            for subnet in &subnets {
                match adb::scanner::scan_network(subnet).await {
                    Ok((devices, _)) => all_devices.extend(devices),
                    Err(e) => log::warn!("Scan error: {}", e),
                }
            }
            serde_json::json!({"devices": all_devices.iter().map(|d| serde_json::json!({"ip": d.ip, "model": d.model})).collect::<Vec<_>>(), "subnets": subnets})
        }

        // === Settings ===
        "get_setting" => {
            let key = params.as_ref().and_then(|p| p["key"].as_str()).unwrap_or("").to_string();
            with_adb(state, move |adb| {
                let val = adb.get_setting(&key).map_err(|e| e.to_string())?;
                Ok(serde_json::json!({"key": key, "value": val}))
            }).await
        }
        "set_setting" => with_adb(state, |adb| {
            let key = params.as_ref().and_then(|p| p["key"].as_str()).unwrap_or("");
            let value = params.as_ref().and_then(|p| p["value"].as_str()).unwrap_or("");
            adb.put_setting(key, value).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"success": true}))
        }).await,
        "get_picture_settings" => with_adb(state, |adb| {
            use crate::state::constants::*;
            let keys = vec![SET_PICTURE_MODE, SET_BACKLIGHT, SET_BRIGHTNESS, SET_CONTRAST,
                SET_SATURATION, SET_HUE, SET_SHARPNESS, SET_COLOR_TEMP,
                SET_LOCAL_DIMMING, SET_DYNAMIC_DEF, SET_RESPONSE_TIME, SET_COLOR_SPACE];
            let values = adb.get_settings_batch(&keys).map_err(|e| e.to_string())?;
            let mut result = serde_json::Map::new();
            for (k, v) in keys.iter().zip(values.iter()) {
                result.insert(k.to_string(), serde_json::Value::String(v.clone()));
            }
            Ok(serde_json::Value::Object(result))
        }).await,
        "get_game_settings" => with_adb(state, |adb| {
            use crate::state::constants::*;
            let keys = vec![SET_CROSSHAIR, SET_DYNAMIC_CROSSHAIR, SET_SNIPER_SCOPE,
                SET_SCOPE_NIGHT, SET_FPS_COUNTER, SET_STOPWATCH, SET_TIMER];
            let values = adb.get_settings_batch(&keys).map_err(|e| e.to_string())?;
            let mut result = serde_json::Map::new();
            for (k, v) in keys.iter().zip(values.iter()) {
                result.insert(k.to_string(), serde_json::Value::String(v.clone()));
            }
            Ok(serde_json::Value::Object(result))
        }).await,
        "get_light_settings" => with_adb(state, |adb| {
            use crate::state::constants::*;
            let keys = vec![SET_LED_MODE, SET_LED_BRIGHTNESS, SET_LED_COLOR_TEMP, SET_LED_COLOR_VALUE];
            let values = adb.get_settings_batch(&keys).map_err(|e| e.to_string())?;
            let mut result = serde_json::Map::new();
            for (k, v) in keys.iter().zip(values.iter()) {
                result.insert(k.to_string(), serde_json::Value::String(v.clone()));
            }
            Ok(serde_json::Value::Object(result))
        }).await,

        // === LED ===
        "set_led" => {
            let p = params.as_ref();
            let mode = p.and_then(|p| p["mode"].as_str()).unwrap_or("");
            let brightness = p.and_then(|p| p["brightness"].as_i64()).unwrap_or(9) as i32;
            let color_temp = p.and_then(|p| p["color_temp"].as_i64()).unwrap_or(1) as i32;
            let color = p.and_then(|p| p["color"].as_i64()).unwrap_or(0) as i32;

            with_adb(state, |adb| {
                match mode {
                    "off" => { adb::jni::led_command(adb, "off", &[]).map_err(|e| e.to_string())?; let _ = adb.put_setting("atmosphere_light_switcher_pm2", "4"); }
                    "ambient" => { adb::jni::led_command(adb, "ambient", &[]).map_err(|e| e.to_string())?; let _ = adb.put_setting("atmosphere_light_switcher_pm2", "1"); }
                    "cycle" => { adb::jni::led_command(adb, "cycle", &[]).map_err(|e| e.to_string())?; let _ = adb.put_setting("atmosphere_light_switcher_pm2", "3"); }
                    "lighting" => {
                        adb::jni::led_command(adb, "lighting", &[&brightness.to_string(), &color_temp.to_string()]).map_err(|e| e.to_string())?;
                        let _ = adb.put_setting("atmosphere_light_switcher_pm2", "0");
                        let _ = adb.put_setting("atmosphere_light_illumination", &brightness.to_string());
                        let _ = adb.put_setting("atmosphere_light_color_temp", &color_temp.to_string());
                    }
                    "solid" => {
                        adb::jni::led_command(adb, "solid", &[&brightness.to_string(), &color.to_string()]).map_err(|e| e.to_string())?;
                        let _ = adb.put_setting("atmosphere_light_switcher_pm2", "2");
                        let _ = adb.put_setting("atmosphere_light_illumination", &brightness.to_string());
                        let _ = adb.put_setting("atmosphere_light_color_value", &color.to_string());
                    }
                    _ => return Err(format!("Unknown LED mode: {}", mode)),
                }
                Ok(serde_json::json!({"success": true}))
            }).await
        }

        // === Input Source ===
        "set_input_source" => with_adb(state, |adb| {
            let source = params.as_ref().and_then(|p| p["source"].as_str()).unwrap_or("");
            let source_id = match source {
                "hdmi1" => 23, "hdmi2" => 24, "dp" => 29, "usbc" => 30,
                _ => return Err(format!("Unknown source: {}", source)),
            };
            let _ = adb.shell("am force-stop com.xiaomi.mitv.tvplayer");
            adb.put_setting("mitv.tvplayer.hdmi.last.source", &source_id.to_string()).map_err(|e| e.to_string())?;
            let _ = adb.shell(&format!("am start -n com.xiaomi.mitv.tvplayer/.ExternalSourceActivity --ei input_source {}", source_id));
            Ok(serde_json::json!({"success": true, "source": source}))
        }).await,

        // === Remote ===
        "send_key" => with_adb(state, |adb| {
            let key = params.as_ref().and_then(|p| p["key"].as_str()).unwrap_or("");
            let keycode = match key {
                "power" => "KEYCODE_POWER", "home" => "KEYCODE_HOME", "menu" => "KEYCODE_MENU",
                "back" => "KEYCODE_BACK", "up" => "KEYCODE_DPAD_UP", "down" => "KEYCODE_DPAD_DOWN",
                "left" => "KEYCODE_DPAD_LEFT", "right" => "KEYCODE_DPAD_RIGHT", "ok" => "KEYCODE_DPAD_CENTER",
                "volume_up" => "KEYCODE_VOLUME_UP", "volume_down" => "KEYCODE_VOLUME_DOWN", "mute" => "KEYCODE_VOLUME_MUTE",
                _ => return Err(format!("Unknown key: {}", key)),
            };
            adb.send_key(keycode).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"success": true}))
        }).await,

        _ => serde_json::json!({"error": format!("Unknown method: {}", method)}),
    }
}
