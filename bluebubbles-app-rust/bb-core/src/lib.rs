//! BlueBubbles Core - Foundation types, error handling, configuration, and logging.
//!
//! This crate provides the shared foundation used by all other BlueBubbles crates:
//! - Application configuration (server URL, auth, settings)
//! - Global error types covering all error categories
//! - Structured logging with tracing
//! - Platform detection utilities
//! - Common constants and type aliases

pub mod config;
pub mod error;
pub mod logging;
pub mod platform;
pub mod constants;

// Re-export commonly used items at the crate root
pub use config::AppConfig;
pub use error::{BbError, BbResult};
pub use logging::init_logging;
pub use platform::Platform;
