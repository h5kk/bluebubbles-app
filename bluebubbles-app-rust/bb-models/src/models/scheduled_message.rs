//! Scheduled message entity model.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Row};
use bb_core::error::{BbError, BbResult};

/// Represents a scheduled message cached locally.
///
/// Scheduled messages are managed by the server. The client caches them
/// for display purposes and status tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledMessage {
    pub id: Option<i64>,
    pub schedule_type: String,
    pub chat_guid: String,
    pub message: String,
    pub scheduled_for: String,
    pub repeat_type: Option<String>,
    pub repeat_interval: Option<i64>,
    pub repeat_interval_type: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub sent_at: Option<String>,
    pub created_at: String,
}

/// Known status values for scheduled messages.
pub mod status {
    pub const PENDING: &str = "pending";
    pub const SENT: &str = "sent";
    pub const FAILED: &str = "failed";
    pub const CANCELLED: &str = "cancelled";
}

impl ScheduledMessage {
    /// Create a ScheduledMessage from a server JSON map.
    pub fn from_server_map(map: &serde_json::Value) -> BbResult<Self> {
        Ok(Self {
            id: map.get("id").and_then(|v| v.as_i64()),
            schedule_type: map
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("send-message")
                .to_string(),
            chat_guid: map
                .get("payload")
                .and_then(|p| p.get("chatGuid"))
                .and_then(|v| v.as_str())
                .or_else(|| map.get("chatGuid").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string(),
            message: map
                .get("payload")
                .and_then(|p| p.get("message"))
                .and_then(|v| v.as_str())
                .or_else(|| map.get("message").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string(),
            scheduled_for: map
                .get("scheduledFor")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            repeat_type: map.get("schedule").and_then(|s| s.get("type")).and_then(|v| v.as_str()).map(String::from),
            repeat_interval: map.get("schedule").and_then(|s| s.get("interval")).and_then(|v| v.as_i64()),
            repeat_interval_type: map.get("schedule").and_then(|s| s.get("intervalType")).and_then(|v| v.as_str()).map(String::from),
            status: map
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or(status::PENDING)
                .to_string(),
            error: map.get("error").and_then(|v| v.as_str()).map(String::from),
            sent_at: map.get("sentAt").and_then(|v| v.as_str()).map(String::from),
            created_at: map
                .get("createdAt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    /// Construct a ScheduledMessage from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: Some(row.get("id")?),
            schedule_type: row.get("type")?,
            chat_guid: row.get("chat_guid")?,
            message: row.get("message")?,
            scheduled_for: row.get("scheduled_for")?,
            repeat_type: row.get("schedule_type")?,
            repeat_interval: row.get("schedule_interval")?,
            repeat_interval_type: row.get("schedule_interval_type")?,
            status: row.get("status")?,
            error: row.get("error")?,
            sent_at: row.get("sent_at")?,
            created_at: row.get("created_at")?,
        })
    }

    /// Whether this scheduled message is still pending.
    pub fn is_pending(&self) -> bool {
        self.status == status::PENDING
    }

    /// Whether this scheduled message has been sent.
    pub fn is_sent(&self) -> bool {
        self.status == status::SENT
    }

    /// Whether this scheduled message failed.
    pub fn is_failed(&self) -> bool {
        self.status == status::FAILED
    }

    /// Save or update this scheduled message in the database.
    pub fn save(&mut self, conn: &rusqlite::Connection) -> BbResult<i64> {
        let row_id = self.id.unwrap_or(0);
        conn.execute(
            "INSERT OR REPLACE INTO scheduled_messages (
                id, type, chat_guid, message, scheduled_for,
                schedule_type, schedule_interval, schedule_interval_type,
                status, error, sent_at, created_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                row_id,
                self.schedule_type,
                self.chat_guid,
                self.message,
                self.scheduled_for,
                self.repeat_type,
                self.repeat_interval,
                self.repeat_interval_type,
                self.status,
                self.error,
                self.sent_at,
                self.created_at,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        if self.id.is_none() {
            self.id = Some(conn.last_insert_rowid());
        }
        Ok(self.id.unwrap_or(0))
    }

    /// Load all scheduled messages from the database.
    pub fn load_all(conn: &rusqlite::Connection) -> BbResult<Vec<Self>> {
        let mut stmt = conn
            .prepare("SELECT * FROM scheduled_messages ORDER BY scheduled_for ASC")
            .map_err(|e| BbError::Database(e.to_string()))?;

        let messages = stmt
            .query_map([], Self::from_row)
            .map_err(|e| BbError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(messages)
    }

    /// Delete a scheduled message by ID.
    pub fn delete(conn: &rusqlite::Connection, id: i64) -> BbResult<bool> {
        let changed = conn
            .execute("DELETE FROM scheduled_messages WHERE id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_message_from_json() {
        let json = serde_json::json!({
            "id": 1,
            "type": "send-message",
            "payload": {
                "chatGuid": "iMessage;-;+1234",
                "message": "Hello!"
            },
            "scheduledFor": "2025-01-01T12:00:00Z",
            "status": "pending",
            "createdAt": "2025-01-01T00:00:00Z"
        });
        let msg = ScheduledMessage::from_server_map(&json).unwrap();
        assert!(msg.is_pending());
        assert_eq!(msg.chat_guid, "iMessage;-;+1234");
    }

    #[test]
    fn test_scheduled_message_status() {
        let json = serde_json::json!({
            "status": "sent",
            "createdAt": "2025-01-01"
        });
        let msg = ScheduledMessage::from_server_map(&json).unwrap();
        assert!(msg.is_sent());
        assert!(!msg.is_pending());
    }
}
