//! End-to-end event flow integration tests.
//!
//! Tests the complete event pipeline: SocketEvent -> ActionHandler -> DB save -> AppEvent,
//! typing indicator flow, updated message flow, chat read status flow,
//! participant change flow, FaceTime flow, and EventDispatcher -> ActionHandler integration.

mod common;

use std::sync::Arc;
use std::time::Duration;

use bb_models::queries;
use bb_services::event_bus::AppEvent;
use bb_services::action_handler::ActionHandler;
use bb_services::service::{Service, ServiceState};
use bb_socket::{SocketEvent, SocketEventType};

// ---- Full incoming message pipeline ----

#[tokio::test]
async fn e2e_incoming_message_saves_chat_message_and_emits_event() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    let event = SocketEvent {
        event_type: SocketEventType::NewMessage,
        data: serde_json::json!({
            "guid": "e2e-msg-001",
            "text": "End-to-end test message",
            "isFromMe": false,
            "dateCreated": "2024-07-01T10:30:00Z",
            "chats": [{
                "guid": "iMessage;-;+15559876543",
                "chatIdentifier": "+15559876543",
                "displayName": "E2E Test Chat",
            }],
            "handle": {
                "address": "+15559876543",
                "service": "iMessage",
            },
        }),
    };

    handler.handle_event(event).await.unwrap();

    // 1. Verify the AppEvent was emitted
    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::MessageReceived {
            message_guid,
            chat_guid,
            is_from_me,
        } => {
            assert_eq!(message_guid, "e2e-msg-001");
            assert_eq!(chat_guid, "iMessage;-;+15559876543");
            assert!(!is_from_me);
        }
        _ => panic!("expected MessageReceived event"),
    }

    // 2. Verify the chat was saved to the database
    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;+15559876543")
        .unwrap()
        .expect("chat should be saved from incoming message event");
    assert_eq!(chat.display_name.as_deref(), Some("E2E Test Chat"));

    // 3. Verify the message was saved to the database
    let msg = queries::find_message_by_guid(&conn, "e2e-msg-001")
        .unwrap()
        .expect("message should be saved from incoming message event");
    assert_eq!(msg.text.as_deref(), Some("End-to-end test message"));
    assert!(!msg.is_from_me);
}

#[tokio::test]
async fn e2e_incoming_message_with_attachment_saves_attachment() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let handler = ActionHandler::new(db.clone(), bus);

    let event = SocketEvent {
        event_type: SocketEventType::NewMessage,
        data: serde_json::json!({
            "guid": "e2e-msg-att-001",
            "text": "Photo message",
            "isFromMe": false,
            "dateCreated": "2024-07-01T11:00:00Z",
            "chats": [{"guid": "iMessage;-;+15559876543"}],
            "attachments": [{
                "guid": "att-e2e-001",
                "mimeType": "image/jpeg",
                "transferName": "photo.jpg",
                "totalBytes": 4096,
            }],
        }),
    };

    handler.handle_event(event).await.unwrap();

    let conn = db.conn().unwrap();
    let att = queries::find_attachment_by_guid(&conn, "att-e2e-001")
        .unwrap()
        .expect("attachment should be saved from incoming message event");
    assert_eq!(att.mime_type.as_deref(), Some("image/jpeg"));
    assert_eq!(att.transfer_name.as_deref(), Some("photo.jpg"));
}

#[tokio::test]
async fn e2e_incoming_from_me_message_sets_is_from_me() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    let event = SocketEvent {
        event_type: SocketEventType::NewMessage,
        data: serde_json::json!({
            "guid": "e2e-from-me-001",
            "text": "Sent from another device",
            "isFromMe": true,
            "dateCreated": "2024-07-01T12:00:00Z",
            "chats": [{"guid": "iMessage;-;+15551112222"}],
        }),
    };

    handler.handle_event(event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::MessageReceived { is_from_me, .. } => {
            assert!(is_from_me, "is_from_me should be true for sent messages");
        }
        _ => panic!("expected MessageReceived"),
    }
}

// ---- Updated message pipeline ----

#[tokio::test]
async fn e2e_updated_message_emits_message_updated_event() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    // First, create the original message
    let new_event = SocketEvent {
        event_type: SocketEventType::NewMessage,
        data: serde_json::json!({
            "guid": "e2e-update-orig",
            "text": "Original text",
            "isFromMe": false,
            "dateCreated": "2024-07-01T10:00:00Z",
            "chats": [{"guid": "iMessage;-;chat-update"}],
        }),
    };
    handler.handle_event(new_event).await.unwrap();
    let _ = rx.recv().await.unwrap(); // consume MessageReceived

    // Now send an update event
    let update_event = SocketEvent {
        event_type: SocketEventType::UpdatedMessage,
        data: serde_json::json!({
            "guid": "e2e-update-orig",
            "text": "Edited text",
            "isFromMe": false,
            "dateCreated": "2024-07-01T10:00:00Z",
            "dateEdited": "2024-07-01T10:05:00Z",
            "chats": [{"guid": "iMessage;-;chat-update"}],
        }),
    };
    handler.handle_event(update_event).await.unwrap();

    let app_event = rx.recv().await.unwrap();
    match app_event {
        AppEvent::MessageUpdated {
            message_guid,
            chat_guid,
        } => {
            assert_eq!(message_guid, "e2e-update-orig");
            assert_eq!(chat_guid, "iMessage;-;chat-update");
        }
        _ => panic!("expected MessageUpdated event"),
    }
}

// ---- Typing indicator pipeline ----

#[tokio::test]
async fn e2e_typing_indicator_start_and_stop() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    // Start typing
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::TypingIndicator,
            data: serde_json::json!({"guid": "iMessage;-;+15551234567", "display": true}),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::TypingChanged {
            chat_guid,
            is_typing,
        } => {
            assert_eq!(chat_guid, "iMessage;-;+15551234567");
            assert!(is_typing);
        }
        _ => panic!("expected TypingChanged start"),
    }

    // Stop typing
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::TypingIndicator,
            data: serde_json::json!({"guid": "iMessage;-;+15551234567", "display": false}),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::TypingChanged {
            chat_guid,
            is_typing,
        } => {
            assert_eq!(chat_guid, "iMessage;-;+15551234567");
            assert!(!is_typing);
        }
        _ => panic!("expected TypingChanged stop"),
    }
}

#[tokio::test]
async fn e2e_typing_indicator_with_empty_guid_is_ignored() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::TypingIndicator,
            data: serde_json::json!({"guid": "", "display": true}),
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "empty guid typing indicator should not emit any event"
    );
}

// ---- Chat read status pipeline ----

#[tokio::test]
async fn e2e_chat_read_status_updates_database_and_emits_event() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    // Mark chat 1 as having unread messages first
    {
        let conn = db.conn().unwrap();
        conn.execute(
            "UPDATE chats SET has_unread_message = 1 WHERE guid = 'iMessage;-;chat-1'",
            [],
        )
        .unwrap();
    }

    // Send read status event
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::ChatReadStatusChanged,
            data: serde_json::json!({"chatGuid": "iMessage;-;chat-1", "read": true}),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::ChatUpdated { chat_guid } => {
            assert_eq!(chat_guid, "iMessage;-;chat-1");
        }
        _ => panic!("expected ChatUpdated event from read status change"),
    }

    // Verify the DB was updated
    let conn = db.conn().unwrap();
    let has_unread: i32 = conn
        .query_row(
            "SELECT has_unread_message FROM chats WHERE guid = 'iMessage;-;chat-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(has_unread, 0, "has_unread_message should be 0 after read status update");
}

#[tokio::test]
async fn e2e_chat_read_status_false_does_not_emit() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::ChatReadStatusChanged,
            data: serde_json::json!({"chatGuid": "iMessage;-;chat-1", "read": false}),
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "read=false should not emit an event"
    );
}

// ---- Group name change pipeline ----

#[tokio::test]
async fn e2e_group_name_change_updates_db_and_emits() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::GroupNameChange,
            data: serde_json::json!({
                "chatGuid": "iMessage;-;chat-2",
                "newName": "The Best Group",
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::GroupNameChanged {
            chat_guid,
            new_name,
        } => {
            assert_eq!(chat_guid, "iMessage;-;chat-2");
            assert_eq!(new_name, "The Best Group");
        }
        _ => panic!("expected GroupNameChanged"),
    }

    // Verify the DB was updated
    let conn = db.conn().unwrap();
    let chat = queries::find_chat_by_guid(&conn, "iMessage;-;chat-2")
        .unwrap()
        .unwrap();
    assert_eq!(chat.display_name.as_deref(), Some("The Best Group"));
}

#[tokio::test]
async fn e2e_group_name_change_with_display_name_fallback() {
    let (db, _dir) = common::create_test_db();
    common::seed_test_data(&db);
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db.clone(), bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::GroupNameChange,
            data: serde_json::json!({
                "chatGuid": "iMessage;-;chat-3",
                "displayName": "Fallback Name",
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::GroupNameChanged { new_name, .. } => {
            assert_eq!(new_name, "Fallback Name");
        }
        _ => panic!("expected GroupNameChanged"),
    }
}

// ---- Participant change pipeline ----

#[tokio::test]
async fn e2e_participant_added_emits_correct_event() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::ParticipantAdded,
            data: serde_json::json!({
                "chatGuid": "iMessage;-;group-chat",
                "handle": "+15559999999",
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::ParticipantAdded {
            chat_guid,
            address,
        } => {
            assert_eq!(chat_guid, "iMessage;-;group-chat");
            assert_eq!(address, "+15559999999");
        }
        _ => panic!("expected ParticipantAdded"),
    }
}

#[tokio::test]
async fn e2e_participant_removed_emits_correct_event() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::ParticipantRemoved,
            data: serde_json::json!({
                "chatGuid": "iMessage;-;group-chat",
                "handle": "+15558888888",
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::ParticipantRemoved {
            chat_guid,
            address,
        } => {
            assert_eq!(chat_guid, "iMessage;-;group-chat");
            assert_eq!(address, "+15558888888");
        }
        _ => panic!("expected ParticipantRemoved"),
    }
}

#[tokio::test]
async fn e2e_participant_left_emits_participant_removed_event() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::ParticipantLeft,
            data: serde_json::json!({
                "chatGuid": "iMessage;-;group-chat",
                "address": "+15557777777",
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::ParticipantRemoved {
            chat_guid,
            address,
        } => {
            assert_eq!(chat_guid, "iMessage;-;group-chat");
            assert_eq!(address, "+15557777777");
        }
        _ => panic!("expected ParticipantRemoved from participant-left"),
    }
}

#[tokio::test]
async fn e2e_participant_change_with_empty_chat_guid_is_ignored() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::ParticipantAdded,
            data: serde_json::json!({
                "chatGuid": "",
                "handle": "+15559999999",
            }),
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "participant event with empty chatGuid should not emit"
    );
}

// ---- FaceTime event pipeline ----

#[tokio::test]
async fn e2e_incoming_facetime_audio_call() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::IncomingFaceTime,
            data: serde_json::json!({
                "uuid": "ft-e2e-audio",
                "handle": {"address": "+15551234567"},
                "isAudio": true,
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::IncomingFaceTime {
            call_uuid,
            caller,
            is_audio,
        } => {
            assert_eq!(call_uuid, "ft-e2e-audio");
            assert_eq!(caller, "+15551234567");
            assert!(is_audio);
        }
        _ => panic!("expected IncomingFaceTime"),
    }
}

#[tokio::test]
async fn e2e_incoming_facetime_video_call() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::IncomingFaceTime,
            data: serde_json::json!({
                "uuid": "ft-e2e-video",
                "handle": {"address": "+15559876543"},
                "isAudio": false,
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::IncomingFaceTime {
            is_audio, ..
        } => {
            assert!(!is_audio, "video call should have is_audio = false");
        }
        _ => panic!("expected IncomingFaceTime"),
    }
}

#[tokio::test]
async fn e2e_facetime_call_status_changed() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::FtCallStatusChanged,
            data: serde_json::json!({
                "uuid": "ft-status-e2e",
                "status_id": 6,
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::FaceTimeStatusChanged {
            call_uuid,
            status,
        } => {
            assert_eq!(call_uuid, "ft-status-e2e");
            assert_eq!(status, 6, "status 6 = call ended");
        }
        _ => panic!("expected FaceTimeStatusChanged"),
    }
}

#[tokio::test]
async fn e2e_incoming_facetime_with_empty_uuid_is_ignored() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::IncomingFaceTime,
            data: serde_json::json!({
                "uuid": "",
                "handle": {"address": "+15551234567"},
            }),
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "FaceTime event with empty uuid should not emit"
    );
}

// ---- Aliases removed pipeline ----

#[tokio::test]
async fn e2e_aliases_removed_emits_with_all_aliases() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::IMessageAliasesRemoved,
            data: serde_json::json!({
                "aliases": ["user@icloud.com", "+15551234567", "user@me.com"],
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::AliasesRemoved { aliases } => {
            assert_eq!(aliases.len(), 3);
            assert!(aliases.contains(&"user@icloud.com".to_string()));
            assert!(aliases.contains(&"+15551234567".to_string()));
            assert!(aliases.contains(&"user@me.com".to_string()));
        }
        _ => panic!("expected AliasesRemoved"),
    }
}

#[tokio::test]
async fn e2e_aliases_removed_empty_list_is_ignored() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::IMessageAliasesRemoved,
            data: serde_json::json!({
                "aliases": [],
            }),
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "empty aliases list should not emit an event"
    );
}

// ---- Deduplication robustness ----

#[tokio::test]
async fn e2e_deduplication_allows_same_guid_for_new_and_update() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    // Send new message
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::NewMessage,
            data: serde_json::json!({
                "guid": "dedup-both-001",
                "text": "original",
                "isFromMe": false,
                "dateCreated": "2024-07-01T10:00:00Z",
                "chats": [{"guid": "iMessage;-;chat-dedup"}],
            }),
        })
        .await
        .unwrap();

    let event1 = rx.recv().await.unwrap();
    assert!(matches!(event1, AppEvent::MessageReceived { .. }));

    // Same GUID but as an update should still go through (different dedup prefix)
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::UpdatedMessage,
            data: serde_json::json!({
                "guid": "dedup-both-001",
                "text": "updated",
                "isFromMe": false,
                "dateCreated": "2024-07-01T10:00:00Z",
                "chats": [{"guid": "iMessage;-;chat-dedup"}],
            }),
        })
        .await
        .unwrap();

    let event2 = rx.recv().await.unwrap();
    assert!(
        matches!(event2, AppEvent::MessageUpdated { .. }),
        "update of same GUID should not be deduplicated against the new-message"
    );
}

#[tokio::test]
async fn e2e_deduplication_blocks_duplicate_new_message() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let data = serde_json::json!({
        "guid": "dedup-repeat-001",
        "text": "first arrival",
        "isFromMe": false,
        "dateCreated": "2024-07-01T10:00:00Z",
        "chats": [{"guid": "iMessage;-;chat-dedup"}],
    });

    // First send
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::NewMessage,
            data: data.clone(),
        })
        .await
        .unwrap();
    let _ = rx.recv().await.unwrap();

    // Duplicate send
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
        "duplicate new-message with same GUID should be blocked"
    );
}

#[tokio::test]
async fn e2e_deduplication_blocks_duplicate_updated_message() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let data = serde_json::json!({
        "guid": "dedup-upd-repeat",
        "text": "updated text",
        "isFromMe": false,
        "dateCreated": "2024-07-01T10:00:00Z",
        "chats": [{"guid": "iMessage;-;chat-dedup"}],
    });

    // First update
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::UpdatedMessage,
            data: data.clone(),
        })
        .await
        .unwrap();
    let _ = rx.recv().await.unwrap();

    // Duplicate update
    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::UpdatedMessage,
            data,
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "duplicate updated-message with same GUID should be blocked"
    );
}

// ---- New message edge cases ----

#[tokio::test]
async fn e2e_new_message_with_empty_guid_is_ignored() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::NewMessage,
            data: serde_json::json!({
                "guid": "",
                "text": "no guid",
                "isFromMe": false,
                "dateCreated": "2024-07-01T10:00:00Z",
                "chats": [{"guid": "iMessage;-;chat-1"}],
            }),
        })
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        result.is_err(),
        "message with empty guid should be silently ignored"
    );
}

#[tokio::test]
async fn e2e_new_message_without_chats_still_emits() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::NewMessage,
            data: serde_json::json!({
                "guid": "no-chat-msg",
                "text": "orphan message",
                "isFromMe": false,
                "dateCreated": "2024-07-01T10:00:00Z",
            }),
        })
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::MessageReceived { chat_guid, .. } => {
            assert!(
                chat_guid.is_empty(),
                "message without chats array should have empty chat_guid"
            );
        }
        _ => panic!("expected MessageReceived"),
    }
}

// ---- Unknown event type is handled gracefully ----

#[tokio::test]
async fn e2e_unknown_event_type_does_not_error() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut rx = bus.subscribe();
    let handler = ActionHandler::new(db, bus);

    let result = handler
        .handle_event(SocketEvent {
            event_type: SocketEventType::ServerUpdate,
            data: serde_json::json!({"version": "2.0.0"}),
        })
        .await;

    assert!(result.is_ok(), "unknown/unhandled event types should not error");

    let timeout = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(
        timeout.is_err(),
        "unhandled event type should not emit any AppEvent"
    );
}

// ---- EventDispatcher -> ActionHandler integration ----

#[tokio::test]
async fn e2e_dispatcher_to_handler_pipeline() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut app_rx = bus.subscribe();
    let handler = Arc::new(ActionHandler::new(db.clone(), bus));
    let dispatcher = common::create_test_dispatcher();

    // Start the listener that bridges dispatcher -> handler
    let _listener = ActionHandler::start_listener(Arc::clone(&handler), &dispatcher);

    // Dispatch a socket event
    dispatcher.dispatch(SocketEvent {
        event_type: SocketEventType::TypingIndicator,
        data: serde_json::json!({"guid": "iMessage;-;+15551234567", "display": true}),
    });

    // The event should flow through: dispatcher -> handler -> event_bus -> app_rx
    let app_event = tokio::time::timeout(Duration::from_secs(2), app_rx.recv())
        .await
        .expect("should receive event within timeout")
        .unwrap();

    match app_event {
        AppEvent::TypingChanged {
            chat_guid,
            is_typing,
        } => {
            assert_eq!(chat_guid, "iMessage;-;+15551234567");
            assert!(is_typing);
        }
        _ => panic!("expected TypingChanged from dispatcher pipeline"),
    }
}

#[tokio::test]
async fn e2e_dispatcher_to_handler_new_message_saves_to_db() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut app_rx = bus.subscribe();
    let handler = Arc::new(ActionHandler::new(db.clone(), bus));
    let dispatcher = common::create_test_dispatcher();

    let _listener = ActionHandler::start_listener(Arc::clone(&handler), &dispatcher);

    dispatcher.dispatch(SocketEvent {
        event_type: SocketEventType::NewMessage,
        data: serde_json::json!({
            "guid": "dispatch-msg-001",
            "text": "From dispatcher",
            "isFromMe": false,
            "dateCreated": "2024-07-15T10:00:00Z",
            "chats": [{"guid": "iMessage;-;chat-dispatch"}],
        }),
    });

    let app_event = tokio::time::timeout(Duration::from_secs(2), app_rx.recv())
        .await
        .expect("should receive event within timeout")
        .unwrap();

    assert!(matches!(app_event, AppEvent::MessageReceived { .. }));

    // Give a moment for the DB write to complete
    tokio::time::sleep(Duration::from_millis(50)).await;

    let conn = db.conn().unwrap();
    let msg = queries::find_message_by_guid(&conn, "dispatch-msg-001")
        .unwrap()
        .expect("message from dispatcher pipeline should be saved to DB");
    assert_eq!(msg.text.as_deref(), Some("From dispatcher"));
}

// ---- ActionHandler service lifecycle ----

#[test]
fn action_handler_service_lifecycle() {
    let (db, _dir) = common::create_test_db();
    let bus = common::create_test_event_bus();
    let mut handler = ActionHandler::new(db, bus);

    assert_eq!(handler.name(), "action_handler");
    assert_eq!(handler.state(), ServiceState::Created);
    assert!(!handler.is_healthy());

    handler.init().unwrap();
    assert_eq!(handler.state(), ServiceState::Running);
    assert!(handler.is_healthy());

    handler.shutdown().unwrap();
    assert_eq!(handler.state(), ServiceState::Stopped);
    assert!(!handler.is_healthy());
}
