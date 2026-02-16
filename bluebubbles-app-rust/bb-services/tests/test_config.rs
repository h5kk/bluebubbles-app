//! Integration tests for application configuration.
//!
//! Tests config loading from TOML, saving and reloading, default values,
//! URL sanitization, ConfigHandle thread-safe access, and platform detection.

mod common;

use bb_core::config::{AppConfig, ConfigHandle};
use bb_core::platform::Platform;
use tempfile::TempDir;

// ---- Default values ----

#[test]
fn default_config_has_expected_server_defaults() {
    let config = AppConfig::default();
    assert!(config.server.address.is_empty(), "default address should be empty");
    assert!(config.server.guid_auth_key.is_empty(), "default auth key should be empty");
    assert_eq!(config.server.api_timeout_ms, 30_000);
    assert!(!config.server.accept_self_signed_certs);
    assert!(config.server.custom_headers.is_empty());
}

#[test]
fn default_config_has_expected_database_defaults() {
    let config = AppConfig::default();
    assert!(config.database.path.is_empty(), "default db path should be empty");
    assert!(config.database.wal_mode, "WAL mode should be enabled by default");
    assert_eq!(config.database.pool_size, 4);
    assert!(config.database.integrity_check_on_startup);
}

#[test]
fn default_config_has_expected_logging_defaults() {
    let config = AppConfig::default();
    assert_eq!(config.logging.level, "info");
    assert!(config.logging.directory.is_empty());
    assert_eq!(config.logging.max_file_size_bytes, 1_048_576);
    assert_eq!(config.logging.max_rotated_files, 5);
    assert!(!config.logging.json_output);
}

#[test]
fn default_config_has_expected_sync_defaults() {
    let config = AppConfig::default();
    assert!(!config.sync.finished_setup);
    assert_eq!(config.sync.last_incremental_sync, 0);
    assert_eq!(config.sync.last_incremental_sync_row_id, 0);
    assert_eq!(config.sync.messages_per_page, 25);
    assert!(config.sync.skip_empty_chats);
    assert!(!config.sync.sync_contacts_automatically);
}

#[test]
fn default_config_has_expected_notification_defaults() {
    let config = AppConfig::default();
    assert!(config.notifications.notify_reactions);
    assert!(!config.notifications.notify_on_chat_list);
    assert!(!config.notifications.filter_unknown_senders);
    assert!(config.notifications.global_text_detection.is_empty());
}

#[test]
fn default_config_has_expected_display_defaults() {
    let config = AppConfig::default();
    assert_eq!(config.display.user_name, "You");
    assert!(!config.display.use_24hr_format);
    assert!(!config.display.redacted_mode);
    assert!(!config.display.dense_chat_tiles);
    assert!(!config.display.hide_dividers);
    assert!((config.display.scroll_velocity - 1.0).abs() < f64::EPSILON);
    assert!(!config.display.show_delivery_timestamps);
    assert!(!config.display.reduced_force_touch);
}

#[test]
fn default_config_has_expected_theme_defaults() {
    let config = AppConfig::default();
    assert_eq!(config.theme.selected_theme, "OLED Dark");
    assert_eq!(config.theme.skin, "iOS");
    assert!(!config.theme.colorful_avatars);
    assert!(!config.theme.colorful_bubbles);
    assert!(!config.theme.monet_theming);
}

#[test]
fn default_config_has_expected_privacy_defaults() {
    let config = AppConfig::default();
    assert!(!config.privacy.incognito_mode);
    assert!(!config.privacy.hide_message_preview);
    assert!(!config.privacy.generate_fake_contact_names);
    assert!(!config.privacy.generate_fake_message_content);
    assert!(!config.privacy.private_mark_chat_as_read);
    assert!(!config.privacy.private_send_typing_indicators);
    assert!(!config.privacy.private_subject_line);
}

#[test]
fn default_config_has_expected_conversation_defaults() {
    let config = AppConfig::default();
    assert!(config.conversation.swipable_conversation_tiles);
    assert!(!config.conversation.smart_reply);
    assert!(!config.conversation.move_to_trash);
    assert!(!config.conversation.swipe_to_close);
    assert!(!config.conversation.double_tap_for_details);
    assert!(config.conversation.auto_play_gifs);
    assert!(config.conversation.auto_open_keyboard);
    assert!(config.conversation.swipe_to_reply);
    assert!(!config.conversation.swipe_to_archive);
    assert!(!config.conversation.move_chat_creator_to_header);
}

// ---- is_server_configured ----

#[test]
fn is_server_configured_returns_false_when_empty() {
    let config = AppConfig::default();
    assert!(!config.is_server_configured());
}

#[test]
fn is_server_configured_returns_false_with_only_address() {
    let mut config = AppConfig::default();
    config.server.address = "https://example.com".into();
    assert!(!config.is_server_configured());
}

#[test]
fn is_server_configured_returns_false_with_only_auth_key() {
    let mut config = AppConfig::default();
    config.server.guid_auth_key = "some-key".into();
    assert!(!config.is_server_configured());
}

#[test]
fn is_server_configured_returns_true_when_both_set() {
    let mut config = AppConfig::default();
    config.server.address = "https://example.com".into();
    config.server.guid_auth_key = "some-key".into();
    assert!(config.is_server_configured());
}

// ---- URL sanitization ----

#[test]
fn sanitize_empty_address_returns_empty() {
    assert_eq!(AppConfig::sanitize_server_address(""), "");
    assert_eq!(AppConfig::sanitize_server_address("  "), "");
}

#[test]
fn sanitize_strips_trailing_slashes() {
    assert_eq!(
        AppConfig::sanitize_server_address("http://192.168.1.1:1234/"),
        "http://192.168.1.1:1234"
    );
    assert_eq!(
        AppConfig::sanitize_server_address("http://example.com///"),
        "http://example.com"
    );
}

#[test]
fn sanitize_adds_https_for_cloudflare_tunnels() {
    assert_eq!(
        AppConfig::sanitize_server_address("abc123.trycloudflare.com"),
        "https://abc123.trycloudflare.com"
    );
}

#[test]
fn sanitize_adds_https_for_ngrok() {
    assert_eq!(
        AppConfig::sanitize_server_address("abc-def.ngrok.io"),
        "https://abc-def.ngrok.io"
    );
}

#[test]
fn sanitize_adds_https_for_zrok() {
    assert_eq!(
        AppConfig::sanitize_server_address("myservice.zrok.io"),
        "https://myservice.zrok.io"
    );
}

#[test]
fn sanitize_adds_http_for_plain_ip() {
    assert_eq!(
        AppConfig::sanitize_server_address("192.168.1.5:1234"),
        "http://192.168.1.5:1234"
    );
}

#[test]
fn sanitize_preserves_existing_http_scheme() {
    assert_eq!(
        AppConfig::sanitize_server_address("http://192.168.1.5:1234"),
        "http://192.168.1.5:1234"
    );
}

#[test]
fn sanitize_preserves_existing_https_scheme() {
    assert_eq!(
        AppConfig::sanitize_server_address("https://secure.example.com"),
        "https://secure.example.com"
    );
}

#[test]
fn sanitize_strips_surrounding_whitespace_and_quotes() {
    assert_eq!(
        AppConfig::sanitize_server_address("  \"https://example.com/\"  "),
        "https://example.com"
    );
}

// ---- TOML serialization round-trip ----

#[test]
fn config_toml_roundtrip_preserves_all_fields() {
    let mut config = AppConfig::default();
    config.server.address = "https://myserver.trycloudflare.com".into();
    config.server.guid_auth_key = "abc-def-ghi".into();
    config.server.api_timeout_ms = 60_000;
    config.server.accept_self_signed_certs = true;
    config.server.custom_headers.insert("X-Custom".into(), "value".into());
    config.database.pool_size = 8;
    config.database.wal_mode = false;
    config.logging.level = "debug".into();
    config.logging.json_output = true;
    config.sync.finished_setup = true;
    config.sync.messages_per_page = 50;
    config.notifications.notify_reactions = false;
    config.notifications.filter_unknown_senders = true;
    config.display.user_name = "TestUser".into();
    config.display.use_24hr_format = true;
    config.display.scroll_velocity = 2.5;
    config.theme.selected_theme = "Bright White".into();
    config.theme.skin = "Material".into();
    config.privacy.incognito_mode = true;
    config.conversation.smart_reply = true;

    let toml_str = toml::to_string_pretty(&config).unwrap();
    let deserialized: AppConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(deserialized.server.address, "https://myserver.trycloudflare.com");
    assert_eq!(deserialized.server.guid_auth_key, "abc-def-ghi");
    assert_eq!(deserialized.server.api_timeout_ms, 60_000);
    assert!(deserialized.server.accept_self_signed_certs);
    assert_eq!(deserialized.server.custom_headers.get("X-Custom").unwrap(), "value");
    assert_eq!(deserialized.database.pool_size, 8);
    assert!(!deserialized.database.wal_mode);
    assert_eq!(deserialized.logging.level, "debug");
    assert!(deserialized.logging.json_output);
    assert!(deserialized.sync.finished_setup);
    assert_eq!(deserialized.sync.messages_per_page, 50);
    assert!(!deserialized.notifications.notify_reactions);
    assert!(deserialized.notifications.filter_unknown_senders);
    assert_eq!(deserialized.display.user_name, "TestUser");
    assert!(deserialized.display.use_24hr_format);
    assert!((deserialized.display.scroll_velocity - 2.5).abs() < f64::EPSILON);
    assert_eq!(deserialized.theme.selected_theme, "Bright White");
    assert_eq!(deserialized.theme.skin, "Material");
    assert!(deserialized.privacy.incognito_mode);
    assert!(deserialized.conversation.smart_reply);
}

#[test]
fn config_toml_deserialization_applies_defaults_for_missing_fields() {
    // Minimal TOML that only sets one field
    let toml_str = r#"
[server]
address = "https://test.com"
"#;

    let config: AppConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.server.address, "https://test.com");
    // All other fields should have defaults
    assert_eq!(config.server.api_timeout_ms, 30_000);
    assert!(config.database.wal_mode);
    assert_eq!(config.logging.level, "info");
    assert_eq!(config.display.user_name, "You");
    assert_eq!(config.theme.selected_theme, "OLED Dark");
}

#[test]
fn config_toml_empty_string_deserializes_to_defaults() {
    let config: AppConfig = toml::from_str("").unwrap();
    assert!(config.server.address.is_empty());
    assert_eq!(config.server.api_timeout_ms, 30_000);
    assert!(config.database.wal_mode);
}

// ---- File save and load ----

#[test]
fn config_save_and_load_from_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");

    let mut config = AppConfig::default();
    config.server.address = "https://saved-server.com".into();
    config.server.guid_auth_key = "file-test-key".into();
    config.display.user_name = "FileTestUser".into();
    config.sync.finished_setup = true;

    config.save_to_file(&path).unwrap();
    assert!(path.exists(), "config file should be created");

    let loaded = AppConfig::load_from_file(&path).unwrap();
    assert_eq!(loaded.server.address, "https://saved-server.com");
    assert_eq!(loaded.server.guid_auth_key, "file-test-key");
    assert_eq!(loaded.display.user_name, "FileTestUser");
    assert!(loaded.sync.finished_setup);
}

#[test]
fn config_save_creates_parent_directories() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested").join("deep").join("config.toml");

    let config = AppConfig::default();
    config.save_to_file(&path).unwrap();
    assert!(path.exists(), "should create nested directories");
}

#[test]
fn config_load_nonexistent_file_returns_error() {
    let result = AppConfig::load_from_file(std::path::Path::new("/tmp/nonexistent_config_42.toml"));
    assert!(result.is_err(), "loading nonexistent file should fail");
}

#[test]
fn config_load_invalid_toml_returns_error() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("invalid.toml");
    std::fs::write(&path, "this is not valid {{ toml }}").unwrap();

    let result = AppConfig::load_from_file(&path);
    assert!(result.is_err(), "loading invalid TOML should fail");
}

// ---- ConfigHandle async access ----

#[tokio::test]
async fn config_handle_read_returns_default_values() {
    let handle = ConfigHandle::new(AppConfig::default());
    let config = handle.read().await;
    assert_eq!(config.server.api_timeout_ms, 30_000);
    assert_eq!(config.display.user_name, "You");
}

#[tokio::test]
async fn config_handle_write_updates_values() {
    let handle = ConfigHandle::new(AppConfig::default());

    {
        let mut config = handle.write().await;
        config.server.address = "https://updated.com".into();
        config.display.user_name = "UpdatedUser".into();
    }

    let config = handle.read().await;
    assert_eq!(config.server.address, "https://updated.com");
    assert_eq!(config.display.user_name, "UpdatedUser");
}

#[tokio::test]
async fn config_handle_clone_shares_state() {
    let handle1 = ConfigHandle::new(AppConfig::default());
    let handle2 = handle1.clone();

    {
        let mut config = handle1.write().await;
        config.server.address = "https://shared.com".into();
    }

    let config = handle2.read().await;
    assert_eq!(
        config.server.address, "https://shared.com",
        "cloned ConfigHandle should share the same underlying state"
    );
}

#[tokio::test]
async fn config_handle_concurrent_reads_do_not_block() {
    let handle = ConfigHandle::new(AppConfig::default());

    let h1 = handle.clone();
    let h2 = handle.clone();

    let (r1, r2) = tokio::join!(
        async { h1.read().await.server.api_timeout_ms },
        async { h2.read().await.server.api_timeout_ms },
    );

    assert_eq!(r1, 30_000);
    assert_eq!(r2, 30_000);
}

// ---- Effective paths ----

#[test]
fn effective_db_path_uses_custom_when_set() {
    let mut config = AppConfig::default();
    config.database.path = "/custom/path/my.db".into();
    let path = config.effective_db_path().unwrap();
    assert_eq!(path, std::path::PathBuf::from("/custom/path/my.db"));
}

#[test]
fn effective_db_path_uses_default_when_empty() {
    let config = AppConfig::default();
    let path = config.effective_db_path().unwrap();
    assert!(
        path.to_string_lossy().contains("bluebubbles.db"),
        "default db path should contain 'bluebubbles.db'"
    );
}

#[test]
fn effective_log_dir_uses_custom_when_set() {
    let mut config = AppConfig::default();
    config.logging.directory = "/custom/logs".into();
    let path = config.effective_log_dir().unwrap();
    assert_eq!(path, std::path::PathBuf::from("/custom/logs"));
}

#[test]
fn effective_log_dir_uses_default_when_empty() {
    let config = AppConfig::default();
    let path = config.effective_log_dir().unwrap();
    assert!(
        path.to_string_lossy().contains("logs"),
        "default log dir should contain 'logs'"
    );
}

// ---- Platform detection ----

#[test]
fn platform_current_returns_valid_platform() {
    let platform = Platform::current();
    assert!(
        matches!(platform, Platform::Windows | Platform::MacOs | Platform::Linux),
        "should detect a known platform"
    );
}

#[test]
fn platform_name_returns_human_readable_string() {
    assert_eq!(Platform::Windows.name(), "Windows");
    assert_eq!(Platform::MacOs.name(), "macOS");
    assert_eq!(Platform::Linux.name(), "Linux");
}

#[test]
fn platform_display_matches_name() {
    assert_eq!(format!("{}", Platform::Windows), "Windows");
    assert_eq!(format!("{}", Platform::MacOs), "macOS");
    assert_eq!(format!("{}", Platform::Linux), "Linux");
}

#[test]
fn platform_data_dir_returns_valid_path() {
    let data_dir = Platform::data_dir().unwrap();
    assert!(
        data_dir.to_string_lossy().contains("BlueBubbles"),
        "data dir should contain 'BlueBubbles'"
    );
}

#[test]
fn platform_config_dir_returns_valid_path() {
    let config_dir = Platform::config_dir().unwrap();
    assert!(
        config_dir.to_string_lossy().contains("BlueBubbles"),
        "config dir should contain 'BlueBubbles'"
    );
}

#[test]
fn platform_cache_dir_returns_valid_path() {
    let cache_dir = Platform::cache_dir().unwrap();
    assert!(
        cache_dir.to_string_lossy().contains("BlueBubbles"),
        "cache dir should contain 'BlueBubbles'"
    );
}

#[test]
fn platform_hostname_returns_non_empty_string() {
    let hostname = Platform::hostname();
    assert!(!hostname.is_empty(), "hostname should not be empty");
}

#[cfg(target_os = "windows")]
#[test]
fn platform_current_is_windows() {
    assert_eq!(Platform::current(), Platform::Windows);
}
