//! Server management endpoints.

use serde::{Deserialize, Serialize};
use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

/// Server info returned by `/server/info`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub os_version: Option<String>,
    pub server_version: Option<String>,
    pub private_api: Option<bool>,
    pub helper_connected: Option<bool>,
    pub proxy_service: Option<String>,
    pub detected_icloud: Option<String>,
    pub local_ipv4s: Option<Vec<String>>,
    pub local_ipv6s: Option<Vec<String>>,
}

/// Server statistics (database totals).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTotals {
    pub handles: Option<i64>,
    pub messages: Option<i64>,
    pub chats: Option<i64>,
    pub attachments: Option<i64>,
}

/// Server media statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTotals {
    pub images: Option<i64>,
    pub videos: Option<i64>,
    pub locations: Option<i64>,
}

impl ApiClient {
    /// Ping the server. Returns true if server responds with "pong".
    pub async fn ping(&self) -> BbResult<bool> {
        let resp: ServerResponse = self.get_json("/ping").await?;
        Ok(resp.is_success())
    }

    /// Get server info (version, capabilities, local IPs).
    pub async fn server_info(&self) -> BbResult<ServerInfo> {
        let resp: ServerResponse<ServerInfo> = self.get_json("/server/info").await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http("missing server info data".into()))
    }

    /// Soft restart the server (restart services).
    pub async fn server_restart_soft(&self) -> BbResult<()> {
        self.get("/server/restart/soft").await?;
        Ok(())
    }

    /// Hard restart the server (full application restart).
    pub async fn server_restart_hard(&self) -> BbResult<()> {
        self.get("/server/restart/hard").await?;
        Ok(())
    }

    /// Check for server updates.
    pub async fn server_check_update(&self) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json("/server/update/check").await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Install a server update.
    pub async fn server_install_update(&self) -> BbResult<()> {
        self.post("/server/update/install", &serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Get server database totals.
    pub async fn server_totals(&self) -> BbResult<ServerTotals> {
        let resp: ServerResponse<ServerTotals> =
            self.get_json("/server/statistics/totals").await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http("missing totals data".into()))
    }

    /// Get server media totals.
    pub async fn server_media_totals(&self) -> BbResult<MediaTotals> {
        let resp: ServerResponse<MediaTotals> =
            self.get_json("/server/statistics/media").await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http("missing media totals data".into()))
    }

    /// Get server media totals by chat.
    pub async fn server_media_totals_by_chat(&self) -> BbResult<serde_json::Value> {
        let resp: ServerResponse =
            self.get_json("/server/statistics/media/chat").await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Get server logs.
    pub async fn server_logs(&self, count: u32) -> BbResult<Vec<serde_json::Value>> {
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.get_json(&format!("/server/logs?count={count}")).await?;
        Ok(resp.data.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info_deserialize() {
        let json = serde_json::json!({
            "os_version": "14.0",
            "server_version": "1.9.0",
            "private_api": true,
            "local_ipv4s": ["192.168.1.100"]
        });
        let info: ServerInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.server_version.as_deref(), Some("1.9.0"));
        assert_eq!(info.private_api, Some(true));
    }
}
