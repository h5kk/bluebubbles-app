//! Scheduled message service for managing scheduled messages via the server API.
//!
//! Provides CRUD operations for scheduled messages, local caching of the
//! schedule list, and checking for messages that are due for sending.

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, debug};

use bb_core::error::BbResult;
use bb_models::{Database, ScheduledMessage};
use bb_models::models::scheduled_message::status as msg_status;
use bb_api::ApiClient;
use bb_api::endpoints::messages::ScheduleMessageParams;

use crate::event_bus::EventBus;
use crate::service::{Service, ServiceState};

/// Service for scheduled message management.
///
/// Scheduled messages are primarily managed on the server side. This service
/// provides a client-side interface for creating, updating, deleting, and
/// listing scheduled messages while maintaining a local cache for offline
/// access and status display.
pub struct ScheduledMessageService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
    /// Locally cached scheduled messages.
    cached: Arc<Mutex<Vec<ScheduledMessage>>>,
}

impl ScheduledMessageService {
    /// Create a new ScheduledMessageService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
            cached: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Fetch all scheduled messages from the server and update the local cache.
    pub async fn fetch_all(&self, api: &ApiClient) -> BbResult<Vec<ScheduledMessage>> {
        let raw_list = api.get_scheduled_messages().await?;
        let mut messages = Vec::new();

        let conn = self.database.conn()?;

        for raw in &raw_list {
            match ScheduledMessage::from_server_map(raw) {
                Ok(mut msg) => {
                    // Persist to local database for offline access
                    if let Err(e) = msg.save(&conn) {
                        warn!("failed to cache scheduled message: {e}");
                    }
                    messages.push(msg);
                }
                Err(e) => {
                    warn!("failed to parse scheduled message: {e}");
                }
            }
        }

        info!("fetched {} scheduled messages from server", messages.len());
        let mut cached = self.cached.lock().await;
        *cached = messages.clone();
        Ok(messages)
    }

    /// Create a new scheduled message on the server.
    pub async fn create(
        &self,
        api: &ApiClient,
        params: &ScheduleMessageParams,
    ) -> BbResult<ScheduledMessage> {
        let result = api.create_scheduled_message(params).await?;
        let mut msg = ScheduledMessage::from_server_map(&result)?;

        // Cache locally
        let conn = self.database.conn()?;
        msg.save(&conn)?;

        let mut cached = self.cached.lock().await;
        cached.push(msg.clone());

        info!("created scheduled message for chat: {}", msg.chat_guid);
        Ok(msg)
    }

    /// Update an existing scheduled message on the server.
    pub async fn update(
        &self,
        api: &ApiClient,
        id: i64,
        params: &ScheduleMessageParams,
    ) -> BbResult<ScheduledMessage> {
        let result = api.update_scheduled_message(id, params).await?;
        let mut msg = ScheduledMessage::from_server_map(&result)?;

        // Update local cache
        let conn = self.database.conn()?;
        msg.save(&conn)?;

        let mut cached = self.cached.lock().await;
        if let Some(pos) = cached.iter().position(|m| m.id == Some(id)) {
            cached[pos] = msg.clone();
        }

        info!("updated scheduled message id={id}");
        Ok(msg)
    }

    /// Delete a scheduled message from the server and local cache.
    pub async fn delete(&self, api: &ApiClient, id: i64) -> BbResult<()> {
        api.delete_scheduled_message(id).await?;

        // Remove from local database
        let conn = self.database.conn()?;
        ScheduledMessage::delete(&conn, id)?;

        // Remove from in-memory cache
        let mut cached = self.cached.lock().await;
        cached.retain(|m| m.id != Some(id));

        info!("deleted scheduled message id={id}");
        Ok(())
    }

    /// List all locally cached scheduled messages.
    pub async fn list_cached(&self) -> Vec<ScheduledMessage> {
        self.cached.lock().await.clone()
    }

    /// List all scheduled messages from the local database.
    pub fn list_from_db(&self) -> BbResult<Vec<ScheduledMessage>> {
        let conn = self.database.conn()?;
        ScheduledMessage::load_all(&conn)
    }

    /// Get pending scheduled messages that are due (scheduled_for <= now).
    pub fn get_due_messages(&self) -> BbResult<Vec<ScheduledMessage>> {
        let conn = self.database.conn()?;
        let all = ScheduledMessage::load_all(&conn)?;

        let now = chrono::Utc::now();

        let due: Vec<ScheduledMessage> = all
            .into_iter()
            .filter(|msg| {
                if msg.status != msg_status::PENDING {
                    return false;
                }
                // Parse the scheduled_for timestamp and check if it is in the past
                match chrono::DateTime::parse_from_rfc3339(&msg.scheduled_for) {
                    Ok(scheduled_time) => scheduled_time <= now,
                    Err(_) => {
                        // Try parsing as epoch milliseconds
                        if let Ok(ms) = msg.scheduled_for.parse::<i64>() {
                            let ts = chrono::DateTime::from_timestamp_millis(ms);
                            ts.map(|t| t <= now).unwrap_or(false)
                        } else {
                            false
                        }
                    }
                }
            })
            .collect();

        debug!("{} scheduled messages are due", due.len());
        Ok(due)
    }

    /// Get the count of pending scheduled messages.
    pub async fn pending_count(&self) -> usize {
        let cached = self.cached.lock().await;
        cached.iter().filter(|m| m.status == msg_status::PENDING).count()
    }

    /// Find a cached scheduled message by its server ID.
    pub async fn find_by_id(&self, id: i64) -> Option<ScheduledMessage> {
        let cached = self.cached.lock().await;
        cached.iter().find(|m| m.id == Some(id)).cloned()
    }
}

impl Service for ScheduledMessageService {
    fn name(&self) -> &str {
        "scheduled_messages"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Initializing;

        // Load any cached scheduled messages from the local database
        match self.list_from_db() {
            Ok(messages) => {
                if let Ok(mut cached) = self.cached.try_lock() {
                    *cached = messages;
                }
                debug!("loaded cached scheduled messages from database");
            }
            Err(e) => {
                warn!("failed to load cached scheduled messages: {e}");
            }
        }

        self.state = ServiceState::Running;
        info!("scheduled message service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("scheduled message service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&path, &config).unwrap();
        std::mem::forget(dir);
        db
    }

    #[test]
    fn test_scheduled_service_name() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ScheduledMessageService::new(db, bus);
        assert_eq!(svc.name(), "scheduled_messages");
    }

    #[test]
    fn test_scheduled_service_init() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let mut svc = ScheduledMessageService::new(db, bus);
        svc.init().unwrap();
        assert_eq!(svc.state(), ServiceState::Running);
    }

    #[tokio::test]
    async fn test_empty_cache_initially() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ScheduledMessageService::new(db, bus);
        assert!(svc.list_cached().await.is_empty());
        assert_eq!(svc.pending_count().await, 0);
    }

    #[test]
    fn test_list_from_empty_db() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ScheduledMessageService::new(db, bus);
        let list = svc.list_from_db().unwrap();
        assert!(list.is_empty());
    }
}
