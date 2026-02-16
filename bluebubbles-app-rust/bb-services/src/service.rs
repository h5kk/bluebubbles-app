//! Service trait and lifecycle management.
//!
//! All services implement the `Service` trait which provides a standard
//! lifecycle (init, shutdown) and health checking interface.

use bb_core::error::BbResult;

/// Lifecycle state of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Service has been created but not initialized.
    Created,
    /// Service is initializing.
    Initializing,
    /// Service is running and ready.
    Running,
    /// Service is shutting down.
    ShuttingDown,
    /// Service has been stopped.
    Stopped,
    /// Service encountered a fatal error.
    Failed,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Initializing => write!(f, "initializing"),
            Self::Running => write!(f, "running"),
            Self::ShuttingDown => write!(f, "shutting_down"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Trait that all BlueBubbles services must implement.
///
/// Provides a standard lifecycle and health-checking interface.
/// Services are initialized in dependency order by the ServiceRegistry.
// Note: async_trait is not in workspace deps, so we use a sync trait
// with methods returning BoxFutures, or keep it sync for now.
pub trait Service: Send + Sync {
    /// Human-readable name of this service.
    fn name(&self) -> &str;

    /// Current state of this service.
    fn state(&self) -> ServiceState;

    /// Initialize the service. Called once during application startup.
    fn init(&mut self) -> BbResult<()>;

    /// Gracefully shut down the service. Called during application teardown.
    fn shutdown(&mut self) -> BbResult<()>;

    /// Health check. Returns true if the service is operational.
    fn is_healthy(&self) -> bool {
        self.state() == ServiceState::Running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        state: ServiceState,
    }

    impl Service for TestService {
        fn name(&self) -> &str { "test" }
        fn state(&self) -> ServiceState { self.state }
        fn init(&mut self) -> BbResult<()> {
            self.state = ServiceState::Running;
            Ok(())
        }
        fn shutdown(&mut self) -> BbResult<()> {
            self.state = ServiceState::Stopped;
            Ok(())
        }
    }

    #[test]
    fn test_service_lifecycle() {
        let mut svc = TestService { state: ServiceState::Created };
        assert!(!svc.is_healthy());
        svc.init().unwrap();
        assert!(svc.is_healthy());
        svc.shutdown().unwrap();
        assert!(!svc.is_healthy());
    }
}
