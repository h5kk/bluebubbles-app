//! FaceTime endpoints.

use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

impl ApiClient {
    /// Answer a FaceTime call. Returns the join link.
    pub async fn answer_facetime(&self, call_uuid: &str) -> BbResult<Option<String>> {
        let resp: ServerResponse = self
            .post_json(&format!("/facetime/answer/{call_uuid}"), &serde_json::json!({}))
            .await?;
        Ok(resp
            .data
            .and_then(|d| d.get("link").and_then(|v| v.as_str()).map(String::from)))
    }

    /// Leave a FaceTime call.
    pub async fn leave_facetime(&self, call_uuid: &str) -> BbResult<()> {
        self.post(&format!("/facetime/leave/{call_uuid}"), &serde_json::json!({}))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_facetime_endpoints_exist() {
        // Compile-time verification
    }
}
