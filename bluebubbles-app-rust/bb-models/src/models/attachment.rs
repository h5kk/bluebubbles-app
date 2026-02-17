//! Attachment entity model.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Row};
use bb_core::error::{BbError, BbResult};

/// Represents a file attachment in the BlueBubbles system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Option<i64>,
    pub original_rowid: Option<i64>,
    pub guid: Option<String>,
    pub message_id: Option<i64>,
    pub uti: Option<String>,
    pub mime_type: Option<String>,
    pub is_outgoing: Option<bool>,
    pub transfer_name: Option<String>,
    pub total_bytes: Option<i64>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub web_url: Option<String>,
    pub has_live_photo: bool,
    pub metadata: Option<String>,
}

impl Attachment {
    /// Create an Attachment from a server JSON map.
    pub fn from_server_map(map: &serde_json::Value) -> BbResult<Self> {
        let mut mime_type = map.get("mimeType").and_then(|v| v.as_str()).map(String::from);

        // Special-case .caf audio files
        let transfer_name = map.get("transferName").and_then(|v| v.as_str()).map(String::from);
        if let Some(ref name) = transfer_name {
            if name.ends_with(".caf") && mime_type.is_none() {
                mime_type = Some("audio/caf".to_string());
            }
        }

        Ok(Self {
            id: None,
            original_rowid: map.get("ROWID").and_then(|v| v.as_i64())
                .or_else(|| map.get("originalROWID").and_then(|v| v.as_i64())),
            guid: map.get("guid").and_then(|v| v.as_str()).map(String::from),
            message_id: None,
            uti: map.get("uti").and_then(|v| v.as_str()).map(String::from),
            mime_type,
            is_outgoing: map.get("isOutgoing").and_then(|v| v.as_bool()),
            transfer_name,
            total_bytes: map.get("totalBytes").and_then(|v| v.as_i64()),
            height: map.get("height").and_then(|v| v.as_i64()).map(|v| v as i32),
            width: map.get("width").and_then(|v| v.as_i64()).map(|v| v as i32),
            web_url: map.get("webUrl").and_then(|v| v.as_str()).map(String::from),
            has_live_photo: map.get("hasLivePhoto").and_then(|v| v.as_bool()).unwrap_or(false),
            metadata: map.get("metadata").map(|v| v.to_string()),
        })
    }

    /// Construct an Attachment from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            original_rowid: row.get("original_rowid")?,
            guid: row.get("guid")?,
            message_id: row.get("message_id")?,
            uti: row.get("uti")?,
            mime_type: row.get("mime_type")?,
            is_outgoing: row.get::<_, Option<i32>>("is_outgoing")?.map(|v| v != 0),
            transfer_name: row.get("transfer_name")?,
            total_bytes: row.get("total_bytes")?,
            height: row.get("height")?,
            width: row.get("width")?,
            web_url: row.get("web_url")?,
            has_live_photo: row.get::<_, i32>("has_live_photo")? != 0,
            metadata: row.get("metadata")?,
        })
    }

    // ─── Static finders ──────────────────────────────────────────────────

    /// Find an attachment by its local database ID.
    pub fn find_by_id(conn: &Connection, id: i64) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM attachments WHERE id = ?1", [id], Self::from_row) {
            Ok(a) => Ok(Some(a)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find an attachment by its GUID.
    pub fn find_by_guid(conn: &Connection, guid: &str) -> BbResult<Option<Self>> {
        match conn.query_row(
            "SELECT * FROM attachments WHERE guid = ?1",
            [guid],
            Self::from_row,
        ) {
            Ok(a) => Ok(Some(a)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Delete an attachment by its local database ID.
    pub fn delete(conn: &Connection, id: i64) -> BbResult<bool> {
        let changed = conn
            .execute("DELETE FROM attachments WHERE id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    // ─── Computed properties ─────────────────────────────────────────────

    /// Whether this attachment is an image based on its MIME type.
    pub fn is_image(&self) -> bool {
        self.mime_type.as_deref().map_or(false, |m| m.starts_with("image/"))
    }

    /// Whether this attachment is a video based on its MIME type.
    pub fn is_video(&self) -> bool {
        self.mime_type.as_deref().map_or(false, |m| m.starts_with("video/"))
    }

    /// Whether this attachment is an audio file based on its MIME type.
    pub fn is_audio(&self) -> bool {
        self.mime_type.as_deref().map_or(false, |m| m.starts_with("audio/"))
    }

    /// Whether this is a contact card (vCard).
    pub fn is_contact_card(&self) -> bool {
        self.mime_type.as_deref().map_or(false, |m| m == "text/x-vlocation" || m == "text/vcard")
            || self.uti.as_deref().map_or(false, |u| u == "public.vcard")
    }

    /// Whether this is a location sharing attachment.
    pub fn is_location(&self) -> bool {
        self.uti.as_deref().map_or(false, |u| u.contains("location") || u.contains("map"))
    }

    /// Whether this attachment has a valid size (width and height are both positive).
    pub fn has_valid_size(&self) -> bool {
        matches!((self.width, self.height), (Some(w), Some(h)) if w > 0 && h > 0)
    }

    /// Compute the aspect ratio (width / height), accounting for orientation.
    pub fn aspect_ratio(&self) -> Option<f64> {
        match (self.width, self.height) {
            (Some(w), Some(h)) if w > 0 && h > 0 => Some(w as f64 / h as f64),
            _ => None,
        }
    }

    /// Get the MIME type prefix (e.g., "image", "video", "audio").
    pub fn mime_start(&self) -> Option<&str> {
        self.mime_type.as_deref().and_then(|m| m.split('/').next())
    }

    /// Get the file extension from the transfer name.
    pub fn file_extension(&self) -> Option<&str> {
        self.transfer_name.as_deref().and_then(|name| name.rsplit('.').next())
    }

    /// Get a human-readable file size string.
    pub fn human_file_size(&self) -> String {
        match self.total_bytes {
            Some(bytes) if bytes >= 1_073_741_824 => {
                format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
            }
            Some(bytes) if bytes >= 1_048_576 => {
                format!("{:.1} MB", bytes as f64 / 1_048_576.0)
            }
            Some(bytes) if bytes >= 1024 => {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            }
            Some(bytes) => format!("{bytes} B"),
            None => "Unknown".to_string(),
        }
    }

    /// Merge non-null fields from another attachment into this one.
    pub fn merge(&mut self, other: &Attachment) {
        if other.mime_type.is_some() {
            self.mime_type = other.mime_type.clone();
        }
        if other.web_url.is_some() {
            self.web_url = other.web_url.clone();
        }
        if other.metadata.is_some() {
            self.metadata = other.metadata.clone();
        }
        if other.total_bytes.is_some() {
            self.total_bytes = other.total_bytes;
        }
        if other.height.is_some() {
            self.height = other.height;
        }
        if other.width.is_some() {
            self.width = other.width;
        }
    }

    // ─── Persistence ─────────────────────────────────────────────────────

    /// Upsert this attachment into the database. Returns the local database ID.
    pub fn save(&mut self, conn: &Connection) -> BbResult<i64> {
        conn.execute(
            "INSERT INTO attachments (
                original_rowid, guid, message_id, uti, mime_type,
                is_outgoing, transfer_name, total_bytes, height, width,
                web_url, has_live_photo, metadata
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
            ON CONFLICT(guid) DO UPDATE SET
                mime_type = COALESCE(excluded.mime_type, mime_type),
                web_url = COALESCE(excluded.web_url, web_url),
                metadata = COALESCE(excluded.metadata, metadata)",
            params![
                self.original_rowid,
                self.guid,
                self.message_id,
                self.uti,
                self.mime_type,
                self.is_outgoing.map(|v| v as i32),
                self.transfer_name,
                self.total_bytes,
                self.height,
                self.width,
                self.web_url,
                self.has_live_photo as i32,
                self.metadata,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        // Always query for the real ID - last_insert_rowid() is unreliable for upserts
        if let Some(ref guid) = self.guid {
            let real_id: i64 = conn
                .query_row("SELECT id FROM attachments WHERE guid = ?1", [guid], |row| row.get(0))
                .map_err(|e| BbError::Database(e.to_string()))?;
            self.id = Some(real_id);
        }

        Ok(self.id.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_from_json() {
        let json = serde_json::json!({
            "guid": "att-123",
            "mimeType": "image/jpeg",
            "transferName": "photo.jpg",
            "totalBytes": 1024000
        });
        let att = Attachment::from_server_map(&json).unwrap();
        assert!(att.is_image());
        assert!(!att.is_video());
        assert_eq!(att.file_extension(), Some("jpg"));
    }

    #[test]
    fn test_caf_audio_detection() {
        let json = serde_json::json!({
            "guid": "att-caf",
            "transferName": "recording.caf"
        });
        let att = Attachment::from_server_map(&json).unwrap();
        assert_eq!(att.mime_type.as_deref(), Some("audio/caf"));
        assert!(att.is_audio());
    }

    #[test]
    fn test_human_file_size() {
        let mut att = Attachment::from_server_map(&serde_json::json!({"guid": "t"})).unwrap();
        att.total_bytes = Some(1_500_000);
        assert_eq!(att.human_file_size(), "1.4 MB");

        att.total_bytes = Some(512);
        assert_eq!(att.human_file_size(), "512 B");
    }

    #[test]
    fn test_attachment_types() {
        let mut att = Attachment::from_server_map(&serde_json::json!({"guid": "t"})).unwrap();
        att.mime_type = Some("video/mp4".into());
        assert!(att.is_video());
        assert!(!att.is_image());

        att.mime_type = Some("audio/aac".into());
        assert!(att.is_audio());
    }

    #[test]
    fn test_aspect_ratio() {
        let mut att = Attachment::from_server_map(&serde_json::json!({"guid": "t"})).unwrap();
        att.width = Some(1920);
        att.height = Some(1080);
        assert!(att.has_valid_size());
        let ratio = att.aspect_ratio().unwrap();
        assert!((ratio - 1.777).abs() < 0.01);
    }
}
