//! BlueBubbles Services - Business logic and service layer.
//!
//! This crate provides the service trait, service registry for dependency
//! injection, and all concrete service implementations covering:
//! - Sync (full and incremental, ROWID-based)
//! - Chat management (CRUD, mute, pin, archive, participants)
//! - Message handling (send, receive, edit, react, retry)
//! - Contact management (phone suffix matching, two-pass sync)
//! - Attachment management (download queue, live photos, caching)
//! - Notifications (grouped, filtered, FaceTime)
//! - Settings persistence (typed accessors for all config sections)
//! - Theme management (CRUD, presets, server backup)
//! - Message queue and retry (exponential backoff, GUID tracking)
//! - Action handling (socket event routing)
//! - Event bus (typed intra-service communication)
//! - Lifecycle management (startup, shutdown, foreground/background)
//! - FCM registration and push notification management
//! - FindMy device and friend location tracking
//! - FaceTime call handling
//! - Settings and theme backup/restore
//! - Global search across messages, chats, contacts
//! - Cache management with LRU eviction
//! - Scheduled message management
//! - Handle/address management and availability checks

pub mod service;
pub mod registry;
pub mod event_bus;
pub mod action_handler;
pub mod lifecycle;
pub mod sync;
pub mod chat;
pub mod message;
pub mod contact;
pub mod attachment;
pub mod notification;
pub mod settings;
pub mod queue;
pub mod theme;
pub mod fcm;
pub mod findmy;
pub mod facetime;
pub mod backup;
pub mod search;
pub mod cache;
pub mod scheduled;
pub mod handle;

// Re-export key types
pub use service::{Service, ServiceState};
pub use registry::ServiceRegistry;
pub use event_bus::{AppEvent, EventBus};
pub use action_handler::ActionHandler;
pub use lifecycle::{LifecycleService, LifecyclePhase};
pub use fcm::FcmService;
pub use findmy::FindMyService;
pub use facetime::FaceTimeService;
pub use backup::BackupService;
pub use search::SearchService;
pub use cache::CacheService;
pub use scheduled::ScheduledMessageService;
pub use handle::HandleService;
