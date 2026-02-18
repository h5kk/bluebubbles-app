//! MCP server state management.
//!
//! Houses the `McpServer` runtime state and the `McpState` wrapper
//! that is registered as Tauri managed state.

use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use tokio::sync::{watch, RwLock};
use tokio::task::JoinHandle;

use crate::mcp_auth::McpAuth;

/// Runtime state for a running MCP server instance.
pub struct McpServer {
    pub auth: McpAuth,
    pub port: u16,
    pub shutdown_tx: watch::Sender<bool>,
    pub active_connections: Arc<AtomicU32>,
    #[allow(dead_code)]
    pub server_handle: Option<JoinHandle<()>>,
}

impl McpServer {
    /// Get the number of active SSE connections.
    pub fn connected_clients(&self) -> u32 {
        self.active_connections.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Signal the server to shut down.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

/// Tauri managed state wrapping the optional MCP server.
pub struct McpState {
    pub server: Arc<RwLock<Option<McpServer>>>,
}

impl McpState {
    /// Create a new empty McpState (no server running).
    pub fn new() -> Self {
        Self {
            server: Arc::new(RwLock::new(None)),
        }
    }
}
