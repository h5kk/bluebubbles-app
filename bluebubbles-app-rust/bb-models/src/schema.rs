//! Database schema definitions and table creation.
//!
//! Defines the complete SQLite schema matching all 9 entities from the
//! original BlueBubbles ObjectBox database, translated to relational tables
//! with proper indexes for performance.

use rusqlite::Connection;
use bb_core::error::{BbError, BbResult};
use tracing::info;

/// Create all database tables and indexes if they do not exist.
pub fn create_tables(conn: &Connection) -> BbResult<()> {
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| BbError::Database(format!("failed to create schema: {e}")))?;
    info!("database schema verified");
    Ok(())
}

/// Drop all tables (used for database reset).
pub fn drop_tables(conn: &Connection) -> BbResult<()> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS chat_handle_join;
         DROP TABLE IF EXISTS attachments;
         DROP TABLE IF EXISTS messages;
         DROP TABLE IF EXISTS handles;
         DROP TABLE IF EXISTS chats;
         DROP TABLE IF EXISTS contacts;
         DROP TABLE IF EXISTS fcm_data;
         DROP TABLE IF EXISTS themes;
         DROP TABLE IF EXISTS theme_entries;
         DROP TABLE IF EXISTS scheduled_messages;
         DROP TABLE IF EXISTS settings;
         DROP TABLE IF EXISTS schema_version;",
    )
    .map_err(|e| BbError::Database(format!("failed to drop tables: {e}")))?;
    Ok(())
}

/// Complete SQL schema for all tables.
const SCHEMA_SQL: &str = r#"
-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);

-- Chats (conversations)
CREATE TABLE IF NOT EXISTS chats (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    original_rowid                  INTEGER,
    guid                            TEXT NOT NULL UNIQUE,
    chat_identifier                 TEXT,
    display_name                    TEXT,
    is_archived                     INTEGER NOT NULL DEFAULT 0,
    mute_type                       TEXT,
    mute_args                       TEXT,
    is_pinned                       INTEGER NOT NULL DEFAULT 0,
    has_unread_message              INTEGER NOT NULL DEFAULT 0,
    pin_index                       INTEGER,
    auto_send_read_receipts         INTEGER,
    auto_send_typing_indicators     INTEGER,
    text_field_text                 TEXT,
    text_field_attachments          TEXT NOT NULL DEFAULT '[]',
    latest_message_date             TEXT,
    date_deleted                    TEXT,
    style                           INTEGER,
    lock_chat_name                  INTEGER NOT NULL DEFAULT 0,
    lock_chat_icon                  INTEGER NOT NULL DEFAULT 0,
    last_read_message_guid          TEXT,
    custom_avatar_path              TEXT
);

CREATE INDEX IF NOT EXISTS idx_chats_guid ON chats(guid);
CREATE INDEX IF NOT EXISTS idx_chats_latest_message_date ON chats(latest_message_date);
CREATE INDEX IF NOT EXISTS idx_chats_is_pinned ON chats(is_pinned);
CREATE INDEX IF NOT EXISTS idx_chats_date_deleted ON chats(date_deleted);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    original_rowid                  INTEGER,
    guid                            TEXT UNIQUE,
    chat_id                         INTEGER REFERENCES chats(id),
    handle_id                       INTEGER,
    other_handle                    INTEGER,
    text                            TEXT,
    subject                         TEXT,
    country                         TEXT,
    error                           INTEGER NOT NULL DEFAULT 0,
    date_created                    TEXT,
    date_read                       TEXT,
    date_delivered                  TEXT,
    is_delivered                    INTEGER NOT NULL DEFAULT 0,
    is_from_me                      INTEGER NOT NULL DEFAULT 1,
    has_dd_results                  INTEGER NOT NULL DEFAULT 0,
    date_played                     TEXT,
    item_type                       INTEGER NOT NULL DEFAULT 0,
    group_title                     TEXT,
    group_action_type               INTEGER NOT NULL DEFAULT 0,
    balloon_bundle_id               TEXT,
    associated_message_guid         TEXT,
    associated_message_part         INTEGER,
    associated_message_type         TEXT,
    expressive_send_style_id        TEXT,
    has_attachments                 INTEGER NOT NULL DEFAULT 0,
    has_reactions                   INTEGER NOT NULL DEFAULT 0,
    date_deleted                    TEXT,
    thread_originator_guid          TEXT,
    thread_originator_part          TEXT,
    big_emoji                       INTEGER,
    attributed_body                 TEXT,
    message_summary_info            TEXT,
    payload_data                    TEXT,
    metadata                        TEXT,
    has_apple_payload_data          INTEGER NOT NULL DEFAULT 0,
    date_edited                     TEXT,
    was_delivered_quietly           INTEGER NOT NULL DEFAULT 0,
    did_notify_recipient            INTEGER NOT NULL DEFAULT 0,
    is_bookmarked                   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_messages_guid ON messages(guid);
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_date_created ON messages(date_created);
CREATE INDEX IF NOT EXISTS idx_messages_handle_id ON messages(handle_id);
CREATE INDEX IF NOT EXISTS idx_messages_associated ON messages(associated_message_guid);
CREATE INDEX IF NOT EXISTS idx_messages_thread_originator ON messages(thread_originator_guid);
CREATE INDEX IF NOT EXISTS idx_messages_chat_date ON messages(chat_id, date_created);

-- Handles (phone numbers / email addresses)
CREATE TABLE IF NOT EXISTS handles (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    original_rowid                  INTEGER,
    address                         TEXT NOT NULL,
    service                         TEXT NOT NULL DEFAULT 'iMessage',
    unique_address_service          TEXT NOT NULL UNIQUE,
    formatted_address               TEXT,
    country                         TEXT,
    color                           TEXT,
    default_phone                   TEXT,
    default_email                   TEXT,
    contact_id                      INTEGER REFERENCES contacts(id)
);

CREATE INDEX IF NOT EXISTS idx_handles_address ON handles(address);
CREATE INDEX IF NOT EXISTS idx_handles_unique ON handles(unique_address_service);
CREATE INDEX IF NOT EXISTS idx_handles_contact ON handles(contact_id);

-- Attachments
CREATE TABLE IF NOT EXISTS attachments (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    original_rowid                  INTEGER,
    guid                            TEXT UNIQUE,
    message_id                      INTEGER REFERENCES messages(id),
    uti                             TEXT,
    mime_type                       TEXT,
    is_outgoing                     INTEGER,
    transfer_name                   TEXT,
    total_bytes                     INTEGER,
    height                          INTEGER,
    width                           INTEGER,
    web_url                         TEXT,
    has_live_photo                  INTEGER NOT NULL DEFAULT 0,
    metadata                        TEXT
);

CREATE INDEX IF NOT EXISTS idx_attachments_guid ON attachments(guid);
CREATE INDEX IF NOT EXISTS idx_attachments_message_id ON attachments(message_id);
CREATE INDEX IF NOT EXISTS idx_attachments_mime_type ON attachments(mime_type);

-- Contacts
CREATE TABLE IF NOT EXISTS contacts (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    external_id                     TEXT UNIQUE,
    display_name                    TEXT NOT NULL,
    phones                          TEXT NOT NULL DEFAULT '[]',
    emails                          TEXT NOT NULL DEFAULT '[]',
    avatar                          BLOB,
    structured_name                 TEXT
);

CREATE INDEX IF NOT EXISTS idx_contacts_external_id ON contacts(external_id);
CREATE INDEX IF NOT EXISTS idx_contacts_display_name ON contacts(display_name);

-- Chat-Handle join table (many-to-many for chat participants)
CREATE TABLE IF NOT EXISTS chat_handle_join (
    chat_id                         INTEGER NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    handle_id                       INTEGER NOT NULL REFERENCES handles(id) ON DELETE CASCADE,
    PRIMARY KEY (chat_id, handle_id)
);

CREATE INDEX IF NOT EXISTS idx_chj_chat ON chat_handle_join(chat_id);
CREATE INDEX IF NOT EXISTS idx_chj_handle ON chat_handle_join(handle_id);

-- FCM configuration data
CREATE TABLE IF NOT EXISTS fcm_data (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id                      TEXT,
    storage_bucket                  TEXT,
    api_key                         TEXT,
    firebase_url                    TEXT,
    client_id                       TEXT,
    application_id                  TEXT
);

-- Theme definitions
CREATE TABLE IF NOT EXISTS themes (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    name                            TEXT NOT NULL UNIQUE,
    gradient_bg                     INTEGER NOT NULL DEFAULT 0,
    google_font                     TEXT NOT NULL DEFAULT 'Default',
    theme_data                      TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_themes_name ON themes(name);

-- Theme entries (legacy, kept for migration compatibility)
CREATE TABLE IF NOT EXISTS theme_entries (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    theme_id                        INTEGER,
    name                            TEXT,
    is_font                         INTEGER,
    font_size                       INTEGER,
    font_weight                     INTEGER,
    color                           TEXT
);

-- Scheduled messages (not locally stored, but cached)
CREATE TABLE IF NOT EXISTS scheduled_messages (
    id                              INTEGER PRIMARY KEY,
    type                            TEXT NOT NULL,
    chat_guid                       TEXT NOT NULL,
    message                         TEXT NOT NULL,
    send_method                     TEXT NOT NULL DEFAULT 'private-api',
    scheduled_for                   TEXT NOT NULL,
    schedule_type                   TEXT,
    schedule_interval               INTEGER,
    schedule_interval_type          TEXT,
    status                          TEXT NOT NULL DEFAULT 'pending',
    error                           TEXT,
    sent_at                         TEXT,
    created_at                      TEXT NOT NULL
);

-- Settings key-value store
CREATE TABLE IF NOT EXISTS settings (
    key                             TEXT PRIMARY KEY NOT NULL,
    value                           TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tables() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Verify key tables exist
        let tables = ["chats", "messages", "handles", "attachments", "contacts",
                       "fcm_data", "themes", "scheduled_messages", "settings",
                       "chat_handle_join", "schema_version"];
        for table in &tables {
            let count: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{table}'"),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table {table} should exist");
        }
    }

    #[test]
    fn test_create_tables_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        create_tables(&conn).unwrap(); // Should not error
    }

    #[test]
    fn test_drop_and_recreate() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        drop_tables(&conn).unwrap();
        create_tables(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chats'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_indexes_created() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_messages_chat_date'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
