//! Settings service for application configuration persistence.
//!
//! Wraps the ConfigHandle to provide typed accessors for all configuration
//! sections: server, database, sync, notifications, display, and logging.

use tracing::{info, debug};
use bb_core::config::{ConfigHandle, AppConfig};
use bb_core::error::BbResult;

use crate::service::{Service, ServiceState};

/// Service for managing application settings.
///
/// Wraps the ConfigHandle to provide a service-compatible interface
/// for reading and writing application settings with typed accessors.
/// Settings are persisted to a TOML configuration file.
pub struct SettingsService {
    state: ServiceState,
    config: ConfigHandle,
}

impl SettingsService {
    /// Create a new SettingsService.
    pub fn new(config: ConfigHandle) -> Self {
        Self {
            state: ServiceState::Created,
            config,
        }
    }

    /// Get the config handle for direct access.
    pub fn config(&self) -> &ConfigHandle {
        &self.config
    }

    // ─── Server settings ────────────────────────────────────────────────

    /// Get the server address.
    pub async fn server_address(&self) -> String {
        self.config.read().await.server.address.clone()
    }

    /// Set the server address (sanitized).
    pub async fn set_server_address(&self, address: String) {
        let mut config = self.config.write().await;
        config.server.address = AppConfig::sanitize_server_address(&address);
        debug!("server address updated");
    }

    /// Get the GUID auth key.
    pub async fn guid_auth_key(&self) -> String {
        self.config.read().await.server.guid_auth_key.clone()
    }

    /// Set the GUID auth key.
    pub async fn set_guid_auth_key(&self, key: String) {
        let mut config = self.config.write().await;
        config.server.guid_auth_key = key;
    }

    /// Get the API timeout in milliseconds.
    pub async fn api_timeout_ms(&self) -> u64 {
        self.config.read().await.server.api_timeout_ms
    }

    /// Set the API timeout in milliseconds.
    pub async fn set_api_timeout_ms(&self, ms: u64) {
        let mut config = self.config.write().await;
        config.server.api_timeout_ms = ms;
    }

    /// Whether self-signed SSL certificates are accepted.
    pub async fn accept_self_signed_certs(&self) -> bool {
        self.config.read().await.server.accept_self_signed_certs
    }

    /// Set whether to accept self-signed SSL certificates.
    pub async fn set_accept_self_signed_certs(&self, accept: bool) {
        let mut config = self.config.write().await;
        config.server.accept_self_signed_certs = accept;
    }

    /// Get custom headers.
    pub async fn custom_headers(&self) -> std::collections::HashMap<String, String> {
        self.config.read().await.server.custom_headers.clone()
    }

    /// Set a custom header.
    pub async fn set_custom_header(&self, key: String, value: String) {
        let mut config = self.config.write().await;
        config.server.custom_headers.insert(key, value);
    }

    /// Remove a custom header.
    pub async fn remove_custom_header(&self, key: &str) {
        let mut config = self.config.write().await;
        config.server.custom_headers.remove(key);
    }

    /// Whether the server connection is configured.
    pub async fn is_server_configured(&self) -> bool {
        self.config.read().await.is_server_configured()
    }

    // ─── Sync settings ──────────────────────────────────────────────────

    /// Check if initial setup has been completed.
    pub async fn is_setup_complete(&self) -> bool {
        self.config.read().await.sync.finished_setup
    }

    /// Mark initial setup as complete.
    pub async fn mark_setup_complete(&self) {
        let mut config = self.config.write().await;
        config.sync.finished_setup = true;
        info!("initial setup marked as complete");
    }

    /// Get the last incremental sync timestamp.
    pub async fn last_incremental_sync(&self) -> i64 {
        self.config.read().await.sync.last_incremental_sync
    }

    /// Get the last incremental sync ROWID.
    pub async fn last_incremental_sync_row_id(&self) -> i64 {
        self.config.read().await.sync.last_incremental_sync_row_id
    }

    /// Get the number of messages fetched per page during sync.
    pub async fn messages_per_page(&self) -> u32 {
        self.config.read().await.sync.messages_per_page
    }

    /// Set the number of messages fetched per page during sync.
    pub async fn set_messages_per_page(&self, count: u32) {
        let mut config = self.config.write().await;
        config.sync.messages_per_page = count;
    }

    /// Whether to skip empty chats during full sync.
    pub async fn skip_empty_chats(&self) -> bool {
        self.config.read().await.sync.skip_empty_chats
    }

    /// Set whether to skip empty chats during full sync.
    pub async fn set_skip_empty_chats(&self, skip: bool) {
        let mut config = self.config.write().await;
        config.sync.skip_empty_chats = skip;
    }

    /// Whether to automatically sync contacts.
    pub async fn sync_contacts_automatically(&self) -> bool {
        self.config.read().await.sync.sync_contacts_automatically
    }

    /// Set whether to automatically sync contacts.
    pub async fn set_sync_contacts_automatically(&self, auto_sync: bool) {
        let mut config = self.config.write().await;
        config.sync.sync_contacts_automatically = auto_sync;
    }

    // ─── Notification settings ──────────────────────────────────────────

    /// Whether reaction notifications are enabled.
    pub async fn notify_reactions(&self) -> bool {
        self.config.read().await.notifications.notify_reactions
    }

    /// Set whether to show reaction notifications.
    pub async fn set_notify_reactions(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.notifications.notify_reactions = enabled;
    }

    /// Whether to notify when on the chat list screen.
    pub async fn notify_on_chat_list(&self) -> bool {
        self.config.read().await.notifications.notify_on_chat_list
    }

    /// Set whether to notify when on the chat list screen.
    pub async fn set_notify_on_chat_list(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.notifications.notify_on_chat_list = enabled;
    }

    /// Whether unknown sender notifications are filtered.
    pub async fn filter_unknown_senders(&self) -> bool {
        self.config.read().await.notifications.filter_unknown_senders
    }

    /// Set whether to filter unknown sender notifications.
    pub async fn set_filter_unknown_senders(&self, filter: bool) {
        let mut config = self.config.write().await;
        config.notifications.filter_unknown_senders = filter;
    }

    /// Get the global text detection keywords.
    pub async fn global_text_detection(&self) -> String {
        self.config.read().await.notifications.global_text_detection.clone()
    }

    /// Set global text detection keywords (comma-separated).
    pub async fn set_global_text_detection(&self, keywords: String) {
        let mut config = self.config.write().await;
        config.notifications.global_text_detection = keywords;
    }

    // ─── Display settings ───────────────────────────────────────────────

    /// Get the user display name.
    pub async fn user_name(&self) -> String {
        self.config.read().await.display.user_name.clone()
    }

    /// Set the user display name.
    pub async fn set_user_name(&self, name: String) {
        let mut config = self.config.write().await;
        config.display.user_name = name;
    }

    /// Whether 24-hour time format is used.
    pub async fn use_24hr_format(&self) -> bool {
        self.config.read().await.display.use_24hr_format
    }

    /// Set whether to use 24-hour time format.
    pub async fn set_use_24hr_format(&self, use_24hr: bool) {
        let mut config = self.config.write().await;
        config.display.use_24hr_format = use_24hr;
    }

    /// Whether redacted/privacy mode is enabled.
    pub async fn redacted_mode(&self) -> bool {
        self.config.read().await.display.redacted_mode
    }

    /// Set whether to enable redacted/privacy mode.
    pub async fn set_redacted_mode(&self, redacted: bool) {
        let mut config = self.config.write().await;
        config.display.redacted_mode = redacted;
    }

    // ─── Theme settings ──────────────────────────────────────────────────

    /// Get the currently selected theme name.
    pub async fn selected_theme(&self) -> String {
        self.config.read().await.theme.selected_theme.clone()
    }

    /// Set the selected theme.
    pub async fn set_selected_theme(&self, theme: String) {
        let mut config = self.config.write().await;
        config.theme.selected_theme = theme;
    }

    /// Get the UI skin (iOS, Material, Samsung).
    pub async fn skin(&self) -> String {
        self.config.read().await.theme.skin.clone()
    }

    /// Set the UI skin.
    pub async fn set_skin(&self, skin: String) {
        let mut config = self.config.write().await;
        config.theme.skin = skin;
    }

    /// Whether colorful avatars are enabled.
    pub async fn colorful_avatars(&self) -> bool {
        self.config.read().await.theme.colorful_avatars
    }

    /// Set whether to use colorful avatars.
    pub async fn set_colorful_avatars(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.theme.colorful_avatars = enabled;
    }

    /// Whether colorful message bubbles are enabled.
    pub async fn colorful_bubbles(&self) -> bool {
        self.config.read().await.theme.colorful_bubbles
    }

    /// Set whether to use colorful bubbles.
    pub async fn set_colorful_bubbles(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.theme.colorful_bubbles = enabled;
    }

    /// Whether Monet/Material You theming is enabled.
    pub async fn monet_theming(&self) -> bool {
        self.config.read().await.theme.monet_theming
    }

    /// Set whether to use Monet theming.
    pub async fn set_monet_theming(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.theme.monet_theming = enabled;
    }

    // ─── Privacy settings ──────────────────────────────────────────────

    /// Whether incognito mode is enabled.
    pub async fn incognito_mode(&self) -> bool {
        self.config.read().await.privacy.incognito_mode
    }

    /// Set incognito mode.
    pub async fn set_incognito_mode(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.privacy.incognito_mode = enabled;
    }

    /// Whether message previews are hidden in notifications.
    pub async fn hide_message_preview(&self) -> bool {
        self.config.read().await.privacy.hide_message_preview
    }

    /// Set whether to hide message previews.
    pub async fn set_hide_message_preview(&self, hidden: bool) {
        let mut config = self.config.write().await;
        config.privacy.hide_message_preview = hidden;
    }

    /// Whether fake contact names are generated (redacted mode).
    pub async fn generate_fake_contact_names(&self) -> bool {
        self.config.read().await.privacy.generate_fake_contact_names
    }

    /// Set whether to generate fake contact names.
    pub async fn set_generate_fake_contact_names(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.privacy.generate_fake_contact_names = enabled;
    }

    /// Whether fake message content is generated (redacted mode).
    pub async fn generate_fake_message_content(&self) -> bool {
        self.config.read().await.privacy.generate_fake_message_content
    }

    /// Set whether to generate fake message content.
    pub async fn set_generate_fake_message_content(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.privacy.generate_fake_message_content = enabled;
    }

    /// Whether chats are marked as read privately.
    pub async fn private_mark_chat_as_read(&self) -> bool {
        self.config.read().await.privacy.private_mark_chat_as_read
    }

    /// Set whether to privately mark chats as read.
    pub async fn set_private_mark_chat_as_read(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.privacy.private_mark_chat_as_read = enabled;
    }

    /// Whether typing indicators are sent privately.
    pub async fn private_send_typing_indicators(&self) -> bool {
        self.config.read().await.privacy.private_send_typing_indicators
    }

    /// Set whether to send typing indicators privately.
    pub async fn set_private_send_typing_indicators(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.privacy.private_send_typing_indicators = enabled;
    }

    /// Whether the subject line input is enabled.
    pub async fn private_subject_line(&self) -> bool {
        self.config.read().await.privacy.private_subject_line
    }

    /// Set whether to enable the subject line.
    pub async fn set_private_subject_line(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.privacy.private_subject_line = enabled;
    }

    // ─── Conversation settings ─────────────────────────────────────────

    /// Whether swipable conversation tiles are enabled.
    pub async fn swipable_conversation_tiles(&self) -> bool {
        self.config.read().await.conversation.swipable_conversation_tiles
    }

    /// Set whether conversation tiles are swipable.
    pub async fn set_swipable_conversation_tiles(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.conversation.swipable_conversation_tiles = enabled;
    }

    /// Whether smart reply suggestions are enabled.
    pub async fn smart_reply(&self) -> bool {
        self.config.read().await.conversation.smart_reply
    }

    /// Set whether to use smart reply.
    pub async fn set_smart_reply(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.conversation.smart_reply = enabled;
    }

    /// Whether messages go to trash instead of being permanently deleted.
    pub async fn move_to_trash(&self) -> bool {
        self.config.read().await.conversation.move_to_trash
    }

    /// Set whether to move to trash on delete.
    pub async fn set_move_to_trash(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.conversation.move_to_trash = enabled;
    }

    /// Whether swipe-to-close is enabled for conversations.
    pub async fn swipe_to_close(&self) -> bool {
        self.config.read().await.conversation.swipe_to_close
    }

    /// Set whether swipe-to-close is enabled.
    pub async fn set_swipe_to_close(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.conversation.swipe_to_close = enabled;
    }

    /// Whether double-tap-for-details is enabled.
    pub async fn double_tap_for_details(&self) -> bool {
        self.config.read().await.conversation.double_tap_for_details
    }

    /// Set whether double-tap-for-details is enabled.
    pub async fn set_double_tap_for_details(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.conversation.double_tap_for_details = enabled;
    }

    /// Whether auto-play GIFs is enabled.
    pub async fn auto_play_gifs(&self) -> bool {
        self.config.read().await.conversation.auto_play_gifs
    }

    /// Set whether GIFs auto-play.
    pub async fn set_auto_play_gifs(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.conversation.auto_play_gifs = enabled;
    }

    // ─── Extended display settings ─────────────────────────────────────

    /// Whether dense chat tiles are enabled.
    pub async fn dense_chat_tiles(&self) -> bool {
        self.config.read().await.display.dense_chat_tiles
    }

    /// Set dense chat tiles.
    pub async fn set_dense_chat_tiles(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.display.dense_chat_tiles = enabled;
    }

    /// Whether date dividers are hidden.
    pub async fn hide_dividers(&self) -> bool {
        self.config.read().await.display.hide_dividers
    }

    /// Set whether to hide dividers.
    pub async fn set_hide_dividers(&self, hidden: bool) {
        let mut config = self.config.write().await;
        config.display.hide_dividers = hidden;
    }

    /// Get the scroll velocity multiplier.
    pub async fn scroll_velocity(&self) -> f64 {
        self.config.read().await.display.scroll_velocity
    }

    /// Set the scroll velocity multiplier.
    pub async fn set_scroll_velocity(&self, velocity: f64) {
        let mut config = self.config.write().await;
        config.display.scroll_velocity = velocity;
    }

    /// Whether delivery timestamps are shown.
    pub async fn show_delivery_timestamps(&self) -> bool {
        self.config.read().await.display.show_delivery_timestamps
    }

    /// Set whether to show delivery timestamps.
    pub async fn set_show_delivery_timestamps(&self, show: bool) {
        let mut config = self.config.write().await;
        config.display.show_delivery_timestamps = show;
    }

    /// Whether reduced force-touch is enabled.
    pub async fn reduced_force_touch(&self) -> bool {
        self.config.read().await.display.reduced_force_touch
    }

    /// Set reduced force-touch.
    pub async fn set_reduced_force_touch(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.display.reduced_force_touch = enabled;
    }

    // ─── Logging settings ───────────────────────────────────────────────

    /// Get the log level.
    pub async fn log_level(&self) -> String {
        self.config.read().await.logging.level.clone()
    }

    /// Set the log level.
    pub async fn set_log_level(&self, level: String) {
        let mut config = self.config.write().await;
        config.logging.level = level;
    }

    /// Whether JSON structured logging is enabled.
    pub async fn json_logging(&self) -> bool {
        self.config.read().await.logging.json_output
    }

    /// Set whether to use JSON structured logging.
    pub async fn set_json_logging(&self, json: bool) {
        let mut config = self.config.write().await;
        config.logging.json_output = json;
    }

    // ─── Persistence ────────────────────────────────────────────────────

    /// Save the current configuration to disk.
    pub async fn save(&self) -> BbResult<()> {
        let config = self.config.read().await;
        let path = bb_core::platform::Platform::config_dir()?.join("config.toml");
        config.save_to_file(&path)?;
        debug!("settings saved to {}", path.display());
        Ok(())
    }

    /// Export all settings as a JSON value for server backup.
    pub async fn export_as_json(&self) -> serde_json::Value {
        let config = self.config.read().await;
        serde_json::to_value(&*config).unwrap_or(serde_json::Value::Null)
    }

    /// Import settings from a JSON value (server backup restore).
    pub async fn import_from_json(&self, data: &serde_json::Value) -> BbResult<()> {
        if let Ok(config) = serde_json::from_value::<AppConfig>(data.clone()) {
            let mut current = self.config.write().await;
            // Preserve server credentials and database path
            let preserved_address = current.server.address.clone();
            let preserved_key = current.server.guid_auth_key.clone();
            let preserved_db_path = current.database.path.clone();

            *current = config;

            // Restore preserved fields
            current.server.address = preserved_address;
            current.server.guid_auth_key = preserved_key;
            current.database.path = preserved_db_path;

            info!("settings imported from backup");
        }
        Ok(())
    }
}

impl Service for SettingsService {
    fn name(&self) -> &str { "settings" }
    fn state(&self) -> ServiceState { self.state }
    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("settings service initialized");
        Ok(())
    }
    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> ConfigHandle {
        ConfigHandle::new(AppConfig::default())
    }

    #[test]
    fn test_settings_service_name() {
        let svc = SettingsService::new(make_config());
        assert_eq!(svc.name(), "settings");
    }

    #[tokio::test]
    async fn test_server_address() {
        let svc = SettingsService::new(make_config());
        assert!(svc.server_address().await.is_empty());
        svc.set_server_address("https://example.trycloudflare.com".into()).await;
        assert_eq!(
            svc.server_address().await,
            "https://example.trycloudflare.com"
        );
    }

    #[tokio::test]
    async fn test_notification_settings() {
        let svc = SettingsService::new(make_config());
        assert!(svc.notify_reactions().await);
        svc.set_notify_reactions(false).await;
        assert!(!svc.notify_reactions().await);
    }

    #[tokio::test]
    async fn test_display_settings() {
        let svc = SettingsService::new(make_config());
        assert_eq!(svc.user_name().await, "You");
        svc.set_user_name("Alice".into()).await;
        assert_eq!(svc.user_name().await, "Alice");

        assert!(!svc.use_24hr_format().await);
        svc.set_use_24hr_format(true).await;
        assert!(svc.use_24hr_format().await);
    }

    #[tokio::test]
    async fn test_sync_settings() {
        let svc = SettingsService::new(make_config());
        assert!(!svc.is_setup_complete().await);
        svc.mark_setup_complete().await;
        assert!(svc.is_setup_complete().await);

        assert_eq!(svc.messages_per_page().await, 25);
        svc.set_messages_per_page(50).await;
        assert_eq!(svc.messages_per_page().await, 50);
    }

    #[tokio::test]
    async fn test_export_as_json() {
        let svc = SettingsService::new(make_config());
        let json = svc.export_as_json().await;
        assert!(json.is_object());
        assert!(json.get("server").is_some());
    }

    #[tokio::test]
    async fn test_import_preserves_credentials() {
        let config = make_config();
        {
            let mut c = config.write().await;
            c.server.address = "https://original.com".into();
            c.server.guid_auth_key = "secret-key".into();
        }

        let svc = SettingsService::new(config);

        // Import a config that has different server address
        let import_data = serde_json::json!({
            "server": {
                "address": "https://overwritten.com",
                "guid_auth_key": "overwritten-key"
            },
            "display": {
                "user_name": "Imported User"
            }
        });

        svc.import_from_json(&import_data).await.unwrap();

        // Credentials should be preserved
        assert_eq!(svc.server_address().await, "https://original.com");
        assert_eq!(svc.guid_auth_key().await, "secret-key");
        // Other settings should be imported
        assert_eq!(svc.user_name().await, "Imported User");
    }

    #[tokio::test]
    async fn test_custom_headers() {
        let svc = SettingsService::new(make_config());
        assert!(svc.custom_headers().await.is_empty());

        svc.set_custom_header("X-Custom".into(), "value".into()).await;
        let headers = svc.custom_headers().await;
        assert_eq!(headers.get("X-Custom"), Some(&"value".to_string()));

        svc.remove_custom_header("X-Custom").await;
        assert!(svc.custom_headers().await.is_empty());
    }
}
