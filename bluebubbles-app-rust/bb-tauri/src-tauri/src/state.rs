//! Application state management for the Tauri backend.
//!
//! Holds shared references to the service registry, database, API client,
//! and socket manager, accessible from Tauri command handlers.

use std::sync::Arc;
use tokio::sync::RwLock;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use bb_models::Database;
use bb_api::ApiClient;
use bb_socket::{SocketManager, EventDispatcher};
use bb_services::ServiceRegistry;

/// Shared application state managed by Tauri.
pub struct AppState {
    /// The service registry holding all backend services.
    pub registry: Arc<RwLock<ServiceRegistry>>,
    /// Configuration handle for reading/writing app config.
    pub config: ConfigHandle,
    /// Database connection pool.
    pub database: Database,
    /// Socket manager for real-time events.
    pub socket_manager: Arc<RwLock<Option<SocketManager>>>,
    /// Whether the initial setup has been completed.
    pub setup_complete: Arc<RwLock<bool>>,
}

impl AppState {
    /// Create a new AppState with the given infrastructure.
    pub fn new(
        config: ConfigHandle,
        database: Database,
        dispatcher: EventDispatcher,
    ) -> Self {
        let mut registry = ServiceRegistry::new(
            config.clone(),
            database.clone(),
            dispatcher,
        );

        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("bluebubbles")
            .join("cache");

        registry.register_all(cache_dir);

        Self {
            registry: Arc::new(RwLock::new(registry)),
            config,
            database,
            socket_manager: Arc::new(RwLock::new(None)),
            setup_complete: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize all services.
    pub async fn init_services(&self) -> BbResult<()> {
        let registry = self.registry.read().await;
        registry.init_all().await
    }

    /// Get a reference to the API client from the registry.
    pub async fn api_client(&self) -> BbResult<ApiClient> {
        let registry = self.registry.read().await;
        registry.api_client().await
    }

    /// Set the API client on the registry.
    pub async fn set_api_client(&self, client: ApiClient) {
        let registry = self.registry.read().await;
        registry.set_api_client(client).await;
    }

    /// Check if setup is complete.
    pub async fn is_setup_complete(&self) -> bool {
        *self.setup_complete.read().await
    }

    /// Mark setup as complete.
    pub async fn mark_setup_complete(&self) {
        let mut complete = self.setup_complete.write().await;
        *complete = true;
    }
}
