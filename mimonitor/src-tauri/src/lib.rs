use std::sync::Arc;
use tauri::Manager;

mod adb;
mod state;
mod commands;
mod socket;
mod tray;
mod hdr;

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(state::AppState::new())
        .setup(|app| {
            // Start socket server in background
            let socket_state = app.state::<state::AppState>();
            let adb = socket_state.adb.clone();
            let connected = socket_state.connected.clone();
            let connection_ip = socket_state.connection_ip.clone();
            let config = socket_state.config.clone();
            let hdr_memory = socket_state.hdr_memory.clone();
            let socket_app_state = Arc::new(state::AppState {
                config,
                adb,
                connected,
                connection_ip,
                hdr_memory,
            });
            tauri::async_runtime::spawn(async move {
                if let Err(e) = socket::start_server(socket_app_state).await {
                    log::error!("Socket server error: {}", e);
                }
            });

            // Setup system tray
            tray::setup_tray(app.handle()).expect("Failed to setup system tray");

            // Auto-connect if saved IP exists
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(900)).await;
                let state = handle.try_state::<state::AppState>();
                if let Some(state) = state {
                    let config = state.config.read().await;
                    let saved_ip = config.saved_ip.clone();
                    drop(config);

                    if !saved_ip.is_empty() {
                        log::info!("Auto-connecting to {}", saved_ip);
                        let mut adb_guard = state.adb.write().await;
                        let mut client = adb::AdbClient::new(&saved_ip);
                        match client.connect(&saved_ip) {
                            Ok(_) => {
                                match client.get_model() {
                                    Ok(model) => log::info!("Connected to {} ({})", saved_ip, model),
                                    Err(_) => log::info!("Connected to {}", saved_ip),
                                }
                                *state.connected.write().await = true;
                                *state.connection_ip.write().await = saved_ip;
                                *adb_guard = Some(client);
                            }
                            Err(e) => log::warn!("Auto-connect failed: {}", e),
                        }
                    }
                }
            });

            // ADB keepalive timer (every 15 seconds)
            let keepalive_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
                interval.tick().await;
                loop {
                    interval.tick().await;
                    let state = keepalive_handle.try_state::<state::AppState>();
                    if let Some(state) = state {
                        let connected = *state.connected.read().await;
                        if connected {
                            let guard = state.adb.read().await;
                            if let Some(ref adb) = *guard {
                                match adb.shell("echo ok") {
                                    Ok(output) => {
                                        if !output.trim().contains("ok") {
                                            log::warn!("ADB device not responding, attempting reconnect");
                                            drop(guard);
                                            let ip = state.connection_ip.read().await.clone();
                                            let mut adb_guard = state.adb.write().await;
                                            if let Some(ref mut adb) = *adb_guard {
                                                if let Err(e) = adb.connect(&ip) {
                                                    log::error!("Reconnect failed: {}", e);
                                                    *state.connected.write().await = false;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("Keepalive check failed: {}", e);
                                        drop(guard);
                                        let ip = state.connection_ip.read().await.clone();
                                        let mut adb_guard = state.adb.write().await;
                                        if let Some(ref mut adb) = *adb_guard {
                                            if let Err(e) = adb.connect(&ip) {
                                                log::error!("Reconnect failed: {}", e);
                                                *state.connected.write().await = false;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // Handle window close event — minimize to tray instead of exit
            let close_handle = app.handle().clone();
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let state = close_handle.try_state::<state::AppState>();
                        if let Some(state) = state {
                            let config = state.config.blocking_read();
                            if config.close_behavior == "tray" {
                                api.prevent_close();
                                if let Some(w) = close_handle.get_webview_window("main") {
                                    let _ = w.hide();
                                }
                            }
                        }
                    }
                });
            }

            // Handle --minimized flag
            let args: Vec<String> = std::env::args().collect();
            if args.contains(&"--minimized".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connection
            commands::connection::connect,
            commands::connection::disconnect,
            commands::connection::get_connection_status,
            commands::connection::scan_network,
            // Picture
            commands::picture::get_picture_settings,
            commands::picture::set_picture_slider,
            commands::picture::set_picture_mode,
            commands::picture::set_color_temperature,
            commands::picture::set_local_dimming,
            commands::picture::set_color_space,
            commands::picture::set_response_time,
            commands::picture::set_dynamic_definition,
            commands::picture::set_hdr_tone_mapping,
            commands::picture::set_color_gains,
            commands::picture::reset_picture_mode,
            commands::picture::refresh_pq,
            // Game
            commands::game::get_game_settings,
            commands::game::set_crosshair,
            commands::game::set_dynamic_crosshair,
            commands::game::set_sniper_scope,
            commands::game::set_scope_night_vision,
            commands::game::set_320hz_mode,
            commands::game::set_freesync,
            commands::game::set_fps_counter,
            commands::game::set_stopwatch,
            commands::game::set_timer,
            // Source
            commands::source::get_input_source,
            commands::source::set_input_source,
            // Light
            commands::light::get_light_settings,
            commands::light::set_led_mode,
            commands::light::set_led_lighting,
            commands::light::set_led_solid,
            // Remote
            commands::remote::send_remote_key,
            // Settings
            commands::settings::get_config,
            commands::settings::update_config,
            commands::settings::open_adb_shell,
            commands::settings::install_apk,
            // Guardian
            commands::guardian::get_guardian_status,
            commands::guardian::deploy_guardian,
            // HDR
            commands::hdr::get_hdr_status,
            commands::hdr::set_hdr_memory_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
