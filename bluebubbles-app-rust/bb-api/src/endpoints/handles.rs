//! Handle endpoints.

use serde::Serialize;
use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

/// Query parameters for listing handles.
#[derive(Debug, Clone, Serialize)]
pub struct HandleQuery {
    #[serde(rename = "with", skip_serializing_if = "Vec::is_empty")]
    pub with: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    pub offset: i64,
    pub limit: i64,
}

impl ApiClient {
    /// Get total handle count.
    pub async fn handle_count(&self) -> BbResult<i64> {
        let resp: ServerResponse<serde_json::Value> = self.get_json("/handle/count").await?;
        Ok(resp
            .data
            .and_then(|d| d.get("total").and_then(|v| v.as_i64()))
            .unwrap_or(0))
    }

    /// Query handles with pagination.
    pub async fn query_handles(&self, query: &HandleQuery) -> BbResult<Vec<serde_json::Value>> {
        let body = serde_json::to_value(query)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.post_json("/handle/query", &body).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Get a single handle by address (GUID).
    pub async fn get_handle(&self, address: &str) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json(&format!("/handle/{address}")).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http(format!("handle not found: {address}")))
    }

    /// Get the focus/DND state of a handle.
    pub async fn get_handle_focus(&self, address: &str) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json(&format!("/handle/{address}/focus")).await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Check iMessage availability for an address.
    pub async fn check_imessage_availability(&self, address: &str) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self
            .get_json(&format!("/handle/availability/imessage?address={address}"))
            .await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Check FaceTime availability for an address.
    pub async fn check_facetime_availability(&self, address: &str) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self
            .get_json(&format!("/handle/availability/facetime?address={address}"))
            .await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_query_serialize() {
        let q = HandleQuery {
            with: vec!["chats".into()],
            address: Some("+15551234".into()),
            offset: 0,
            limit: 50,
        };
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(json["limit"], 50);
    }
}
