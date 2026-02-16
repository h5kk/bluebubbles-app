//! Settings key-value store with typed accessors.
//!
//! Implements the 120+ BlueBubbles settings as a simple key-value table in SQLite.
//! Each setting is stored as a TEXT value; typed accessors handle parsing.

use rusqlite::{params, Connection};
use bb_core::error::{BbError, BbResult};
use std::collections::HashMap;

/// Settings key-value store backed by the `settings` table.
pub struct Settings;

impl Settings {
    /// Get a raw string value for a key.
    pub fn get(conn: &Connection, key: &str) -> BbResult<Option<String>> {
        match conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Set a raw string value for a key (upsert).
    pub fn set(conn: &Connection, key: &str, value: &str) -> BbResult<()> {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(())
    }

    /// Delete a setting by key.
    pub fn delete(conn: &Connection, key: &str) -> BbResult<bool> {
        let changed = conn
            .execute("DELETE FROM settings WHERE key = ?1", [key])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    /// Get all settings as a HashMap.
    pub fn get_all(conn: &Connection) -> BbResult<HashMap<String, String>> {
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| BbError::Database(e.to_string()))?;

        let map = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| BbError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(map)
    }

    /// Set multiple settings at once (batch upsert).
    pub fn set_many(conn: &Connection, entries: &[(&str, &str)]) -> BbResult<()> {
        let mut stmt = conn
            .prepare(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .map_err(|e| BbError::Database(e.to_string()))?;

        for (key, value) in entries {
            stmt.execute(params![key, value])
                .map_err(|e| BbError::Database(e.to_string()))?;
        }
        Ok(())
    }

    /// Clear all settings.
    pub fn clear(conn: &Connection) -> BbResult<()> {
        conn.execute("DELETE FROM settings", [])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(())
    }

    // ─── Typed accessors ─────────────────────────────────────────────────

    /// Get a boolean setting (stored as "true"/"false" or "1"/"0").
    pub fn get_bool(conn: &Connection, key: &str) -> BbResult<Option<bool>> {
        Ok(Self::get(conn, key)?.map(|v| v == "true" || v == "1"))
    }

    /// Set a boolean setting.
    pub fn set_bool(conn: &Connection, key: &str, value: bool) -> BbResult<()> {
        Self::set(conn, key, if value { "true" } else { "false" })
    }

    /// Get an integer setting.
    pub fn get_i64(conn: &Connection, key: &str) -> BbResult<Option<i64>> {
        Ok(Self::get(conn, key)?.and_then(|v| v.parse().ok()))
    }

    /// Set an integer setting.
    pub fn set_i64(conn: &Connection, key: &str, value: i64) -> BbResult<()> {
        Self::set(conn, key, &value.to_string())
    }

    /// Get a float setting.
    pub fn get_f64(conn: &Connection, key: &str) -> BbResult<Option<f64>> {
        Ok(Self::get(conn, key)?.and_then(|v| v.parse().ok()))
    }

    /// Set a float setting.
    pub fn set_f64(conn: &Connection, key: &str, value: f64) -> BbResult<()> {
        Self::set(conn, key, &value.to_string())
    }

    /// Get a JSON-deserialized setting.
    pub fn get_json<T: serde::de::DeserializeOwned>(conn: &Connection, key: &str) -> BbResult<Option<T>> {
        match Self::get(conn, key)? {
            Some(v) => {
                let parsed = serde_json::from_str(&v)
                    .map_err(|e| BbError::Serialization(format!("failed to parse setting {key}: {e}")))?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a JSON-serialized setting.
    pub fn set_json<T: serde::Serialize>(conn: &Connection, key: &str, value: &T) -> BbResult<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| BbError::Serialization(e.to_string()))?;
        Self::set(conn, key, &json)
    }
}

// ─── Setting key constants ───────────────────────────────────────────────────

/// Connection and authentication setting keys.
pub mod keys {
    // Connection & Auth
    pub const ICLOUD_ACCOUNT: &str = "iCloudAccount";
    pub const GUID_AUTH_KEY: &str = "guidAuthKey";
    pub const SERVER_ADDRESS: &str = "serverAddress";
    pub const CUSTOM_HEADERS: &str = "customHeaders";
    pub const FINISHED_SETUP: &str = "finishedSetup";
    pub const REACHED_CONVERSATION_LIST: &str = "reachedConversationList";
    pub const FIRST_FCM_REGISTER_DATE: &str = "firstFcmRegisterDate";

    // Download & Save
    pub const AUTO_DOWNLOAD: &str = "autoDownload";
    pub const ONLY_WIFI_DOWNLOAD: &str = "onlyWifiDownload";
    pub const AUTO_SAVE: &str = "autoSave";
    pub const AUTO_SAVE_PICS_LOCATION: &str = "autoSavePicsLocation";
    pub const AUTO_SAVE_DOCS_LOCATION: &str = "autoSaveDocsLocation";
    pub const ASK_WHERE_TO_SAVE: &str = "askWhereToSave";

    // UI & Appearance
    pub const SKIN: &str = "skin";
    pub const THEME: &str = "theme";
    pub const COLORFUL_AVATARS: &str = "colorfulAvatars";
    pub const COLORFUL_BUBBLES: &str = "colorfulBubbles";
    pub const HIDE_DIVIDERS: &str = "hideDividers";
    pub const DENSE_CHAT_TILES: &str = "denseChatTiles";
    pub const ALWAYS_SHOW_AVATARS: &str = "alwaysShowAvatars";
    pub const AVATAR_SCALE: &str = "avatarScale";
    pub const HIGH_PERF_MODE: &str = "highPerfMode";
    pub const IMMERSIVE_MODE: &str = "immersiveMode";
    pub const COLORS_FROM_MEDIA: &str = "colorsFromMedia";
    pub const MONET_THEMING: &str = "monetTheming";
    pub const FILTERED_CHAT_LIST: &str = "filteredChatList";
    pub const SHOW_DELIVERY_TIMESTAMPS: &str = "showDeliveryTimestamps";
    pub const SHOW_CONNECTION_INDICATOR: &str = "showConnectionIndicator";
    pub const SHOW_SYNC_INDICATOR: &str = "showSyncIndicator";
    pub const STATUS_INDICATORS_ON_CHATS: &str = "statusIndicatorsOnChats";
    pub const TABLET_MODE: &str = "tabletMode";
    pub const HIGHLIGHT_SELECTED_CHAT: &str = "highlightSelectedChat";
    pub const MAX_AVATARS_IN_GROUP_WIDGET: &str = "maxAvatarsInGroupWidget";

    // Input & Keyboard
    pub const AUTO_OPEN_KEYBOARD: &str = "autoOpenKeyboard";
    pub const SEND_WITH_RETURN: &str = "sendWithReturn";
    pub const HIDE_KEYBOARD_ON_SCROLL: &str = "hideKeyboardOnScroll";
    pub const SWIPE_TO_CLOSE_KEYBOARD: &str = "swipeToCloseKeyboard";
    pub const SWIPE_TO_OPEN_KEYBOARD: &str = "swipeToOpenKeyboard";
    pub const OPEN_KEYBOARD_ON_STB: &str = "openKeyboardOnSTB";
    pub const MOVE_CHAT_CREATOR_TO_HEADER: &str = "moveChatCreatorToHeader";
    pub const CAMERA_FAB: &str = "cameraFAB";
    pub const RECIPIENT_AS_PLACEHOLDER: &str = "recipientAsPlaceholder";
    pub const REPLACE_EMOTICONS_WITH_EMOJI: &str = "replaceEmoticonsWithEmoji";

    // Messaging Behavior
    pub const SEND_DELAY: &str = "sendDelay";
    pub const HIDE_TEXT_PREVIEWS: &str = "hideTextPreviews";
    pub const CANCEL_QUEUED_MESSAGES: &str = "cancelQueuedMessages";
    pub const REPLIES_TO_PREVIOUS: &str = "repliesToPrevious";
    pub const SCROLL_TO_BOTTOM_ON_SEND: &str = "scrollToBottomOnSend";
    pub const SCROLL_TO_LAST_UNREAD: &str = "scrollToLastUnread";
    pub const UNARCHIVE_ON_NEW_MESSAGE: &str = "unarchiveOnNewMessage";

    // Notifications
    pub const NOTIFY_ON_CHAT_LIST: &str = "notifyOnChatList";
    pub const NOTIFY_REACTIONS: &str = "notifyReactions";
    pub const NOTIFICATION_SOUND: &str = "notificationSound";
    pub const GLOBAL_TEXT_DETECTION: &str = "globalTextDetection";
    pub const FILTER_UNKNOWN_SENDERS: &str = "filterUnknownSenders";
    pub const SELECTED_ACTION_INDICES: &str = "selectedActionIndices";
    pub const ACTION_LIST: &str = "actionList";

    // Private API
    pub const SERVER_PRIVATE_API: &str = "serverPrivateAPI";
    pub const ENABLE_PRIVATE_API: &str = "enablePrivateAPI";
    pub const PRIVATE_SEND_TYPING_INDICATORS: &str = "privateSendTypingIndicators";
    pub const PRIVATE_MARK_CHAT_AS_READ: &str = "privateMarkChatAsRead";
    pub const PRIVATE_MANUAL_MARK_AS_READ: &str = "privateManualMarkAsRead";
    pub const PRIVATE_SUBJECT_LINE: &str = "privateSubjectLine";
    pub const PRIVATE_API_SEND: &str = "privateAPISend";
    pub const PRIVATE_API_ATTACHMENT_SEND: &str = "privateAPIAttachmentSend";
    pub const EDIT_LAST_SENT_MESSAGE_ON_UP_ARROW: &str = "editLastSentMessageOnUpArrow";

    // Redacted Mode
    pub const REDACTED_MODE: &str = "redactedMode";
    pub const HIDE_ATTACHMENTS: &str = "hideAttachments";
    pub const HIDE_CONTACT_INFO: &str = "hideContactInfo";
    pub const GENERATE_FAKE_CONTACT_NAMES: &str = "generateFakeContactNames";
    pub const HIDE_MESSAGE_CONTENT: &str = "hideMessageContent";

    // Security
    pub const SHOULD_SECURE: &str = "shouldSecure";
    pub const SECURITY_LEVEL: &str = "securityLevel";
    pub const INCOGNITO_KEYBOARD: &str = "incognitoKeyboard";

    // Quick Tapback
    pub const ENABLE_QUICK_TAPBACK: &str = "enableQuickTapback";
    pub const QUICK_TAPBACK_TYPE: &str = "quickTapbackType";

    // Swipe Actions
    pub const MATERIAL_RIGHT_ACTION: &str = "materialRightAction";
    pub const MATERIAL_LEFT_ACTION: &str = "materialLeftAction";
    pub const SWIPABLE_CONVERSATION_TILES: &str = "swipableConversationTiles";

    // Video
    pub const START_VIDEOS_MUTED: &str = "startVideosMuted";
    pub const START_VIDEOS_MUTED_FULLSCREEN: &str = "startVideosMutedFullscreen";

    // Time & Display
    pub const USE_24HR_FORMAT: &str = "use24HrFormat";
    pub const SCROLL_VELOCITY: &str = "scrollVelocity";
    pub const REFRESH_RATE: &str = "refreshRate";
    pub const FULLSCREEN_VIEWER_SWIPE_DIR: &str = "fullscreenViewerSwipeDir";
    pub const DOUBLE_TAP_FOR_DETAILS: &str = "doubleTapForDetails";

    // Pinned Chats Layout
    pub const PIN_ROWS_PORTRAIT: &str = "pinRowsPortrait";
    pub const PIN_COLUMNS_PORTRAIT: &str = "pinColumnsPortrait";
    pub const PIN_ROWS_LANDSCAPE: &str = "pinRowsLandscape";
    pub const PIN_COLUMNS_LANDSCAPE: &str = "pinColumnsLandscape";

    // Desktop-Specific
    pub const LAUNCH_AT_STARTUP: &str = "launchAtStartup";
    pub const LAUNCH_AT_STARTUP_MINIMIZED: &str = "launchAtStartupMinimized";
    pub const MINIMIZE_TO_TRAY: &str = "minimizeToTray";
    pub const CLOSE_TO_TRAY: &str = "closeToTray";
    pub const SPELLCHECK: &str = "spellcheck";
    pub const SPELLCHECK_LANGUAGE: &str = "spellcheckLanguage";
    pub const WINDOW_EFFECT: &str = "windowEffect";
    pub const WINDOW_EFFECT_CUSTOM_OPACITY_LIGHT: &str = "windowEffectCustomOpacityLight";
    pub const WINDOW_EFFECT_CUSTOM_OPACITY_DARK: &str = "windowEffectCustomOpacityDark";
    pub const USE_CUSTOM_TITLE_BAR: &str = "useCustomTitleBar";
    pub const USE_WINDOWS_ACCENT: &str = "useWindowsAccent";

    // Sync & Networking
    pub const SHOW_INCREMENTAL_SYNC: &str = "showIncrementalSync";
    pub const LAST_INCREMENTAL_SYNC: &str = "lastIncrementalSync";
    pub const LAST_INCREMENTAL_SYNC_ROW_ID: &str = "lastIncrementalSyncRowId";
    pub const API_TIMEOUT: &str = "apiTimeout";
    pub const USE_LOCALHOST: &str = "useLocalhost";
    pub const USE_LOCAL_IPV6: &str = "useLocalIpv6";
    pub const SYNC_CONTACTS_AUTOMATICALLY: &str = "syncContactsAutomatically";

    // Sound
    pub const SEND_SOUND_PATH: &str = "sendSoundPath";
    pub const RECEIVE_SOUND_PATH: &str = "receiveSoundPath";
    pub const SOUND_VOLUME: &str = "soundVolume";

    // Unified Push
    pub const ENABLE_UNIFIED_PUSH: &str = "enableUnifiedPush";
    pub const ENDPOINT_UNIFIED_PUSH: &str = "endpointUnifiedPush";

    // Misc
    pub const SMART_REPLY: &str = "smartReply";
    pub const KEEP_APP_ALIVE: &str = "keepAppAlive";
    pub const SEND_EVENTS_TO_TASKER: &str = "sendEventsToTasker";
    pub const USER_NAME: &str = "userName";
    pub const USER_AVATAR_PATH: &str = "userAvatarPath";
    pub const HIDE_NAMES_FOR_REACTIONS: &str = "hideNamesForReactions";
    pub const LOG_LEVEL: &str = "logLevel";
    pub const ALLOW_UPSIDE_DOWN_ROTATION: &str = "allowUpsideDownRotation";
    pub const LAST_REVIEW_REQUEST_TIMESTAMP: &str = "lastReviewRequestTimestamp";
    pub const DETAILS_MENU_ACTIONS: &str = "detailsMenuActions";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_settings_crud() {
        let conn = setup();

        // Set and get
        Settings::set(&conn, "testKey", "testValue").unwrap();
        assert_eq!(Settings::get(&conn, "testKey").unwrap(), Some("testValue".to_string()));

        // Update
        Settings::set(&conn, "testKey", "updatedValue").unwrap();
        assert_eq!(Settings::get(&conn, "testKey").unwrap(), Some("updatedValue".to_string()));

        // Delete
        assert!(Settings::delete(&conn, "testKey").unwrap());
        assert_eq!(Settings::get(&conn, "testKey").unwrap(), None);
    }

    #[test]
    fn test_settings_typed_bool() {
        let conn = setup();
        Settings::set_bool(&conn, keys::FINISHED_SETUP, true).unwrap();
        assert_eq!(Settings::get_bool(&conn, keys::FINISHED_SETUP).unwrap(), Some(true));

        Settings::set_bool(&conn, keys::FINISHED_SETUP, false).unwrap();
        assert_eq!(Settings::get_bool(&conn, keys::FINISHED_SETUP).unwrap(), Some(false));
    }

    #[test]
    fn test_settings_typed_i64() {
        let conn = setup();
        Settings::set_i64(&conn, keys::SEND_DELAY, 3000).unwrap();
        assert_eq!(Settings::get_i64(&conn, keys::SEND_DELAY).unwrap(), Some(3000));
    }

    #[test]
    fn test_settings_typed_f64() {
        let conn = setup();
        Settings::set_f64(&conn, keys::AVATAR_SCALE, 1.5).unwrap();
        let val = Settings::get_f64(&conn, keys::AVATAR_SCALE).unwrap().unwrap();
        assert!((val - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_settings_json() {
        let conn = setup();
        let actions = vec!["love", "like", "laugh"];
        Settings::set_json(&conn, keys::ACTION_LIST, &actions).unwrap();
        let loaded: Vec<String> = Settings::get_json(&conn, keys::ACTION_LIST).unwrap().unwrap();
        assert_eq!(loaded, vec!["love", "like", "laugh"]);
    }

    #[test]
    fn test_settings_get_all() {
        let conn = setup();
        Settings::set(&conn, "a", "1").unwrap();
        Settings::set(&conn, "b", "2").unwrap();
        let all = Settings::get_all(&conn).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("a").unwrap(), "1");
    }

    #[test]
    fn test_settings_set_many() {
        let conn = setup();
        Settings::set_many(&conn, &[("x", "10"), ("y", "20")]).unwrap();
        assert_eq!(Settings::get(&conn, "x").unwrap(), Some("10".to_string()));
        assert_eq!(Settings::get(&conn, "y").unwrap(), Some("20".to_string()));
    }

    #[test]
    fn test_settings_clear() {
        let conn = setup();
        Settings::set(&conn, "a", "1").unwrap();
        Settings::clear(&conn).unwrap();
        assert_eq!(Settings::get(&conn, "a").unwrap(), None);
    }

    #[test]
    fn test_settings_missing_key() {
        let conn = setup();
        assert_eq!(Settings::get(&conn, "nonexistent").unwrap(), None);
        assert_eq!(Settings::get_bool(&conn, "nonexistent").unwrap(), None);
        assert_eq!(Settings::get_i64(&conn, "nonexistent").unwrap(), None);
    }
}
