//! Chat (conversation) entity model.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Row};
use bb_core::error::{BbError, BbResult};

/// Represents a chat/conversation in the BlueBubbles system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: Option<i64>,
    pub original_rowid: Option<i64>,
    pub guid: String,
    pub chat_identifier: Option<String>,
    pub display_name: Option<String>,
    pub is_archived: bool,
    pub mute_type: Option<String>,
    pub mute_args: Option<String>,
    pub is_pinned: bool,
    pub has_unread_message: bool,
    pub pin_index: Option<i64>,
    pub auto_send_read_receipts: Option<bool>,
    pub auto_send_typing_indicators: Option<bool>,
    pub text_field_text: Option<String>,
    /// JSON array of draft attachment paths.
    pub text_field_attachments: String,
    pub latest_message_date: Option<String>,
    pub date_deleted: Option<String>,
    pub style: Option<i32>,
    pub lock_chat_name: bool,
    pub lock_chat_icon: bool,
    pub last_read_message_guid: Option<String>,
    pub custom_avatar_path: Option<String>,

    /// Transient: loaded participants (not stored directly on this table).
    #[serde(default)]
    pub participants: Vec<super::handle::Handle>,

    /// Transient: latest message (loaded separately).
    #[serde(skip)]
    pub latest_message: Option<Box<super::message::Message>>,
}

impl Chat {
    /// Create a Chat from a server JSON map.
    pub fn from_server_map(map: &serde_json::Value) -> BbResult<Self> {
        let guid = map["guid"]
            .as_str()
            .ok_or_else(|| BbError::Serialization("chat missing guid".into()))?
            .to_string();

        let participants = if let Some(arr) = map.get("participants").and_then(|v| v.as_array()) {
            arr.iter()
                .filter_map(|h| super::handle::Handle::from_server_map(h).ok())
                .collect()
        } else {
            vec![]
        };

        Ok(Self {
            id: None,
            original_rowid: map.get("ROWID").and_then(|v| v.as_i64())
                .or_else(|| map.get("originalROWID").and_then(|v| v.as_i64())),
            guid,
            chat_identifier: map.get("chatIdentifier").and_then(|v| v.as_str()).map(String::from),
            display_name: map.get("displayName").and_then(|v| v.as_str()).and_then(|s| {
                if s.is_empty() { None } else { Some(String::from(s)) }
            }),
            is_archived: map.get("isArchived").and_then(|v| v.as_bool()).unwrap_or(false),
            mute_type: map.get("muteType").and_then(|v| v.as_str()).map(String::from),
            mute_args: map.get("muteArgs").and_then(|v| v.as_str()).map(String::from),
            is_pinned: map.get("isPinned").and_then(|v| v.as_bool()).unwrap_or(false),
            has_unread_message: map.get("hasUnreadMessage").and_then(|v| v.as_bool()).unwrap_or(false),
            pin_index: map.get("pinIndex").and_then(|v| v.as_i64()),
            auto_send_read_receipts: map.get("autoSendReadReceipts").and_then(|v| v.as_bool()),
            auto_send_typing_indicators: map.get("autoSendTypingIndicators").and_then(|v| v.as_bool()),
            text_field_text: None,
            text_field_attachments: "[]".to_string(),
            latest_message_date: map.get("lastMessageDate").and_then(|v| v.as_str()).map(String::from)
                .or_else(|| {
                    // Server sends lastMessage as a full message object; extract dateCreated
                    map.get("lastMessage")
                        .and_then(|lm| lm.get("dateCreated"))
                        .and_then(|v| v.as_i64())
                        .map(|ts| ts.to_string())
                })
                .or_else(|| {
                    // Also try direct numeric lastMessageDate
                    map.get("lastMessageDate").and_then(|v| v.as_i64()).map(|ts| ts.to_string())
                }),
            date_deleted: map.get("dateDeleted").and_then(|v| v.as_str()).map(String::from),
            style: map.get("style").and_then(|v| v.as_i64()).map(|v| v as i32),
            lock_chat_name: false,
            lock_chat_icon: false,
            last_read_message_guid: map.get("lastReadMessageGuid").and_then(|v| v.as_str()).map(String::from),
            custom_avatar_path: None,
            participants,
            latest_message: None,
        })
    }

    /// Construct a Chat from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            original_rowid: row.get("original_rowid")?,
            guid: row.get("guid")?,
            chat_identifier: row.get("chat_identifier")?,
            display_name: row.get("display_name")?,
            is_archived: row.get::<_, i32>("is_archived")? != 0,
            mute_type: row.get("mute_type")?,
            mute_args: row.get("mute_args")?,
            is_pinned: row.get::<_, i32>("is_pinned")? != 0,
            has_unread_message: row.get::<_, i32>("has_unread_message")? != 0,
            pin_index: row.get("pin_index")?,
            auto_send_read_receipts: row.get::<_, Option<i32>>("auto_send_read_receipts")?
                .map(|v| v != 0),
            auto_send_typing_indicators: row.get::<_, Option<i32>>("auto_send_typing_indicators")?
                .map(|v| v != 0),
            text_field_text: row.get("text_field_text")?,
            text_field_attachments: row.get::<_, String>("text_field_attachments")
                .unwrap_or_else(|_| "[]".to_string()),
            latest_message_date: row.get("latest_message_date")?,
            date_deleted: row.get("date_deleted")?,
            style: row.get("style")?,
            lock_chat_name: row.get::<_, i32>("lock_chat_name")? != 0,
            lock_chat_icon: row.get::<_, i32>("lock_chat_icon")? != 0,
            last_read_message_guid: row.get("last_read_message_guid")?,
            custom_avatar_path: row.get("custom_avatar_path")?,
            participants: vec![],
            latest_message: None,
        })
    }

    // ─── Static finders ──────────────────────────────────────────────────

    /// Find a chat by its local database ID.
    pub fn find_by_id(conn: &Connection, id: i64) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM chats WHERE id = ?1", [id], Self::from_row) {
            Ok(chat) => Ok(Some(chat)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find a chat by its GUID.
    pub fn find_by_guid(conn: &Connection, guid: &str) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM chats WHERE guid = ?1", [guid], Self::from_row) {
            Ok(chat) => Ok(Some(chat)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find a chat by its chat identifier.
    pub fn find_by_identifier(conn: &Connection, identifier: &str) -> BbResult<Option<Self>> {
        match conn.query_row(
            "SELECT * FROM chats WHERE chat_identifier = ?1",
            [identifier],
            Self::from_row,
        ) {
            Ok(chat) => Ok(Some(chat)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Delete a chat by its local database ID. Returns true if a row was deleted.
    pub fn delete(conn: &Connection, id: i64) -> BbResult<bool> {
        // Delete join table entries first
        conn.execute("DELETE FROM chat_handle_join WHERE chat_id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        let changed = conn
            .execute("DELETE FROM chats WHERE id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    /// Soft-delete a chat by setting date_deleted.
    pub fn soft_delete(conn: &Connection, id: i64, timestamp: &str) -> BbResult<bool> {
        let changed = conn
            .execute(
                "UPDATE chats SET date_deleted = ?1 WHERE id = ?2",
                params![timestamp, id],
            )
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    /// Un-delete a soft-deleted chat.
    pub fn undelete(conn: &Connection, id: i64) -> BbResult<bool> {
        let changed = conn
            .execute(
                "UPDATE chats SET date_deleted = NULL WHERE id = ?1",
                [id],
            )
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    // ─── Computed properties ─────────────────────────────────────────────

    /// Whether this is a group chat.
    pub fn is_group(&self) -> bool {
        self.participants.len() > 1 || self.style == Some(bb_core::constants::chat_style::GROUP)
    }

    /// Whether this is an SMS text forwarding chat.
    pub fn is_text_forwarding(&self) -> bool {
        self.guid.starts_with("SMS")
    }

    /// Whether this is an iMessage chat.
    pub fn is_imessage(&self) -> bool {
        !self.is_text_forwarding()
    }

    /// Get the display title for this chat.
    pub fn title(&self) -> String {
        // 1. Explicit display name (set by user or group name)
        if let Some(ref name) = self.display_name {
            if !name.is_empty() {
                return name.clone();
            }
        }
        // 2. Participant display names (resolved from contacts)
        if !self.participants.is_empty() {
            let names: Vec<String> = self.participants
                .iter()
                .map(|h| h.display_name())
                .collect();
            // Only use participant names if at least one resolved to a contact name
            // (not just a raw phone/email address)
            if names.iter().any(|n| !n.starts_with('+') && !n.contains('@')) {
                return names.join(", ");
            }
        }
        if !self.participants.is_empty() {
            return self.participants
                .iter()
                .map(|h| h.display_name())
                .collect::<Vec<_>>()
                .join(", ");
        }
        // 4. Chat identifier (raw phone/email from GUID)
        if let Some(ref ident) = self.chat_identifier {
            return ident.clone();
        }
        "Unknown".to_string()
    }

    /// Compute participant names as a separate list (for Tauri API).
    pub fn participant_name_list(&self) -> Vec<String> {
        self.participants
            .iter()
            .map(|h| h.display_name())
            .collect()
    }

    /// Parse text_field_attachments JSON into a list of paths.
    pub fn draft_attachment_paths(&self) -> Vec<String> {
        serde_json::from_str(&self.text_field_attachments).unwrap_or_default()
    }

    // ─── Persistence ─────────────────────────────────────────────────────

    /// Upsert this chat into the database. Returns the local database ID.
    pub fn save(&mut self, conn: &Connection) -> BbResult<i64> {
        conn.execute(
            "INSERT INTO chats (
                original_rowid, guid, chat_identifier, display_name,
                is_archived, mute_type, mute_args, is_pinned,
                has_unread_message, pin_index, auto_send_read_receipts,
                auto_send_typing_indicators, text_field_text, text_field_attachments,
                latest_message_date, date_deleted, style, lock_chat_name, lock_chat_icon,
                last_read_message_guid, custom_avatar_path
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)
            ON CONFLICT(guid) DO UPDATE SET
                display_name = COALESCE(excluded.display_name, display_name),
                is_archived = excluded.is_archived,
                mute_type = excluded.mute_type,
                mute_args = excluded.mute_args,
                has_unread_message = excluded.has_unread_message,
                latest_message_date = COALESCE(excluded.latest_message_date, latest_message_date),
                date_deleted = excluded.date_deleted,
                style = COALESCE(excluded.style, style),
                last_read_message_guid = COALESCE(excluded.last_read_message_guid, last_read_message_guid)",
            params![
                self.original_rowid,
                self.guid,
                self.chat_identifier,
                self.display_name,
                self.is_archived as i32,
                self.mute_type,
                self.mute_args,
                self.is_pinned as i32,
                self.has_unread_message as i32,
                self.pin_index,
                self.auto_send_read_receipts.map(|v| v as i32),
                self.auto_send_typing_indicators.map(|v| v as i32),
                self.text_field_text,
                self.text_field_attachments,
                self.latest_message_date,
                self.date_deleted,
                self.style,
                self.lock_chat_name as i32,
                self.lock_chat_icon as i32,
                self.last_read_message_guid,
                self.custom_avatar_path,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        // Always query for the real ID - last_insert_rowid() is unreliable for upserts
        let real_id: i64 = conn
            .query_row(
                "SELECT id FROM chats WHERE guid = ?1",
                [&self.guid],
                |row| row.get(0),
            )
            .map_err(|e| BbError::Database(e.to_string()))?;
        self.id = Some(real_id);

        Ok(real_id)
    }

    /// Update mutable local-only fields (pinned, archived, mute, draft, etc).
    pub fn update(&self, conn: &Connection) -> BbResult<()> {
        let id = self.id.ok_or_else(|| BbError::Database("chat has no id for update".into()))?;
        conn.execute(
            "UPDATE chats SET
                is_pinned = ?1, pin_index = ?2, is_archived = ?3,
                mute_type = ?4, mute_args = ?5, has_unread_message = ?6,
                text_field_text = ?7, text_field_attachments = ?8,
                lock_chat_name = ?9, lock_chat_icon = ?10,
                custom_avatar_path = ?11, auto_send_read_receipts = ?12,
                auto_send_typing_indicators = ?13, last_read_message_guid = ?14,
                display_name = ?15
            WHERE id = ?16",
            params![
                self.is_pinned as i32,
                self.pin_index,
                self.is_archived as i32,
                self.mute_type,
                self.mute_args,
                self.has_unread_message as i32,
                self.text_field_text,
                self.text_field_attachments,
                self.lock_chat_name as i32,
                self.lock_chat_icon as i32,
                self.custom_avatar_path,
                self.auto_send_read_receipts.map(|v| v as i32),
                self.auto_send_typing_indicators.map(|v| v as i32),
                self.last_read_message_guid,
                self.display_name,
                id,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(())
    }

    /// Save participant relationships for this chat.
    pub fn save_participants(&self, conn: &Connection) -> BbResult<()> {
        let chat_id = self.id.ok_or_else(|| BbError::Database("chat has no id".into()))?;

        for handle in &self.participants {
            if let Some(handle_id) = handle.id {
                conn.execute(
                    "INSERT OR IGNORE INTO chat_handle_join (chat_id, handle_id) VALUES (?1, ?2)",
                    params![chat_id, handle_id],
                )
                .map_err(|e| BbError::Database(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Remove a participant from this chat.
    pub fn remove_participant(&self, conn: &Connection, handle_id: i64) -> BbResult<()> {
        let chat_id = self.id.ok_or_else(|| BbError::Database("chat has no id".into()))?;
        conn.execute(
            "DELETE FROM chat_handle_join WHERE chat_id = ?1 AND handle_id = ?2",
            params![chat_id, handle_id],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(())
    }

    /// Merge non-null fields from another Chat instance into this one.
    pub fn merge(&mut self, other: &Chat) {
        if other.display_name.is_some() {
            self.display_name = other.display_name.clone();
        }
        if other.chat_identifier.is_some() {
            self.chat_identifier = other.chat_identifier.clone();
        }
        if other.latest_message_date.is_some() {
            self.latest_message_date = other.latest_message_date.clone();
        }
        if other.style.is_some() {
            self.style = other.style;
        }
        if other.last_read_message_guid.is_some() {
            self.last_read_message_guid = other.last_read_message_guid.clone();
        }
        self.is_archived = other.is_archived;
        self.has_unread_message = other.has_unread_message;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_from_json() {
        let json = serde_json::json!({
            "guid": "iMessage;-;+1234567890",
            "chatIdentifier": "+1234567890",
            "displayName": "Test Chat",
            "isArchived": false,
            "isPinned": true,
            "participants": []
        });
        let chat = Chat::from_server_map(&json).unwrap();
        assert_eq!(chat.guid, "iMessage;-;+1234567890");
        assert!(chat.is_pinned);
        assert!(!chat.is_group());
    }

    #[test]
    fn test_chat_title() {
        let mut chat = Chat::from_server_map(&serde_json::json!({
            "guid": "test",
            "displayName": "My Chat"
        }))
        .unwrap();
        assert_eq!(chat.title(), "My Chat");

        chat.display_name = None;
        chat.chat_identifier = Some("+15551234".into());
        assert_eq!(chat.title(), "+15551234");
    }

    #[test]
    fn test_chat_merge() {
        let mut chat = Chat::from_server_map(&serde_json::json!({"guid": "a"})).unwrap();
        let other = Chat::from_server_map(&serde_json::json!({
            "guid": "a",
            "displayName": "Updated Name",
            "style": 43
        })).unwrap();
        chat.merge(&other);
        assert_eq!(chat.display_name.as_deref(), Some("Updated Name"));
        assert_eq!(chat.style, Some(43));
    }

    #[test]
    fn test_text_forwarding() {
        let chat = Chat::from_server_map(&serde_json::json!({"guid": "SMS;-;+1234"})).unwrap();
        assert!(chat.is_text_forwarding());
        assert!(!chat.is_imessage());
    }
}
