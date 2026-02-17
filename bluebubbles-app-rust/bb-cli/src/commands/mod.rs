//! CLI command implementations.

pub mod connect;
pub mod status;
pub mod chats;
pub mod messages;
pub mod contacts;
pub mod attachments;
pub mod sync;
pub mod settings;
pub mod server;
pub mod logs;
pub mod db;
pub mod findmy;
pub mod facetime;
pub mod scheduled;
pub mod backup;
pub mod private_api;
pub mod diagnose;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use bb_core::platform::Platform;
use bb_api::ApiClient;
use bb_models::Database;

/// Helper to initialize the database from config.
pub async fn init_database(config: &ConfigHandle) -> BbResult<Database> {
    let db_path = Platform::data_dir()?.join("bluebubbles.db");
    let db_config = config.read().await.database.clone();
    Database::init(&db_path, &db_config)
}

/// Helper to create an API client from config.
/// Falls back to server credentials stored in the SQLite database (from the Tauri app)
/// when the config file doesn't have a server address.
pub async fn create_api_client(config: &ConfigHandle) -> BbResult<ApiClient> {
    let mut server_config = config.read().await.server.clone();

    // If config doesn't have server address, try reading from SQLite settings
    // (the Tauri app stores credentials there via the connect command)
    if server_config.address.is_empty() {
        if let Ok(db) = init_database(config).await {
            let conn = db.conn()?;
            if let Ok(Some(addr)) = bb_models::Settings::get(&conn, bb_models::models::settings::keys::SERVER_ADDRESS) {
                if !addr.is_empty() {
                    server_config.address = addr;
                    if let Ok(Some(key)) = bb_models::Settings::get(&conn, bb_models::models::settings::keys::GUID_AUTH_KEY) {
                        server_config.guid_auth_key = key;
                    }
                    tracing::info!("using server credentials from database: {}", server_config.address);
                }
            }
        }
    }

    if server_config.address.is_empty() {
        return Err(bb_core::error::BbError::Config(
            "no server address configured. use 'bluebubbles connect <address> <password>' first, or connect via the Tauri app".into()
        ));
    }

    ApiClient::new(&server_config)
}

/// Format a byte count as a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Truncate a string to a maximum length, appending an ellipsis if truncated.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}
