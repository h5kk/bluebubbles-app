//! Service registry for dependency injection and lifecycle management.
//!
//! The registry holds all services, initializes them in order, and provides
//! access to services by type. It also handles ordered shutdown.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

use bb_core::error::{BbError, BbResult};
use bb_core::config::ConfigHandle;
use bb_models::Database;
use bb_api::ApiClient;
use bb_socket::EventDispatcher;

use crate::service::{Service, ServiceState};
use crate::event_bus::EventBus;
use crate::action_handler::ActionHandler;
use crate::lifecycle::LifecycleService;
use crate::sync::SyncService;
use crate::chat::ChatService;
use crate::message::MessageService;
use crate::contact::ContactService;
use crate::attachment::AttachmentService;
use crate::notification::NotificationService;
use crate::settings::SettingsService;
use crate::queue::QueueService;
use crate::theme::ThemeService;
use crate::fcm::FcmService;
use crate::findmy::FindMyService;
use crate::facetime::FaceTimeService;
use crate::backup::BackupService;
use crate::search::SearchService;
use crate::cache::CacheService;
use crate::scheduled::ScheduledMessageService;
use crate::handle::HandleService;

/// Central service registry that manages all application services.
///
/// Provides dependency injection by holding shared references to core
/// infrastructure (database, API client, event dispatcher, config) and
/// managing service lifecycle in the correct order.
pub struct ServiceRegistry {
    /// Application configuration.
    pub config: ConfigHandle,
    /// Database connection pool.
    pub database: Database,
    /// HTTP API client.
    pub api_client: Arc<RwLock<Option<ApiClient>>>,
    /// Event dispatcher for socket events.
    pub dispatcher: EventDispatcher,
    /// Application-level event bus.
    pub event_bus: EventBus,
    /// Registered services in initialization order.
    services: Vec<(String, Arc<RwLock<Box<dyn Service>>>)>,
}

impl ServiceRegistry {
    /// Create a new ServiceRegistry with core infrastructure.
    pub fn new(
        config: ConfigHandle,
        database: Database,
        dispatcher: EventDispatcher,
    ) -> Self {
        Self {
            config,
            database,
            api_client: Arc::new(RwLock::new(None)),
            dispatcher,
            event_bus: EventBus::new(256),
            services: Vec::new(),
        }
    }

    /// Register a service. Services are initialized in registration order.
    pub fn register<S: Service + 'static>(&mut self, service: S) {
        let name = service.name().to_string();
        info!("registered service: {name}");
        self.services
            .push((name, Arc::new(RwLock::new(Box::new(service)))));
    }

    /// Register all default services in the correct dependency order.
    ///
    /// Initialization order:
    /// 1. Settings (no deps)
    /// 2. Queue (no deps)
    /// 3. Notification (config)
    /// 4. Contact (database, event_bus)
    /// 5. Chat (database, event_bus)
    /// 6. Message (database, event_bus)
    /// 7. Attachment (database, event_bus)
    /// 8. Theme (database, event_bus)
    /// 9. FCM (database, event_bus)
    /// 10. Handle (database, event_bus)
    /// 11. FindMy (event_bus)
    /// 12. FaceTime (event_bus)
    /// 13. Backup (config, event_bus)
    /// 14. Search (database, event_bus)
    /// 15. Cache (event_bus, cache_dir)
    /// 16. ScheduledMessages (database, event_bus)
    /// 17. Sync (config, database, event_bus)
    /// 18. ActionHandler (database, event_bus)
    /// 19. Lifecycle (config, database, event_bus)
    pub fn register_all(&mut self, cache_dir: PathBuf) {
        let bus = self.event_bus.clone();

        // 1. Settings
        self.register(SettingsService::new(self.config.clone()));

        // 2. Queue
        self.register(QueueService::new());

        // 3. Notification
        self.register(NotificationService::new(self.config.clone()));

        // 4. Contact
        self.register(ContactService::new(self.database.clone(), bus.clone()));

        // 5. Chat
        self.register(ChatService::new(self.database.clone(), bus.clone()));

        // 6. Message
        self.register(MessageService::new(self.database.clone(), bus.clone()));

        // 7. Attachment
        self.register(AttachmentService::new(
            self.database.clone(),
            bus.clone(),
            cache_dir.clone(),
        ));

        // 8. Theme
        self.register(ThemeService::new(self.database.clone(), bus.clone()));

        // 9. FCM
        self.register(FcmService::new(self.database.clone(), bus.clone()));

        // 10. Handle
        self.register(HandleService::new(self.database.clone(), bus.clone()));

        // 11. FindMy
        self.register(FindMyService::new(bus.clone()));

        // 12. FaceTime
        self.register(FaceTimeService::new(bus.clone()));

        // 13. Backup
        self.register(BackupService::new(self.config.clone(), bus.clone()));

        // 14. Search
        self.register(SearchService::new(self.database.clone(), bus.clone()));

        // 15. Cache
        self.register(CacheService::new(bus.clone(), cache_dir));

        // 16. ScheduledMessages
        self.register(ScheduledMessageService::new(self.database.clone(), bus.clone()));

        // 17. Sync
        self.register(SyncService::new(
            self.config.clone(),
            self.database.clone(),
            bus.clone(),
        ));

        // 18. ActionHandler
        self.register(ActionHandler::new(self.database.clone(), bus.clone()));

        // 19. Lifecycle
        self.register(LifecycleService::new(
            self.config.clone(),
            self.database.clone(),
            bus,
        ));

        info!("registered {} default services", self.services.len());
    }

    /// Initialize all registered services in order.
    pub async fn init_all(&self) -> BbResult<()> {
        info!("initializing {} services", self.services.len());

        for (name, service) in &self.services {
            info!("initializing service: {name}");
            let mut svc = service.write().await;
            if let Err(e) = svc.init() {
                error!("failed to initialize service {name}: {e}");
                return Err(BbError::ServiceInit(format!("{name}: {e}")));
            }
        }

        info!("all services initialized");
        Ok(())
    }

    /// Shut down all services in reverse order.
    pub async fn shutdown_all(&self) -> BbResult<()> {
        info!("shutting down services");

        for (name, service) in self.services.iter().rev() {
            info!("shutting down service: {name}");
            let mut svc = service.write().await;
            if let Err(e) = svc.shutdown() {
                error!("error shutting down service {name}: {e}");
                // Continue shutting down other services
            }
        }

        info!("all services shut down");
        Ok(())
    }

    /// Set the API client (after server configuration is available).
    pub async fn set_api_client(&self, client: ApiClient) {
        let mut api = self.api_client.write().await;
        *api = Some(client);
        info!("API client configured");
    }

    /// Get a reference to the API client.
    pub async fn api_client(&self) -> BbResult<ApiClient> {
        let api = self.api_client.read().await;
        api.clone()
            .ok_or_else(|| BbError::ServiceNotInitialized("API client not configured".into()))
    }

    /// Get a reference to the event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Get the health status of all services.
    pub async fn health_check(&self) -> Vec<(String, ServiceState, bool)> {
        let mut results = Vec::new();
        for (name, service) in &self.services {
            let svc = service.read().await;
            results.push((name.clone(), svc.state(), svc.is_healthy()));
        }
        results
    }

    /// Get the number of registered services.
    pub fn service_count(&self) -> usize {
        self.services.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_state_display() {
        assert_eq!(ServiceState::Running.to_string(), "running");
        assert_eq!(ServiceState::Failed.to_string(), "failed");
    }

    #[tokio::test]
    async fn test_register_all() {
        let config = ConfigHandle::new(bb_core::config::AppConfig::default());
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let db_config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&db_path, &db_config).unwrap();
        let dispatcher = EventDispatcher::new(64);

        let mut registry = ServiceRegistry::new(config, db, dispatcher);
        registry.register_all(dir.path().join("cache"));

        assert_eq!(registry.service_count(), 19);
    }

    #[tokio::test]
    async fn test_init_and_shutdown() {
        let config = ConfigHandle::new(bb_core::config::AppConfig::default());
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let db_config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&db_path, &db_config).unwrap();
        let dispatcher = EventDispatcher::new(64);

        let mut registry = ServiceRegistry::new(config, db, dispatcher);
        registry.register_all(dir.path().join("cache"));

        registry.init_all().await.unwrap();

        let health = registry.health_check().await;
        for (name, state, healthy) in &health {
            assert!(healthy, "service {name} is not healthy (state: {state})");
        }

        registry.shutdown_all().await.unwrap();
    }
}
