//! Contact entity model.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Row};
use bb_core::error::{BbError, BbResult};

/// Represents a contact in the BlueBubbles system.
///
/// Contacts are synced from the macOS server's address book. They hold
/// display names, phone numbers, email addresses, and optional avatars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<i64>,
    pub external_id: Option<String>,
    pub display_name: String,
    /// JSON array of phone number strings.
    pub phones: String,
    /// JSON array of email strings.
    pub emails: String,
    /// Raw avatar image bytes (JPEG/PNG).
    #[serde(skip)]
    pub avatar: Option<Vec<u8>>,
    /// JSON-encoded structured name (first, last, middle, etc).
    pub structured_name: Option<String>,
}

/// Parsed structured name components.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredName {
    #[serde(rename = "namePrefix")]
    pub prefix: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    #[serde(rename = "middleName")]
    pub middle_name: Option<String>,
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
    #[serde(rename = "nameSuffix")]
    pub suffix: Option<String>,
}

impl Contact {
    /// Create a Contact from a server JSON map.
    pub fn from_server_map(map: &serde_json::Value) -> BbResult<Self> {
        let display_name = map
            .get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let phones = map
            .get("phoneNumbers")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "[]".to_string());

        let emails = map
            .get("emails")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "[]".to_string());

        Ok(Self {
            id: None,
            external_id: map.get("id").and_then(|v| v.as_str()).map(String::from),
            display_name,
            phones,
            emails,
            avatar: None,
            structured_name: map.get("structuredName").map(|v| v.to_string()),
        })
    }

    /// Construct a Contact from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            external_id: row.get("external_id")?,
            display_name: row.get("display_name")?,
            phones: row.get("phones")?,
            emails: row.get("emails")?,
            avatar: row.get("avatar")?,
            structured_name: row.get("structured_name")?,
        })
    }

    // ─── Static finders ──────────────────────────────────────────────────

    /// Find a contact by its local database ID.
    pub fn find_by_id(conn: &Connection, id: i64) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM contacts WHERE id = ?1", [id], Self::from_row) {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find a contact by its external ID.
    pub fn find_by_external_id(conn: &Connection, external_id: &str) -> BbResult<Option<Self>> {
        match conn.query_row(
            "SELECT * FROM contacts WHERE external_id = ?1",
            [external_id],
            Self::from_row,
        ) {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Delete a contact by its local database ID.
    pub fn delete(conn: &Connection, id: i64) -> BbResult<bool> {
        let changed = conn
            .execute("DELETE FROM contacts WHERE id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    // ─── Parsing helpers ─────────────────────────────────────────────────

    /// Parse the phones JSON array into a list of phone number strings.
    pub fn phone_list(&self) -> Vec<String> {
        serde_json::from_str::<Vec<serde_json::Value>>(&self.phones)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else {
                    v.get("address").and_then(|a| a.as_str()).map(String::from)
                }
            })
            .collect()
    }

    /// Parse the emails JSON array into a list of email strings.
    pub fn email_list(&self) -> Vec<String> {
        serde_json::from_str::<Vec<serde_json::Value>>(&self.emails)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else {
                    v.get("address").and_then(|a| a.as_str()).map(String::from)
                }
            })
            .collect()
    }

    /// Parse the structured name JSON.
    pub fn structured_name_parsed(&self) -> Option<StructuredName> {
        self.structured_name.as_deref().and_then(|s| serde_json::from_str(s).ok())
    }

    /// Get initials from structured name or display name.
    pub fn initials(&self) -> String {
        if let Some(sn) = self.structured_name_parsed() {
            let first = sn.given_name.as_deref()
                .and_then(|n| n.chars().next())
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default();
            let last = sn.family_name.as_deref()
                .and_then(|n| n.chars().next())
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default();
            if !first.is_empty() || !last.is_empty() {
                return format!("{first}{last}");
            }
        }
        self.display_name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    /// Whether this contact has an avatar loaded.
    pub fn has_avatar(&self) -> bool {
        self.avatar.as_ref().map_or(false, |a| !a.is_empty())
    }

    /// Whether this contact matches a given address (phone or email).
    pub fn matches_address(&self, address: &str) -> bool {
        let normalized = normalize_address(address);
        self.phone_list()
            .iter()
            .any(|p| normalize_address(p) == normalized)
            || self
                .email_list()
                .iter()
                .any(|e| e.eq_ignore_ascii_case(address))
    }

    // ─── Persistence ─────────────────────────────────────────────────────

    /// Upsert this contact into the database. Returns the local database ID.
    pub fn save(&mut self, conn: &Connection) -> BbResult<i64> {
        conn.execute(
            "INSERT INTO contacts (
                external_id, display_name, phones, emails, avatar, structured_name
            ) VALUES (?1,?2,?3,?4,?5,?6)
            ON CONFLICT(external_id) DO UPDATE SET
                display_name = excluded.display_name,
                phones = excluded.phones,
                emails = excluded.emails,
                avatar = COALESCE(excluded.avatar, avatar),
                structured_name = COALESCE(excluded.structured_name, structured_name)",
            params![
                self.external_id,
                self.display_name,
                self.phones,
                self.emails,
                self.avatar,
                self.structured_name,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        if id > 0 {
            self.id = Some(id);
        } else if let Some(ref ext_id) = self.external_id {
            let existing_id: i64 = conn
                .query_row(
                    "SELECT id FROM contacts WHERE external_id = ?1",
                    [ext_id],
                    |row| row.get(0),
                )
                .map_err(|e| BbError::Database(e.to_string()))?;
            self.id = Some(existing_id);
        }

        Ok(self.id.unwrap_or(0))
    }
}

/// Normalize a phone number by stripping non-digit characters for comparison.
pub fn normalize_address(addr: &str) -> String {
    addr.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_from_json() {
        let json = serde_json::json!({
            "id": "ext-123",
            "displayName": "John Doe",
            "phoneNumbers": ["+15551234567"],
            "emails": ["john@example.com"]
        });
        let contact = Contact::from_server_map(&json).unwrap();
        assert_eq!(contact.display_name, "John Doe");
        assert_eq!(contact.phone_list(), vec!["+15551234567"]);
        assert_eq!(contact.email_list(), vec!["john@example.com"]);
    }

    #[test]
    fn test_contact_matches_address() {
        let json = serde_json::json!({
            "displayName": "Jane",
            "phoneNumbers": ["+15551234567"],
            "emails": ["jane@test.com"]
        });
        let contact = Contact::from_server_map(&json).unwrap();
        assert!(contact.matches_address("+15551234567"));
        assert!(contact.matches_address("jane@test.com"));
        assert!(!contact.matches_address("unknown@test.com"));
    }

    #[test]
    fn test_normalize_address() {
        assert_eq!(normalize_address("+1 (555) 123-4567"), "+15551234567");
    }

    #[test]
    fn test_initials() {
        let contact = Contact {
            id: None,
            external_id: None,
            display_name: "John".to_string(),
            phones: "[]".to_string(),
            emails: "[]".to_string(),
            avatar: None,
            structured_name: Some(r#"{"givenName":"John","familyName":"Doe"}"#.to_string()),
        };
        assert_eq!(contact.initials(), "JD");
    }
}
