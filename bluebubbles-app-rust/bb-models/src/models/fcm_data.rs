//! FCM (Firebase Cloud Messaging) configuration data model.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Row};
use bb_core::error::{BbError, BbResult};

/// Firebase Cloud Messaging configuration for push notifications.
///
/// This data is provided by the BlueBubbles server and used to register
/// the client for push notifications via Firebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmData {
    pub id: Option<i64>,
    pub project_id: Option<String>,
    pub storage_bucket: Option<String>,
    pub api_key: Option<String>,
    pub firebase_url: Option<String>,
    pub client_id: Option<String>,
    pub application_id: Option<String>,
}

impl FcmData {
    /// Create FcmData from a server JSON map.
    pub fn from_server_map(map: &serde_json::Value) -> BbResult<Self> {
        Ok(Self {
            id: None,
            project_id: map.get("projectID").and_then(|v| v.as_str()).map(String::from),
            storage_bucket: map.get("storageBucket").and_then(|v| v.as_str()).map(String::from),
            api_key: map.get("apiKey").and_then(|v| v.as_str()).map(String::from),
            firebase_url: map.get("firebaseURL").and_then(|v| v.as_str()).map(String::from),
            client_id: map.get("clientID").and_then(|v| v.as_str()).map(String::from),
            application_id: map.get("applicationID").and_then(|v| v.as_str()).map(String::from),
        })
    }

    /// Construct FcmData from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            storage_bucket: row.get("storage_bucket")?,
            api_key: row.get("api_key")?,
            firebase_url: row.get("firebase_url")?,
            client_id: row.get("client_id")?,
            application_id: row.get("application_id")?,
        })
    }

    /// Whether this FCM data has sufficient fields to register for notifications.
    pub fn is_valid(&self) -> bool {
        self.project_id.is_some()
            && self.api_key.is_some()
            && self.application_id.is_some()
    }

    /// Save or update FCM data. Only one row should ever exist.
    pub fn save(&mut self, conn: &rusqlite::Connection) -> BbResult<i64> {
        // Clear existing FCM data and insert fresh
        conn.execute("DELETE FROM fcm_data", [])
            .map_err(|e| BbError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO fcm_data (
                project_id, storage_bucket, api_key,
                firebase_url, client_id, application_id
            ) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                self.project_id,
                self.storage_bucket,
                self.api_key,
                self.firebase_url,
                self.client_id,
                self.application_id,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    /// Load the FCM data from the database (returns None if not configured).
    pub fn load(conn: &rusqlite::Connection) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM fcm_data LIMIT 1", [], Self::from_row) {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcm_from_json() {
        let json = serde_json::json!({
            "projectID": "my-project",
            "apiKey": "key-123",
            "applicationID": "app-456"
        });
        let fcm = FcmData::from_server_map(&json).unwrap();
        assert!(fcm.is_valid());
        assert_eq!(fcm.project_id.as_deref(), Some("my-project"));
    }

    #[test]
    fn test_fcm_invalid() {
        let json = serde_json::json!({});
        let fcm = FcmData::from_server_map(&json).unwrap();
        assert!(!fcm.is_valid());
    }
}
