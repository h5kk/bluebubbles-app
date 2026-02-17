//! iCloud service endpoints.

use bb_core::error::BbResult;
use bb_models::FindMyLocationItem;
use crate::client::ApiClient;
use crate::response::ServerResponse;

impl ApiClient {
    /// Get FindMy devices (raw JSON).
    pub async fn get_findmy_devices_raw(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.get_json("/icloud/findmy/devices").await?;

        tracing::debug!("FindMy devices server response - status: {}, message: {}, data is_some: {}",
            resp.status, resp.message, resp.data.is_some());

        if let Some(ref data) = resp.data {
            tracing::debug!("FindMy devices server response - data length: {}", data.len());
            if !data.is_empty() {
                tracing::trace!("FindMy devices sample: {:?}", data.first());
            }
        } else {
            tracing::warn!("FindMy devices server response - data is null (cache may be encrypted on macOS 14.4+)");
        }

        Ok(resp.data.unwrap_or_default())
    }

    /// Get FindMy devices (typed).
    pub async fn get_findmy_devices(&self) -> BbResult<Vec<FindMyLocationItem>> {
        let raw = self.get_findmy_devices_raw().await?;
        let devices: Vec<FindMyLocationItem> = raw
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(devices)
    }

    /// Refresh FindMy device locations (raw JSON). Uses extended timeout (12x).
    pub async fn refresh_findmy_devices_raw(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp = self
            .post_extended("/icloud/findmy/devices/refresh", &serde_json::json!({}))
            .await?;
        let resp: ServerResponse<Vec<serde_json::Value>> =
            ApiClient::parse_response(resp).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Refresh FindMy device locations. Uses extended timeout (12x).
    pub async fn refresh_findmy_devices(&self) -> BbResult<Vec<FindMyLocationItem>> {
        let raw = self.refresh_findmy_devices_raw().await?;
        let devices: Vec<FindMyLocationItem> = raw
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(devices)
    }

    /// Get FindMy friends (raw JSON).
    pub async fn get_findmy_friends_raw(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.get_json("/icloud/findmy/friends").await?;

        tracing::debug!("FindMy friends server response - status: {}, message: {}, data is_some: {}",
            resp.status, resp.message, resp.data.is_some());

        if let Some(ref data) = resp.data {
            tracing::debug!("FindMy friends server response - data length: {}", data.len());
            if !data.is_empty() {
                tracing::trace!("FindMy friends sample: {:?}", data.first());
            }
        } else {
            tracing::debug!("FindMy friends server response - data is null or empty");
        }

        Ok(resp.data.unwrap_or_default())
    }

    /// Get FindMy friends (typed).
    pub async fn get_findmy_friends(&self) -> BbResult<Vec<FindMyLocationItem>> {
        let raw = self.get_findmy_friends_raw().await?;
        let friends: Vec<FindMyLocationItem> = raw
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(friends)
    }

    /// Refresh FindMy friend locations (raw JSON). Uses extended timeout (12x).
    pub async fn refresh_findmy_friends_raw(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp = self
            .post_extended("/icloud/findmy/friends/refresh", &serde_json::json!({}))
            .await?;
        let resp: ServerResponse<Vec<serde_json::Value>> =
            ApiClient::parse_response(resp).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Refresh FindMy friend locations. Uses extended timeout (12x).
    pub async fn refresh_findmy_friends(&self) -> BbResult<Vec<FindMyLocationItem>> {
        let raw = self.refresh_findmy_friends_raw().await?;
        let friends: Vec<FindMyLocationItem> = raw
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(friends)
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
    use super::*;
    use crate::response::ServerResponse;

    #[test]
    fn test_icloud_endpoints_exist() {
        // Compile-time verification
    }

    #[test]
    fn test_parse_findmy_null_data() {
        // Test that null data is handled correctly
        let json = r#"{"status": 200, "message": "Success", "data": null}"#;
        let resp: ServerResponse<Vec<serde_json::Value>> = serde_json::from_str(json).unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Success");
        assert!(resp.data.is_none());

        // Verify unwrap_or_default returns empty Vec
        let devices = resp.data.unwrap_or_default();
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_parse_findmy_empty_array() {
        // Test that empty array is handled correctly
        let json = r#"{"status": 200, "message": "Success", "data": []}"#;
        let resp: ServerResponse<Vec<serde_json::Value>> = serde_json::from_str(json).unwrap();

        assert_eq!(resp.status, 200);
        assert!(resp.data.is_some());

        let devices = resp.data.unwrap_or_default();
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_parse_findmy_with_devices() {
        // Test that actual device data is parsed
        let json = r#"{
            "status": 200,
            "message": "Success",
            "data": [
                {
                    "id": "device123",
                    "name": "iPhone 13",
                    "deviceDisplayName": "iPhone"
                }
            ]
        }"#;
        let resp: ServerResponse<Vec<serde_json::Value>> = serde_json::from_str(json).unwrap();

        assert_eq!(resp.status, 200);
        assert!(resp.data.is_some());

        let devices = resp.data.unwrap_or_default();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0]["id"].as_str().unwrap(), "device123");
    }
}
