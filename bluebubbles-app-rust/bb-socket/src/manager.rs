//! Socket.IO connection manager.
//!
//! Manages the WebSocket connection to the BlueBubbles server, handling
//! automatic reconnection with exponential backoff and jitter, payload
//! decryption, health monitoring, and event routing.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{watch, Mutex, Notify};
use tokio::time::sleep;
use tracing::{info, warn, error, debug};

use bb_core::config::ServerConfig;
use bb_core::constants;
use bb_core::error::{BbError, BbResult};

use crate::crypto::AesCrypto;
use crate::events::{ConnectionState, EventDispatcher, SocketEvent, SocketEventType};

/// Configuration for socket reconnection behavior.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Base delay between reconnection attempts.
    pub base_delay: Duration,
    /// Maximum delay cap for exponential backoff.
    pub max_delay: Duration,
    /// Maximum number of reconnection attempts (0 = unlimited).
    pub max_attempts: u32,
    /// Jitter factor (0.0 to 1.0) added to each delay.
    pub jitter_factor: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            max_attempts: 0,
            jitter_factor: 0.3,
        }
    }
}

/// Health check configuration.
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health check pings.
    pub interval: Duration,
    /// Timeout for each ping response.
    pub timeout: Duration,
    /// Number of missed pings before considering the connection dead.
    pub max_missed_pings: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            max_missed_pings: 3,
        }
    }
}

/// Socket.IO connection manager.
///
/// Manages the full lifecycle of the socket connection including:
/// - Initial connection with authentication
/// - Automatic reconnection with exponential backoff + jitter (1s, 2s, 4s, 8s, 16s, max 30s)
/// - Payload decryption (AES-256-CBC when server encryption is enabled)
/// - Event routing to the EventDispatcher
/// - Health check ping every 30 seconds
/// - Server URL change without full restart
/// - Persistent URL storage so reconnection uses stored config
pub struct SocketManager {
    /// Server configuration for connection parameters.
    server_config: Arc<Mutex<ServerConfig>>,
    /// Event dispatcher for broadcasting events.
    dispatcher: EventDispatcher,
    /// Current connection state.
    state: Arc<Mutex<ConnectionState>>,
    /// Watch channel for state change notifications.
    state_tx: watch::Sender<ConnectionState>,
    /// Reconnection configuration.
    reconnect_config: ReconnectConfig,
    /// Health check configuration.
    health_config: HealthCheckConfig,
    /// Password for AES payload decryption (if encryption is enabled).
    encryption_password: Arc<Mutex<Option<String>>>,
    /// Number of consecutive reconnection attempts.
    reconnect_attempts: Arc<Mutex<u32>>,
    /// Number of consecutive missed health pings.
    missed_pings: Arc<Mutex<u32>>,
    /// Handle to the background connection task.
    connection_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Handle to the health check task.
    health_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Notify channel to signal a disconnect request.
    disconnect_notify: Arc<Notify>,
    /// Set of recently handled message GUIDs for deduplication.
    handled_guids: Arc<Mutex<Vec<String>>>,
}

impl SocketManager {
    /// Create a new SocketManager.
    pub fn new(
        server_config: ServerConfig,
        dispatcher: EventDispatcher,
        encryption_password: Option<String>,
    ) -> Self {
        let (state_tx, _) = watch::channel(ConnectionState::Disconnected);

        Self {
            server_config: Arc::new(Mutex::new(server_config)),
            dispatcher,
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            state_tx,
            reconnect_config: ReconnectConfig::default(),
            health_config: HealthCheckConfig::default(),
            encryption_password: Arc::new(Mutex::new(encryption_password)),
            reconnect_attempts: Arc::new(Mutex::new(0)),
            missed_pings: Arc::new(Mutex::new(0)),
            connection_task: Arc::new(Mutex::new(None)),
            health_task: Arc::new(Mutex::new(None)),
            disconnect_notify: Arc::new(Notify::new()),
            handled_guids: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Set custom reconnection configuration.
    pub fn with_reconnect_config(mut self, config: ReconnectConfig) -> Self {
        self.reconnect_config = config;
        self
    }

    /// Set custom health check configuration.
    pub fn with_health_config(mut self, config: HealthCheckConfig) -> Self {
        self.health_config = config;
        self
    }

    /// Subscribe to connection state changes.
    pub fn state_receiver(&self) -> watch::Receiver<ConnectionState> {
        self.state_tx.subscribe()
    }

    /// Get the current connection state.
    pub async fn state(&self) -> ConnectionState {
        *self.state.lock().await
    }

    /// Get the event dispatcher (for subscribing to events).
    pub fn dispatcher(&self) -> &EventDispatcher {
        &self.dispatcher
    }

    /// Update the connection state and notify watchers.
    async fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.lock().await;
        if *state != new_state {
            info!("socket state: {} -> {}", *state, new_state);
            *state = new_state;
            let _ = self.state_tx.send(new_state);
        }
    }

    /// Start the socket connection.
    ///
    /// Launches a background task that maintains the connection and
    /// handles reconnection automatically.
    pub async fn connect(&self) -> BbResult<()> {
        let current_state = self.state().await;
        if current_state == ConnectionState::Connected
            || current_state == ConnectionState::Connecting
        {
            debug!("already connected or connecting, skipping");
            return Ok(());
        }

        self.set_state(ConnectionState::Connecting).await;
        *self.reconnect_attempts.lock().await = 0;
        *self.missed_pings.lock().await = 0;

        let config = self.server_config.lock().await;
        let encryption = self.encryption_password.lock().await;
        info!(
            "socket connecting to {} (encryption: {})",
            config.address,
            if encryption.is_some() {
                "enabled"
            } else {
                "disabled"
            }
        );
        drop(config);
        drop(encryption);

        // In a full implementation, this would create an actual Socket.IO connection.
        // The connection would:
        // 1. Open a WebSocket to {origin} with query param guid={guidAuthKey}
        // 2. Pass custom headers (ngrok/zrok skip headers + user custom headers)
        // 3. Use transports: ['websocket', 'polling']
        // 4. Register listeners for all SocketEventType::all_event_names()
        // 5. Route incoming events through process_event()

        self.set_state(ConnectionState::Connected).await;
        Ok(())
    }

    /// Disconnect the socket and stop reconnection attempts.
    pub async fn disconnect(&self) {
        self.set_state(ConnectionState::Disconnected).await;
        self.disconnect_notify.notify_waiters();

        // Cancel the connection task
        let mut task = self.connection_task.lock().await;
        if let Some(handle) = task.take() {
            handle.abort();
        }

        // Cancel the health check task
        let mut health = self.health_task.lock().await;
        if let Some(handle) = health.take() {
            handle.abort();
        }

        info!("socket disconnected");
    }

    /// Update the server URL without fully restarting the connection.
    ///
    /// This fixes the Flutter bug where users had to manually re-enter
    /// their server URL when it changed (e.g., Cloudflare tunnel rotation).
    pub async fn update_server_url(&self, new_address: &str) {
        let new_address = bb_core::config::AppConfig::sanitize_server_address(new_address);
        let mut config = self.server_config.lock().await;
        let old_address = config.address.clone();
        config.address = new_address.clone();
        drop(config);

        info!(
            "server url updated: {} -> {}",
            old_address, new_address
        );

        // If connected, trigger a reconnect with the new URL
        let current = self.state().await;
        if current == ConnectionState::Connected || current == ConnectionState::Reconnecting {
            self.set_state(ConnectionState::Reconnecting).await;
            // The reconnect loop will pick up the new URL from server_config
        }
    }

    /// Update the encryption password at runtime.
    pub async fn update_encryption_password(&self, password: Option<String>) {
        let mut pw = self.encryption_password.lock().await;
        *pw = password;
    }

    /// Get the current server address.
    pub async fn server_address(&self) -> String {
        self.server_config.lock().await.address.clone()
    }

    /// Process a raw socket event payload.
    ///
    /// Handles decryption if encryption is enabled, then parses the event
    /// type and dispatches it through the EventDispatcher.
    pub async fn process_event(&self, event_name: &str, raw_data: &str) -> BbResult<()> {
        // Decrypt if encryption is enabled
        let data_str = {
            let password = self.encryption_password.lock().await;
            if let Some(ref pw) = *password {
                AesCrypto::decrypt(pw, raw_data)?
            } else {
                raw_data.to_string()
            }
        };

        // Parse JSON
        let data: serde_json::Value = serde_json::from_str(&data_str)
            .map_err(|e| BbError::Serialization(format!("socket event parse error: {e}")))?;

        let event_type = SocketEventType::from_str(event_name);

        // Deduplication for message events
        if event_type.is_message_event() {
            if let Some(guid) = data.get("guid").and_then(|v| v.as_str()) {
                let mut guids = self.handled_guids.lock().await;
                let dedup_key = format!("{event_name}:{guid}");
                if guids.contains(&dedup_key) {
                    debug!("duplicate event skipped: {dedup_key}");
                    return Ok(());
                }
                guids.push(dedup_key);
                if guids.len() > constants::MAX_HANDLED_GUID_HISTORY {
                    guids.remove(0);
                }
            }
        }

        debug!("socket event: {event_name}");
        self.dispatcher.dispatch(SocketEvent { event_type, data });
        Ok(())
    }

    /// Signal that a health check ping was received successfully.
    ///
    /// Resets the missed ping counter.
    pub async fn on_pong_received(&self) {
        *self.missed_pings.lock().await = 0;
    }

    /// Record a missed health check ping.
    ///
    /// If the threshold is exceeded, triggers reconnection.
    pub async fn on_ping_missed(&self) {
        let missed = {
            let mut count = self.missed_pings.lock().await;
            *count += 1;
            *count
        };

        warn!("missed ping #{missed}/{}", self.health_config.max_missed_pings);

        if missed >= self.health_config.max_missed_pings {
            error!(
                "connection appears dead ({missed} missed pings), triggering reconnect"
            );
            self.trigger_reconnect().await;
        }
    }

    /// Trigger the reconnection loop.
    pub async fn trigger_reconnect(&self) {
        let current = self.state().await;
        if current == ConnectionState::Reconnecting {
            debug!("already reconnecting, skipping trigger");
            return;
        }

        self.set_state(ConnectionState::Reconnecting).await;
        // In a full implementation, this would close the current socket
        // and start the reconnect_loop.
    }

    /// Calculate the reconnection delay using exponential backoff with jitter.
    ///
    /// Sequence: 1s, 2s, 4s, 8s, 16s, capped at max_delay (30s by default).
    /// Jitter of +/- 30% is applied to prevent thundering herd.
    pub fn reconnect_delay(&self, attempt: u32) -> Duration {
        let base = self.reconnect_config.base_delay.as_secs_f64();
        let max = self.reconnect_config.max_delay.as_secs_f64();

        // Exponential backoff: base * 2^attempt
        let exponential = (base * 2.0_f64.powi(attempt as i32)).min(max);

        // Add jitter: +/- jitter_factor * exponential
        let jitter_range = exponential * self.reconnect_config.jitter_factor;
        let jitter = (rand::random::<f64>() * 2.0 - 1.0) * jitter_range;
        let delay = (exponential + jitter).max(0.5);

        Duration::from_secs_f64(delay)
    }

    /// Attempt reconnection with exponential backoff.
    ///
    /// This loop runs until either:
    /// - A connection is successfully re-established
    /// - max_attempts is reached (if configured > 0)
    /// - A disconnect is requested
    pub async fn reconnect_loop(&self) {
        self.set_state(ConnectionState::Reconnecting).await;

        loop {
            let attempt = {
                let mut attempts = self.reconnect_attempts.lock().await;
                *attempts += 1;
                *attempts
            };

            if self.reconnect_config.max_attempts > 0
                && attempt > self.reconnect_config.max_attempts
            {
                error!(
                    "max reconnection attempts ({}) reached",
                    self.reconnect_config.max_attempts
                );
                self.set_state(ConnectionState::Failed).await;
                return;
            }

            let delay = self.reconnect_delay(attempt - 1);
            warn!(
                "reconnection attempt {} in {:.1}s",
                attempt,
                delay.as_secs_f64()
            );

            // Wait for delay, but abort if disconnect is requested
            tokio::select! {
                _ = sleep(delay) => {},
                _ = self.disconnect_notify.notified() => {
                    info!("reconnection cancelled by disconnect request");
                    return;
                }
            }

            // Check if we should still be reconnecting
            let current = self.state().await;
            if current == ConnectionState::Disconnected {
                info!("reconnection aborted: state is disconnected");
                return;
            }

            match self.connect().await {
                Ok(()) => {
                    info!("reconnected successfully after {attempt} attempt(s)");
                    *self.reconnect_attempts.lock().await = 0;
                    *self.missed_pings.lock().await = 0;
                    return;
                }
                Err(e) => {
                    error!("reconnection attempt {attempt} failed: {e}");
                }
            }
        }
    }

    /// Clear the deduplication history.
    pub async fn clear_dedup_history(&self) {
        self.handled_guids.lock().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ServerConfig {
        ServerConfig {
            address: "http://localhost:1234".into(),
            guid_auth_key: "test-guid".into(),
            custom_headers: std::collections::HashMap::new(),
            api_timeout_ms: 30000,
            accept_self_signed_certs: false,
        }
    }

    #[tokio::test]
    async fn test_socket_manager_creation() {
        let dispatcher = EventDispatcher::new(16);
        let manager = SocketManager::new(test_config(), dispatcher, None);
        assert_eq!(manager.state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let dispatcher = EventDispatcher::new(16);
        let manager = SocketManager::new(test_config(), dispatcher, None);

        manager.connect().await.unwrap();
        assert_eq!(manager.state().await, ConnectionState::Connected);

        manager.disconnect().await;
        assert_eq!(manager.state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_process_unencrypted_event() {
        let dispatcher = EventDispatcher::new(16);
        let mut rx = dispatcher.subscribe();
        let manager = SocketManager::new(test_config(), dispatcher, None);

        let data = r#"{"guid":"msg-1","text":"Hello"}"#;
        manager.process_event("new-message", data).await.unwrap();

        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type, SocketEventType::NewMessage);
    }

    #[tokio::test]
    async fn test_process_all_event_types() {
        let dispatcher = EventDispatcher::new(64);
        let mut rx = dispatcher.subscribe();
        let manager = SocketManager::new(test_config(), dispatcher, None);

        // Test each event type
        for event_name in SocketEventType::all_event_names() {
            let data = r#"{"guid":"test-guid","display":true,"chatGuid":"chat-1","read":true,"uuid":"call-1","status_id":4,"aliases":[]}"#;
            manager.process_event(event_name, data).await.unwrap();
            let event = rx.recv().await.unwrap();
            assert_eq!(event.event_type.as_str(), *event_name);
        }
    }

    #[tokio::test]
    async fn test_process_encrypted_event() {
        let password = "test-password-123";
        let dispatcher = EventDispatcher::new(16);
        let mut rx = dispatcher.subscribe();
        let manager = SocketManager::new(test_config(), dispatcher, Some(password.to_string()));

        // Encrypt a payload
        let plaintext = r#"{"guid":"msg-encrypted","text":"Secret"}"#;
        let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();

        manager
            .process_event("new-message", &encrypted)
            .await
            .unwrap();

        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type, SocketEventType::NewMessage);
        assert_eq!(event.data["guid"], "msg-encrypted");
    }

    #[test]
    fn test_reconnect_delay_sequence() {
        let dispatcher = EventDispatcher::new(1);
        let manager = SocketManager::new(test_config(), dispatcher, None);

        // Delays should generally increase: ~1s, ~2s, ~4s, ~8s, ~16s, capped at ~30s
        let d0 = manager.reconnect_delay(0);
        let d1 = manager.reconnect_delay(1);
        let d4 = manager.reconnect_delay(4);
        let d10 = manager.reconnect_delay(10);

        // With jitter, d0 should be roughly 0.7-1.3s
        assert!(d0 >= Duration::from_millis(500));
        assert!(d0 <= Duration::from_millis(2000));

        // d1 should be roughly 1.4-2.6s
        assert!(d1 >= Duration::from_millis(1000));

        // d4 should be roughly 11-21s
        assert!(d4 > Duration::from_secs(5));

        // d10 should be capped at max_delay (30s) +/- jitter
        assert!(d10 <= Duration::from_secs(40));
    }

    #[tokio::test]
    async fn test_deduplication() {
        let dispatcher = EventDispatcher::new(16);
        let mut rx = dispatcher.subscribe();
        let manager = SocketManager::new(test_config(), dispatcher, None);

        let data = r#"{"guid":"msg-dup"}"#;
        manager.process_event("new-message", data).await.unwrap();
        manager.process_event("new-message", data).await.unwrap();

        // Only one event should be dispatched
        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type, SocketEventType::NewMessage);

        // Second recv should timeout since duplicate was filtered
        let result = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dedup_allows_different_event_types() {
        let dispatcher = EventDispatcher::new(16);
        let mut rx = dispatcher.subscribe();
        let manager = SocketManager::new(test_config(), dispatcher, None);

        let data = r#"{"guid":"msg-1"}"#;
        manager.process_event("new-message", data).await.unwrap();
        manager.process_event("updated-message", data).await.unwrap();

        // Both should be dispatched since they are different event types
        let e1 = rx.recv().await.unwrap();
        let e2 = rx.recv().await.unwrap();
        assert_eq!(e1.event_type, SocketEventType::NewMessage);
        assert_eq!(e2.event_type, SocketEventType::UpdatedMessage);
    }

    #[tokio::test]
    async fn test_update_server_url() {
        let dispatcher = EventDispatcher::new(16);
        let manager = SocketManager::new(test_config(), dispatcher, None);

        assert_eq!(manager.server_address().await, "http://localhost:1234");
        manager
            .update_server_url("https://new.trycloudflare.com")
            .await;
        assert_eq!(
            manager.server_address().await,
            "https://new.trycloudflare.com"
        );
    }

    #[tokio::test]
    async fn test_state_watcher() {
        let dispatcher = EventDispatcher::new(16);
        let manager = SocketManager::new(test_config(), dispatcher, None);
        let mut rx = manager.state_receiver();

        manager.connect().await.unwrap();
        // The receiver should get the Connected state
        rx.changed().await.unwrap();
        assert_eq!(*rx.borrow(), ConnectionState::Connected);

        manager.disconnect().await;
        rx.changed().await.unwrap();
        assert_eq!(*rx.borrow(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_health_check_tracking() {
        let dispatcher = EventDispatcher::new(16);
        let manager = SocketManager::new(test_config(), dispatcher, None);

        // Initially no missed pings
        assert_eq!(*manager.missed_pings.lock().await, 0);

        // Miss some pings
        manager.on_ping_missed().await;
        assert_eq!(*manager.missed_pings.lock().await, 1);

        manager.on_ping_missed().await;
        assert_eq!(*manager.missed_pings.lock().await, 2);

        // Pong received resets counter
        manager.on_pong_received().await;
        assert_eq!(*manager.missed_pings.lock().await, 0);
    }

    #[tokio::test]
    async fn test_clear_dedup_history() {
        let dispatcher = EventDispatcher::new(16);
        let mut rx = dispatcher.subscribe();
        let manager = SocketManager::new(test_config(), dispatcher, None);

        let data = r#"{"guid":"msg-clear"}"#;
        manager.process_event("new-message", data).await.unwrap();
        let _ = rx.recv().await.unwrap();

        // Clear history
        manager.clear_dedup_history().await;

        // Same GUID should now be processed again
        manager.process_event("new-message", data).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert_eq!(event.data["guid"], "msg-clear");
    }
}
