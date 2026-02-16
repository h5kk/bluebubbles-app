//! Message service for sending, receiving, and managing messages.
//!
//! Handles text/attachment/reaction sending with temp GUID management,
//! message editing, unsending, retry via the outgoing queue, and
//! incoming message processing.

use std::path::Path;
use tracing::{info, debug, warn};
use bb_core::error::{BbError, BbResult, MessageError};
use bb_models::{Database, Message};
use bb_models::queries;
use bb_api::ApiClient;
use bb_api::endpoints::messages::{SendTextParams, SendReactionParams, EditMessageParams};

use crate::event_bus::{AppEvent, EventBus};
use crate::queue::{QueueService, QueuedMessage};
use crate::service::{Service, ServiceState};

/// Service for managing messages.
///
/// Handles message sending (text, attachment, reaction), receiving,
/// editing, unsending, and local database operations. Manages temp
/// GUIDs for optimistic UI updates and integrates with the queue
/// service for retry logic.
pub struct MessageService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
}

impl MessageService {
    /// Create a new MessageService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
        }
    }

    /// List messages for a chat from the local database.
    pub fn list_messages(
        &self,
        chat_id: i64,
        offset: i64,
        limit: i64,
    ) -> BbResult<Vec<Message>> {
        let conn = self.database.conn()?;
        queries::list_messages_for_chat(&conn, chat_id, offset, limit, queries::SortDirection::Desc)
    }

    /// Find a message by GUID.
    pub fn find_message(&self, guid: &str) -> BbResult<Option<Message>> {
        let conn = self.database.conn()?;
        queries::find_message_by_guid(&conn, guid)
    }

    /// Search messages by text content.
    pub fn search_messages(&self, query: &str, limit: i64) -> BbResult<Vec<Message>> {
        let conn = self.database.conn()?;
        queries::search_messages(&conn, query, limit)
    }

    /// Send a text message via the server API.
    ///
    /// Creates a temporary message in the local database immediately for
    /// optimistic UI, then sends via the server. On success, replaces the
    /// temp GUID with the real server GUID. On failure, marks the message
    /// as errored and optionally enqueues for retry.
    pub async fn send_text(
        &self,
        api: &ApiClient,
        chat_guid: &str,
        text: &str,
        method: &str,
        effect_id: Option<String>,
        subject: Option<String>,
        reply_guid: Option<String>,
    ) -> BbResult<Message> {
        let temp_guid = format!("temp-{}", uuid::Uuid::new_v4());

        let params = SendTextParams {
            chat_guid: chat_guid.to_string(),
            temp_guid: temp_guid.clone(),
            message: text.to_string(),
            method: method.to_string(),
            effect_id,
            subject,
            selected_message_guid: reply_guid,
            part_index: None,
            dd_scan: None,
        };

        // Create temp message in local DB for optimistic UI
        let mut temp_msg = Message::from_server_map(&serde_json::json!({
            "guid": temp_guid,
            "text": text,
            "isFromMe": true,
            "dateCreated": chrono::Utc::now().to_rfc3339(),
        }))?;

        {
            let conn = self.database.conn()?;
            if let Some(chat) = queries::find_chat_by_guid(&conn, chat_guid)? {
                temp_msg.chat_id = chat.id;
            }
            temp_msg.save(&conn)?;
        }

        debug!("sending text message (temp_guid: {temp_guid})");

        // Send via API
        match api.send_text(&params).await {
            Ok(msg_json) => {
                let mut msg = Message::from_server_map(&msg_json)?;
                let conn = self.database.conn()?;

                // Replace the temp GUID with the real GUID
                if let Some(real_guid) = msg.guid.as_deref() {
                    conn.execute(
                        "DELETE FROM messages WHERE guid = ?1",
                        rusqlite::params![temp_guid],
                    )
                    .map_err(|e| BbError::Database(e.to_string()))?;

                    // Resolve chat_id for the real message
                    if let Some(chat) = queries::find_chat_by_guid(&conn, chat_guid)? {
                        msg.chat_id = chat.id;
                    }

                    self.event_bus.emit(AppEvent::MessageSent {
                        temp_guid: temp_guid.clone(),
                        real_guid: real_guid.to_string(),
                        chat_guid: chat_guid.to_string(),
                    });
                }

                msg.save(&conn)?;
                info!("message sent: {:?}", msg.guid);
                Ok(msg)
            }
            Err(e) => {
                // Mark temp message as errored
                let conn = self.database.conn()?;
                let error_guid = format!("error-{temp_guid}");
                conn.execute(
                    "UPDATE messages SET guid = ?1 WHERE guid = ?2",
                    rusqlite::params![error_guid, temp_guid],
                )
                .map_err(|e2| BbError::Database(e2.to_string()))?;

                self.event_bus.emit(AppEvent::MessageFailed {
                    temp_guid,
                    chat_guid: chat_guid.to_string(),
                    error: e.to_string(),
                });

                warn!("failed to send message: {e}");
                Err(e)
            }
        }
    }

    /// Send a text message with automatic retry via the queue service.
    ///
    /// On failure, the message is enqueued for retry with exponential backoff.
    pub async fn send_text_with_retry(
        &self,
        api: &ApiClient,
        queue: &QueueService,
        chat_guid: &str,
        text: &str,
        method: &str,
    ) -> BbResult<Message> {
        match self
            .send_text(api, chat_guid, text, method, None, None, None)
            .await
        {
            Ok(msg) => Ok(msg),
            Err(e) => {
                let error_code = classify_send_error(&e);
                if error_code.should_retry() {
                    let queued = QueuedMessage {
                        id: format!("q-{}", uuid::Uuid::new_v4()),
                        chat_guid: chat_guid.to_string(),
                        text: Some(text.to_string()),
                        file_path: None,
                        attempts: 1,
                        max_attempts: 5,
                        last_attempt: Some(std::time::Instant::now()),
                    };
                    queue.enqueue(queued).await;
                    debug!("message enqueued for retry");
                }
                Err(e)
            }
        }
    }

    /// Send an attachment via the server API.
    ///
    /// Reads the file, uploads it, and returns the sent message.
    pub async fn send_attachment(
        &self,
        api: &ApiClient,
        chat_guid: &str,
        file_path: &Path,
        mime_type: &str,
        method: &str,
    ) -> BbResult<Message> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        let file_bytes = std::fs::read(file_path)?;
        let temp_guid = format!("temp-{}", uuid::Uuid::new_v4());

        debug!("sending attachment: {file_name} ({} bytes)", file_bytes.len());

        let msg_json = api
            .send_attachment(chat_guid, &temp_guid, file_name, file_bytes, mime_type, method)
            .await?;

        let mut msg = Message::from_server_map(&msg_json)?;
        let conn = self.database.conn()?;

        if let Some(chat) = queries::find_chat_by_guid(&conn, chat_guid)? {
            msg.chat_id = chat.id;
        }
        msg.save(&conn)?;

        // Save attachments from the response
        if let Some(attachments) = msg_json.get("attachments").and_then(|v| v.as_array()) {
            for att_json in attachments {
                if let Ok(mut att) = bb_models::Attachment::from_server_map(att_json) {
                    att.message_id = msg.id;
                    let _ = att.save(&conn);
                }
            }
        }

        if let Some(real_guid) = msg.guid.as_deref() {
            self.event_bus.emit(AppEvent::MessageSent {
                temp_guid,
                real_guid: real_guid.to_string(),
                chat_guid: chat_guid.to_string(),
            });
        }

        info!("attachment sent: {:?}", msg.guid);
        Ok(msg)
    }

    /// Send a reaction / tapback.
    pub async fn send_reaction(
        &self,
        api: &ApiClient,
        chat_guid: &str,
        message_guid: &str,
        message_text: &str,
        reaction: &str,
        part_index: Option<i32>,
    ) -> BbResult<Message> {
        let params = SendReactionParams {
            chat_guid: chat_guid.to_string(),
            selected_message_text: message_text.to_string(),
            selected_message_guid: message_guid.to_string(),
            reaction: reaction.to_string(),
            part_index,
        };

        let msg_json = api.send_reaction(&params).await?;
        let mut msg = Message::from_server_map(&msg_json)?;

        let conn = self.database.conn()?;
        msg.save(&conn)?;

        info!("reaction sent: {reaction} on {message_guid}");
        Ok(msg)
    }

    /// Edit a previously sent message.
    pub async fn edit_message(
        &self,
        api: &ApiClient,
        guid: &str,
        new_text: &str,
        part_index: i32,
    ) -> BbResult<Message> {
        let params = EditMessageParams {
            edited_message: new_text.to_string(),
            backwards_compatibility_message: format!("Edited to \"{new_text}\""),
            part_index,
        };

        let msg_json = api.edit_message(guid, &params).await?;
        let mut msg = Message::from_server_map(&msg_json)?;

        let conn = self.database.conn()?;
        msg.save(&conn)?;

        info!("message edited: {guid}");
        self.event_bus.emit(AppEvent::MessageUpdated {
            message_guid: guid.to_string(),
            chat_guid: String::new(), // chat_guid not available from edit response
        });
        Ok(msg)
    }

    /// Unsend a message.
    pub async fn unsend_message(
        &self,
        api: &ApiClient,
        guid: &str,
        part_index: i32,
    ) -> BbResult<()> {
        api.unsend_message(guid, part_index).await?;

        let conn = self.database.conn()?;
        conn.execute(
            "UPDATE messages SET date_deleted = ?1 WHERE guid = ?2",
            rusqlite::params![chrono::Utc::now().to_rfc3339(), guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        info!("message unsent: {guid}");
        self.event_bus.emit(AppEvent::MessageUpdated {
            message_guid: guid.to_string(),
            chat_guid: String::new(),
        });
        Ok(())
    }

    /// Retry sending a failed message from the queue.
    ///
    /// Looks up the queued message details, re-sends, and either removes from
    /// queue on success or re-enqueues with incremented attempt count.
    pub async fn retry_failed(
        &self,
        api: &ApiClient,
        queue: &QueueService,
        msg: QueuedMessage,
    ) -> BbResult<()> {
        if let Some(text) = &msg.text {
            match self
                .send_text(api, &msg.chat_guid, text, "private-api", None, None, None)
                .await
            {
                Ok(_) => {
                    queue.remove(&msg.id).await;
                    info!("retry succeeded for queue item: {}", msg.id);
                }
                Err(e) => {
                    let error_code = classify_send_error(&e);
                    if msg.should_retry() && error_code.should_retry() {
                        let retry_msg = QueuedMessage {
                            attempts: msg.attempts + 1,
                            last_attempt: Some(std::time::Instant::now()),
                            ..msg
                        };
                        queue.enqueue(retry_msg).await;
                        debug!("re-enqueued failed message for retry");
                    } else {
                        queue.remove(&msg.id).await;
                        warn!("message retry exhausted: {}", msg.id);
                    }
                }
            }
        } else if msg.file_path.is_some() {
            // Attachment retries would go here
            warn!("attachment retry not yet supported for queue item: {}", msg.id);
            queue.remove(&msg.id).await;
        }
        Ok(())
    }

    /// Process an incoming message event from the socket.
    pub fn handle_incoming_message(&self, msg_json: &serde_json::Value) -> BbResult<Message> {
        let mut msg = Message::from_server_map(msg_json)?;
        let conn = self.database.conn()?;
        msg.save(&conn)?;
        debug!("incoming message saved: {:?}", msg.guid);
        Ok(msg)
    }

    /// Get the message count for a chat.
    pub fn count_for_chat(&self, chat_id: i64) -> BbResult<i64> {
        let conn = self.database.conn()?;
        queries::count_messages_for_chat(&conn, chat_id)
    }
}

/// Classify a send error into a MessageError code for retry decisions.
fn classify_send_error(error: &BbError) -> MessageError {
    match error {
        BbError::Timeout(_) => MessageError::Timeout,
        BbError::Socket(_) | BbError::SocketDisconnected => MessageError::NoConnection,
        BbError::ServerError { status, .. } if *status >= 500 => MessageError::ServerError,
        BbError::ServerError { status, .. } if *status == 400 => MessageError::BadRequest,
        BbError::ServerError { status, .. } if *status == 403 => {
            MessageError::NoAccessToConversation
        }
        BbError::SendFailed(_) => MessageError::FailedToSend,
        _ => MessageError::Unknown,
    }
}

/// Extension trait for MessageError to determine retry eligibility.
trait RetryEligible {
    fn should_retry(&self) -> bool;
}

impl RetryEligible for MessageError {
    fn should_retry(&self) -> bool {
        matches!(
            self,
            MessageError::Timeout
                | MessageError::NoConnection
                | MessageError::ServerError
                | MessageError::Unknown
        )
    }
}

impl Service for MessageService {
    fn name(&self) -> &str { "message" }
    fn state(&self) -> ServiceState { self.state }
    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("message service initialized");
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

    fn create_test_db() -> Database {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&path, &config).unwrap();
        std::mem::forget(dir);
        db
    }

    #[test]
    fn test_message_service_name() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = MessageService::new(db, bus);
        assert_eq!(svc.name(), "message");
    }

    #[test]
    fn test_classify_send_error() {
        assert_eq!(
            classify_send_error(&BbError::Timeout("timeout".into())),
            MessageError::Timeout
        );
        assert_eq!(
            classify_send_error(&BbError::SocketDisconnected),
            MessageError::NoConnection
        );
        assert_eq!(
            classify_send_error(&BbError::ServerError {
                status: 500,
                message: "internal".into()
            }),
            MessageError::ServerError
        );
        assert_eq!(
            classify_send_error(&BbError::ServerError {
                status: 400,
                message: "bad".into()
            }),
            MessageError::BadRequest
        );
    }

    #[test]
    fn test_retry_eligibility() {
        assert!(MessageError::Timeout.should_retry());
        assert!(MessageError::NoConnection.should_retry());
        assert!(MessageError::ServerError.should_retry());
        assert!(!MessageError::BadRequest.should_retry());
        assert!(!MessageError::NoAccessToConversation.should_retry());
        assert!(!MessageError::FailedToSend.should_retry());
    }
}
