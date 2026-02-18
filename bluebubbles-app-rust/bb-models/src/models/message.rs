//! Message entity model.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Row};
use bb_core::error::{BbError, BbResult};

/// Represents a single message in the BlueBubbles system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<i64>,
    pub original_rowid: Option<i64>,
    pub guid: Option<String>,
    pub chat_id: Option<i64>,
    pub handle_id: Option<i64>,
    pub other_handle: Option<i64>,
    pub text: Option<String>,
    pub subject: Option<String>,
    pub country: Option<String>,
    pub error: i32,
    pub date_created: Option<String>,
    pub date_read: Option<String>,
    pub date_delivered: Option<String>,
    pub is_delivered: bool,
    pub is_from_me: bool,
    pub has_dd_results: bool,
    pub date_played: Option<String>,
    pub item_type: i32,
    pub group_title: Option<String>,
    pub group_action_type: i32,
    pub balloon_bundle_id: Option<String>,
    pub associated_message_guid: Option<String>,
    pub associated_message_part: Option<i32>,
    pub associated_message_type: Option<String>,
    pub expressive_send_style_id: Option<String>,
    pub has_attachments: bool,
    pub has_reactions: bool,
    pub date_deleted: Option<String>,
    pub thread_originator_guid: Option<String>,
    pub thread_originator_part: Option<String>,
    pub big_emoji: Option<bool>,
    pub attributed_body: Option<String>,
    pub message_summary_info: Option<String>,
    pub payload_data: Option<String>,
    pub metadata: Option<String>,
    pub has_apple_payload_data: bool,
    pub date_edited: Option<String>,
    pub was_delivered_quietly: bool,
    pub did_notify_recipient: bool,
    pub is_bookmarked: bool,

    // Transient fields (not persisted directly)
    #[serde(skip)]
    pub handle: Option<super::handle::Handle>,
    #[serde(default)]
    pub attachments: Vec<super::attachment::Attachment>,
    #[serde(default)]
    pub associated_messages: Vec<Message>,
}

impl Message {
    /// Create a Message from a server JSON map.
    pub fn from_server_map(map: &serde_json::Value) -> BbResult<Self> {
        let mut result = Ok(Self {
            id: None,
            original_rowid: map.get("ROWID").and_then(|v| v.as_i64())
                .or_else(|| map.get("originalROWID").and_then(|v| v.as_i64())),
            guid: map.get("guid").and_then(|v| v.as_str()).map(String::from),
            chat_id: None,
            handle_id: map.get("handleId").and_then(|v| v.as_i64()),
            other_handle: map.get("otherHandle").and_then(|v| v.as_i64()),
            text: map.get("text").and_then(|v| v.as_str()).map(String::from),
            subject: map.get("subject").and_then(|v| v.as_str()).map(String::from),
            country: map.get("country").and_then(|v| v.as_str()).map(String::from),
            error: map.get("error").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            date_created: map.get("dateCreated").and_then(|v| v.as_str()).map(String::from)
                .or_else(|| map.get("dateCreated").and_then(|v| v.as_i64()).map(|ts| ts.to_string())),
            date_read: map.get("dateRead").and_then(|v| v.as_str()).map(String::from)
                .or_else(|| map.get("dateRead").and_then(|v| v.as_i64()).map(|ts| ts.to_string())),
            date_delivered: map.get("dateDelivered").and_then(|v| v.as_str()).map(String::from)
                .or_else(|| map.get("dateDelivered").and_then(|v| v.as_i64()).map(|ts| ts.to_string())),
            is_delivered: map.get("isDelivered").and_then(|v| v.as_bool()).unwrap_or(false),
            is_from_me: map.get("isFromMe").and_then(|v| v.as_bool()).unwrap_or(false),
            has_dd_results: map.get("hasDdResults").and_then(|v| v.as_bool()).unwrap_or(false),
            date_played: map.get("datePlayed").and_then(|v| v.as_str()).map(String::from),
            item_type: map.get("itemType").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            group_title: map.get("groupTitle").and_then(|v| v.as_str()).map(String::from),
            group_action_type: map.get("groupActionType").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            balloon_bundle_id: map.get("balloonBundleId").and_then(|v| v.as_str()).map(String::from),
            associated_message_guid: map.get("associatedMessageGuid").and_then(|v| v.as_str()).map(String::from),
            associated_message_part: map.get("associatedMessagePart").and_then(|v| v.as_i64()).map(|v| v as i32),
            associated_message_type: map.get("associatedMessageType").and_then(|v| v.as_str()).map(String::from),
            expressive_send_style_id: map.get("expressiveSendStyleId").and_then(|v| v.as_str()).map(String::from),
            has_reactions: map.get("hasReactions").and_then(|v| v.as_bool()).unwrap_or(false),
            date_deleted: map.get("dateDeleted").and_then(|v| v.as_str()).map(String::from),
            thread_originator_guid: map.get("threadOriginatorGuid").and_then(|v| v.as_str()).map(String::from),
            thread_originator_part: map.get("threadOriginatorPart").and_then(|v| v.as_str()).map(String::from),
            big_emoji: map.get("bigEmoji").and_then(|v| v.as_bool()),
            attributed_body: map.get("attributedBody").map(|v| v.to_string()),
            message_summary_info: map.get("messageSummaryInfo").map(|v| v.to_string()),
            payload_data: map.get("payloadData").map(|v| v.to_string()),
            metadata: map.get("metadata").map(|v| v.to_string()),
            has_apple_payload_data: map.get("hasApplePayloadData").and_then(|v| v.as_bool()).unwrap_or(false),
            date_edited: map.get("dateEdited").and_then(|v| v.as_str()).map(String::from),
            was_delivered_quietly: map.get("wasDeliveredQuietly").and_then(|v| v.as_bool()).unwrap_or(false),
            did_notify_recipient: map.get("didNotifyRecipient").and_then(|v| v.as_bool()).unwrap_or(false),
            is_bookmarked: map.get("isBookmarked").and_then(|v| v.as_bool()).unwrap_or(false),
            handle: None,
            // Parse attachments first, then derive has_attachments from both the
            // server field (if present) AND the actual parsed attachment count.
            // The BB server often omits the hasAttachments field entirely.
            attachments: {
                let parsed: Vec<super::attachment::Attachment> = map.get("attachments")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|a| super::attachment::Attachment::from_server_map(a).ok())
                            .collect()
                    })
                    .unwrap_or_default();
                parsed
            },
            has_attachments: false, // placeholder, set below
            associated_messages: vec![],
        });
        // Derive has_attachments from the server field OR the actual attachment count
        if let Ok(ref mut msg) = result {
            let server_has = map.get("hasAttachments").and_then(|v| v.as_bool()).unwrap_or(false);
            msg.has_attachments = server_has || !msg.attachments.is_empty();
        }
        result
    }

    /// Construct a Message from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            original_rowid: row.get("original_rowid")?,
            guid: row.get("guid")?,
            chat_id: row.get("chat_id")?,
            handle_id: row.get("handle_id")?,
            other_handle: row.get("other_handle")?,
            text: row.get("text")?,
            subject: row.get("subject")?,
            country: row.get("country")?,
            error: row.get("error")?,
            date_created: row.get("date_created")?,
            date_read: row.get("date_read")?,
            date_delivered: row.get("date_delivered")?,
            is_delivered: row.get::<_, i32>("is_delivered")? != 0,
            is_from_me: row.get::<_, i32>("is_from_me")? != 0,
            has_dd_results: row.get::<_, i32>("has_dd_results")? != 0,
            date_played: row.get("date_played")?,
            item_type: row.get("item_type")?,
            group_title: row.get("group_title")?,
            group_action_type: row.get("group_action_type")?,
            balloon_bundle_id: row.get("balloon_bundle_id")?,
            associated_message_guid: row.get("associated_message_guid")?,
            associated_message_part: row.get("associated_message_part")?,
            associated_message_type: row.get("associated_message_type")?,
            expressive_send_style_id: row.get("expressive_send_style_id")?,
            has_attachments: row.get::<_, i32>("has_attachments")? != 0,
            has_reactions: row.get::<_, i32>("has_reactions")? != 0,
            date_deleted: row.get("date_deleted")?,
            thread_originator_guid: row.get("thread_originator_guid")?,
            thread_originator_part: row.get("thread_originator_part")?,
            big_emoji: row.get::<_, Option<i32>>("big_emoji")?.map(|v| v != 0),
            attributed_body: row.get("attributed_body")?,
            message_summary_info: row.get("message_summary_info")?,
            payload_data: row.get("payload_data")?,
            metadata: row.get("metadata")?,
            has_apple_payload_data: row.get::<_, i32>("has_apple_payload_data")? != 0,
            date_edited: row.get("date_edited")?,
            was_delivered_quietly: row.get::<_, i32>("was_delivered_quietly")? != 0,
            did_notify_recipient: row.get::<_, i32>("did_notify_recipient")? != 0,
            is_bookmarked: row.get::<_, i32>("is_bookmarked")? != 0,
            handle: None,
            attachments: vec![],
            associated_messages: vec![],
        })
    }

    // ─── Static finders ──────────────────────────────────────────────────

    /// Find a message by its local database ID.
    pub fn find_by_id(conn: &Connection, id: i64) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM messages WHERE id = ?1", [id], Self::from_row) {
            Ok(msg) => Ok(Some(msg)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find a message by its GUID.
    pub fn find_by_guid(conn: &Connection, guid: &str) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM messages WHERE guid = ?1", [guid], Self::from_row) {
            Ok(msg) => Ok(Some(msg)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Delete a message by its local database ID. Returns true if a row was deleted.
    pub fn delete(conn: &Connection, id: i64) -> BbResult<bool> {
        let changed = conn
            .execute("DELETE FROM messages WHERE id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    /// Soft-delete a message by GUID.
    pub fn soft_delete(conn: &Connection, guid: &str, timestamp: &str) -> BbResult<bool> {
        let changed = conn
            .execute(
                "UPDATE messages SET date_deleted = ?1 WHERE guid = ?2",
                params![timestamp, guid],
            )
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    /// Replace a temp message (identified by old_guid) with server data.
    pub fn replace_temp(conn: &Connection, old_guid: &str, new_msg: &mut Message) -> BbResult<i64> {
        // Find existing temp message
        if let Some(existing) = Self::find_by_guid(conn, old_guid)? {
            new_msg.id = existing.id;
            new_msg.chat_id = new_msg.chat_id.or(existing.chat_id);
            // Delete the temp and insert the real one
            if let Some(id) = existing.id {
                Self::delete(conn, id)?;
            }
        }
        new_msg.save(conn)
    }

    // ─── Computed properties ─────────────────────────────────────────────

    /// Whether this message is a group event (name change, participant change, etc).
    pub fn is_group_event(&self) -> bool {
        self.item_type != 0
    }

    /// Whether this message is a reaction/tapback.
    pub fn is_reaction(&self) -> bool {
        self.associated_message_type.is_some()
    }

    /// Whether this is a temporary (not yet sent) message.
    pub fn is_temp(&self) -> bool {
        self.guid.as_deref().map_or(false, |g| g.starts_with("temp"))
    }

    /// Whether this message resulted in an error.
    pub fn is_error(&self) -> bool {
        self.guid.as_deref().map_or(false, |g| g.starts_with("error"))
    }

    /// Get the full text including subject.
    pub fn full_text(&self) -> String {
        match (&self.subject, &self.text) {
            (Some(s), Some(t)) if !s.is_empty() => format!("{s}\n{t}"),
            (Some(s), None) if !s.is_empty() => s.clone(),
            (_, Some(t)) => t.clone(),
            _ => String::new(),
        }
    }

    /// Whether this message has an interactive balloon bundle.
    pub fn is_interactive(&self) -> bool {
        self.balloon_bundle_id.is_some() && !self.is_legacy_url_preview()
    }

    /// Whether this is a legacy-style URL preview message.
    pub fn is_legacy_url_preview(&self) -> bool {
        self.balloon_bundle_id
            .as_deref()
            .map_or(false, |id| id == "com.apple.messages.URLBalloonProvider")
    }

    /// Whether this message is a participant event.
    pub fn is_participant_event(&self) -> bool {
        self.item_type == 1 || self.item_type == 3
    }

    /// Delivery indicator to show: "read", "delivered", "sent", or "none".
    pub fn indicator_to_show(&self) -> &'static str {
        if !self.is_from_me {
            return "none";
        }
        if self.date_read.is_some() {
            return "read";
        }
        if self.date_delivered.is_some() || self.is_delivered {
            return "delivered";
        }
        if self.date_created.is_some() {
            return "sent";
        }
        "none"
    }

    /// Parse the normalized thread originator part index.
    pub fn normalized_thread_part(&self) -> Option<i32> {
        self.thread_originator_part
            .as_deref()
            .and_then(|s| s.chars().next())
            .and_then(|c| c.to_digit(10))
            .map(|d| d as i32)
    }

    /// Merge non-null fields from another message into this one.
    pub fn merge(&mut self, other: &Message) {
        if other.text.is_some() {
            self.text = other.text.clone();
        }
        if other.date_read.is_some() {
            self.date_read = other.date_read.clone();
        }
        if other.date_delivered.is_some() {
            self.date_delivered = other.date_delivered.clone();
        }
        if other.date_edited.is_some() {
            self.date_edited = other.date_edited.clone();
        }
        if other.date_deleted.is_some() {
            self.date_deleted = other.date_deleted.clone();
        }
        if other.message_summary_info.is_some() {
            self.message_summary_info = other.message_summary_info.clone();
        }
        self.error = other.error;
        self.is_delivered = other.is_delivered;
        self.has_reactions = other.has_reactions;
        self.is_bookmarked = other.is_bookmarked;
    }

    // ─── Persistence ─────────────────────────────────────────────────────

    /// Upsert this message into the database. Returns the local database ID.
    pub fn save(&mut self, conn: &Connection) -> BbResult<i64> {
        conn.execute(
            "INSERT INTO messages (
                original_rowid, guid, chat_id, handle_id, other_handle,
                text, subject, country, error, date_created, date_read,
                date_delivered, is_delivered, is_from_me, has_dd_results,
                date_played, item_type, group_title, group_action_type,
                balloon_bundle_id, associated_message_guid, associated_message_part,
                associated_message_type, expressive_send_style_id, has_attachments,
                has_reactions, date_deleted, thread_originator_guid, thread_originator_part,
                big_emoji, attributed_body, message_summary_info, payload_data,
                metadata, has_apple_payload_data, date_edited, was_delivered_quietly,
                did_notify_recipient, is_bookmarked
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,?37,?38,?39)
            ON CONFLICT(guid) DO UPDATE SET
                text = COALESCE(excluded.text, text),
                error = excluded.error,
                date_read = COALESCE(excluded.date_read, date_read),
                date_delivered = COALESCE(excluded.date_delivered, date_delivered),
                is_delivered = excluded.is_delivered,
                has_reactions = excluded.has_reactions,
                date_deleted = excluded.date_deleted,
                date_edited = COALESCE(excluded.date_edited, date_edited),
                message_summary_info = COALESCE(excluded.message_summary_info, message_summary_info),
                is_bookmarked = excluded.is_bookmarked",
            params![
                self.original_rowid, self.guid, self.chat_id, self.handle_id,
                self.other_handle, self.text, self.subject, self.country,
                self.error, self.date_created, self.date_read, self.date_delivered,
                self.is_delivered as i32, self.is_from_me as i32, self.has_dd_results as i32,
                self.date_played, self.item_type, self.group_title, self.group_action_type,
                self.balloon_bundle_id, self.associated_message_guid, self.associated_message_part,
                self.associated_message_type, self.expressive_send_style_id,
                self.has_attachments as i32, self.has_reactions as i32,
                self.date_deleted, self.thread_originator_guid, self.thread_originator_part,
                self.big_emoji.map(|b| b as i32), self.attributed_body,
                self.message_summary_info, self.payload_data, self.metadata,
                self.has_apple_payload_data as i32, self.date_edited,
                self.was_delivered_quietly as i32, self.did_notify_recipient as i32,
                self.is_bookmarked as i32,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        // Always query for the real ID - last_insert_rowid() is unreliable for upserts
        if let Some(ref guid) = self.guid {
            let real_id: i64 = conn
                .query_row("SELECT id FROM messages WHERE guid = ?1", [guid], |row| row.get(0))
                .map_err(|e| BbError::Database(e.to_string()))?;
            self.id = Some(real_id);
        }

        Ok(self.id.unwrap_or(0))
    }

    /// Update specific mutable fields on an existing message.
    pub fn update(&self, conn: &Connection) -> BbResult<()> {
        let id = self.id.ok_or_else(|| BbError::Database("message has no id for update".into()))?;
        conn.execute(
            "UPDATE messages SET
                error = ?1, date_read = ?2, date_delivered = ?3,
                is_delivered = ?4, has_reactions = ?5, date_deleted = ?6,
                date_edited = ?7, message_summary_info = ?8, is_bookmarked = ?9
            WHERE id = ?10",
            params![
                self.error,
                self.date_read,
                self.date_delivered,
                self.is_delivered as i32,
                self.has_reactions as i32,
                self.date_deleted,
                self.date_edited,
                self.message_summary_info,
                self.is_bookmarked as i32,
                id,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_from_json() {
        let json = serde_json::json!({
            "guid": "msg-123",
            "text": "Hello world",
            "isFromMe": true,
            "dateCreated": "2024-01-01T00:00:00Z"
        });
        let msg = Message::from_server_map(&json).unwrap();
        assert_eq!(msg.guid.as_deref(), Some("msg-123"));
        assert!(msg.is_from_me);
        assert!(!msg.is_reaction());
    }

    #[test]
    fn test_message_full_text() {
        let mut msg = Message::from_server_map(&serde_json::json!({"guid": "t"})).unwrap();
        msg.subject = Some("Re:".into());
        msg.text = Some("Hello".into());
        assert_eq!(msg.full_text(), "Re:\nHello");
    }

    #[test]
    fn test_temp_detection() {
        let mut msg = Message::from_server_map(&serde_json::json!({"guid": "temp-abc"})).unwrap();
        assert!(msg.is_temp());
        msg.guid = Some("error-timeout".into());
        assert!(msg.is_error());
    }

    #[test]
    fn test_indicator_to_show() {
        let mut msg = Message::from_server_map(&serde_json::json!({"guid": "m1", "isFromMe": true})).unwrap();
        msg.date_created = Some("2024-01-01".into());
        assert_eq!(msg.indicator_to_show(), "sent");

        msg.date_delivered = Some("2024-01-01".into());
        assert_eq!(msg.indicator_to_show(), "delivered");

        msg.date_read = Some("2024-01-01".into());
        assert_eq!(msg.indicator_to_show(), "read");
    }

    #[test]
    fn test_message_merge() {
        let mut msg = Message::from_server_map(&serde_json::json!({"guid": "m1"})).unwrap();
        msg.text = Some("original".into());

        let other = Message::from_server_map(&serde_json::json!({
            "guid": "m1",
            "text": "updated",
            "dateRead": "2024-01-02"
        })).unwrap();
        msg.merge(&other);
        assert_eq!(msg.text.as_deref(), Some("updated"));
        assert_eq!(msg.date_read.as_deref(), Some("2024-01-02"));
    }
}
