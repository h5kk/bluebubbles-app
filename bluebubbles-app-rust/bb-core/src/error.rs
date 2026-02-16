//! Global error types for the BlueBubbles application.
//!
//! All error categories across the application are unified into a single
//! `BbError` enum with conversions from underlying library errors.

use thiserror::Error;

/// Convenience type alias for Results using BbError.
pub type BbResult<T> = Result<T, BbError>;

/// Unified error type covering all error categories in BlueBubbles.
#[derive(Error, Debug)]
pub enum BbError {
    // -- Configuration errors --
    /// Failed to load or parse application configuration.
    #[error("configuration error: {0}")]
    Config(String),

    /// A required configuration value is missing.
    #[error("missing configuration: {0}")]
    MissingConfig(String),

    // -- Database errors --
    /// SQLite database error.
    #[error("database error: {0}")]
    Database(String),

    /// Database migration failed.
    #[error("migration error: {0}")]
    Migration(String),

    /// Database connection pool error.
    #[error("connection pool error: {0}")]
    Pool(String),

    /// Database integrity check failed.
    #[error("database integrity check failed: {0}")]
    IntegrityCheck(String),

    // -- Network errors --
    /// HTTP request failed.
    #[error("http error: {0}")]
    Http(String),

    /// HTTP request timed out.
    #[error("request timeout: {0}")]
    Timeout(String),

    /// Socket.IO connection error.
    #[error("socket error: {0}")]
    Socket(String),

    /// Socket.IO disconnected unexpectedly.
    #[error("socket disconnected")]
    SocketDisconnected,

    /// Server returned an error response.
    #[error("server error (status {status}): {message}")]
    ServerError {
        /// HTTP status code.
        status: u16,
        /// Error message from server.
        message: String,
    },

    /// Authentication failed.
    #[error("authentication failed: {0}")]
    AuthFailed(String),

    /// SSL/TLS certificate error.
    #[error("certificate error: {0}")]
    CertificateError(String),

    // -- Sync errors --
    /// Full sync operation failed.
    #[error("full sync failed: {0}")]
    FullSync(String),

    /// Incremental sync operation failed.
    #[error("incremental sync failed: {0}")]
    IncrementalSync(String),

    // -- Message errors --
    /// Failed to send a message.
    #[error("send failed: {0}")]
    SendFailed(String),

    /// Message not found.
    #[error("message not found: {0}")]
    MessageNotFound(String),

    /// Chat not found.
    #[error("chat not found: {0}")]
    ChatNotFound(String),

    // -- Crypto errors --
    /// AES encryption/decryption error.
    #[error("crypto error: {0}")]
    Crypto(String),

    // -- File/IO errors --
    /// File system operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    // -- Service errors --
    /// A service failed to initialize.
    #[error("service init error: {0}")]
    ServiceInit(String),

    /// A service is not yet initialized.
    #[error("service not initialized: {0}")]
    ServiceNotInitialized(String),

    /// A service operation failed.
    #[error("service error: {0}")]
    Service(String),

    // -- Notification errors --
    /// Desktop notification failed.
    #[error("notification error: {0}")]
    Notification(String),

    // -- Generic --
    /// An unexpected internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Wrapping anyhow errors for interop.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<serde_json::Error> for BbError {
    fn from(e: serde_json::Error) -> Self {
        BbError::Serialization(e.to_string())
    }
}

impl From<toml::de::Error> for BbError {
    fn from(e: toml::de::Error) -> Self {
        BbError::Config(e.to_string())
    }
}

/// Message-level error codes matching the Flutter app's MessageError enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i32)]
pub enum MessageError {
    /// No error.
    NoError = 0,
    /// Request timed out.
    Timeout = 4,
    /// No network connection.
    NoConnection = 1000,
    /// Bad request (400).
    BadRequest = 1001,
    /// Server error (500).
    ServerError = 1002,
    /// No access to this conversation.
    NoAccessToConversation = 1003,
    /// Generic send failure.
    FailedToSend = 1004,
    /// Unknown error.
    Unknown = 9999,
}

impl MessageError {
    /// Convert an integer code to a MessageError variant.
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Self::NoError,
            4 => Self::Timeout,
            1000 => Self::NoConnection,
            1001 => Self::BadRequest,
            1002 => Self::ServerError,
            1003 => Self::NoAccessToConversation,
            1004 => Self::FailedToSend,
            _ => Self::Unknown,
        }
    }

    /// Get the integer code for this error.
    pub fn code(&self) -> i32 {
        *self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_error_roundtrip() {
        let err = MessageError::Timeout;
        assert_eq!(err.code(), 4);
        assert_eq!(MessageError::from_code(4), MessageError::Timeout);
    }

    #[test]
    fn test_message_error_unknown_code() {
        assert_eq!(MessageError::from_code(42), MessageError::Unknown);
    }

    #[test]
    fn test_bb_error_display() {
        let err = BbError::Config("bad value".to_string());
        assert_eq!(err.to_string(), "configuration error: bad value");
    }
}
