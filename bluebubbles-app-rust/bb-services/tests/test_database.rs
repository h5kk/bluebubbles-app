//! Integration tests for the complete data layer.
//!
//! Tests database creation, schema, CRUD operations for all models,
//! cursor-based pagination, search, settings, and integrity checks.

mod common;

use bb_core::constants::DB_SCHEMA_VERSION;
use bb_models::models::chat::Chat;
use bb_models::models::message::Message;
use bb_models::models::handle::Handle;
use bb_models::models::attachment::Attachment;
use bb_models::models::contact::Contact;
use bb_models::models::theme::ThemeStruct;
use bb_models::models::settings::Settings;
use bb_models::queries::{self, SortDirection};
use bb_models::migrations;

// ---- Database initialization and WAL mode ----

#[test]
fn database_init_creates_file_and_wal_mode() {
    let (db, dir) = common::create_test_db();
    let db_path = dir.path().join("test.db");
    assert!(db_path.exists(), "database file should exist after init");

    let conn = db.conn().unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal", "database should be in WAL mode");
}

#[test]
fn database_init_creates_all_tables() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let expected_tables = [
        "chats",
        "messages",
        "handles",
        "attachments",
        "contacts",
        "chat_handle_join",
        "fcm_data",
        "themes",
        "theme_entries",
        "scheduled_messages",
        "settings",
        "schema_version",
    ];

    for table in &expected_tables {
        let count: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'",
                    table
                ),
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "table '{}' should exist", table);
    }
}

#[test]
fn database_init_creates_indexes() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let expected_indexes = [
        "idx_chats_guid",
        "idx_messages_guid",
        "idx_messages_chat_id",
        "idx_messages_chat_date",
        "idx_handles_address",
        "idx_attachments_guid",
        "idx_contacts_external_id",
        "idx_settings_key",
    ];

    for idx in &expected_indexes {
        let count: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='{}'",
                    idx
                ),
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "index '{}' should exist", idx);
    }
}

// ---- Schema version and migrations ----

#[test]
fn schema_version_is_set_after_migration() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let version: i32 = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(version, DB_SCHEMA_VERSION);
}

#[test]
fn migration_is_idempotent() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    // Run migrations again; should be a no-op
    migrations::run_migrations(&conn).unwrap();

    let version: i32 = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(version, DB_SCHEMA_VERSION);
}

#[test]
fn migration_seeds_default_themes() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM themes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2, "migration should seed 2 default themes");
}

// ---- Chat CRUD ----

#[test]
fn chat_create_and_find_by_guid() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut chat = Chat::from_server_map(&serde_json::json!({
        "guid": "iMessage;-;+15551234567",
        "displayName": "Test Chat",
        "chatIdentifier": "+15551234567",
    }))
    .unwrap();

    let id = chat.save(&conn).unwrap();
    assert!(id > 0, "save should return a positive id");

    let found = queries::find_chat_by_guid(&conn, "iMessage;-;+15551234567")
        .unwrap()
        .expect("chat should be found by guid");
    assert_eq!(found.guid, "iMessage;-;+15551234567");
    assert_eq!(found.display_name.as_deref(), Some("Test Chat"));
}

#[test]
fn chat_upsert_updates_existing() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut chat = Chat::from_server_map(&serde_json::json!({
        "guid": "chat-upsert-test",
        "displayName": "Original",
    }))
    .unwrap();
    chat.save(&conn).unwrap();

    let mut updated = Chat::from_server_map(&serde_json::json!({
        "guid": "chat-upsert-test",
        "displayName": "Updated",
    }))
    .unwrap();
    updated.save(&conn).unwrap();

    let found = queries::find_chat_by_guid(&conn, "chat-upsert-test")
        .unwrap()
        .unwrap();
    assert_eq!(found.display_name.as_deref(), Some("Updated"));
}

#[test]
fn chat_list_excludes_archived_and_deleted_by_default() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();

    let non_archived = queries::list_chats(&conn, 0, 100, false).unwrap();
    let all = queries::list_chats(&conn, 0, 100, true).unwrap();

    assert!(
        non_archived.len() < all.len(),
        "excluding archived/deleted should return fewer chats"
    );

    // Verify no archived or deleted chats in non_archived list
    for chat in &non_archived {
        assert!(!chat.is_archived, "should not include archived chats");
        assert!(
            chat.date_deleted.is_none(),
            "should not include soft-deleted chats"
        );
    }
}

#[test]
fn chat_list_with_details_includes_unread_count() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let details = queries::list_chats_with_details(&conn, 0, 100, true).unwrap();

    assert!(!details.is_empty(), "should return chats with details");

    // At least some chats should have unread messages (messages from others without date_read)
    let any_unread = details.iter().any(|d| d.unread_count > 0);
    assert!(any_unread, "at least one chat should have unread messages");
}

#[test]
fn chat_count_returns_correct_value() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let count = queries::count_chats(&conn).unwrap();
    assert_eq!(count, 10, "should have 10 seeded chats");
}

#[test]
fn chat_search_finds_by_display_name() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let results = queries::search_chats(&conn, "Chat 5", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].display_name.as_deref(), Some("Chat 5"));
}

// ---- Message CRUD ----

#[test]
fn message_create_and_find_by_guid() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut chat = Chat::from_server_map(&serde_json::json!({"guid": "chat-msg-test"})).unwrap();
    let chat_id = chat.save(&conn).unwrap();

    let mut msg = Message::from_server_map(&serde_json::json!({
        "guid": "msg-find-test",
        "text": "hello world",
        "dateCreated": "2024-06-15T10:00:00Z",
        "isFromMe": true,
    }))
    .unwrap();
    msg.chat_id = Some(chat_id);
    msg.save(&conn).unwrap();

    let found = queries::find_message_by_guid(&conn, "msg-find-test")
        .unwrap()
        .expect("message should be found by guid");
    assert_eq!(found.text.as_deref(), Some("hello world"));
    assert!(found.is_from_me);
}

#[test]
fn cursor_based_pagination_with_100_messages() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut chat = Chat::from_server_map(&serde_json::json!({"guid": "chat-pagination"})).unwrap();
    let chat_id = chat.save(&conn).unwrap();

    // Insert 100 messages with sequential timestamps
    for i in 1..=100 {
        let mut msg = Message::from_server_map(&serde_json::json!({
            "guid": format!("pag-msg-{:03}", i),
            "text": format!("Message {}", i),
            "dateCreated": format!("2024-01-01T{:02}:{:02}:00Z", i / 60, i % 60),
            "isFromMe": true,
        }))
        .unwrap();
        msg.chat_id = Some(chat_id);
        msg.save(&conn).unwrap();
    }

    // Page 1: newest 25 messages (no cursor)
    let page1 = queries::messages_for_chat_cursor(&conn, chat_id, None, 25, SortDirection::Desc).unwrap();
    assert_eq!(page1.len(), 25, "first page should have 25 messages");

    // Verify they are ordered newest first
    for window in page1.windows(2) {
        assert!(
            window[0].date_created >= window[1].date_created,
            "messages should be sorted newest first"
        );
    }

    // Page 2: use cursor from last message of page 1
    let cursor1 = page1.last().unwrap().date_created.as_deref();
    let page2 = queries::messages_for_chat_cursor(&conn, chat_id, cursor1, 25, SortDirection::Desc).unwrap();
    assert_eq!(page2.len(), 25, "second page should have 25 messages");

    // Verify no overlap between pages
    let page1_guids: Vec<_> = page1.iter().map(|m| m.guid.as_deref().unwrap()).collect();
    let page2_guids: Vec<_> = page2.iter().map(|m| m.guid.as_deref().unwrap()).collect();
    for guid in &page2_guids {
        assert!(
            !page1_guids.contains(guid),
            "page 2 should not overlap with page 1"
        );
    }

    // Continue paging through all 100 messages
    let mut all_guids = page1_guids;
    all_guids.extend(page2_guids);

    let cursor2 = page2.last().unwrap().date_created.as_deref();
    let page3 = queries::messages_for_chat_cursor(&conn, chat_id, cursor2, 25, SortDirection::Desc).unwrap();
    all_guids.extend(page3.iter().map(|m| m.guid.as_deref().unwrap()));

    let cursor3 = page3.last().unwrap().date_created.as_deref();
    let page4 = queries::messages_for_chat_cursor(&conn, chat_id, cursor3, 25, SortDirection::Desc).unwrap();
    all_guids.extend(page4.iter().map(|m| m.guid.as_deref().unwrap()));

    assert_eq!(all_guids.len(), 100, "should have paged through all 100 messages");

    // Verify all unique
    let unique_count = {
        let mut set = std::collections::HashSet::new();
        all_guids.iter().for_each(|g| { set.insert(*g); });
        set.len()
    };
    assert_eq!(unique_count, 100, "all 100 messages should be unique across pages");
}

#[test]
fn message_search_by_text() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let results = queries::search_messages(&conn, "message number 5", 10).unwrap();
    assert!(
        !results.is_empty(),
        "search should find messages containing the query text"
    );
    for msg in &results {
        assert!(
            msg.text
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains("message number 5"),
            "all results should contain the search term"
        );
    }
}

#[test]
fn message_search_in_specific_chat() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    let chat_id = chat.id.unwrap();

    let results = queries::search_messages_in_chat(&conn, chat_id, "Test message", 100).unwrap();
    assert!(!results.is_empty());
    for msg in &results {
        assert_eq!(msg.chat_id, Some(chat_id), "results should be from the queried chat");
    }
}

#[test]
fn unread_count_for_chat() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    let chat_id = chat.id.unwrap();

    let unread = queries::unread_count_for_chat(&conn, chat_id).unwrap();
    // We have some received messages (is_from_me = false) without date_read
    assert!(unread >= 0, "unread count should be non-negative");
}

#[test]
fn total_unread_count() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let total = queries::total_unread_count(&conn).unwrap();
    assert!(total > 0, "should have some unread messages across all chats");
}

#[test]
fn latest_message_for_chat() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    let chat_id = chat.id.unwrap();

    let latest = queries::latest_message_for_chat(&conn, chat_id).unwrap();
    assert!(latest.is_some(), "chat with messages should have a latest message");
}

#[test]
fn message_count_for_chat() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    let count = queries::count_messages_for_chat(&conn, chat.id.unwrap()).unwrap();
    assert_eq!(count, 10, "each chat should have 10 messages from seeding");
}

// ---- Handle CRUD ----

#[test]
fn handle_create_and_find() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut handle = Handle::from_server_map(&serde_json::json!({
        "address": "+15559876543",
        "service": "iMessage",
    }))
    .unwrap();
    handle.save(&conn).unwrap();

    let found = queries::find_handle(&conn, "+15559876543", "iMessage")
        .unwrap()
        .expect("handle should be found");
    assert_eq!(found.address, "+15559876543");
    assert_eq!(found.service, "iMessage");
}

#[test]
fn handle_list_and_search() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();

    let all = queries::list_handles(&conn).unwrap();
    assert_eq!(all.len(), 15, "should have 15 seeded handles");

    // Handles are +1555000XXXX where XXXX = 0001..0015
    let results = queries::search_handles(&conn, "0010", 10).unwrap();
    assert!(!results.is_empty(), "should find handles matching partial address");
}

// ---- Attachment queries ----

#[test]
fn attachment_create_and_find_by_guid() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut chat = Chat::from_server_map(&serde_json::json!({"guid": "chat-att"})).unwrap();
    let chat_id = chat.save(&conn).unwrap();

    let mut msg = Message::from_server_map(&serde_json::json!({
        "guid": "msg-att",
        "dateCreated": "2024-01-01T00:00:00Z",
    }))
    .unwrap();
    msg.chat_id = Some(chat_id);
    let msg_id = msg.save(&conn).unwrap();

    let mut att = Attachment::from_server_map(&serde_json::json!({
        "guid": "att-find-test",
        "mimeType": "image/png",
        "transferName": "photo.png",
        "totalBytes": 2048,
    }))
    .unwrap();
    att.message_id = Some(msg_id);
    att.save(&conn).unwrap();

    let found = queries::find_attachment_by_guid(&conn, "att-find-test")
        .unwrap()
        .expect("attachment should be found");
    assert_eq!(found.mime_type.as_deref(), Some("image/png"));
    assert_eq!(found.transfer_name.as_deref(), Some("photo.png"));
}

#[test]
fn attachment_queries_by_message_chat_and_mime() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();

    // By message
    let atts = queries::load_attachments_for_message(&conn, 1).unwrap();
    assert_eq!(atts.len(), 1, "first message should have 1 attachment");

    // By chat (chat 1 has messages 1..=10 but only msg 1-5 have attachments in chat 1)
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-1").unwrap().unwrap();
    let chat_atts = queries::load_attachments_for_chat(&conn, chat.id.unwrap(), 100).unwrap();
    assert!(!chat_atts.is_empty(), "chat should have attachments");

    // By mime type prefix
    let images = queries::load_attachments_by_mime(&conn, "image/", 100).unwrap();
    assert!(!images.is_empty(), "should find image attachments");
    for att in &images {
        assert!(
            att.mime_type.as_deref().unwrap().starts_with("image/"),
            "filtered attachments should match mime prefix"
        );
    }
}

// ---- Contact CRUD ----

#[test]
fn contact_create_and_find_by_external_id() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut contact = Contact::from_server_map(&serde_json::json!({
        "id": "ext-abc",
        "displayName": "Alice Bob",
        "phoneNumbers": ["+15551112222"],
        "emails": ["alice@test.com"],
    }))
    .unwrap();
    contact.save(&conn).unwrap();

    let found = queries::find_contact_by_external_id(&conn, "ext-abc")
        .unwrap()
        .expect("contact should be found");
    assert_eq!(found.display_name, "Alice Bob");
}

#[test]
fn contact_phone_suffix_matching() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();

    // Phone numbers are +1555000XXXX where XXXX = 0001..0020
    // Use full phone for unique match
    let results = queries::search_contacts_by_phone_suffix(&conn, "+15550000001", 10).unwrap();
    assert!(results.len() >= 1, "should find at least one contact by full phone");
    assert!(
        results.iter().any(|c| c.display_name == "Contact 1"),
        "should include Contact 1"
    );

    // Search by full phone number with formatting
    let results = queries::search_contacts_by_phone_suffix(&conn, "+1 (555) 000-0005", 10).unwrap();
    assert!(results.len() >= 1, "should find contact by formatted phone");
    assert!(
        results.iter().any(|c| c.display_name == "Contact 5"),
        "should include Contact 5"
    );

    // Non-matching suffix
    let results = queries::search_contacts_by_phone_suffix(&conn, "9999999", 10).unwrap();
    assert!(results.is_empty(), "no contacts should match non-existent suffix");
}

#[test]
fn contact_search_by_display_name() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let results = queries::search_contacts(&conn, "Contact 1", 100).unwrap();
    // "Contact 1" matches Contact 1, Contact 10, Contact 11, ..., Contact 19
    assert!(results.len() >= 1, "should find contacts matching name pattern");
}

#[test]
fn contact_search_by_email() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let results = queries::search_contacts_by_email(&conn, "contact3@example.com", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].display_name, "Contact 3");
}

#[test]
fn delete_all_contacts() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let before = queries::list_contacts(&conn).unwrap().len();
    assert!(before > 0);

    queries::delete_all_contacts(&conn).unwrap();

    let after = queries::list_contacts(&conn).unwrap().len();
    assert_eq!(after, 0, "all contacts should be deleted");
}

// ---- Settings key-value store ----

#[test]
fn settings_string_crud() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    Settings::set(&conn, "myKey", "myValue").unwrap();
    assert_eq!(Settings::get(&conn, "myKey").unwrap(), Some("myValue".to_string()));

    Settings::set(&conn, "myKey", "updated").unwrap();
    assert_eq!(Settings::get(&conn, "myKey").unwrap(), Some("updated".to_string()));

    Settings::delete(&conn, "myKey").unwrap();
    assert_eq!(Settings::get(&conn, "myKey").unwrap(), None);
}

#[test]
fn settings_typed_bool_accessor() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    Settings::set_bool(&conn, "flag", true).unwrap();
    assert_eq!(Settings::get_bool(&conn, "flag").unwrap(), Some(true));

    Settings::set_bool(&conn, "flag", false).unwrap();
    assert_eq!(Settings::get_bool(&conn, "flag").unwrap(), Some(false));
}

#[test]
fn settings_typed_i64_accessor() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    Settings::set_i64(&conn, "count", 9999).unwrap();
    assert_eq!(Settings::get_i64(&conn, "count").unwrap(), Some(9999));
}

#[test]
fn settings_typed_f64_accessor() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    Settings::set_f64(&conn, "scale", 2.75).unwrap();
    let val = Settings::get_f64(&conn, "scale").unwrap().unwrap();
    assert!((val - 2.75).abs() < f64::EPSILON);
}

#[test]
fn settings_json_accessor() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let data = vec!["alpha", "beta", "gamma"];
    Settings::set_json(&conn, "list", &data).unwrap();
    let loaded: Vec<String> = Settings::get_json(&conn, "list").unwrap().unwrap();
    assert_eq!(loaded, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn settings_get_all_and_set_many() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    Settings::set_many(&conn, &[("a", "1"), ("b", "2"), ("c", "3")]).unwrap();
    let all = Settings::get_all(&conn).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all.get("b").unwrap(), "2");
}

#[test]
fn settings_clear_removes_all() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    Settings::set(&conn, "x", "1").unwrap();
    Settings::set(&conn, "y", "2").unwrap();
    Settings::clear(&conn).unwrap();

    let all = Settings::get_all(&conn).unwrap();
    assert!(all.is_empty(), "clear should remove all settings");
}

#[test]
fn settings_nonexistent_key_returns_none() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    assert_eq!(Settings::get(&conn, "nope").unwrap(), None);
    assert_eq!(Settings::get_bool(&conn, "nope").unwrap(), None);
    assert_eq!(Settings::get_i64(&conn, "nope").unwrap(), None);
    assert_eq!(Settings::get_f64(&conn, "nope").unwrap(), None);
}

// ---- Theme CRUD ----

#[test]
fn theme_save_find_and_delete() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    let mut theme = ThemeStruct {
        id: None,
        name: "Custom Night".to_string(),
        gradient_bg: true,
        google_font: "Roboto".to_string(),
        theme_data: r#"{"colorScheme":{"brightness":0,"primary":255}}"#.to_string(),
    };

    let id = theme.save(&conn).unwrap();
    assert!(id > 0);

    let found = ThemeStruct::find_by_name(&conn, "Custom Night").unwrap().unwrap();
    assert_eq!(found.name, "Custom Night");
    assert!(found.gradient_bg);
    assert_eq!(found.google_font, "Roboto");

    let deleted = ThemeStruct::delete_by_name(&conn, "Custom Night").unwrap();
    assert!(deleted);
    assert!(ThemeStruct::find_by_name(&conn, "Custom Night").unwrap().is_none());
}

#[test]
fn theme_preset_cannot_be_deleted() {
    let (db, _dir) = common::create_test_db();
    let conn = db.conn().unwrap();

    // Presets are seeded by migration
    let result = ThemeStruct::delete_by_name(&conn, "OLED Dark");
    assert!(result.is_err(), "deleting a preset theme should fail");
}

// ---- Database integrity check ----

#[test]
fn integrity_check_passes_on_fresh_db() {
    let (db, _dir) = common::create_test_db();
    assert!(db.run_integrity_check().is_ok());
}

#[test]
fn integrity_check_passes_after_seeding() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    assert!(db.run_integrity_check().is_ok());
}

// ---- Database stats ----

#[test]
fn database_stats_reflect_seeded_data() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let stats = db.stats().unwrap();
    assert_eq!(stats.chats, 10);
    assert_eq!(stats.messages, 100);
    assert_eq!(stats.handles, 15);
    assert_eq!(stats.attachments, 5);
    assert_eq!(stats.contacts, 20);
    assert_eq!(stats.settings, 3);
}

// ---- Database reset ----

#[test]
fn database_reset_clears_all_data() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    db.reset().unwrap();

    let stats = db.stats().unwrap();
    assert_eq!(stats.chats, 0);
    assert_eq!(stats.messages, 0);
    assert_eq!(stats.handles, 0);
    assert_eq!(stats.contacts, 0);
}

// ---- Database transaction ----

#[test]
fn transaction_commits_on_success() {
    let (db, _dir) = common::create_test_db();

    db.transaction(|conn| {
        conn.execute(
            "INSERT INTO handles (address, service, unique_address_service) VALUES ('tx@test.com', 'iMessage', 'tx@test.com/iMessage')",
            [],
        )
        .map_err(|e| bb_core::error::BbError::Database(e.to_string()))?;
        Ok(())
    })
    .unwrap();

    let conn = db.conn().unwrap();
    let found = queries::find_handle(&conn, "tx@test.com", "iMessage")
        .unwrap()
        .expect("handle should exist after committed transaction");
    assert_eq!(found.address, "tx@test.com");
}

#[test]
fn transaction_rolls_back_on_error() {
    let (db, _dir) = common::create_test_db();

    let result: bb_core::error::BbResult<()> = db.transaction(|conn| {
        conn.execute(
            "INSERT INTO handles (address, service, unique_address_service) VALUES ('rollback@test.com', 'iMessage', 'rollback@test.com/iMessage')",
            [],
        )
        .map_err(|e| bb_core::error::BbError::Database(e.to_string()))?;
        // Force an error
        Err(bb_core::error::BbError::Database("intentional error".into()))
    });

    assert!(result.is_err());

    let conn = db.conn().unwrap();
    let found = queries::find_handle(&conn, "rollback@test.com", "iMessage").unwrap();
    assert!(found.is_none(), "handle should not exist after rolled back transaction");
}

// ---- Chat participants ----

#[test]
fn chat_participants_loaded_correctly() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-1").unwrap().unwrap();
    let participants = queries::load_chat_participants(&conn, chat.id.unwrap()).unwrap();
    assert!(
        participants.len() >= 1,
        "chat should have at least one participant"
    );
}

// ---- Messages around date ----

#[test]
fn messages_around_date_returns_surrounding_messages() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);

    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-1").unwrap().unwrap();
    let chat_id = chat.id.unwrap();

    // Get the middle-ish message date
    let all_msgs = queries::list_messages_for_chat(&conn, chat_id, 0, 100, SortDirection::Asc).unwrap();
    let mid_date = all_msgs[all_msgs.len() / 2].date_created.as_deref().unwrap();

    let around = queries::messages_around_date(&conn, chat_id, mid_date, 3).unwrap();
    assert!(
        around.len() >= 2,
        "should return messages around the target date"
    );
}
