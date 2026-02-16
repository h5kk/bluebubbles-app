//! Integration tests for service coordination.
//!
//! Tests EventBus publish/subscribe, ChatService local operations,
//! MessageService handling, ContactService phone matching, QueueService,
//! SettingsService, ThemeService, and ServiceRegistry initialization.

mod common;

use std::time::Duration;
use bb_core::config::ConfigHandle;
use bb_core::error::MessageError;
use bb_services::event_bus::AppEvent;
use bb_services::chat::ChatService;
use bb_services::message::MessageService;
use bb_services::contact::ContactService;
use bb_services::queue::{QueueService, QueuedMessage, SendStatus};
use bb_services::settings::SettingsService;
use bb_services::theme::ThemeService;
use bb_services::service::{Service, ServiceState};
use bb_services::registry::ServiceRegistry;
use bb_services::action_handler::ActionHandler;
use bb_socket::{SocketEvent, SocketEventType};

// ---- EventBus publish/subscribe ----

#[tokio::test]
async fn event_bus_single_subscriber_receives_event() {
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();

    bus.emit(AppEvent::ChatUpdated {
        chat_guid: "test-chat".into(),
    });

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::ChatUpdated { chat_guid } => assert_eq!(chat_guid, "test-chat"),
        _ => panic!("expected ChatUpdated event"),
    }
}

#[tokio::test]
async fn event_bus_multiple_subscribers_all_receive() {
    let bus = common::create_test_event_bus();
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();
    let mut rx3 = bus.subscribe();

    assert_eq!(bus.subscriber_count(), 3);

    bus.emit(AppEvent::ContactsUpdated { count: 42 });

    for rx in [&mut rx1, &mut rx2, &mut rx3] {
        let event = rx.recv().await.unwrap();
        match event {
            AppEvent::ContactsUpdated { count } => assert_eq!(count, 42),
            _ => panic!("all subscribers should receive the same event"),
        }
    }
}

#[tokio::test]
async fn event_bus_different_event_types() {
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();

    bus.emit(AppEvent::MessageReceived {
        message_guid: "msg-1".into(),
        chat_guid: "chat-1".into(),
        is_from_me: false,
    });

    bus.emit(AppEvent::TypingChanged {
        chat_guid: "chat-1".into(),
        is_typing: true,
    });

    bus.emit(AppEvent::ConnectionStateChanged {
        connected: true,
        message: "connected".into(),
    });

    let e1 = rx.recv().await.unwrap();
    let e2 = rx.recv().await.unwrap();
    let e3 = rx.recv().await.unwrap();

    assert!(matches!(e1, AppEvent::MessageReceived { .. }));
    assert!(matches!(e2, AppEvent::TypingChanged { .. }));
    assert!(matches!(e3, AppEvent::ConnectionStateChanged { .. }));
}

#[tokio::test]
async fn event_bus_no_subscribers_does_not_panic() {
    let bus = common::create_test_event_bus();
    // No subscribers, should not panic
    bus.emit(AppEvent::SyncComplete {
        is_full_sync: true,
        messages_synced: 500,
    });
    assert_eq!(bus.subscriber_count(), 0);
}

// ---- ChatService local operations ----

#[test]
fn chat_service_list_chats_empty() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let svc = ChatService::new(db, bus);

    let chats = svc.list_chats(0, 100, false).unwrap();
    assert!(chats.is_empty());
}

#[test]
fn chat_service_count() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ChatService::new(db, bus);

    assert_eq!(svc.count().unwrap(), 10);
}

#[test]
fn chat_service_find_chat() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ChatService::new(db, bus);

    let found = svc.find_chat("iMessage;-;chat-5").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().display_name.as_deref(), Some("Chat 5"));

    let not_found = svc.find_chat("nonexistent").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn chat_service_toggle_pin_emits_event() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let svc = ChatService::new(db, bus);

    svc.toggle_pin("iMessage;-;chat-5", true).unwrap();

    let event = rx.try_recv().unwrap();
    match event {
        AppEvent::ChatUpdated { chat_guid } => assert_eq!(chat_guid, "iMessage;-;chat-5"),
        _ => panic!("expected ChatUpdated"),
    }
}

#[test]
fn chat_service_toggle_archive_emits_event() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let svc = ChatService::new(db, bus);

    svc.toggle_archive("iMessage;-;chat-5", true).unwrap();

    let event = rx.try_recv().unwrap();
    assert!(matches!(event, AppEvent::ChatUpdated { .. }));
}

#[test]
fn chat_service_mute_and_unmute() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ChatService::new(db.clone(), bus);

    svc.set_muted("iMessage;-;chat-1", true).unwrap();

    let conn = db.conn().unwrap();
    let chat = bb_models::queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    assert_eq!(chat.mute_type.as_deref(), Some("mute"));

    svc.set_muted("iMessage;-;chat-1", false).unwrap();
    let chat = bb_models::queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    assert!(chat.mute_type.is_none());
}

#[test]
fn chat_service_soft_delete_and_restore() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let svc = ChatService::new(db.clone(), bus);

    svc.soft_delete("iMessage;-;chat-5").unwrap();

    let event = rx.try_recv().unwrap();
    assert!(matches!(event, AppEvent::ChatDeleted { .. }));

    let conn = db.conn().unwrap();
    let chat = bb_models::queries::find_chat_by_guid(&conn, "iMessage;-;chat-5")
        .unwrap()
        .unwrap();
    assert!(chat.date_deleted.is_some(), "chat should have date_deleted set");

    svc.restore_deleted("iMessage;-;chat-5").unwrap();

    let chat = bb_models::queries::find_chat_by_guid(&conn, "iMessage;-;chat-5")
        .unwrap()
        .unwrap();
    assert!(chat.date_deleted.is_none(), "chat should have date_deleted cleared");
}

#[test]
fn chat_service_search() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ChatService::new(db, bus);

    let results = svc.search_chats("Chat 7", 10).unwrap();
    assert_eq!(results.len(), 1);
}

// ---- MessageService operations ----

#[test]
fn message_service_handle_incoming_message() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let svc = MessageService::new(db.clone(), bus);

    // Insert a chat first
    let conn = db.conn().unwrap();
    conn.execute(
        "INSERT INTO chats (guid, display_name) VALUES ('chat-incoming', 'Incoming Test')",
        [],
    )
    .unwrap();

    let msg_json = serde_json::json!({
        "guid": "incoming-msg-1",
        "text": "Hello from the server",
        "dateCreated": "2024-06-15T12:00:00Z",
        "isFromMe": false,
    });

    let msg = svc.handle_incoming_message(&msg_json).unwrap();
    assert_eq!(msg.guid.as_deref(), Some("incoming-msg-1"));
    assert_eq!(msg.text.as_deref(), Some("Hello from the server"));

    // Verify it was saved to the database
    let found = svc.find_message("incoming-msg-1").unwrap();
    assert!(found.is_some());
}

#[test]
fn message_service_search() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = MessageService::new(db, bus);

    let results = svc.search_messages("Test message", 50).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn message_service_list_and_count() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = MessageService::new(db.clone(), bus);

    let conn = db.conn().unwrap();
    let chat = bb_models::queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    let chat_id = chat.id.unwrap();

    let messages = svc.list_messages(chat_id, 0, 100).unwrap();
    assert_eq!(messages.len(), 10);

    let count = svc.count_for_chat(chat_id).unwrap();
    assert_eq!(count, 10);
}

// ---- ContactService phone matching ----

#[test]
fn contact_service_find_by_phone_suffix() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ContactService::new(db, bus);

    // Last 7 digits: 5550001 from "+15550001xxxx"
    let found = svc.find_contact_by_phone("+15550000001").unwrap();
    assert!(found.is_some(), "should find contact by phone suffix");
    assert_eq!(found.unwrap().display_name, "Contact 1");
}

#[test]
fn contact_service_short_phone_returns_none() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let svc = ContactService::new(db, bus);

    let result = svc.find_contact_by_phone("123").unwrap();
    assert!(result.is_none(), "phone with fewer than 7 digits should return None");
}

#[test]
fn contact_service_resolve_display_name_by_phone() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ContactService::new(db, bus);

    // Phone numbers are +1555000XXXX, resolve_display_name uses phone suffix matching
    let name = svc.resolve_display_name("+15550000003").unwrap();
    assert_eq!(name.as_deref(), Some("Contact 3"));
}

#[test]
fn contact_service_resolve_display_name_email_returns_none_without_display_name_match() {
    // resolve_display_name for email addresses only searches by display_name,
    // so an email like "contact3@example.com" won't match display_name "Contact 3"
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ContactService::new(db, bus);

    let name = svc.resolve_display_name("contact3@example.com").unwrap();
    assert!(name.is_none(), "email resolve only searches display_name, not email field");
}

#[test]
fn contact_service_resolve_unknown_returns_none() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ContactService::new(db, bus);

    let name = svc.resolve_display_name("unknown@nobody.com").unwrap();
    assert!(name.is_none());
}

#[test]
fn contact_service_batch_resolve() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ContactService::new(db, bus);

    let addresses = vec![
        "contact1@example.com".to_string(),
        "unknown@nowhere.com".to_string(),
    ];
    let resolved = svc.resolve_batch(&addresses).unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].1, "Contact 1");
    // Unknown address should resolve to itself
    assert_eq!(resolved[1].1, "unknown@nowhere.com");
}

#[test]
fn contact_service_count() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let svc = ContactService::new(db, bus);

    assert_eq!(svc.count().unwrap(), 20);
}

// ---- QueueService ----

#[tokio::test]
async fn queue_enqueue_dequeue_lifecycle() {
    let svc = QueueService::new();

    let msg = QueuedMessage {
        id: "q-test-1".into(),
        chat_guid: "chat-1".into(),
        text: Some("retry me".into()),
        file_path: None,
        attempts: 0,
        max_attempts: 5,
        last_attempt: None,
    };

    svc.enqueue(msg).await;
    assert_eq!(svc.len().await, 1);
    assert!(!svc.is_empty().await);

    let dequeued = svc.dequeue().await.unwrap();
    assert_eq!(dequeued.id, "q-test-1");
    assert_eq!(dequeued.text.as_deref(), Some("retry me"));
    assert!(svc.is_empty().await);
}

#[tokio::test]
async fn queue_remove_by_id() {
    let svc = QueueService::new();

    svc.enqueue(QueuedMessage {
        id: "q-rm-1".into(),
        chat_guid: "c".into(),
        text: None,
        file_path: None,
        attempts: 0,
        max_attempts: 3,
        last_attempt: None,
    })
    .await;

    assert!(svc.remove("q-rm-1").await);
    assert!(svc.is_empty().await);
    assert!(!svc.remove("nonexistent").await);
}

#[tokio::test]
async fn queue_clear() {
    let svc = QueueService::new();

    for i in 0..5 {
        svc.enqueue(QueuedMessage {
            id: format!("q-clear-{}", i),
            chat_guid: "c".into(),
            text: None,
            file_path: None,
            attempts: 0,
            max_attempts: 3,
            last_attempt: None,
        })
        .await;
    }

    assert_eq!(svc.len().await, 5);
    svc.clear().await;
    assert!(svc.is_empty().await);
}

#[tokio::test]
async fn queue_guid_tracking_sent() {
    let svc = QueueService::new();

    svc.mark_sent("temp-guid-1", "real-guid-1").await;

    let status = svc.get_status("temp-guid-1").await.unwrap();
    assert!(matches!(status, SendStatus::Sent { real_guid } if real_guid == "real-guid-1"));

    let resolved = svc.resolve_guid("temp-guid-1").await;
    assert_eq!(resolved, Some("real-guid-1".to_string()));
}

#[tokio::test]
async fn queue_guid_tracking_failed() {
    let svc = QueueService::new();

    svc.mark_failed("temp-guid-2", "timeout error", MessageError::Timeout)
        .await;

    let status = svc.get_status("temp-guid-2").await.unwrap();
    match status {
        SendStatus::Failed { error, error_code } => {
            assert_eq!(error, "timeout error");
            assert_eq!(error_code, MessageError::Timeout);
        }
        _ => panic!("expected Failed status"),
    }

    // Failed GUIDs should not resolve
    assert!(svc.resolve_guid("temp-guid-2").await.is_none());
}

#[test]
fn queued_message_retry_delay_exponential_backoff() {
    let delays: Vec<Duration> = (0..6)
        .map(|attempts| {
            QueuedMessage {
                id: "t".into(),
                chat_guid: "c".into(),
                text: None,
                file_path: None,
                attempts,
                max_attempts: 10,
                last_attempt: None,
            }
            .retry_delay()
        })
        .collect();

    assert_eq!(delays[0], Duration::from_secs(1));
    assert_eq!(delays[1], Duration::from_secs(2));
    assert_eq!(delays[2], Duration::from_secs(4));
    assert_eq!(delays[3], Duration::from_secs(8));
    assert_eq!(delays[4], Duration::from_secs(16));
    assert_eq!(delays[5], Duration::from_secs(32));
}

#[test]
fn queued_message_should_retry_respects_max_attempts() {
    let under = QueuedMessage {
        id: "t".into(),
        chat_guid: "c".into(),
        text: None,
        file_path: None,
        attempts: 2,
        max_attempts: 5,
        last_attempt: None,
    };
    assert!(under.should_retry());

    let at_limit = QueuedMessage {
        attempts: 5,
        ..under.clone()
    };
    assert!(!at_limit.should_retry());
}

#[tokio::test]
async fn queue_stats() {
    let svc = QueueService::new();

    svc.enqueue(QueuedMessage {
        id: "q-stats-1".into(),
        chat_guid: "c".into(),
        text: Some("msg".into()),
        file_path: None,
        attempts: 0,
        max_attempts: 3,
        last_attempt: None,
    })
    .await;

    svc.mark_sent("t-1", "r-1").await;
    svc.mark_failed("t-2", "err", MessageError::Unknown).await;

    let stats = svc.stats().await;
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.sent_count, 1);
    assert_eq!(stats.failed_count, 1);
    assert!(stats.to_string().contains("pending=1"));
}

// ---- SettingsService typed accessors and persistence ----

#[tokio::test]
async fn settings_service_server_address() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let svc = SettingsService::new(config);

    assert!(svc.server_address().await.is_empty());

    svc.set_server_address("https://test.trycloudflare.com/".into())
        .await;
    // Should be sanitized (trailing slash removed)
    assert_eq!(
        svc.server_address().await,
        "https://test.trycloudflare.com"
    );
}

#[tokio::test]
async fn settings_service_notification_settings() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let svc = SettingsService::new(config);

    // Default: notify_reactions = true
    assert!(svc.notify_reactions().await);

    svc.set_notify_reactions(false).await;
    assert!(!svc.notify_reactions().await);
}

#[tokio::test]
async fn settings_service_display_settings() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let svc = SettingsService::new(config);

    assert_eq!(svc.user_name().await, "You");
    svc.set_user_name("TestUser".into()).await;
    assert_eq!(svc.user_name().await, "TestUser");

    assert!(!svc.use_24hr_format().await);
    svc.set_use_24hr_format(true).await;
    assert!(svc.use_24hr_format().await);

    assert!(!svc.redacted_mode().await);
    svc.set_redacted_mode(true).await;
    assert!(svc.redacted_mode().await);
}

#[tokio::test]
async fn settings_service_sync_settings() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let svc = SettingsService::new(config);

    assert!(!svc.is_setup_complete().await);
    svc.mark_setup_complete().await;
    assert!(svc.is_setup_complete().await);

    assert_eq!(svc.messages_per_page().await, 25);
    svc.set_messages_per_page(50).await;
    assert_eq!(svc.messages_per_page().await, 50);
}

#[tokio::test]
async fn settings_service_custom_headers() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let svc = SettingsService::new(config);

    assert!(svc.custom_headers().await.is_empty());

    svc.set_custom_header("X-Api-Key".into(), "abc123".into())
        .await;
    let headers = svc.custom_headers().await;
    assert_eq!(headers.get("X-Api-Key").unwrap(), "abc123");

    svc.remove_custom_header("X-Api-Key").await;
    assert!(svc.custom_headers().await.is_empty());
}

#[tokio::test]
async fn settings_service_export_import_preserves_credentials() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    {
        let mut c = config.write().await;
        c.server.address = "https://original-server.com".into();
        c.server.guid_auth_key = "secret-auth-key".into();
    }

    let svc = SettingsService::new(config);

    // Export
    let json = svc.export_as_json().await;
    assert!(json.is_object());

    // Import different config that tries to overwrite credentials
    let import_data = serde_json::json!({
        "server": {
            "address": "https://attacker.com",
            "guid_auth_key": "stolen-key"
        },
        "display": {
            "user_name": "Imported"
        }
    });

    svc.import_from_json(&import_data).await.unwrap();

    // Credentials should be preserved
    assert_eq!(svc.server_address().await, "https://original-server.com");
    assert_eq!(svc.guid_auth_key().await, "secret-auth-key");
    // Non-credential settings should be imported
    assert_eq!(svc.user_name().await, "Imported");
}

// ---- ThemeService ----

#[test]
fn theme_service_ensure_presets_creates_both_themes() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let svc = ThemeService::new(db, bus);
    svc.ensure_presets().unwrap();

    let themes = svc.list_themes().unwrap();
    let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"OLED Dark"));
    assert!(names.contains(&"Bright White"));
}

#[test]
fn theme_service_save_find_delete() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let svc = ThemeService::new(db, bus);

    let theme = bb_models::ThemeStruct {
        id: None,
        name: "My Custom".to_string(),
        gradient_bg: false,
        google_font: "Default".to_string(),
        theme_data: "{}".to_string(),
    };

    let id = svc.save_theme(theme).unwrap();
    assert!(id > 0);

    let found = svc.find_theme("My Custom").unwrap();
    assert!(found.is_some());

    let deleted = svc.delete_theme("My Custom").unwrap();
    assert!(deleted);
    assert!(svc.find_theme("My Custom").unwrap().is_none());
}

#[test]
fn theme_service_apply_emits_event() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let mut svc = ThemeService::new(db, bus);
    svc.ensure_presets().unwrap();

    let theme = svc.apply_theme("Bright White").unwrap();
    assert_eq!(theme.name, "Bright White");
    assert_eq!(svc.active_theme_name(), "Bright White");

    let event = rx.try_recv().unwrap();
    match event {
        AppEvent::ThemeChanged { theme_name } => assert_eq!(theme_name, "Bright White"),
        _ => panic!("expected ThemeChanged event"),
    }
}

#[test]
fn theme_service_apply_nonexistent_fails() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut svc = ThemeService::new(db, bus);

    let result = svc.apply_theme("Does Not Exist");
    assert!(result.is_err());
}

#[test]
fn theme_service_cannot_delete_preset() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let svc = ThemeService::new(db, bus);
    svc.ensure_presets().unwrap();

    let result = svc.delete_theme("OLED Dark");
    assert!(result.is_err());
}

// ---- ServiceRegistry initialization ----

#[tokio::test]
async fn service_registry_registers_all_default_services() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let (db, dir) = common::create_test_db();
    let dispatcher = common::create_test_dispatcher();

    let mut registry = ServiceRegistry::new(config, db, dispatcher);
    registry.register_all(dir.path().join("cache"));

    assert!(
        registry.service_count() >= 11,
        "should register at least 11 default services, got {}",
        registry.service_count()
    );
}

#[tokio::test]
async fn service_registry_init_and_shutdown_all() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let (db, dir) = common::create_test_db();
    let dispatcher = common::create_test_dispatcher();

    let mut registry = ServiceRegistry::new(config, db, dispatcher);
    registry.register_all(dir.path().join("cache"));

    registry.init_all().await.unwrap();

    let health = registry.health_check().await;
    for (name, state, healthy) in &health {
        assert!(
            *healthy,
            "service '{}' should be healthy after init (state: {})",
            name, state
        );
    }

    registry.shutdown_all().await.unwrap();
}

#[tokio::test]
async fn service_registry_api_client_not_configured_initially() {
    let config = ConfigHandle::new(bb_core::config::AppConfig::default());
    let (db, _dir) = common::create_test_db();
    let dispatcher = common::create_test_dispatcher();

    let registry = ServiceRegistry::new(config, db, dispatcher);

    let result = registry.api_client().await;
    assert!(
        result.is_err(),
        "API client should not be available before configuration"
    );
}

// ---- Service trait lifecycle ----

#[test]
fn service_lifecycle_created_to_running_to_stopped() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut svc = ChatService::new(db, bus);

    assert_eq!(svc.state(), ServiceState::Created);
    assert!(!svc.is_healthy());

    svc.init().unwrap();
    assert_eq!(svc.state(), ServiceState::Running);
    assert!(svc.is_healthy());

    svc.shutdown().unwrap();
    assert_eq!(svc.state(), ServiceState::Stopped);
    assert!(!svc.is_healthy());
}

// ---- ActionHandler event routing ----

#[tokio::test]
async fn action_handler_routes_typing_indicator() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let event = SocketEvent {
        event_type: SocketEventType::TypingIndicator,
        data: serde_json::json!({"guid": "iMessage;-;chat-1", "display": true}),
    };

    handler.handle_event(event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::TypingChanged {
            chat_guid,
            is_typing,
        } => {
            assert_eq!(chat_guid, "iMessage;-;chat-1");
            assert!(is_typing);
        }
        _ => panic!("expected TypingChanged event"),
    }
}

#[tokio::test]
async fn action_handler_routes_new_message_and_saves_to_db() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    let event = SocketEvent {
        event_type: SocketEventType::NewMessage,
        data: serde_json::json!({
            "guid": "new-msg-from-socket",
            "text": "Hello from socket",
            "isFromMe": false,
            "dateCreated": "2024-06-15T10:00:00Z",
            "chats": [{"guid": "iMessage;-;+15551234567"}],
        }),
    };

    handler.handle_event(event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::MessageReceived {
            message_guid,
            chat_guid,
            is_from_me,
        } => {
            assert_eq!(message_guid, "new-msg-from-socket");
            assert_eq!(chat_guid, "iMessage;-;+15551234567");
            assert!(!is_from_me);
        }
        _ => panic!("expected MessageReceived event"),
    }

    // Verify message was saved
    let conn = db.conn().unwrap();
    let msg = bb_models::queries::find_message_by_guid(&conn, "new-msg-from-socket")
        .unwrap()
        .expect("message should be saved to database");
    assert_eq!(msg.text.as_deref(), Some("Hello from socket"));
}

#[tokio::test]
async fn action_handler_deduplicates_messages() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let data = serde_json::json!({
        "guid": "dedup-msg-1",
        "isFromMe": false,
        "dateCreated": "2024-06-15T10:00:00Z",
        "chats": [{"guid": "chat-dedup"}],
    });

    // First event should be processed
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::NewMessage,
            data: data.clone(),
        })
        .await
        .unwrap();
    let _ = rx.recv().await.unwrap();

    // Second identical event should be deduplicated
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::NewMessage,
            data,
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "duplicate event should not produce a second AppEvent"
    );
}

#[tokio::test]
async fn action_handler_routes_group_name_change() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    let event = SocketEvent {
        event_type: SocketEventType::GroupNameChange,
        data: serde_json::json!({
            "chatGuid": "iMessage;-;chat-1",
            "newName": "Renamed Group",
        }),
    };

    handler.handle_event(event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::GroupNameChanged {
            chat_guid,
            new_name,
        } => {
            assert_eq!(chat_guid, "iMessage;-;chat-1");
            assert_eq!(new_name, "Renamed Group");
        }
        _ => panic!("expected GroupNameChanged event"),
    }

    // Verify DB was updated
    let conn = db.conn().unwrap();
    let chat = bb_models::queries::find_chat_by_guid(&conn, "iMessage;-;chat-1")
        .unwrap()
        .unwrap();
    assert_eq!(chat.display_name.as_deref(), Some("Renamed Group"));
}

#[tokio::test]
async fn action_handler_routes_participant_added() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let event = SocketEvent {
        event_type: SocketEventType::ParticipantAdded,
        data: serde_json::json!({
            "chatGuid": "chat-group-1",
            "handle": "+15559999999",
        }),
    };

    handler.handle_event(event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::ParticipantAdded {
            chat_guid,
            address,
        } => {
            assert_eq!(chat_guid, "chat-group-1");
            assert_eq!(address, "+15559999999");
        }
        _ => panic!("expected ParticipantAdded event"),
    }
}

#[tokio::test]
async fn action_handler_routes_incoming_facetime() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let event = SocketEvent {
        event_type: SocketEventType::IncomingFaceTime,
        data: serde_json::json!({
            "uuid": "ft-call-123",
            "handle": {"address": "+15551234567"},
            "isAudio": true,
        }),
    };

    handler.handle_event(event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::IncomingFaceTime {
            call_uuid,
            caller,
            is_audio,
        } => {
            assert_eq!(call_uuid, "ft-call-123");
            assert_eq!(caller, "+15551234567");
            assert!(is_audio);
        }
        _ => panic!("expected IncomingFaceTime event"),
    }
}

#[tokio::test]
async fn action_handler_routes_aliases_removed() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let event = SocketEvent {
        event_type: SocketEventType::IMessageAliasesRemoved,
        data: serde_json::json!({
            "aliases": ["user@icloud.com", "+15551234567"],
        }),
    };

    handler.handle_event(event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::AliasesRemoved { aliases } => {
            assert_eq!(aliases.len(), 2);
            assert!(aliases.contains(&"user@icloud.com".to_string()));
        }
        _ => panic!("expected AliasesRemoved event"),
    }
}
