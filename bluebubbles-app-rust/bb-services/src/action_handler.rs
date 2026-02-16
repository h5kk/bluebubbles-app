//! Action handler that routes incoming socket events to appropriate services.
//!
//! This is the central coordinator for real-time events. It receives raw socket
//! events from the EventDispatcher, processes them (save to DB, resolve contacts),
//! and re-emits processed AppEvents through the EventBus.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, debug, error};

use bb_core::constants;
use bb_core::error::{BbError, BbResult};
use bb_models::Database;
use bb_socket::{EventDispatcher, SocketEvent, SocketEventType};

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Routes incoming socket events to the appropriate service logic.
///
/// Responsibilities:
/// - Receives raw socket events from the bb-socket EventDispatcher
/// - Deduplicates message events using a rolling GUID history
/// - Saves new/updated messages to the local database
/// - Emits processed AppEvents for UI and other services
/// - Handles group membership changes, typing indicators, FaceTime events
pub struct ActionHandler {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
    /// Rolling history of recently handled GUIDs for deduplication.
    handled_guids: Arc<Mutex<VecDeque<String>>>,
}

impl ActionHandler {
    /// Create a new ActionHandler.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
            handled_guids: Arc::new(Mutex::new(VecDeque::with_capacity(
                constants::MAX_HANDLED_GUID_HISTORY + 1,
            ))),
        }
    }

    /// Process a socket event, routing it to the appropriate handler.
    ///
    /// This is the main entry point called by the socket listener task.
    pub async fn handle_event(&self, event: SocketEvent) -> BbResult<()> {
        match event.event_type {
            SocketEventType::NewMessage => {
                self.handle_new_message(&event.data).await
            }
            SocketEventType::UpdatedMessage => {
                self.handle_updated_message(&event.data).await
            }
            SocketEventType::TypingIndicator => {
                self.handle_typing_indicator(&event.data)
            }
            SocketEventType::ChatReadStatusChanged => {
                self.handle_chat_read_status(&event.data)
            }
            SocketEventType::GroupNameChange => {
                self.handle_group_name_change(&event.data)
            }
            SocketEventType::ParticipantAdded => {
                self.handle_participant_change("added", &event.data)
            }
            SocketEventType::ParticipantRemoved => {
                self.handle_participant_change("removed", &event.data)
            }
            SocketEventType::ParticipantLeft => {
                self.handle_participant_change("left", &event.data)
            }
            SocketEventType::IncomingFaceTime => {
                self.handle_incoming_facetime(&event.data)
            }
            SocketEventType::FtCallStatusChanged => {
                self.handle_facetime_status(&event.data)
            }
            SocketEventType::IMessageAliasesRemoved => {
                self.handle_aliases_removed(&event.data)
            }
            _ => {
                debug!("unhandled event type: {:?}", event.event_type);
                Ok(())
            }
        }
    }

    /// Handle a new incoming message event.
    ///
    /// Flow:
    /// 1. Check deduplication history
    /// 2. Parse message and chat from JSON
    /// 3. Save/update chat in database
    /// 4. Save message in database
    /// 5. Emit MessageReceived event
    async fn handle_new_message(&self, data: &serde_json::Value) -> BbResult<()> {
        let guid = data
            .get("guid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if guid.is_empty() {
            warn!("new-message event missing guid");
            return Ok(());
        }

        // Deduplication check
        {
            let mut guids = self.handled_guids.lock().await;
            if guids.contains(&guid) {
                debug!("duplicate new-message skipped: {guid}");
                return Ok(());
            }
            guids.push_back(guid.clone());
            if guids.len() > constants::MAX_HANDLED_GUID_HISTORY {
                guids.pop_front();
            }
        }

        let is_from_me = data
            .get("isFromMe")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Extract chat GUID from the chats array
        let chat_guid = data
            .get("chats")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("guid"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Save the chat if present
        if !chat_guid.is_empty() {
            if let Some(chat_data) = data
                .get("chats")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
            {
                let conn = self.database.conn()?;
                if let Ok(mut chat) = bb_models::Chat::from_server_map(chat_data) {
                    if let Err(e) = chat.save(&conn) {
                        warn!("failed to save chat from new-message: {e}");
                    }
                }
            }
        }

        // Save the message
        let conn = self.database.conn()?;
        match bb_models::Message::from_server_map(data) {
            Ok(mut msg) => {
                // Resolve chat_id from guid
                if !chat_guid.is_empty() {
                    if let Ok(Some(chat)) =
                        bb_models::queries::find_chat_by_guid(&conn, &chat_guid)
                    {
                        msg.chat_id = chat.id;
                    }
                }

                // Save handle if present
                if let Some(handle_data) = data.get("handle") {
                    if let Ok(mut handle) = bb_models::Handle::from_server_map(handle_data) {
                        let _ = handle.save(&conn);
                        msg.handle_id = handle.id;
                    }
                }

                if let Err(e) = msg.save(&conn) {
                    warn!("failed to save message {guid}: {e}");
                } else {
                    debug!("saved incoming message: {guid}");
                }

                // Save attachments if present
                if let Some(attachments) = data.get("attachments").and_then(|v| v.as_array()) {
                    for att_json in attachments {
                        if let Ok(mut att) = bb_models::Attachment::from_server_map(att_json) {
                            att.message_id = msg.id;
                            let _ = att.save(&conn);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("failed to parse message from new-message event: {e}");
            }
        }

        // Emit processed event
        self.event_bus.emit(AppEvent::MessageReceived {
            message_guid: guid,
            chat_guid,
            is_from_me,
        });

        Ok(())
    }

    /// Handle an updated message event (delivered, read, edited, unsent).
    async fn handle_updated_message(&self, data: &serde_json::Value) -> BbResult<()> {
        let guid = data
            .get("guid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if guid.is_empty() {
            return Ok(());
        }

        // Deduplication with a different prefix so new + update both go through
        let dedup_key = format!("updated:{guid}");
        {
            let mut guids = self.handled_guids.lock().await;
            if guids.contains(&dedup_key) {
                debug!("duplicate updated-message skipped: {guid}");
                return Ok(());
            }
            guids.push_back(dedup_key);
            if guids.len() > constants::MAX_HANDLED_GUID_HISTORY {
                guids.pop_front();
            }
        }

        let chat_guid = data
            .get("chats")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("guid"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Update the message in the database
        let conn = self.database.conn()?;
        match bb_models::Message::from_server_map(data) {
            Ok(mut msg) => {
                // Try to find existing message to preserve chat_id
                if let Ok(Some(existing)) =
                    bb_models::queries::find_message_by_guid(&conn, &guid)
                {
                    msg.id = existing.id;
                    msg.chat_id = existing.chat_id;
                }

                if let Err(e) = msg.save(&conn) {
                    warn!("failed to update message {guid}: {e}");
                } else {
                    debug!("updated message: {guid}");
                }
            }
            Err(e) => {
                warn!("failed to parse updated-message event: {e}");
            }
        }

        self.event_bus.emit(AppEvent::MessageUpdated {
            message_guid: guid,
            chat_guid,
        });

        Ok(())
    }

    /// Handle a typing indicator event.
    fn handle_typing_indicator(&self, data: &serde_json::Value) -> BbResult<()> {
        let chat_guid = data
            .get("guid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let display = data
            .get("display")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !chat_guid.is_empty() {
            self.event_bus.emit(AppEvent::TypingChanged {
                chat_guid,
                is_typing: display,
            });
        }
        Ok(())
    }

    /// Handle chat read status changed on another device.
    fn handle_chat_read_status(&self, data: &serde_json::Value) -> BbResult<()> {
        let chat_guid = data
            .get("chatGuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let read = data
            .get("read")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !chat_guid.is_empty() && read {
            let conn = self.database.conn()?;
            conn.execute(
                "UPDATE chats SET has_unread_message = 0 WHERE guid = ?1",
                rusqlite::params![chat_guid],
            )
            .map_err(|e| BbError::Database(e.to_string()))?;

            self.event_bus.emit(AppEvent::ChatUpdated { chat_guid });
        }
        Ok(())
    }

    /// Handle a group name change event.
    fn handle_group_name_change(&self, data: &serde_json::Value) -> BbResult<()> {
        let chat_guid = data
            .get("chatGuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let new_name = data
            .get("newName")
            .or_else(|| data.get("displayName"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if !chat_guid.is_empty() {
            let conn = self.database.conn()?;
            conn.execute(
                "UPDATE chats SET display_name = ?1 WHERE guid = ?2",
                rusqlite::params![new_name, chat_guid],
            )
            .map_err(|e| BbError::Database(e.to_string()))?;

            self.event_bus.emit(AppEvent::GroupNameChanged {
                chat_guid,
                new_name,
            });
        }
        Ok(())
    }

    /// Handle participant added/removed/left events.
    fn handle_participant_change(
        &self,
        change_type: &str,
        data: &serde_json::Value,
    ) -> BbResult<()> {
        let chat_guid = data
            .get("chatGuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let address = data
            .get("handle")
            .or_else(|| data.get("address"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if chat_guid.is_empty() {
            return Ok(());
        }

        info!("participant {change_type}: {address} in {chat_guid}");

        match change_type {
            "added" => {
                self.event_bus.emit(AppEvent::ParticipantAdded {
                    chat_guid,
                    address,
                });
            }
            "removed" | "left" => {
                self.event_bus.emit(AppEvent::ParticipantRemoved {
                    chat_guid,
                    address,
                });
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle an incoming FaceTime call notification.
    fn handle_incoming_facetime(&self, data: &serde_json::Value) -> BbResult<()> {
        let call_uuid = data
            .get("uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let caller = data
            .get("handle")
            .and_then(|v| v.get("address"))
            .and_then(|v| v.as_str())
            .or_else(|| data.get("address").and_then(|v| v.as_str()))
            .unwrap_or("Unknown")
            .to_string();
        let is_audio = data
            .get("isAudio")
            .or_else(|| data.get("is_audio"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !call_uuid.is_empty() {
            info!("incoming FaceTime call from {caller} (audio: {is_audio})");
            self.event_bus.emit(AppEvent::IncomingFaceTime {
                call_uuid,
                caller,
                is_audio,
            });
        }
        Ok(())
    }

    /// Handle FaceTime call status change.
    fn handle_facetime_status(&self, data: &serde_json::Value) -> BbResult<()> {
        let call_uuid = data
            .get("uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let status = data
            .get("status_id")
            .or_else(|| data.get("statusId"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        if !call_uuid.is_empty() {
            self.event_bus.emit(AppEvent::FaceTimeStatusChanged {
                call_uuid,
                status,
            });
        }
        Ok(())
    }

    /// Handle iMessage aliases removed event.
    fn handle_aliases_removed(&self, data: &serde_json::Value) -> BbResult<()> {
        let aliases: Vec<String> = data
            .get("aliases")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        if !aliases.is_empty() {
            warn!("iMessage aliases removed: {:?}", aliases);
            self.event_bus.emit(AppEvent::AliasesRemoved { aliases });
        }
        Ok(())
    }

    /// Get a reference to the event bus for external subscriptions.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Start the background event listener that consumes socket events.
    ///
    /// Spawns a tokio task that subscribes to the socket EventDispatcher
    /// and routes each event through handle_event.
    pub fn start_listener(
        handler: Arc<ActionHandler>,
        dispatcher: &EventDispatcher,
    ) -> tokio::task::JoinHandle<()> {
        let mut rx = dispatcher.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = handler.handle_event(event).await {
                            error!("action handler error: {e}");
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("action handler lagged by {n} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("action handler: socket event channel closed");
                        break;
                    }
                }
            }
        })
    }
}

use tokio::sync::broadcast;

impl Service for ActionHandler {
    fn name(&self) -> &str {
        "action_handler"
    }
    fn state(&self) -> ServiceState {
        self.state
    }
    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("action handler initialized");
        Ok(())
    }
    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("action handler stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let config = bb_core::config::DatabaseConfig::default();
        // Leak the TempDir so it is not dropped before the test finishes
        let db = Database::init(&path, &config).unwrap();
        std::mem::forget(dir);
        db
    }

    #[test]
    fn test_action_handler_name() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let handler = ActionHandler::new(db, bus);
        assert_eq!(handler.name(), "action_handler");
    }

    #[tokio::test]
    async fn test_handle_typing_indicator() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        let handler = ActionHandler::new(db, bus);

        let event = SocketEvent {
            event_type: SocketEventType::TypingIndicator,
            data: serde_json::json!({"guid": "chat-1", "display": true}),
        };

        handler.handle_event(event).await.unwrap();

        let app_event = rx.recv().await.unwrap();
        match app_event {
            AppEvent::TypingChanged {
                chat_guid,
                is_typing,
            } => {
                assert_eq!(chat_guid, "chat-1");
                assert!(is_typing);
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn test_deduplication() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        let handler = ActionHandler::new(db, bus);

        let data = serde_json::json!({
            "guid": "msg-dup-1",
            "isFromMe": false,
            "chats": [{"guid": "chat-1"}],
            "dateCreated": "2024-01-01T00:00:00Z",
        });

        // First call should emit
        let event1 = SocketEvent {
            event_type: SocketEventType::NewMessage,
            data: data.clone(),
        };
        handler.handle_event(event1).await.unwrap();
        let _ = rx.recv().await.unwrap();

        // Second call should be deduplicated
        let event2 = SocketEvent {
            event_type: SocketEventType::NewMessage,
            data,
        };
        handler.handle_event(event2).await.unwrap();

        // Should timeout since no second event emitted
        let result =
            tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
        assert!(result.is_err());
    }
}
