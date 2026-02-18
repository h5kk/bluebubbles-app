//! BlueBubbles Tauri desktop application entry point.
//!
//! Initializes core infrastructure (config, database, socket dispatcher),
//! registers services, builds the Tauri app with IPC commands, and sets up
//! the system tray.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod menu;
mod otp_detector;
mod mcp_auth;
mod mcp_tools;
mod mcp_server;
mod mcp_state;

use std::path::PathBuf;
use tracing::info;

use bb_core::config::{AppConfig, ConfigHandle};
use bb_models::Database;
use bb_socket::EventDispatcher;

use tauri::Manager;
use state::AppState;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("bb_tauri=debug,bb_services=info,bb_api=info,bb_socket=info")
        .init();

    info!("starting BlueBubbles desktop application");

    // Determine data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bluebubbles");

    std::fs::create_dir_all(&data_dir).expect("failed to create data directory");

    // Load or create config
    let config = AppConfig::default();
    let config_handle = ConfigHandle::new(config);

    // Initialize database
    let db_path = data_dir.join("bluebubbles.db");
    let db_config = bb_core::config::DatabaseConfig::default();
    let database = Database::init(&db_path, &db_config)
        .expect("failed to initialize database");

    info!("database initialized at {}", db_path.display());

    // Create event dispatcher for socket events
    let dispatcher = EventDispatcher::new(256);

    // Build shared application state
    let app_state = AppState::new(config_handle, database, dispatcher);

    // Build and run the Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .manage(mcp_state::McpState::new())
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::try_auto_connect,
            commands::get_server_info,
            commands::detect_localhost,
            commands::get_chats,
            commands::refresh_chats,
            commands::mark_chat_read,
            commands::mark_chat_unread,
            commands::update_chat,
            commands::get_messages,
            commands::send_message,
            commands::search_messages,
            commands::get_contacts,
            commands::get_contact_avatar,
            commands::get_all_contact_avatars,
            commands::sync_contact_avatars,
            commands::pick_attachment_file,
            commands::send_attachment_message,
            commands::send_attachment_data,
            commands::download_attachment,
            commands::get_message_reactions,
            commands::get_settings,
            commands::update_setting,
            commands::sync_full,
            commands::sync_messages,
            commands::check_messages_synced,
            commands::get_themes,
            commands::set_theme,
            commands::check_setup_complete,
            commands::complete_setup,
            commands::check_private_api_status,
            commands::send_typing_indicator,
            commands::send_reaction,
            commands::edit_message,
            commands::unsend_message,
            commands::get_findmy_devices,
            commands::refresh_findmy_devices,
            commands::get_findmy_friends,
            commands::refresh_findmy_friends,
            commands::detect_otp_in_message,
            commands::detect_otp_in_text,
            commands::create_scheduled_message,
            commands::get_scheduled_messages,
            commands::delete_scheduled_message,
            commands::start_mcp_server,
            commands::stop_mcp_server,
            commands::get_mcp_status,
            commands::regenerate_mcp_token,
        ])
        .setup(|app| {
            // Setup system tray
            menu::setup_tray(app.handle())?;

            // Initialize services in the background
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                if let Err(e) = state.init_services().await {
                    tracing::error!("failed to initialize services: {e}");
                }
                info!("services initialized");

                // Auto-start MCP server if enabled in settings
                if let Ok(conn) = state.database.conn() {
                    use bb_models::Settings;
                    let mcp_enabled = Settings::get(&conn, "mcp_server_enabled")
                        .ok()
                        .flatten()
                        .map(|v| v == "true")
                        .unwrap_or(false);

                    if mcp_enabled {
                        let port = Settings::get(&conn, "mcp_server_port")
                            .ok()
                            .flatten()
                            .and_then(|p| p.parse::<u16>().ok())
                            .unwrap_or(11111);

                        let token = Settings::get(&conn, "mcp_server_token")
                            .ok()
                            .flatten();

                        // Start the MCP server
                        let mcp_state = handle.state::<mcp_state::McpState>();
                        let auth = match token {
                            Some(t) if !t.is_empty() => mcp_auth::McpAuth::with_token(t),
                            _ => mcp_auth::McpAuth::new(),
                        };

                        let current_token = auth.current_token().await;
                        let auth = std::sync::Arc::new(auth);
                        let active_connections = std::sync::Arc::new(
                            std::sync::atomic::AtomicU32::new(0),
                        );
                        let (shutdown_tx, shutdown_rx) =
                            tokio::sync::watch::channel(false);

                        let proxy_state = std::sync::Arc::new(AppState {
                            registry: state.registry.clone(),
                            config: state.config.clone(),
                            database: state.database.clone(),
                            socket_manager: state.socket_manager.clone(),
                            setup_complete: state.setup_complete.clone(),
                        });

                        let auth_clone = auth.clone();
                        let connections_clone = active_connections.clone();

                        let server_handle = tokio::spawn(async move {
                            if let Err(e) = mcp_server::start(
                                auth_clone,
                                proxy_state,
                                port,
                                connections_clone,
                                shutdown_rx,
                            )
                            .await
                            {
                                tracing::warn!("mcp server auto-start error: {e}");
                            }
                        });

                        let mcp_server_instance = mcp_state::McpServer {
                            auth: mcp_auth::McpAuth::with_token(current_token.clone()),
                            port,
                            shutdown_tx,
                            active_connections,
                            server_handle: Some(server_handle),
                        };

                        let mut guard = mcp_state.server.write().await;
                        *guard = Some(mcp_server_instance);

                        // Persist the token if it was newly generated
                        let _ = Settings::set(&conn, "mcp_server_token", &current_token);

                        info!("mcp server auto-started on port {port}");
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running BlueBubbles");
}
