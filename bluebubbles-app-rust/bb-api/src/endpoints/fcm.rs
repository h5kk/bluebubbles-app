//! FCM (Firebase Cloud Messaging) endpoints.

use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

impl ApiClient {
    /// Register an FCM device with the server.
    pub async fn register_fcm_device(&self, name: &str, identifier: &str) -> BbResult<()> {
        let body = serde_json::json!({
            "name": name,
            "identifier": identifier,
        });
        self.post("/fcm/device", &body).await?;
        Ok(())
    }

    /// Get FCM client configuration from the server.
    pub async fn get_fcm_client(&self) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json("/fcm/client").await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fcm_endpoints_exist() {
        // Compile-time verification
    }
}
