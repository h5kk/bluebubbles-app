//! FCM service for Firebase Cloud Messaging registration and push notification management.
//!
//! Handles fetching FCM configuration from the server, registering this device
//! for push notifications, and managing the FCM token lifecycle.

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, debug, error};

use bb_core::error::BbResult;
use bb_models::{Database, FcmData};
use bb_api::ApiClient;

use crate::event_bus::EventBus;
use crate::service::{Service, ServiceState};

/// Service for Firebase Cloud Messaging registration and configuration.
///
/// Manages the FCM client configuration obtained from the server, registers
/// the device for push notifications, and handles token refresh. FCM data is
/// persisted to the local database so the client can re-register on restart.
pub struct FcmService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
    /// Cached FCM configuration data.
    fcm_data: Arc<Mutex<Option<FcmData>>>,
    /// Device name used for registration.
    device_name: Arc<Mutex<String>>,
    /// Whether the device is currently registered.
    registered: Arc<Mutex<bool>>,
}

impl FcmService {
    /// Create a new FcmService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
            fcm_data: Arc::new(Mutex::new(None)),
            device_name: Arc::new(Mutex::new(String::new())),
            registered: Arc::new(Mutex::new(false)),
        }
    }

    /// Load cached FCM data from the local database.
    ///
    /// Called during init to restore the last known FCM configuration
    /// so registration can proceed without waiting for a server roundtrip.
    pub fn load_cached_fcm_data(&self) -> BbResult<Option<FcmData>> {
        let conn = self.database.conn()?;
        FcmData::load(&conn)
    }

    /// Fetch the FCM client configuration from the server and persist it locally.
    ///
    /// The server provides the Firebase project credentials needed to initialise
    /// the FCM SDK on the client side.
    pub async fn fetch_fcm_config(&self, api: &ApiClient) -> BbResult<FcmData> {
        let data = api.get_fcm_client().await?;
        let mut fcm = FcmData::from_server_map(&data)?;

        // Persist to local database
        let conn = self.database.conn()?;
        fcm.save(&conn)?;

        let mut cached = self.fcm_data.lock().await;
        *cached = Some(fcm.clone());

        info!("fetched and cached FCM configuration (project: {:?})", fcm.project_id);
        Ok(fcm)
    }

    /// Register this device with the BlueBubbles server for push notifications.
    ///
    /// Uses the device name and a unique identifier to register. If registration
    /// fails with stale FCM data, fetches fresh config and retries once.
    pub async fn register_device(
        &self,
        api: &ApiClient,
        device_name: &str,
        device_identifier: &str,
    ) -> BbResult<()> {
        {
            let mut name = self.device_name.lock().await;
            *name = device_name.to_string();
        }

        match api.register_fcm_device(device_name, device_identifier).await {
            Ok(_) => {
                let mut reg = self.registered.lock().await;
                *reg = true;
                info!("registered FCM device: {device_name}");
                Ok(())
            }
            Err(e) => {
                warn!("FCM registration failed, refreshing config and retrying: {e}");

                // Retry with fresh FCM data
                if let Err(fetch_err) = self.fetch_fcm_config(api).await {
                    error!("failed to refresh FCM config: {fetch_err}");
                    return Err(e);
                }

                match api.register_fcm_device(device_name, device_identifier).await {
                    Ok(_) => {
                        let mut reg = self.registered.lock().await;
                        *reg = true;
                        info!("registered FCM device on retry: {device_name}");
                        Ok(())
                    }
                    Err(retry_err) => {
                        error!("FCM registration failed after retry: {retry_err}");
                        Err(retry_err)
                    }
                }
            }
        }
    }

    /// Check if the device is currently registered for push notifications.
    pub async fn is_registered(&self) -> bool {
        *self.registered.lock().await
    }

    /// Get the cached FCM data, if available.
    pub async fn fcm_data(&self) -> Option<FcmData> {
        self.fcm_data.lock().await.clone()
    }

    /// Check if the cached FCM data is valid and sufficient for registration.
    pub async fn has_valid_config(&self) -> bool {
        self.fcm_data
            .lock()
            .await
            .as_ref()
            .map(|d| d.is_valid())
            .unwrap_or(false)
    }
}

impl Service for FcmService {
    fn name(&self) -> &str {
        "fcm"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Initializing;

        // Load any cached FCM data from a previous session
        match self.load_cached_fcm_data() {
            Ok(Some(data)) => {
                debug!("loaded cached FCM data (project: {:?})", data.project_id);
                // Use try_lock since we are in a sync context during init
                if let Ok(mut cached) = self.fcm_data.try_lock() {
                    *cached = Some(data);
                }
            }
            Ok(None) => {
                debug!("no cached FCM data found");
            }
            Err(e) => {
                warn!("failed to load cached FCM data: {e}");
            }
        }

        self.state = ServiceState::Running;
        info!("FCM service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("FCM service stopped");
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
    fn test_fcm_service_name() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = FcmService::new(db, bus);
        assert_eq!(svc.name(), "fcm");
    }

    #[test]
    fn test_fcm_service_init() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let mut svc = FcmService::new(db, bus);
        svc.init().unwrap();
        assert_eq!(svc.state(), ServiceState::Running);
    }

    #[tokio::test]
    async fn test_fcm_not_registered_initially() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = FcmService::new(db, bus);
        assert!(!svc.is_registered().await);
        assert!(!svc.has_valid_config().await);
    }
}
