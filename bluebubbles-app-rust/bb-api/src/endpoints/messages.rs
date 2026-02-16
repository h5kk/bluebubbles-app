//! Message endpoints.

use serde::Serialize;
use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

/// Parameters for sending a text message.
#[derive(Debug, Clone, Serialize)]
pub struct SendTextParams {
    #[serde(rename = "chatGuid")]
    pub chat_guid: String,
    #[serde(rename = "tempGuid")]
    pub temp_guid: String,
    pub message: String,
    pub method: String,
    #[serde(rename = "effectId", skip_serializing_if = "Option::is_none")]
    pub effect_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "selectedMessageGuid", skip_serializing_if = "Option::is_none")]
    pub selected_message_guid: Option<String>,
    #[serde(rename = "partIndex", skip_serializing_if = "Option::is_none")]
    pub part_index: Option<i32>,
    #[serde(rename = "ddScan", skip_serializing_if = "Option::is_none")]
    pub dd_scan: Option<bool>,
}

/// Parameters for sending a reaction / tapback.
#[derive(Debug, Clone, Serialize)]
pub struct SendReactionParams {
    #[serde(rename = "chatGuid")]
    pub chat_guid: String,
    #[serde(rename = "selectedMessageText")]
    pub selected_message_text: String,
    #[serde(rename = "selectedMessageGuid")]
    pub selected_message_guid: String,
    pub reaction: String,
    #[serde(rename = "partIndex", skip_serializing_if = "Option::is_none")]
    pub part_index: Option<i32>,
}

/// Parameters for editing a message.
#[derive(Debug, Clone, Serialize)]
pub struct EditMessageParams {
    #[serde(rename = "editedMessage")]
    pub edited_message: String,
    #[serde(rename = "backwardsCompatibilityMessage")]
    pub backwards_compatibility_message: String,
    #[serde(rename = "partIndex")]
    pub part_index: i32,
}

/// Message query parameters.
#[derive(Debug, Clone, Serialize)]
pub struct MessageQuery {
    #[serde(rename = "with", skip_serializing_if = "Vec::is_empty")]
    pub with: Vec<String>,
    #[serde(rename = "where", skip_serializing_if = "Vec::is_empty")]
    pub where_clauses: Vec<WhereClause>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<i64>,
    #[serde(rename = "chatGuid", skip_serializing_if = "Option::is_none")]
    pub chat_guid: Option<String>,
    pub offset: i64,
    pub limit: i64,
    #[serde(rename = "convertAttachments", skip_serializing_if = "Option::is_none")]
    pub convert_attachments: Option<bool>,
}

/// Where clause for message queries.
#[derive(Debug, Clone, Serialize)]
pub struct WhereClause {
    pub statement: String,
    pub args: serde_json::Value,
}

/// Part of a multipart message with mentions.
#[derive(Debug, Clone, Serialize)]
pub struct MessagePart {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<String>,
    #[serde(rename = "partIndex", skip_serializing_if = "Option::is_none")]
    pub part_index: Option<i32>,
}

/// Parameters for sending a multipart message.
#[derive(Debug, Clone, Serialize)]
pub struct SendMultipartParams {
    #[serde(rename = "chatGuid")]
    pub chat_guid: String,
    #[serde(rename = "tempGuid")]
    pub temp_guid: String,
    pub parts: Vec<MessagePart>,
    #[serde(rename = "effectId", skip_serializing_if = "Option::is_none")]
    pub effect_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "selectedMessageGuid", skip_serializing_if = "Option::is_none")]
    pub selected_message_guid: Option<String>,
    #[serde(rename = "partIndex", skip_serializing_if = "Option::is_none")]
    pub part_index: Option<i32>,
    #[serde(rename = "ddScan", skip_serializing_if = "Option::is_none")]
    pub dd_scan: Option<bool>,
}

/// Parameters for creating a scheduled message.
#[derive(Debug, Clone, Serialize)]
pub struct ScheduleMessageParams {
    #[serde(rename = "type")]
    pub schedule_type: String,
    pub payload: serde_json::Value,
    #[serde(rename = "scheduledFor")]
    pub scheduled_for: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<serde_json::Value>,
}

impl ApiClient {
    /// Get total message count (optionally filtered by date range).
    pub async fn message_count(&self, after: Option<i64>, before: Option<i64>) -> BbResult<i64> {
        let mut params = Vec::new();
        if let Some(a) = after {
            params.push(format!("after={a}"));
        }
        if let Some(b) = before {
            params.push(format!("before={b}"));
        }
        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        let resp: ServerResponse<serde_json::Value> =
            self.get_json(&format!("/message/count{query}")).await?;
        Ok(resp
            .data
            .and_then(|d| d.get("total").and_then(|v| v.as_i64()))
            .unwrap_or(0))
    }

    /// Get count of updated messages (optionally filtered by date range).
    pub async fn message_count_updated(
        &self,
        after: Option<i64>,
        before: Option<i64>,
    ) -> BbResult<i64> {
        let mut params = Vec::new();
        if let Some(a) = after {
            params.push(format!("after={a}"));
        }
        if let Some(b) = before {
            params.push(format!("before={b}"));
        }
        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        let resp: ServerResponse<serde_json::Value> =
            self.get_json(&format!("/message/count/updated{query}"))
                .await?;
        Ok(resp
            .data
            .and_then(|d| d.get("total").and_then(|v| v.as_i64()))
            .unwrap_or(0))
    }

    /// Get count of messages sent by the user (optionally filtered by date range).
    pub async fn message_count_me(
        &self,
        after: Option<i64>,
        before: Option<i64>,
    ) -> BbResult<i64> {
        let mut params = Vec::new();
        if let Some(a) = after {
            params.push(format!("after={a}"));
        }
        if let Some(b) = before {
            params.push(format!("before={b}"));
        }
        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        let resp: ServerResponse<serde_json::Value> =
            self.get_json(&format!("/message/count/me{query}")).await?;
        Ok(resp
            .data
            .and_then(|d| d.get("total").and_then(|v| v.as_i64()))
            .unwrap_or(0))
    }

    /// Query messages with filters and pagination.
    pub async fn query_messages(
        &self,
        query: &MessageQuery,
    ) -> BbResult<(Vec<serde_json::Value>, Option<i64>)> {
        let body = serde_json::to_value(query)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.post_json("/message/query", &body).await?;

        let total = resp
            .metadata
            .as_ref()
            .and_then(|m| m.get("total"))
            .and_then(|v| v.as_i64());

        Ok((resp.data.unwrap_or_default(), total))
    }

    /// Get a single message by GUID.
    pub async fn get_message(&self, guid: &str, with: &[&str]) -> BbResult<serde_json::Value> {
        let with_param = if with.is_empty() {
            String::new()
        } else {
            format!("?with={}", with.join(","))
        };
        let resp: ServerResponse =
            self.get_json(&format!("/message/{guid}{with_param}")).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::MessageNotFound(guid.to_string()))
    }

    /// Get embedded media (digital touch / handwritten messages).
    pub async fn get_embedded_media(&self, guid: &str) -> BbResult<Vec<u8>> {
        let resp = self
            .get(&format!("/message/{guid}/embedded-media"))
            .await?;
        ApiClient::response_bytes(resp).await
    }

    /// Send a text message.
    pub async fn send_text(&self, params: &SendTextParams) -> BbResult<serde_json::Value> {
        let body = serde_json::to_value(params)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse = self.post_json("/message/text", &body).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::SendFailed("no data in response".into()))
    }

    /// Send a multipart message (with mentions).
    pub async fn send_multipart(
        &self,
        params: &SendMultipartParams,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::to_value(params)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse = self.post_json("/message/multipart", &body).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::SendFailed("no data in response".into()))
    }

    /// Send a reaction / tapback.
    pub async fn send_reaction(
        &self,
        params: &SendReactionParams,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::to_value(params)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse = self.post_json("/message/react", &body).await?;
        resp.data
            .ok_or_else(|| {
                bb_core::error::BbError::SendFailed("no data in reaction response".into())
            })
    }

    /// Unsend a message.
    pub async fn unsend_message(
        &self,
        guid: &str,
        part_index: i32,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::json!({ "partIndex": part_index });
        let resp: ServerResponse =
            self.post_json(&format!("/message/{guid}/unsend"), &body)
                .await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::MessageNotFound(guid.to_string()))
    }

    /// Edit a sent message.
    pub async fn edit_message(
        &self,
        guid: &str,
        params: &EditMessageParams,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::to_value(params)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse =
            self.post_json(&format!("/message/{guid}/edit"), &body)
                .await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::MessageNotFound(guid.to_string()))
    }

    /// Send a notify-anyway for a message.
    pub async fn notify_message(&self, guid: &str) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self
            .post_json(
                &format!("/message/{guid}/notify"),
                &serde_json::json!({}),
            )
            .await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::MessageNotFound(guid.to_string()))
    }

    /// Get all scheduled messages.
    pub async fn get_scheduled_messages(&self) -> BbResult<Vec<serde_json::Value>> {
        let resp: ServerResponse<Vec<serde_json::Value>> =
            self.get_json("/message/schedule").await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Create a scheduled message.
    pub async fn create_scheduled_message(
        &self,
        params: &ScheduleMessageParams,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::to_value(params)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp: ServerResponse = self.post_json("/message/schedule", &body).await?;
        resp.data.ok_or_else(|| {
            bb_core::error::BbError::Http("failed to create scheduled message".into())
        })
    }

    /// Update a scheduled message.
    pub async fn update_scheduled_message(
        &self,
        id: i64,
        params: &ScheduleMessageParams,
    ) -> BbResult<serde_json::Value> {
        let body = serde_json::to_value(params)
            .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
        let resp = self.put(&format!("/message/schedule/{id}"), &body).await?;
        let resp: ServerResponse = ApiClient::parse_response(resp).await?;
        resp.data.ok_or_else(|| {
            bb_core::error::BbError::Http("failed to update scheduled message".into())
        })
    }

    /// Delete a scheduled message.
    pub async fn delete_scheduled_message(&self, id: i64) -> BbResult<()> {
        self.delete(&format!("/message/schedule/{id}")).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_text_params_serialize() {
        let params = SendTextParams {
            chat_guid: "iMessage;-;+1234".into(),
            temp_guid: "temp-abc".into(),
            message: "Hello".into(),
            method: "private-api".into(),
            effect_id: None,
            subject: None,
            selected_message_guid: None,
            part_index: None,
            dd_scan: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["chatGuid"], "iMessage;-;+1234");
        assert!(json.get("effectId").is_none());
    }

    #[test]
    fn test_multipart_params_serialize() {
        let params = SendMultipartParams {
            chat_guid: "iMessage;-;+1234".into(),
            temp_guid: "temp-abc".into(),
            parts: vec![MessagePart {
                text: "Hello".into(),
                mention: Some("+15551234".into()),
                part_index: Some(0),
            }],
            effect_id: None,
            subject: None,
            selected_message_guid: None,
            part_index: None,
            dd_scan: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["parts"][0]["text"], "Hello");
    }

    #[test]
    fn test_where_clause_serialize() {
        let clause = WhereClause {
            statement: "message.ROWID > :startRowId".into(),
            args: serde_json::json!({"startRowId": 12345}),
        };
        let json = serde_json::to_value(&clause).unwrap();
        assert_eq!(json["statement"], "message.ROWID > :startRowId");
    }
}
