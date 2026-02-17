//! Chat endpoints.

use serde::Serialize;
use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

/// Query parameters for listing chats.
#[derive(Debug, Clone, Serialize)]
pub struct ChatQuery {
    /// Include options: "participants", "lastmessage", "sms", "archived".
    #[serde(rename = "with")]
    pub with: Vec<String>,
    /// Pagination offset.
    pub offset: i64,
    /// Maximum number of results.
    pub limit: i64,
    /// Sort order: "lastmessage" sorts by latest message.
    pub sort: Option<String>,
}

impl Default for ChatQuery {
    fn default() -> Self {
        Self {
            with: vec!["participants".into(), "lastmessage".into()],
            offset: 0,
            limit: 25,
            sort: Some("lastmessage".into()),
        }
    }
}

/// Parameters for creating a new chat.
#[derive(Debug, Clone, Serialize)]
pub struct CreateChatParams {
    pub addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub service: String,
    pub method: String,
}

impl ApiClient {
    /// Query chats with pagination and includes.
    pub async fn query_chats(&self, query: &ChatQuery) -> BbResult<Vec<serde_json::Value>> {
        let body = serde_json::to_value(query)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.post_json("/chat/query", &body).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Get total chat count.
    pub async fn chat_count(&self) -> BbResult<i64> {
        let resp: ServerResponse<serde_json::Value> = self.get_json("/chat/count").await?;
        Ok(resp
            .data
            .and_then(|d| d.get("total").and_then(|v| v.as_i64()))
            .unwrap_or(0))
    }

    /// Get a single chat by GUID.
    pub async fn get_chat(&self, guid: &str, with: &[&str]) -> BbResult<serde_json::Value> {
        let with_param = if with.is_empty() {
            String::new()
        } else {
            format!("?with={}", with.join(","))
        };
        let resp: ServerResponse = self.get_json(&format!("/chat/{guid}{with_param}")).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::ChatNotFound(guid.to_string()))
    }

    /// Update a chat's display name.
    pub async fn update_chat(
        &self,
        guid: &str,
        display_name: &str,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::json!({ "displayName": display_name });
        let resp = self.put(&format!("/chat/{guid}"), &body).await?;
        let resp: ServerResponse = ApiClient::parse_response(resp).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::ChatNotFound(guid.to_string()))
    }

    /// Delete a chat.
    pub async fn delete_chat(&self, guid: &str) -> BbResult<()> {
        self.delete(&format!("/chat/{guid}")).await?;
        Ok(())
    }

    /// Create a new chat.
    pub async fn create_chat(&self, params: &CreateChatParams) -> BbResult<serde_json::Value> {
        let body = serde_json::to_value(params)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse = self.post_json("/chat/new", &body).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http("failed to create chat".into()))
    }

    /// Mark a chat as read.
    pub async fn mark_chat_read(&self, guid: &str) -> BbResult<()> {
        self.post(&format!("/chat/{guid}/read"), &serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Mark a chat as unread.
    pub async fn mark_chat_unread(&self, guid: &str) -> BbResult<()> {
        self.post(&format!("/chat/{guid}/unread"), &serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Leave a group chat.
    pub async fn leave_chat(&self, guid: &str) -> BbResult<()> {
        self.post(&format!("/chat/{guid}/leave"), &serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Add a participant to a group chat.
    pub async fn add_participant(
        &self,
        chat_guid: &str,
        address: &str,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::json!({ "address": address });
        let resp: ServerResponse =
            self.post_json(&format!("/chat/{chat_guid}/participant/add"), &body)
                .await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http("failed to add participant".into()))
    }

    /// Remove a participant from a group chat.
    pub async fn remove_participant(
        &self,
        chat_guid: &str,
        address: &str,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::json!({ "address": address });
        let resp: ServerResponse = self
            .post_json(&format!("/chat/{chat_guid}/participant/remove"), &body)
            .await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http("failed to remove participant".into()))
    }

    /// Get messages for a chat with pagination and filtering.
    pub async fn get_chat_messages(
        &self,
        guid: &str,
        offset: i64,
        limit: i64,
        sort: &str,
        with: &[&str],
        before: Option<i64>,
        after: Option<i64>,
    ) -> BbResult<Vec<serde_json::Value>> {
        let with_param = if with.is_empty() {
            String::new()
        } else {
            format!("&with={}", with.join(","))
        };
        let before_param = before
            .map(|b| format!("&before={b}"))
            .unwrap_or_default();
        let after_param = after
            .map(|a| format!("&after={a}"))
            .unwrap_or_default();
        let path = format!(
            "/chat/{guid}/message?offset={offset}&limit={limit}&sort={sort}{with_param}{before_param}{after_param}"
        );
        let resp: ServerResponse<Vec<serde_json::Value>> = self.get_json(&path).await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Get the group chat icon as raw bytes.
    pub async fn get_chat_icon(&self, guid: &str) -> BbResult<Vec<u8>> {
        let resp = self.get(&format!("/chat/{guid}/icon")).await?;
        ApiClient::response_bytes(resp).await
    }

    /// Set the group chat icon (multipart upload). Uses extended timeout.
    pub async fn set_chat_icon(
        &self,
        guid: &str,
        icon_bytes: Vec<u8>,
        file_name: &str,
        mime_type: &str,
    ) -> BbResult<()> {
        let file_part = reqwest::multipart::Part::bytes(icon_bytes)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| bb_core::error::BbError::Http(format!("invalid mime type: {e}")))?;

        let form = reqwest::multipart::Form::new().part("icon", file_part);
        self.post_multipart(&format!("/chat/{guid}/icon"), form)
            .await?;
        Ok(())
    }

    /// Delete the group chat icon.
    pub async fn delete_chat_icon(&self, guid: &str) -> BbResult<()> {
        self.delete(&format!("/chat/{guid}/icon")).await?;
        Ok(())
    }

    /// Send a typing indicator for a chat.
    ///
    /// Requires the Private API to be enabled on the server.
    /// `status` should be `"start"` or `"stop"`.
    pub async fn send_typing_indicator(&self, guid: &str, status: &str) -> BbResult<()> {
        let body = serde_json::json!({ "status": status });
        self.post(&format!("/chat/{guid}/typing"), &body).await?;
        Ok(())
    }

    /// Delete a specific message from a chat.
    pub async fn delete_chat_message(
        &self,
        chat_guid: &str,
        message_guid: &str,
    ) -> BbResult<()> {
        self.delete(&format!("/chat/{chat_guid}/{message_guid}"))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_query_default() {
        let q = ChatQuery::default();
        assert_eq!(q.limit, 25);
        assert!(q.with.contains(&"participants".to_string()));
    }

    #[test]
    fn test_chat_query_serialize() {
        let q = ChatQuery::default();
        let json = serde_json::to_value(&q).unwrap();
        assert!(json.get("with").is_some());
    }

    #[test]
    fn test_create_chat_params_serialize() {
        let params = CreateChatParams {
            addresses: vec!["+15551234".into()],
            message: Some("Hello".into()),
            service: "iMessage".into(),
            method: "private-api".into(),
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["method"], "private-api");
    }
}
