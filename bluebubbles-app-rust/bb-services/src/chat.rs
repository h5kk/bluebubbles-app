//! Chat service for managing conversations.
//!
//! Handles chat CRUD operations, participant management, read/unread status,
//! mute/unmute, soft delete, pin/archive, and chat search.

use tracing::{info, debug};
use bb_core::error::{BbError, BbResult};
use bb_models::{Database, Chat, Handle};
use bb_models::queries;
use bb_models::queries::ChatWithDetails;
use bb_api::ApiClient;

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Service for managing chat conversations.
///
/// Handles chat CRUD operations, participant management, read/unread status,
/// and chat search. Coordinates between the local database and the server API.
pub struct ChatService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
}

impl ChatService {
    /// Create a new ChatService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
        }
    }

    /// List chats from the local database.
    pub fn list_chats(&self, offset: i64, limit: i64, include_archived: bool) -> BbResult<Vec<Chat>> {
        let conn = self.database.conn()?;
        queries::list_chats(&conn, offset, limit, include_archived)
    }

    /// List chats with participant details (display names, handles).
    pub fn list_chats_with_details(
        &self,
        offset: i64,
        limit: i64,
        include_archived: bool,
    ) -> BbResult<Vec<ChatWithDetails>> {
        let conn = self.database.conn()?;
        queries::list_chats_with_details(&conn, offset, limit, include_archived)
    }

    /// Find a chat by GUID in the local database.
    pub fn find_chat(&self, guid: &str) -> BbResult<Option<Chat>> {
        let conn = self.database.conn()?;
        queries::find_chat_by_guid(&conn, guid)
    }

    /// Search chats by name or identifier.
    pub fn search_chats(&self, query: &str, limit: i64) -> BbResult<Vec<Chat>> {
        let conn = self.database.conn()?;
        queries::search_chats(&conn, query, limit)
    }

    /// Get the unread message count for a specific chat.
    pub fn unread_count(&self, chat_guid: &str) -> BbResult<i64> {
        let conn = self.database.conn()?;
        if let Some(chat) = queries::find_chat_by_guid(&conn, chat_guid)? {
            queries::unread_count_for_chat(&conn, chat.id.unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    /// Mark a chat as read both locally and on the server.
    pub async fn mark_read(&self, api: &ApiClient, guid: &str) -> BbResult<()> {
        api.mark_chat_read(guid).await?;

        let conn = self.database.conn()?;
        conn.execute(
            "UPDATE chats SET has_unread_message = 0 WHERE guid = ?1",
            rusqlite::params![guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        debug!("marked chat {guid} as read");
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Mark a chat as unread both locally and on the server.
    pub async fn mark_unread(&self, api: &ApiClient, guid: &str) -> BbResult<()> {
        api.mark_chat_unread(guid).await?;

        let conn = self.database.conn()?;
        conn.execute(
            "UPDATE chats SET has_unread_message = 1 WHERE guid = ?1",
            rusqlite::params![guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        debug!("marked chat {guid} as unread");
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Create a new chat via the server API and save it locally.
    pub async fn create_chat(
        &self,
        api: &ApiClient,
        addresses: Vec<String>,
        message: Option<String>,
        service: &str,
    ) -> BbResult<Chat> {
        let params = bb_api::endpoints::chats::CreateChatParams {
            addresses,
            message,
            service: service.to_string(),
            method: "private-api".to_string(),
        };

        let chat_json = api.create_chat(&params).await?;
        let mut chat = Chat::from_server_map(&chat_json)?;

        let conn = self.database.conn()?;
        chat.save(&conn)?;

        info!("created chat: {}", chat.guid);
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: chat.guid.clone(),
        });
        Ok(chat)
    }

    /// Update a chat's display name (group name).
    pub async fn rename_chat(&self, api: &ApiClient, guid: &str, name: &str) -> BbResult<()> {
        api.update_chat(guid, name).await?;

        let conn = self.database.conn()?;
        conn.execute(
            "UPDATE chats SET display_name = ?1 WHERE guid = ?2",
            rusqlite::params![name, guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        info!("renamed chat {guid} to {name}");
        self.event_bus.emit(AppEvent::GroupNameChanged {
            chat_guid: guid.to_string(),
            new_name: name.to_string(),
        });
        Ok(())
    }

    /// Get the total number of chats.
    pub fn count(&self) -> BbResult<i64> {
        let conn = self.database.conn()?;
        queries::count_chats(&conn)
    }

    /// Toggle the pinned state of a chat.
    pub fn toggle_pin(&self, guid: &str, pinned: bool) -> BbResult<()> {
        let conn = self.database.conn()?;
        conn.execute(
            "UPDATE chats SET is_pinned = ?1 WHERE guid = ?2",
            rusqlite::params![pinned as i32, guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        debug!("chat {guid} pinned={pinned}");
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Toggle the archived state of a chat.
    pub fn toggle_archive(&self, guid: &str, archived: bool) -> BbResult<()> {
        let conn = self.database.conn()?;
        conn.execute(
            "UPDATE chats SET is_archived = ?1 WHERE guid = ?2",
            rusqlite::params![archived as i32, guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        debug!("chat {guid} archived={archived}");
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Mute or unmute a chat locally.
    ///
    /// Sets `mute_type` to "mute" or null. The `mute_args` column can hold
    /// additional data such as a mute-until timestamp.
    pub fn set_muted(&self, guid: &str, muted: bool) -> BbResult<()> {
        let conn = self.database.conn()?;
        let mute_type: Option<&str> = if muted { Some("mute") } else { None };
        let mute_args: Option<&str> = None;
        conn.execute(
            "UPDATE chats SET mute_type = ?1, mute_args = ?2 WHERE guid = ?3",
            rusqlite::params![mute_type, mute_args, guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        debug!("chat {guid} muted={muted}");
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Temporarily mute a chat until the given timestamp.
    ///
    /// Stores the timestamp in `mute_args` as a string.
    pub fn mute_until(&self, guid: &str, until_ms: i64) -> BbResult<()> {
        let conn = self.database.conn()?;
        conn.execute(
            "UPDATE chats SET mute_type = 'mute', mute_args = ?1 WHERE guid = ?2",
            rusqlite::params![until_ms.to_string(), guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        debug!("chat {guid} muted until {until_ms}");
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Soft-delete a chat locally.
    ///
    /// Sets `date_deleted` to the current timestamp without removing it
    /// from the database. The server-side chat is unaffected.
    pub fn soft_delete(&self, guid: &str) -> BbResult<()> {
        let conn = self.database.conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE chats SET date_deleted = ?1 WHERE guid = ?2",
            rusqlite::params![now, guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        info!("soft-deleted chat: {guid}");
        self.event_bus.emit(AppEvent::ChatDeleted {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Restore a soft-deleted chat.
    pub fn restore_deleted(&self, guid: &str) -> BbResult<()> {
        let conn = self.database.conn()?;
        let null: Option<&str> = None;
        conn.execute(
            "UPDATE chats SET date_deleted = ?1 WHERE guid = ?2",
            rusqlite::params![null, guid],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        debug!("restored soft-deleted chat: {guid}");
        self.event_bus.emit(AppEvent::ChatUpdated {
            chat_guid: guid.to_string(),
        });
        Ok(())
    }

    /// Leave a group chat via the server API.
    pub async fn leave_chat(&self, api: &ApiClient, guid: &str) -> BbResult<()> {
        api.leave_chat(guid).await?;
        info!("left chat: {guid}");
        Ok(())
    }

    /// Get participants for a chat from the local database.
    pub fn get_participants(&self, chat_guid: &str) -> BbResult<Vec<Handle>> {
        let conn = self.database.conn()?;
        let chat = queries::find_chat_by_guid(&conn, chat_guid)?
            .ok_or_else(|| BbError::ChatNotFound(chat_guid.to_string()))?;
        let chat_id = chat.id.ok_or_else(|| BbError::ChatNotFound(chat_guid.to_string()))?;
        queries::load_chat_participants(&conn, chat_id)
    }

    /// Add a participant to a group chat via the server API.
    pub async fn add_participant(
        &self,
        api: &ApiClient,
        chat_guid: &str,
        address: &str,
    ) -> BbResult<()> {
        api.add_participant(chat_guid, address).await?;

        // Save the handle locally
        let conn = self.database.conn()?;
        let service = if address.contains('@') { "iMessage" } else { "iMessage" };
        let unique = format!("{}/{}", address, service);
        conn.execute(
            "INSERT OR IGNORE INTO handles (address, service, unique_address_service) VALUES (?1, ?2, ?3)",
            rusqlite::params![address, service, unique],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        info!("added participant {address} to chat {chat_guid}");
        self.event_bus.emit(AppEvent::ParticipantAdded {
            chat_guid: chat_guid.to_string(),
            address: address.to_string(),
        });
        Ok(())
    }

    /// Remove a participant from a group chat via the server API.
    pub async fn remove_participant(
        &self,
        api: &ApiClient,
        chat_guid: &str,
        address: &str,
    ) -> BbResult<()> {
        api.remove_participant(chat_guid, address).await?;

        info!("removed participant {address} from chat {chat_guid}");
        self.event_bus.emit(AppEvent::ParticipantRemoved {
            chat_guid: chat_guid.to_string(),
            address: address.to_string(),
        });
        Ok(())
    }

    /// Delete a chat from the server.
    pub async fn delete_chat(&self, api: &ApiClient, guid: &str) -> BbResult<()> {
        api.delete_chat(guid).await?;

        // Also soft-delete locally
        self.soft_delete(guid)?;
        info!("deleted chat: {guid}");
        Ok(())
    }
}

impl Service for ChatService {
    fn name(&self) -> &str { "chat" }
    fn state(&self) -> ServiceState { self.state }
    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("chat service initialized");
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
    fn test_chat_service_name() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = ChatService::new(db, bus);
        assert_eq!(svc.name(), "chat");
    }

    #[test]
    fn test_count_empty() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = ChatService::new(db, bus);
        assert_eq!(svc.count().unwrap(), 0);
    }

    #[test]
    fn test_toggle_pin() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let mut rx = bus.subscribe();
        let svc = ChatService::new(db.clone(), bus);

        // Insert a test chat first
        let conn = db.conn().unwrap();
        conn.execute(
            "INSERT INTO chats (guid, display_name, is_pinned) VALUES ('chat-1', 'Test', 0)",
            [],
        )
        .unwrap();

        svc.toggle_pin("chat-1", true).unwrap();

        // Verify event was emitted
        let event = rx.try_recv().unwrap();
        match event {
            AppEvent::ChatUpdated { chat_guid } => assert_eq!(chat_guid, "chat-1"),
            _ => panic!("expected ChatUpdated"),
        }
    }

    #[test]
    fn test_set_muted() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = ChatService::new(db.clone(), bus);

        let conn = db.conn().unwrap();
        conn.execute(
            "INSERT INTO chats (guid, display_name) VALUES ('chat-mute', 'Test')",
            [],
        )
        .unwrap();

        svc.set_muted("chat-mute", true).unwrap();
        svc.set_muted("chat-mute", false).unwrap();
    }

    #[test]
    fn test_soft_delete_and_restore() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = ChatService::new(db.clone(), bus);

        let conn = db.conn().unwrap();
        conn.execute(
            "INSERT INTO chats (guid, display_name) VALUES ('chat-del', 'Test')",
            [],
        )
        .unwrap();

        svc.soft_delete("chat-del").unwrap();
        svc.restore_deleted("chat-del").unwrap();
    }
}
