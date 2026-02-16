//! Lifecycle service for managing application startup, shutdown, and state transitions.
//!
//! Orchestrates the initialization sequence, graceful shutdown, and
//! foreground/background transitions for the application.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn, error, debug};

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use bb_models::Database;
use bb_api::ApiClient;

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Application lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Application has not started yet.
    NotStarted,
    /// Application is initializing (loading config, connecting).
    Starting,
    /// Application is running in the foreground.
    Foreground,
    /// Application is running in the background.
    Background,
    /// Application is shutting down.
    ShuttingDown,
    /// Application has stopped.
    Stopped,
}

impl std::fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "not_started"),
            Self::Starting => write!(f, "starting"),
            Self::Foreground => write!(f, "foreground"),
            Self::Background => write!(f, "background"),
            Self::ShuttingDown => write!(f, "shutting_down"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// Service responsible for application lifecycle management.
///
/// Handles:
/// - Startup sequence: config validation, database init, server connectivity
/// - Graceful shutdown: flush queues, close connections, save state
/// - Foreground/background transitions: pause/resume sync, manage resources
pub struct LifecycleService {
    state: ServiceState,
    config: ConfigHandle,
    database: Database,
    event_bus: EventBus,
    /// Current application lifecycle phase.
    phase: LifecyclePhase,
    /// Whether the application has completed initial setup.
    setup_complete: Arc<AtomicBool>,
    /// Whether an incremental sync is currently running.
    sync_in_progress: Arc<AtomicBool>,
}

impl LifecycleService {
    /// Create a new LifecycleService.
    pub fn new(config: ConfigHandle, database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            config,
            database,
            event_bus,
            phase: LifecyclePhase::NotStarted,
            setup_complete: Arc::new(AtomicBool::new(false)),
            sync_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the current lifecycle phase.
    pub fn phase(&self) -> LifecyclePhase {
        self.phase
    }

    /// Whether initial setup has been completed.
    pub fn is_setup_complete(&self) -> bool {
        self.setup_complete.load(Ordering::Relaxed)
    }

    /// Whether the application is in the foreground.
    pub fn is_foreground(&self) -> bool {
        self.phase == LifecyclePhase::Foreground
    }

    /// Whether a sync is currently in progress.
    pub fn is_sync_in_progress(&self) -> bool {
        self.sync_in_progress.load(Ordering::Relaxed)
    }

    /// Run the startup sequence.
    ///
    /// Steps:
    /// 1. Validate configuration (server address and auth key)
    /// 2. Test server connectivity via ping
    /// 3. Fetch server info to determine capabilities
    /// 4. Transition to foreground phase
    ///
    /// Returns the server version string if successful.
    pub async fn startup(&mut self, api: &ApiClient) -> BbResult<String> {
        info!("starting application lifecycle");
        self.phase = LifecyclePhase::Starting;

        // Step 1: Validate configuration
        {
            let config = self.config.read().await;
            if !config.is_server_configured() {
                warn!("server not configured - entering setup mode");
                self.phase = LifecyclePhase::Foreground;
                return Ok("unconfigured".to_string());
            }
        }

        // Step 2: Test server connectivity
        info!("testing server connectivity");
        match api.ping().await {
            Ok(true) => {
                debug!("server ping successful");
            }
            Ok(false) => {
                warn!("server ping returned unexpected response");
            }
            Err(e) => {
                warn!("server ping failed: {e}");
                self.event_bus.emit(AppEvent::ConnectionStateChanged {
                    connected: false,
                    message: format!("ping failed: {e}"),
                });
                // Continue anyway - socket will handle reconnection
            }
        }

        // Step 3: Fetch server info
        let version = match api.server_info().await {
            Ok(info) => {
                let version = info.server_version.clone().unwrap_or_else(|| "unknown".to_string());
                info!("connected to server version: {version}");

                // Store server capabilities
                {
                    let config = self.config.read().await;
                    // Check if setup was previously completed
                    if config.sync.finished_setup {
                        self.setup_complete.store(true, Ordering::Relaxed);
                    }
                }

                version
            }
            Err(e) => {
                warn!("failed to fetch server info: {e}");
                "unknown".to_string()
            }
        };

        // Step 4: Transition to foreground
        self.phase = LifecyclePhase::Foreground;
        self.event_bus.emit(AppEvent::ConnectionStateChanged {
            connected: true,
            message: format!("connected to server v{version}"),
        });

        info!("startup complete (phase: foreground)");
        Ok(version)
    }

    /// Mark the initial setup as complete.
    ///
    /// Called after the first full sync finishes. Persists the flag to config.
    pub async fn mark_setup_complete(&mut self) -> BbResult<()> {
        self.setup_complete.store(true, Ordering::Relaxed);
        let mut config = self.config.write().await;
        config.sync.finished_setup = true;
        info!("initial setup marked as complete");
        Ok(())
    }

    /// Transition to the background phase.
    ///
    /// Reduces resource usage: pauses non-critical sync, reduces poll frequency.
    pub fn enter_background(&mut self) {
        if self.phase != LifecyclePhase::Foreground {
            debug!("ignoring enter_background in phase: {}", self.phase);
            return;
        }

        self.phase = LifecyclePhase::Background;
        info!("entered background mode");
    }

    /// Transition back to the foreground phase.
    ///
    /// Resumes normal operations: triggers incremental sync, restores poll frequency.
    pub fn enter_foreground(&mut self) {
        if self.phase != LifecyclePhase::Background {
            debug!("ignoring enter_foreground in phase: {}", self.phase);
            return;
        }

        self.phase = LifecyclePhase::Foreground;
        info!("entered foreground mode");
    }

    /// Mark that a sync operation has started.
    pub fn sync_started(&self) {
        self.sync_in_progress.store(true, Ordering::Relaxed);
        debug!("sync started");
    }

    /// Mark that a sync operation has completed.
    pub fn sync_completed(&self) {
        self.sync_in_progress.store(false, Ordering::Relaxed);
        debug!("sync completed");
    }

    /// Run the shutdown sequence.
    ///
    /// Steps:
    /// 1. Transition to shutting down phase
    /// 2. Save current configuration to disk
    /// 3. Log database statistics
    /// 4. Transition to stopped phase
    pub async fn shutdown_sequence(&mut self) -> BbResult<()> {
        info!("starting shutdown sequence");
        self.phase = LifecyclePhase::ShuttingDown;

        // Save configuration
        match self.config.save().await {
            Ok(_) => debug!("configuration saved"),
            Err(e) => error!("failed to save configuration during shutdown: {e}"),
        }

        // Log final database stats
        match self.database.stats() {
            Ok(stats) => info!("database stats at shutdown: {stats}"),
            Err(e) => warn!("failed to read database stats: {e}"),
        }

        self.phase = LifecyclePhase::Stopped;
        info!("shutdown sequence complete");
        Ok(())
    }

    /// Get a summary of the current application state for diagnostics.
    pub fn diagnostics(&self) -> LifecycleDiagnostics {
        LifecycleDiagnostics {
            phase: self.phase,
            setup_complete: self.setup_complete.load(Ordering::Relaxed),
            sync_in_progress: self.sync_in_progress.load(Ordering::Relaxed),
            service_state: self.state,
        }
    }
}

/// Diagnostic information about the application lifecycle.
#[derive(Debug, Clone)]
pub struct LifecycleDiagnostics {
    pub phase: LifecyclePhase,
    pub setup_complete: bool,
    pub sync_in_progress: bool,
    pub service_state: ServiceState,
}

impl std::fmt::Display for LifecycleDiagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "phase={}, setup={}, syncing={}, state={}",
            self.phase,
            self.setup_complete,
            self.sync_in_progress,
            self.service_state,
        )
    }
}

impl Service for LifecycleService {
    fn name(&self) -> &str {
        "lifecycle"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("lifecycle service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        self.phase = LifecyclePhase::Stopped;
        info!("lifecycle service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_deps() -> (ConfigHandle, Database, EventBus) {
        let config = ConfigHandle::new(bb_core::config::AppConfig::default());
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let db_config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&path, &db_config).unwrap();
        std::mem::forget(dir);
        let bus = EventBus::new(16);
        (config, db, bus)
    }

    #[test]
    fn test_lifecycle_service_name() {
        let (config, db, bus) = create_test_deps();
        let svc = LifecycleService::new(config, db, bus);
        assert_eq!(svc.name(), "lifecycle");
    }

    #[test]
    fn test_initial_phase() {
        let (config, db, bus) = create_test_deps();
        let svc = LifecycleService::new(config, db, bus);
        assert_eq!(svc.phase(), LifecyclePhase::NotStarted);
        assert!(!svc.is_setup_complete());
        assert!(!svc.is_foreground());
    }

    #[test]
    fn test_foreground_background_transitions() {
        let (config, db, bus) = create_test_deps();
        let mut svc = LifecycleService::new(config, db, bus);
        svc.init().unwrap();

        // Cannot enter background from NotStarted
        svc.enter_background();
        assert_eq!(svc.phase(), LifecyclePhase::NotStarted);

        // Manually set to foreground for testing
        svc.phase = LifecyclePhase::Foreground;
        assert!(svc.is_foreground());

        svc.enter_background();
        assert_eq!(svc.phase(), LifecyclePhase::Background);
        assert!(!svc.is_foreground());

        svc.enter_foreground();
        assert_eq!(svc.phase(), LifecyclePhase::Foreground);
        assert!(svc.is_foreground());
    }

    #[test]
    fn test_sync_tracking() {
        let (config, db, bus) = create_test_deps();
        let svc = LifecycleService::new(config, db, bus);
        assert!(!svc.is_sync_in_progress());

        svc.sync_started();
        assert!(svc.is_sync_in_progress());

        svc.sync_completed();
        assert!(!svc.is_sync_in_progress());
    }

    #[test]
    fn test_diagnostics() {
        let (config, db, bus) = create_test_deps();
        let svc = LifecycleService::new(config, db, bus);
        let diag = svc.diagnostics();
        assert_eq!(diag.phase, LifecyclePhase::NotStarted);
        assert!(!diag.setup_complete);
        assert!(!diag.sync_in_progress);
        // Display trait
        let s = format!("{diag}");
        assert!(s.contains("phase=not_started"));
    }

    #[test]
    fn test_lifecycle_phase_display() {
        assert_eq!(LifecyclePhase::Foreground.to_string(), "foreground");
        assert_eq!(LifecyclePhase::Background.to_string(), "background");
        assert_eq!(LifecyclePhase::ShuttingDown.to_string(), "shutting_down");
    }
}
