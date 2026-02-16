//! iCloud service endpoints.

use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

impl ApiClient {
    /// Get FindMy devices.
    pub async fn get_findmy_devices(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.get_json("/icloud/findmy/devices").await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Refresh FindMy device locations. Uses extended timeout (12x).
    pub async fn refresh_findmy_devices(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp = self
            .post_extended("/icloud/findmy/devices/refresh", &serde_json::json!({}))
            .await?;
        let resp: ServerResponse<Vec<serde_json::Value>> =
            ApiClient::parse_response(resp).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Get FindMy friends.
    pub async fn get_findmy_friends(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.get_json("/icloud/findmy/friends").await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Refresh FindMy friend locations. Uses extended timeout (12x).
    pub async fn refresh_findmy_friends(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp = self
            .post_extended("/icloud/findmy/friends/refresh", &serde_json::json!({}))
            .await?;
        let resp: ServerResponse<Vec<serde_json::Value>> =
            ApiClient::parse_response(resp).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Get iCloud account info.
    pub async fn get_icloud_account(&self) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json("/icloud/account").await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Get iCloud contact card.
    pub async fn get_icloud_contact(&self) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json("/icloud/contact").await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Set iCloud account alias.
    pub async fn set_icloud_alias(&self, alias: &str) -> BbResult<()> {
        let body = serde_json::json!({ "alias": alias });
        self.post("/icloud/account/alias", &body).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_icloud_endpoints_exist() {
        // Compile-time verification
    }
}
