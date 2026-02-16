//! Shared test utilities for integration tests.

use bb_core::config::{AppConfig, DatabaseConfig, ConfigHandle};
use bb_models::Database;
use bb_services::event_bus::EventBus;
use bb_socket::EventDispatcher;
use tempfile::TempDir;

/// Create a temporary database with full schema and migrations applied.
/// Returns the Database and the TempDir (must be held alive for the duration of the test).
pub fn create_test_db() -> (Database, TempDir) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("test.db");
    let config = DatabaseConfig::default();
    let db = Database::init(&path, &config).expect("failed to init test database");
    (db, dir)
}

/// Create a default test configuration.
pub fn create_test_config() -> AppConfig {
    AppConfig::default()
}

/// Create a ConfigHandle wrapping a default config.
pub fn create_test_config_handle() -> ConfigHandle {
    ConfigHandle::new(create_test_config())
}

/// Create an EventBus with a small buffer suitable for tests.
pub fn create_test_event_bus() -> EventBus {
    EventBus::new(64)
}

/// Create an EventDispatcher with a small buffer suitable for tests.
pub fn create_test_dispatcher() -> EventDispatcher {
    EventDispatcher::new(64)
}

/// Seed the database with realistic test data.
///
/// Creates:
/// - 10 chats (2 pinned, 1 archived, 1 soft-deleted)
/// - 100 messages spread across the chats
/// - 20 contacts with phones and emails
/// - 15 handles linked to chats
/// - 5 attachments
/// - 3 settings entries
pub fn seed_test_data(db: &Database) {
    let conn = db.conn().expect("failed to get connection for seeding");

    // Insert 15 handles
    for i in 1..=15 {
        let address = format!("+1555000{:04}", i);
        let service = "iMessage";
        let unique = format!("{}/{}", address, service);
        conn.execute(
            "INSERT INTO handles (address, service, unique_address_service) VALUES (?1, ?2, ?3)",
            rusqlite::params![address, service, unique],
        )
        .expect("failed to insert handle");
    }

    // Insert 10 chats
    for i in 1..=10 {
        let guid = format!("iMessage;-;chat-{}", i);
        let display_name = format!("Chat {}", i);
        let is_pinned = if i <= 2 { 1 } else { 0 };
        let is_archived = if i == 3 { 1 } else { 0 };
        let date_deleted: Option<&str> = if i == 4 {
            Some("2024-06-01T00:00:00Z")
        } else {
            None
        };
        let latest_message_date = format!("2024-01-{:02}T12:00:00Z", 10 + i);

        conn.execute(
            "INSERT INTO chats (guid, chat_identifier, display_name, is_pinned, is_archived, date_deleted, latest_message_date, pin_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                guid,
                format!("chat-id-{}", i),
                display_name,
                is_pinned,
                is_archived,
                date_deleted,
                latest_message_date,
                if is_pinned == 1 { Some(i as i64) } else { None::<i64> },
            ],
        )
        .expect("failed to insert chat");

        // Link 1-2 handles to each chat
        let handle_id_1 = ((i - 1) % 15) + 1;
        conn.execute(
            "INSERT OR IGNORE INTO chat_handle_join (chat_id, handle_id) VALUES (?1, ?2)",
            rusqlite::params![i as i64, handle_id_1 as i64],
        )
        .expect("failed to insert chat_handle_join");

        if i <= 5 {
            let handle_id_2 = (i % 15) + 1;
            conn.execute(
                "INSERT OR IGNORE INTO chat_handle_join (chat_id, handle_id) VALUES (?1, ?2)",
                rusqlite::params![i as i64, handle_id_2 as i64],
            )
            .expect("failed to insert second chat_handle_join");
        }
    }

    // Insert 100 messages across the 10 chats (10 per chat)
    for i in 1..=100 {
        let chat_id = ((i - 1) % 10) + 1;
        let msg_index = (i - 1) / 10 + 1;
        let guid = format!("msg-{:04}", i);
        let is_from_me = i % 3 != 0; // 2/3 from me, 1/3 received
        let date_created = format!(
            "2024-01-{:02}T{:02}:{:02}:00Z",
            (chat_id as u32).min(28),
            msg_index % 24,
            (i % 60) as u32
        );
        let text = format!("Test message number {} in chat {}", msg_index, chat_id);
        let handle_id = if !is_from_me {
            Some(((chat_id - 1) % 15 + 1) as i64)
        } else {
            None
        };
        // Mark some received messages as read
        let date_read: Option<String> = if !is_from_me && i % 5 == 0 {
            Some(format!(
                "2024-01-{:02}T{:02}:{:02}:30Z",
                (chat_id as u32).min(28),
                msg_index % 24,
                (i % 60) as u32
            ))
        } else {
            None
        };

        conn.execute(
            "INSERT INTO messages (guid, chat_id, handle_id, text, is_from_me, date_created, date_read, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
            rusqlite::params![
                guid,
                chat_id as i64,
                handle_id,
                text,
                is_from_me as i32,
                date_created,
                date_read,
            ],
        )
        .expect("failed to insert message");
    }

    // Insert 20 contacts
    for i in 1..=20 {
        let external_id = format!("contact-ext-{}", i);
        let display_name = format!("Contact {}", i);
        let phone = format!("+1555000{:04}", i);
        let phones = format!("[\"{}\"]", phone);
        let email = format!("contact{}@example.com", i);
        let emails = format!("[\"{}\"]", email);

        conn.execute(
            "INSERT INTO contacts (external_id, display_name, phones, emails)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![external_id, display_name, phones, emails],
        )
        .expect("failed to insert contact");
    }

    // Insert 5 attachments on the first 5 messages
    for i in 1..=5 {
        let guid = format!("att-{}", i);
        let message_id = i as i64;
        let mime_type = match i % 3 {
            0 => "video/mp4",
            1 => "image/jpeg",
            _ => "image/png",
        };
        let transfer_name = format!("file_{}.{}", i, if mime_type.starts_with("video") { "mp4" } else { "jpg" });

        conn.execute(
            "INSERT INTO attachments (guid, message_id, mime_type, transfer_name, total_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![guid, message_id, mime_type, transfer_name, 1024 * i as i64],
        )
        .expect("failed to insert attachment");
    }

    // Insert settings
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('testBool', 'true')",
        [],
    )
    .expect("failed to insert setting");
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('testInt', '42')",
        [],
    )
    .expect("failed to insert setting");
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('testStr', 'hello world')",
        [],
    )
    .expect("failed to insert setting");
}
