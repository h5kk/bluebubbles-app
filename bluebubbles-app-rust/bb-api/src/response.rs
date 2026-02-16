//! Server response types.
//!
//! All BlueBubbles server REST responses follow a common envelope format
//! with status, message, and optional data/error fields.

use serde::{Deserialize, Serialize};

/// Standard server response envelope.
///
/// All REST API responses from the BlueBubbles server follow this format:
/// ```json
/// { "status": 200, "message": "Success!", "data": { ... } }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse<T = serde_json::Value> {
    /// HTTP-like status code from the server.
    pub status: u16,
    /// Human-readable message.
    #[serde(default)]
    pub message: String,
    /// Response payload data (type varies by endpoint).
    pub data: Option<T>,
    /// Error details (present only on error responses).
    pub error: Option<ServerError>,
    /// Metadata (pagination info, totals, etc).
    pub metadata: Option<serde_json::Value>,
}

/// Server error detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerError {
    /// Error type identifier.
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    /// Error message.
    pub message: Option<String>,
}

impl<T> ServerResponse<T> {
    /// Whether the response indicates success (status 200).
    pub fn is_success(&self) -> bool {
        self.status == 200
    }

    /// Whether the response indicates an error.
    pub fn is_error(&self) -> bool {
        self.status != 200
    }

    /// Get the error message if this is an error response.
    pub fn error_message(&self) -> Option<String> {
        if self.is_error() {
            self.error
                .as_ref()
                .and_then(|e| e.message.clone())
                .or_else(|| Some(self.message.clone()))
        } else {
            None
        }
    }
}

/// Server payload used in Socket.IO events.
///
/// Socket event data is wrapped in this format with event type,
/// encryption state, and payload data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPayload {
    /// The payload type (e.g. "NEW_MESSAGE", "UPDATED_MESSAGE").
    #[serde(rename = "type")]
    pub payload_type: String,
    /// Optional subtype.
    #[serde(default)]
    pub subtype: Option<String>,
    /// Whether the data field is encrypted.
    #[serde(default)]
    pub encrypted: bool,
    /// Whether this is a partial payload.
    #[serde(default)]
    pub partial: bool,
    /// Encoding of the data field.
    #[serde(default)]
    pub encoding: Option<String>,
    /// Encryption type (e.g. "AES_PB").
    #[serde(rename = "encryptionType", default)]
    pub encryption_type: Option<String>,
    /// The event payload data.
    pub data: serde_json::Value,
}

impl ServerPayload {
    /// Whether this payload needs decryption.
    pub fn needs_decryption(&self) -> bool {
        self.encrypted
    }

    /// Whether this payload is a message event.
    pub fn is_message(&self) -> bool {
        self.payload_type == "NEW_MESSAGE"
            || self.payload_type == "UPDATED_MESSAGE"
            || self.payload_type == "MESSAGE"
    }
}

/// Pagination metadata from query responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMetadata {
    /// Total number of results matching the query.
    pub total: Option<i64>,
    /// Current offset.
    pub offset: Option<i64>,
    /// Current limit.
    pub limit: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_response_success() {
        let json = r#"{"status":200,"message":"Success!","data":{"total":42}}"#;
        let resp: ServerResponse = serde_json::from_str(json).unwrap();
        assert!(resp.is_success());
        assert!(resp.error_message().is_none());
    }

    #[test]
    fn test_server_response_error() {
        let json = r#"{"status":401,"message":"Unauthorized","error":{"type":"auth","message":"Bad password"}}"#;
        let resp: ServerResponse = serde_json::from_str(json).unwrap();
        assert!(resp.is_error());
        assert_eq!(resp.error_message().unwrap(), "Bad password");
    }

    #[test]
    fn test_server_payload() {
        let json = r#"{"type":"NEW_MESSAGE","data":{"guid":"msg-1"}}"#;
        let payload: ServerPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.payload_type, "NEW_MESSAGE");
        assert!(payload.is_message());
        assert!(!payload.needs_decryption());
    }

    #[test]
    fn test_server_payload_encrypted() {
        let json =
            r#"{"type":"NEW_MESSAGE","encrypted":true,"encryptionType":"AES_PB","data":"ciphertext"}"#;
        let payload: ServerPayload = serde_json::from_str(json).unwrap();
        assert!(payload.needs_decryption());
    }
}
