//! BlueBubbles Models - Database schema, models, migrations, and query builders.
//!
//! This crate owns all data persistence: SQLite database initialization,
//! entity models matching the BlueBubbles server schema, versioned migrations,
//! and query builders for common access patterns.

pub mod db;
pub mod schema;
pub mod models;
pub mod queries;
pub mod migrations;

// Re-export key types
pub use db::{Database, DbPool};
pub use models::chat::Chat;
pub use models::message::Message;
pub use models::handle::Handle;
pub use models::attachment::Attachment;
pub use models::contact::Contact;
pub use models::fcm_data::FcmData;
pub use models::theme::ThemeStruct;
pub use models::scheduled_message::ScheduledMessage;
pub use models::settings::Settings;
pub use models::findmy::{FindMyLocationItem, FindMyDevice, FindMyLocation, FindMyAddress};
