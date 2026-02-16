//! BlueBubbles Socket - Socket.IO client for real-time event streaming.
//!
//! This crate provides the Socket.IO connection manager that handles:
//! - Real-time event streaming from the BlueBubbles server
//! - Automatic reconnection with exponential backoff and jitter
//! - AES-256-CBC encryption/decryption of socket payloads
//! - Connection health monitoring with configurable ping intervals
//! - Event dispatching via tokio broadcast channels
//! - Server URL change without connection restart

pub mod crypto;
pub mod events;
pub mod manager;

// Re-export key types
pub use events::{
    ConnectionState, EventDispatcher, SocketEvent, SocketEventType,
    TypingIndicatorPayload, ChatReadStatusPayload,
    FtCallStatusPayload, AliasesRemovedPayload,
};
pub use manager::{SocketManager, ReconnectConfig, HealthCheckConfig};
pub use crypto::AesCrypto;
