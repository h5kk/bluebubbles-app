//! Contact endpoints.

use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

impl ApiClient {
    /// Get all contacts from the server. Set `include_avatars` to true for avatar data.
    pub async fn get_contacts(&self, include_avatars: bool) -> BbResult<Vec<serde_json::Value>> {
        let extra = if include_avatars {
            "?extraProperties=avatar"
        } else {
            ""
        };
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.get_json(&format!("/contact{extra}")).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Query contacts by addresses (phone numbers or emails).
    pub async fn query_contacts(
        &self,
        addresses: &[String],
    ) -> BbResult<Vec<serde_json::Value>> {
        let body = serde_json::json!({ "addresses": addresses });
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.post_json("/contact/query", &body).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Upload contacts to the server. Uses extended timeout (12x) due to
    /// potentially large payloads.
    pub async fn upload_contacts(&self, contacts: &[serde_json::Value]) -> BbResult<()> {
        let body = serde_json::Value::Array(contacts.to_vec());
        self.post_extended("/contact", &body).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_contact_endpoints_exist() {
        // Compile-time verification
    }
}
