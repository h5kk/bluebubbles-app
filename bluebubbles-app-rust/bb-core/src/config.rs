//! Application configuration management.
//!
//! Handles loading, saving, and accessing application configuration including
//! server URL, authentication credentials, and user preferences. Configuration
//! is persisted as TOML on disk.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::error::{BbError, BbResult};
use crate::platform::Platform;

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server connection settings.
    #[serde(default)]
    pub server: ServerConfig,

    /// Database settings.
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Logging settings.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Sync settings.
    #[serde(default)]
    pub sync: SyncConfig,

    /// Notification settings.
    #[serde(default)]
    pub notifications: NotificationConfig,

    /// UI/display settings (subset relevant to CLI/backend).
    #[serde(default)]
    pub display: DisplayConfig,

    /// Theme and appearance settings.
    #[serde(default)]
    pub theme: ThemeConfig,

    /// Privacy settings.
    #[serde(default)]
    pub privacy: PrivacyConfig,

    /// Conversation behaviour settings.
    #[serde(default)]
    pub conversation: ConversationConfig,
}

/// Server connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// BlueBubbles server URL (e.g., "https://abc123.trycloudflare.com").
    #[serde(default)]
    pub address: String,

    /// GUID authentication key (server password hash).
    #[serde(default)]
    pub guid_auth_key: String,

    /// Custom HTTP headers as key-value pairs.
    #[serde(default)]
    pub custom_headers: std::collections::HashMap<String, String>,

    /// API request timeout in milliseconds.
    #[serde(default = "default_api_timeout")]
    pub api_timeout_ms: u64,

    /// Whether to accept self-signed SSL certificates from the server.
    #[serde(default)]
    pub accept_self_signed_certs: bool,
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file. If empty, uses default location.
    #[serde(default)]
    pub path: String,

    /// Enable WAL (Write-Ahead Logging) mode. Always recommended.
    #[serde(default = "default_true")]
    pub wal_mode: bool,

    /// Maximum number of connections in the pool.
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Run integrity check on startup.
    #[serde(default = "default_true")]
    pub integrity_check_on_startup: bool,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error.
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Directory for log files. If empty, uses default location.
    #[serde(default)]
    pub directory: String,

    /// Maximum log file size in bytes before rotation.
    #[serde(default = "default_max_log_size")]
    pub max_file_size_bytes: u64,

    /// Maximum number of rotated log files to keep.
    #[serde(default = "default_max_log_files")]
    pub max_rotated_files: u32,

    /// Enable JSON structured logging output.
    #[serde(default)]
    pub json_output: bool,
}

/// Sync configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Whether initial setup/sync has been completed.
    #[serde(default)]
    pub finished_setup: bool,

    /// Timestamp of the last incremental sync (ms since epoch).
    #[serde(default)]
    pub last_incremental_sync: i64,

    /// Last synced message ROWID for row-based incremental sync.
    #[serde(default)]
    pub last_incremental_sync_row_id: i64,

    /// Number of messages to fetch per page during full sync.
    #[serde(default = "default_messages_per_page")]
    pub messages_per_page: u32,

    /// Skip chats with no messages during full sync.
    #[serde(default = "default_true")]
    pub skip_empty_chats: bool,

    /// Automatically sync contacts.
    #[serde(default)]
    pub sync_contacts_automatically: bool,
}

/// Notification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Show notifications for reactions.
    #[serde(default = "default_true")]
    pub notify_reactions: bool,

    /// Show notifications when on the chat list screen.
    #[serde(default)]
    pub notify_on_chat_list: bool,

    /// Filter notifications from unknown senders.
    #[serde(default)]
    pub filter_unknown_senders: bool,

    /// Comma-separated keywords for text-based notification detection.
    #[serde(default)]
    pub global_text_detection: String,
}

/// Display-related settings relevant to backend/CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// User's display name shown for "from me" messages.
    #[serde(default = "default_user_name")]
    pub user_name: String,

    /// Use 24-hour time format.
    #[serde(default)]
    pub use_24hr_format: bool,

    /// Enable redacted/privacy mode.
    #[serde(default)]
    pub redacted_mode: bool,

    /// Use dense chat tiles in the conversation list.
    #[serde(default)]
    pub dense_chat_tiles: bool,

    /// Hide date dividers in conversations.
    #[serde(default)]
    pub hide_dividers: bool,

    /// Scroll velocity multiplier (1.0 = default).
    #[serde(default = "default_scroll_velocity")]
    pub scroll_velocity: f64,

    /// Show delivery timestamps on messages.
    #[serde(default)]
    pub show_delivery_timestamps: bool,

    /// Reduce force-touch pressure threshold.
    #[serde(default)]
    pub reduced_force_touch: bool,
}

/// Theme and appearance settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Name of the currently selected theme.
    #[serde(default = "default_selected_theme")]
    pub selected_theme: String,

    /// UI skin (iOS, Material, Samsung).
    #[serde(default = "default_skin")]
    pub skin: String,

    /// Use colorful avatars based on contact hash.
    #[serde(default)]
    pub colorful_avatars: bool,

    /// Use colorful message bubbles.
    #[serde(default)]
    pub colorful_bubbles: bool,

    /// Enable Monet/Material You dynamic theming.
    #[serde(default)]
    pub monet_theming: bool,
}

/// Privacy settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Enable incognito mode (no history saved locally).
    #[serde(default)]
    pub incognito_mode: bool,

    /// Hide message preview in notifications.
    #[serde(default)]
    pub hide_message_preview: bool,

    /// Generate fake contact names in redacted mode.
    #[serde(default)]
    pub generate_fake_contact_names: bool,

    /// Generate fake message content in redacted mode.
    #[serde(default)]
    pub generate_fake_message_content: bool,

    /// Mark chats as read privately (server-side only, no read receipt).
    #[serde(default)]
    pub private_mark_chat_as_read: bool,

    /// Send typing indicators privately.
    #[serde(default)]
    pub private_send_typing_indicators: bool,

    /// Enable subject line input.
    #[serde(default)]
    pub private_subject_line: bool,
}

/// Conversation behaviour settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfig {
    /// Enable swipable conversation tiles.
    #[serde(default = "default_true")]
    pub swipable_conversation_tiles: bool,

    /// Enable smart reply suggestions.
    #[serde(default)]
    pub smart_reply: bool,

    /// Move messages to trash instead of permanent delete.
    #[serde(default)]
    pub move_to_trash: bool,

    /// Swipe to close conversation view.
    #[serde(default)]
    pub swipe_to_close: bool,

    /// Double tap a message for details.
    #[serde(default)]
    pub double_tap_for_details: bool,

    /// Auto-play GIF attachments.
    #[serde(default = "default_true")]
    pub auto_play_gifs: bool,

    /// Auto-open keyboard when entering a conversation.
    #[serde(default = "default_true")]
    pub auto_open_keyboard: bool,

    /// Enable swipe to reply on messages.
    #[serde(default = "default_true")]
    pub swipe_to_reply: bool,

    /// Enable swipe to archive on chat tiles.
    #[serde(default)]
    pub swipe_to_archive: bool,

    /// Move chat creator button to header bar.
    #[serde(default)]
    pub move_chat_creator_to_header: bool,
}

// Default value functions for serde

fn default_api_timeout() -> u64 {
    30_000
}

fn default_true() -> bool {
    true
}

fn default_pool_size() -> u32 {
    4
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_log_size() -> u64 {
    1_048_576 // 1 MB
}

fn default_max_log_files() -> u32 {
    5
}

fn default_messages_per_page() -> u32 {
    25
}

fn default_user_name() -> String {
    "You".to_string()
}

fn default_scroll_velocity() -> f64 {
    1.0
}

fn default_selected_theme() -> String {
    "OLED Dark".to_string()
}

fn default_skin() -> String {
    "iOS".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            sync: SyncConfig::default(),
            notifications: NotificationConfig::default(),
            display: DisplayConfig::default(),
            theme: ThemeConfig::default(),
            privacy: PrivacyConfig::default(),
            conversation: ConversationConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: String::new(),
            guid_auth_key: String::new(),
            custom_headers: std::collections::HashMap::new(),
            api_timeout_ms: default_api_timeout(),
            accept_self_signed_certs: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            wal_mode: true,
            pool_size: default_pool_size(),
            integrity_check_on_startup: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            directory: String::new(),
            max_file_size_bytes: default_max_log_size(),
            max_rotated_files: default_max_log_files(),
            json_output: false,
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            finished_setup: false,
            last_incremental_sync: 0,
            last_incremental_sync_row_id: 0,
            messages_per_page: default_messages_per_page(),
            skip_empty_chats: true,
            sync_contacts_automatically: false,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            notify_reactions: true,
            notify_on_chat_list: false,
            filter_unknown_senders: false,
            global_text_detection: String::new(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            user_name: default_user_name(),
            use_24hr_format: false,
            redacted_mode: false,
            dense_chat_tiles: false,
            hide_dividers: false,
            scroll_velocity: default_scroll_velocity(),
            show_delivery_timestamps: false,
            reduced_force_touch: false,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            selected_theme: default_selected_theme(),
            skin: default_skin(),
            colorful_avatars: false,
            colorful_bubbles: false,
            monet_theming: false,
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            incognito_mode: false,
            hide_message_preview: false,
            generate_fake_contact_names: false,
            generate_fake_message_content: false,
            private_mark_chat_as_read: false,
            private_send_typing_indicators: false,
            private_subject_line: false,
        }
    }
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            swipable_conversation_tiles: true,
            smart_reply: false,
            move_to_trash: false,
            swipe_to_close: false,
            double_tap_for_details: false,
            auto_play_gifs: true,
            auto_open_keyboard: true,
            swipe_to_reply: true,
            swipe_to_archive: false,
            move_chat_creator_to_header: false,
        }
    }
}

impl AppConfig {
    /// Load configuration from the default config file path.
    pub fn load_default() -> BbResult<Self> {
        let path = Self::default_config_path()?;
        if path.exists() {
            Self::load_from_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific file path.
    pub fn load_from_file(path: &Path) -> BbResult<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to the default config file path.
    pub fn save_default(&self) -> BbResult<()> {
        let path = Self::default_config_path()?;
        self.save_to_file(&path)
    }

    /// Save configuration to a specific file path.
    pub fn save_to_file(&self, path: &Path) -> BbResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)
            .map_err(|e| BbError::Config(format!("failed to serialize config: {e}")))?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get the default configuration file path.
    pub fn default_config_path() -> BbResult<PathBuf> {
        let data_dir = Platform::data_dir()?;
        Ok(data_dir.join("config.toml"))
    }

    /// Get the effective database path, using the configured path or the default.
    pub fn effective_db_path(&self) -> BbResult<PathBuf> {
        if self.database.path.is_empty() {
            let data_dir = Platform::data_dir()?;
            Ok(data_dir.join("bluebubbles.db"))
        } else {
            Ok(PathBuf::from(&self.database.path))
        }
    }

    /// Get the effective log directory, using the configured path or the default.
    pub fn effective_log_dir(&self) -> BbResult<PathBuf> {
        if self.logging.directory.is_empty() {
            let data_dir = Platform::data_dir()?;
            Ok(data_dir.join("logs"))
        } else {
            Ok(PathBuf::from(&self.logging.directory))
        }
    }

    /// Check whether the server connection is configured.
    pub fn is_server_configured(&self) -> bool {
        !self.server.address.is_empty() && !self.server.guid_auth_key.is_empty()
    }

    /// Sanitize and normalize a server address.
    ///
    /// Ensures the address has a scheme, strips trailing slashes,
    /// and applies https for known tunnel providers.
    pub fn sanitize_server_address(address: &str) -> String {
        let trimmed = address.trim().trim_matches('"').trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed.to_string()
        } else if trimmed.contains("ngrok.io")
            || trimmed.contains("trycloudflare.com")
            || trimmed.contains("zrok.io")
        {
            format!("https://{trimmed}")
        } else {
            format!("http://{trimmed}")
        };

        with_scheme.trim_end_matches('/').to_string()
    }
}

/// Thread-safe configuration holder for shared access across services.
#[derive(Clone)]
pub struct ConfigHandle {
    inner: Arc<RwLock<AppConfig>>,
}

impl ConfigHandle {
    /// Create a new configuration handle.
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(config)),
        }
    }

    /// Read the configuration.
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, AppConfig> {
        self.inner.read().await
    }

    /// Write/update the configuration.
    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, AppConfig> {
        self.inner.write().await
    }

    /// Save the current configuration to disk.
    pub async fn save(&self) -> BbResult<()> {
        let config = self.inner.read().await;
        config.save_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.api_timeout_ms, 30_000);
        assert!(config.database.wal_mode);
        assert_eq!(config.logging.level, "info");
        assert!(!config.is_server_configured());
    }

    #[test]
    fn test_sanitize_server_address() {
        assert_eq!(
            AppConfig::sanitize_server_address("abc123.trycloudflare.com"),
            "https://abc123.trycloudflare.com"
        );
        assert_eq!(
            AppConfig::sanitize_server_address("http://192.168.1.100:1234/"),
            "http://192.168.1.100:1234"
        );
        assert_eq!(
            AppConfig::sanitize_server_address("  \"https://example.com/\"  "),
            "https://example.com"
        );
        assert_eq!(
            AppConfig::sanitize_server_address("192.168.1.5:1234"),
            "http://192.168.1.5:1234"
        );
    }

    #[test]
    fn test_roundtrip_toml() {
        let config = AppConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.server.api_timeout_ms, config.server.api_timeout_ms);
    }
}
