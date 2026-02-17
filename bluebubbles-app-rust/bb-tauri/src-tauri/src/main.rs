//! BlueBubbles Tauri desktop application entry point.
//!
//! Initializes core infrastructure (config, database, socket dispatcher),
//! registers services, builds the Tauri app with IPC commands, and sets up
//! the system tray.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod menu;

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
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::try_auto_connect,
            commands::get_server_info,
            commands::get_chats,
            commands::refresh_chats,
            commands::mark_chat_read,
            commands::get_messages,
            commands::send_message,
            commands::search_messages,
            commands::get_contacts,
            commands::get_contact_avatar,
            commands::get_all_contact_avatars,
            commands::sync_contact_avatars,
            commands::download_attachment,
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
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running BlueBubbles");
}
