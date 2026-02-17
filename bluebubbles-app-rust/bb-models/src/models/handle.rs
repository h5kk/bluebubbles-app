//! Handle (phone number / email address) entity model.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Row};
use bb_core::error::{BbError, BbResult};

/// Represents a handle (phone number or email address) in the BlueBubbles system.
///
/// A handle is a unique identifier for a contact endpoint: either a phone number
/// or an email address, paired with the service type (iMessage or SMS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handle {
    pub id: Option<i64>,
    pub original_rowid: Option<i64>,
    pub address: String,
    pub service: String,
    pub unique_address_service: String,
    pub formatted_address: Option<String>,
    pub country: Option<String>,
    pub color: Option<String>,
    pub default_phone: Option<String>,
    pub default_email: Option<String>,
    pub contact_id: Option<i64>,

    /// Transient: linked contact data (loaded separately).
    #[serde(skip)]
    pub contact: Option<Box<super::contact::Contact>>,
}

impl Handle {
    /// Create a Handle from a server JSON map.
    pub fn from_server_map(map: &serde_json::Value) -> BbResult<Self> {
        let address = map
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BbError::Serialization("handle missing address".into()))?
            .to_string();

        let service = map
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or("iMessage")
            .to_string();

        let unique = format!("{}/{}", address, service);

        Ok(Self {
            id: None,
            original_rowid: map.get("ROWID").and_then(|v| v.as_i64())
                .or_else(|| map.get("originalROWID").and_then(|v| v.as_i64())),
            address: address.clone(),
            service,
            unique_address_service: map
                .get("uniqueAddressAndService")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or(unique),
            formatted_address: map.get("formattedAddress").and_then(|v| v.as_str()).map(String::from),
            country: map.get("country").and_then(|v| v.as_str()).map(String::from),
            color: map.get("color").and_then(|v| v.as_str()).map(String::from),
            default_phone: map.get("defaultPhone").and_then(|v| v.as_str()).map(String::from),
            default_email: map.get("defaultEmail").and_then(|v| v.as_str()).map(String::from),
            contact_id: None,
            contact: None,
        })
    }

    /// Construct a Handle from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            original_rowid: row.get("original_rowid")?,
            address: row.get("address")?,
            service: row.get("service")?,
            unique_address_service: row.get("unique_address_service")?,
            formatted_address: row.get("formatted_address")?,
            country: row.get("country")?,
            color: row.get("color")?,
            default_phone: row.get("default_phone")?,
            default_email: row.get("default_email")?,
            contact_id: row.get("contact_id")?,
            contact: None,
        })
    }

    // ─── Static finders ──────────────────────────────────────────────────

    /// Find a handle by its local database ID.
    pub fn find_by_id(conn: &Connection, id: i64) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM handles WHERE id = ?1", [id], Self::from_row) {
            Ok(h) => Ok(Some(h)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find a handle by address and service.
    pub fn find_by_address(conn: &Connection, address: &str, service: &str) -> BbResult<Option<Self>> {
        let unique = format!("{address}/{service}");
        match conn.query_row(
            "SELECT * FROM handles WHERE unique_address_service = ?1",
            [&unique],
            Self::from_row,
        ) {
            Ok(h) => Ok(Some(h)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find a handle by its original server ROWID.
    pub fn find_by_original_rowid(conn: &Connection, rowid: i64) -> BbResult<Option<Self>> {
        match conn.query_row(
            "SELECT * FROM handles WHERE original_rowid = ?1",
            [rowid],
            Self::from_row,
        ) {
            Ok(h) => Ok(Some(h)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Delete a handle by its local database ID.
    pub fn delete(conn: &Connection, id: i64) -> BbResult<bool> {
        let changed = conn
            .execute("DELETE FROM handles WHERE id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    // ─── Computed properties ─────────────────────────────────────────────

    /// Get a human-readable display name for this handle.
    pub fn display_name(&self) -> String {
        if let Some(ref contact) = self.contact {
            return contact.display_name.clone();
        }
        if let Some(ref formatted) = self.formatted_address {
            if !formatted.is_empty() {
                return formatted.clone();
            }
        }
        self.address.clone()
    }

    /// Get initials derived from display name.
    pub fn initials(&self) -> String {
        let name = self.display_name();
        let parts: Vec<&str> = name.split_whitespace().collect();
        match parts.len() {
            0 => "?".to_string(),
            1 => parts[0].chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default(),
            _ => {
                let first = parts[0].chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
                let last = parts.last().unwrap().chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
                format!("{first}{last}")
            }
        }
    }

    /// Whether this handle is an email address.
    pub fn is_email(&self) -> bool {
        self.address.contains('@')
    }

    /// Whether this handle is a phone number.
    pub fn is_phone(&self) -> bool {
        !self.is_email()
    }

    /// Whether this handle uses iMessage.
    pub fn is_imessage(&self) -> bool {
        self.service == "iMessage"
    }

    // ─── Persistence ─────────────────────────────────────────────────────

    /// Upsert this handle into the database. Returns the local database ID.
    pub fn save(&mut self, conn: &Connection) -> BbResult<i64> {
        conn.execute(
            "INSERT INTO handles (
                original_rowid, address, service, unique_address_service,
                formatted_address, country, color, default_phone,
                default_email, contact_id
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
            ON CONFLICT(unique_address_service) DO UPDATE SET
                formatted_address = COALESCE(excluded.formatted_address, formatted_address),
                color = COALESCE(excluded.color, color),
                default_phone = COALESCE(excluded.default_phone, default_phone),
                default_email = COALESCE(excluded.default_email, default_email),
                contact_id = COALESCE(excluded.contact_id, contact_id)",
            params![
                self.original_rowid,
                self.address,
                self.service,
                self.unique_address_service,
                self.formatted_address,
                self.country,
                self.color,
                self.default_phone,
                self.default_email,
                self.contact_id,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        // Always query for the real ID - last_insert_rowid() is unreliable for upserts
        // because ON CONFLICT DO UPDATE doesn't change it, leaving stale values from
        // previous INSERTs on the same connection.
        let real_id: i64 = conn
            .query_row(
                "SELECT id FROM handles WHERE unique_address_service = ?1",
                [&self.unique_address_service],
                |row| row.get(0),
            )
            .map_err(|e| BbError::Database(e.to_string()))?;
        self.id = Some(real_id);

        Ok(real_id)
    }

    /// Update mutable fields on an existing handle.
    pub fn update(&self, conn: &Connection) -> BbResult<()> {
        let id = self.id.ok_or_else(|| BbError::Database("handle has no id for update".into()))?;
        conn.execute(
            "UPDATE handles SET
                formatted_address = ?1, color = ?2,
                default_phone = ?3, default_email = ?4, contact_id = ?5
            WHERE id = ?6",
            params![
                self.formatted_address,
                self.color,
                self.default_phone,
                self.default_email,
                self.contact_id,
                id,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(())
    }

    /// Merge non-null fields from another handle into this one.
    pub fn merge(&mut self, other: &Handle) {
        if other.formatted_address.is_some() {
            self.formatted_address = other.formatted_address.clone();
        }
        if other.color.is_some() {
            self.color = other.color.clone();
        }
        if other.default_phone.is_some() {
            self.default_phone = other.default_phone.clone();
        }
        if other.default_email.is_some() {
            self.default_email = other.default_email.clone();
        }
        if other.contact_id.is_some() {
            self.contact_id = other.contact_id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_from_json() {
        let json = serde_json::json!({
            "address": "+15551234567",
            "service": "iMessage",
            "country": "US"
        });
        let handle = Handle::from_server_map(&json).unwrap();
        assert_eq!(handle.address, "+15551234567");
        assert!(handle.is_phone());
        assert!(handle.is_imessage());
    }

    #[test]
    fn test_handle_email() {
        let json = serde_json::json!({
            "address": "test@example.com",
            "service": "iMessage"
        });
        let handle = Handle::from_server_map(&json).unwrap();
        assert!(handle.is_email());
        assert!(!handle.is_phone());
    }

    #[test]
    fn test_handle_display_name() {
        let json = serde_json::json!({
            "address": "+15551234567",
            "formattedAddress": "(555) 123-4567"
        });
        let handle = Handle::from_server_map(&json).unwrap();
        assert_eq!(handle.display_name(), "(555) 123-4567");
    }

    #[test]
    fn test_handle_initials() {
        let json = serde_json::json!({"address": "+15551234567"});
        let mut handle = Handle::from_server_map(&json).unwrap();
        // With a contact
        handle.contact = Some(Box::new(super::super::contact::Contact {
            id: None,
            external_id: None,
            display_name: "John Doe".to_string(),
            phones: "[]".to_string(),
            emails: "[]".to_string(),
            avatar: None,
            structured_name: None,
        }));
        assert_eq!(handle.initials(), "JD");
    }
}
