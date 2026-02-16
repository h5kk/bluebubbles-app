//! Handle service for managing handles (phone numbers / email addresses).
//!
//! Provides CRUD operations, availability checks (iMessage/FaceTime),
//! and focus state queries for handles via the server API and local database.

use tracing::{info, warn, debug};

use bb_core::error::{BbError, BbResult};
use bb_models::{Database, Handle};
use bb_models::queries;
use bb_api::ApiClient;
use bb_api::endpoints::handles::HandleQuery;

use crate::event_bus::EventBus;
use crate::service::{Service, ServiceState};

/// Result of an iMessage or FaceTime availability check.
#[derive(Debug, Clone)]
pub struct AvailabilityResult {
    /// The address that was checked.
    pub address: String,
    /// Whether the address is available on the service.
    pub available: bool,
    /// Raw response data from the server.
    pub raw: serde_json::Value,
}

/// Focus/DND state of a handle.
#[derive(Debug, Clone)]
pub struct FocusState {
    /// The address queried.
    pub address: String,
    /// Whether focus/DND is active.
    pub is_focused: bool,
    /// Raw response data from the server.
    pub raw: serde_json::Value,
}

/// Service for handle management.
///
/// Handles represent contact endpoints (phone numbers and email addresses)
/// in the iMessage system. This service provides local database CRUD,
/// server-side availability checks, and focus state queries.
pub struct HandleService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
}

impl HandleService {
    /// Create a new HandleService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
        }
    }

    /// List all handles from the local database.
    pub fn list_handles(&self) -> BbResult<Vec<Handle>> {
        let conn = self.database.conn()?;
        queries::list_handles(&conn)
    }

    /// Search handles by address pattern.
    pub fn search_handles(&self, query: &str, limit: i64) -> BbResult<Vec<Handle>> {
        let conn = self.database.conn()?;
        queries::search_handles(&conn, query, limit)
    }

    /// Find a handle by its address and service in the local database.
    pub fn find_by_address(&self, address: &str, service: &str) -> BbResult<Option<Handle>> {
        let conn = self.database.conn()?;
        Handle::find_by_address(&conn, address, service)
    }

    /// Find a handle by its database ID.
    pub fn find_by_id(&self, id: i64) -> BbResult<Option<Handle>> {
        let conn = self.database.conn()?;
        Handle::find_by_id(&conn, id)
    }

    /// Save or update a handle in the local database.
    pub fn save_handle(&self, handle: &mut Handle) -> BbResult<i64> {
        let conn = self.database.conn()?;
        handle.save(&conn)
    }

    /// Query handles from the server with pagination.
    pub async fn query_from_server(
        &self,
        api: &ApiClient,
        offset: i64,
        limit: i64,
    ) -> BbResult<Vec<Handle>> {
        let query = HandleQuery {
            with: vec![],
            address: None,
            offset,
            limit,
        };

        let raw_handles = api.query_handles(&query).await?;
        let mut handles = Vec::new();
        let conn = self.database.conn()?;

        for raw in &raw_handles {
            match Handle::from_server_map(raw) {
                Ok(mut handle) => {
                    // Persist to local DB
                    if let Err(e) = handle.save(&conn) {
                        warn!("failed to save handle from server: {e}");
                    }
                    handles.push(handle);
                }
                Err(e) => {
                    warn!("failed to parse handle from server: {e}");
                }
            }
        }

        debug!("fetched {} handles from server", handles.len());
        Ok(handles)
    }

    /// Check iMessage availability for an address.
    ///
    /// Returns whether the address is registered on iMessage.
    pub async fn check_imessage_availability(
        &self,
        api: &ApiClient,
        address: &str,
    ) -> BbResult<AvailabilityResult> {
        let data = api.check_imessage_availability(address).await?;
        let available = data
            .get("available")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        debug!("iMessage availability for {address}: {available}");

        Ok(AvailabilityResult {
            address: address.to_string(),
            available,
            raw: data,
        })
    }

    /// Check FaceTime availability for an address.
    pub async fn check_facetime_availability(
        &self,
        api: &ApiClient,
        address: &str,
    ) -> BbResult<AvailabilityResult> {
        let data = api.check_facetime_availability(address).await?;
        let available = data
            .get("available")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        debug!("FaceTime availability for {address}: {available}");

        Ok(AvailabilityResult {
            address: address.to_string(),
            available,
            raw: data,
        })
    }

    /// Query the focus/DND state of a handle address.
    pub async fn get_focus_state(
        &self,
        api: &ApiClient,
        address: &str,
    ) -> BbResult<FocusState> {
        let data = api.get_handle_focus(address).await?;
        let is_focused = data
            .get("isFocused")
            .or_else(|| data.get("focused"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(FocusState {
            address: address.to_string(),
            is_focused,
            raw: data,
        })
    }

    /// Get handles for a specific chat from the local database.
    pub fn handles_for_chat(&self, chat_id: i64) -> BbResult<Vec<Handle>> {
        let conn = self.database.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT h.* FROM handles h
                 INNER JOIN chat_handle_join chj ON chj.handle_id = h.id
                 WHERE chj.chat_id = ?1"
            )
            .map_err(|e| BbError::Database(e.to_string()))?;

        let handles = stmt
            .query_map(rusqlite::params![chat_id], Handle::from_row)
            .map_err(|e| BbError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(handles)
    }

    /// Get the display name for a handle, checking the linked contact first.
    pub fn display_name(&self, handle: &Handle) -> String {
        handle.display_name()
    }
}

impl Service for HandleService {
    fn name(&self) -> &str {
        "handle"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("handle service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("handle service stopped");
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
    fn test_handle_service_name() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = HandleService::new(db, bus);
        assert_eq!(svc.name(), "handle");
    }

    #[test]
    fn test_list_handles_empty() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = HandleService::new(db, bus);
        let handles = svc.list_handles().unwrap();
        assert!(handles.is_empty());
    }

    #[test]
    fn test_save_and_find_handle() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = HandleService::new(db, bus);

        let json = serde_json::json!({
            "address": "+15551234567",
            "service": "iMessage"
        });

        let mut handle = Handle::from_server_map(&json).unwrap();
        let id = svc.save_handle(&mut handle).unwrap();
        assert!(id > 0);

        let found = svc.find_by_address("+15551234567", "iMessage").unwrap();
        assert!(found.is_some());
    }
}
