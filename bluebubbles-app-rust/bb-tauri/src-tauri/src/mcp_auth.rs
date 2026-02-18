//! MCP bearer token management and validation.
//!
//! Generates cryptographically random tokens for authenticating
//! external AI tool connections to the local MCP server.

use std::sync::Arc;
use tokio::sync::RwLock;
use base64::Engine;
use rand::Rng;
use tracing::debug;

/// Manages bearer token authentication for the MCP server.
pub struct McpAuth {
    token: Arc<RwLock<String>>,
}

impl McpAuth {
    /// Create a new McpAuth with a freshly generated random token.
    pub fn new() -> Self {
        let token = generate_token();
        debug!("mcp auth initialized with new token");
        Self {
            token: Arc::new(RwLock::new(token)),
        }
    }

    /// Create a McpAuth restoring an existing token (e.g. from settings).
    pub fn with_token(token: String) -> Self {
        debug!("mcp auth restored from saved token");
        Self {
            token: Arc::new(RwLock::new(token)),
        }
    }

    /// Validate an `Authorization` header value.
    /// Expects format: `Bearer <token>`.
    pub async fn validate(&self, header: &str) -> bool {
        let expected = self.token.read().await;
        if let Some(provided) = header.strip_prefix("Bearer ") {
            provided.trim() == expected.as_str()
        } else {
            false
        }
    }

    /// Return the current token value.
    pub async fn current_token(&self) -> String {
        self.token.read().await.clone()
    }

    /// Generate a new token, replacing the old one.
    /// Returns the new token.
    pub async fn regenerate(&self) -> String {
        let new_token = generate_token();
        let mut guard = self.token.write().await;
        *guard = new_token.clone();
        debug!("mcp token regenerated");
        new_token
    }
}

/// Generate a 32-byte random token encoded as base64url (no padding).
/// Produces a 43-character string.
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length() {
        let token = generate_token();
        assert_eq!(token.len(), 43);
    }

    #[test]
    fn test_generate_token_unique() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn test_validate_correct() {
        let auth = McpAuth::new();
        let token = auth.current_token().await;
        assert!(auth.validate(&format!("Bearer {token}")).await);
    }

    #[tokio::test]
    async fn test_validate_wrong() {
        let auth = McpAuth::new();
        assert!(!auth.validate("Bearer wrong-token").await);
    }

    #[tokio::test]
    async fn test_validate_no_prefix() {
        let auth = McpAuth::new();
        let token = auth.current_token().await;
        assert!(!auth.validate(&token).await);
    }

    #[tokio::test]
    async fn test_regenerate() {
        let auth = McpAuth::new();
        let old = auth.current_token().await;
        let new = auth.regenerate().await;
        assert_ne!(old, new);
        assert!(auth.validate(&format!("Bearer {new}")).await);
        assert!(!auth.validate(&format!("Bearer {old}")).await);
    }

    #[tokio::test]
    async fn test_with_token() {
        let auth = McpAuth::with_token("my-custom-token".into());
        assert!(auth.validate("Bearer my-custom-token").await);
    }
}
