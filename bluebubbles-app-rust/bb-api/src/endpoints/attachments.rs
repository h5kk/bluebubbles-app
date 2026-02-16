//! Attachment endpoints.

use serde::Serialize;
use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

/// Parameters for sending an attachment with optional extras.
#[derive(Debug, Clone, Serialize)]
pub struct SendAttachmentParams {
    pub chat_guid: String,
    pub temp_guid: String,
    pub file_name: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_message_guid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_audio_message: Option<bool>,
}

impl ApiClient {
    /// Get total attachment count.
    pub async fn attachment_count(&self) -> BbResult<i64> {
        let resp: ServerResponse<serde_json::Value> =
            self.get_json("/attachment/count").await?;
        Ok(resp
            .data
            .and_then(|d| d.get("total").and_then(|v| v.as_i64()))
            .unwrap_or(0))
    }

    /// Get attachment metadata by GUID.
    pub async fn get_attachment(&self, guid: &str) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json(&format!("/attachment/{guid}")).await?;
        resp.data
            .ok_or_else(|| bb_core::error::BbError::Http(format!("attachment not found: {guid}")))
    }

    /// Download attachment bytes without progress tracking. Uses extended timeout.
    pub async fn download_attachment(&self, guid: &str, original: bool) -> BbResult<Vec<u8>> {
        let original_param = if original { "&original=true" } else { "" };
        let resp = self
            .get_extended(&format!(
                "/attachment/{guid}/download?dummy=1{original_param}"
            ))
            .await?;
        ApiClient::response_bytes(resp).await
    }

    /// Download attachment bytes with progress reporting.
    ///
    /// The progress callback receives (bytes_downloaded, total_bytes).
    /// If the server does not send Content-Length, total_bytes will be 0.
    pub async fn download_attachment_with_progress<F>(
        &self,
        guid: &str,
        original: bool,
        progress: F,
    ) -> BbResult<Vec<u8>>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        let original_param = if original { "&original=true" } else { "" };
        let resp = self
            .get_extended(&format!(
                "/attachment/{guid}/download?dummy=1{original_param}"
            ))
            .await?;
        ApiClient::response_bytes_with_progress(resp, progress).await
    }

    /// Download the live photo video component. Uses extended timeout.
    pub async fn download_live_photo(&self, guid: &str) -> BbResult<Vec<u8>> {
        let resp = self
            .get_extended(&format!("/attachment/{guid}/live"))
            .await?;
        ApiClient::response_bytes(resp).await
    }

    /// Get the blurhash placeholder for an attachment.
    pub async fn get_blurhash(&self, guid: &str) -> BbResult<Vec<u8>> {
        let resp = self
            .get(&format!("/attachment/{guid}/blurhash"))
            .await?;
        ApiClient::response_bytes(resp).await
    }

    /// Upload an attachment and send it in a chat (simple form).
    pub async fn send_attachment(
        &self,
        chat_guid: &str,
        temp_guid: &str,
        file_name: &str,
        file_bytes: Vec<u8>,
        mime_type: &str,
        method: &str,
    ) -> BbResult<serde_json::Value> {
        let file_part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| bb_core::error::BbError::Http(format!("invalid mime type: {e}")))?;

        let form = reqwest::multipart::Form::new()
            .text("chatGuid", chat_guid.to_string())
            .text("tempGuid", temp_guid.to_string())
            .text("name", file_name.to_string())
            .text("method", method.to_string())
            .part("attachment", file_part);

        let resp = self.post_multipart("/message/attachment", form).await?;
        let resp: ServerResponse = ApiClient::parse_response(resp).await?;
        resp.data.ok_or_else(|| {
            bb_core::error::BbError::SendFailed("no data in attachment response".into())
        })
    }

    /// Upload an attachment with full parameters including optional fields.
    pub async fn send_attachment_full(
        &self,
        params: &SendAttachmentParams,
        file_bytes: Vec<u8>,
        mime_type: &str,
    ) -> BbResult<serde_json::Value> {
        let file_part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(params.file_name.clone())
            .mime_str(mime_type)
            .map_err(|e| bb_core::error::BbError::Http(format!("invalid mime type: {e}")))?;

        let mut form = reqwest::multipart::Form::new()
            .text("chatGuid", params.chat_guid.clone())
            .text("tempGuid", params.temp_guid.clone())
            .text("name", params.file_name.clone())
            .text("method", params.method.clone())
            .part("attachment", file_part);

        if let Some(ref eid) = params.effect_id {
            form = form.text("effectId", eid.clone());
        }
        if let Some(ref s) = params.subject {
            form = form.text("subject", s.clone());
        }
        if let Some(ref smg) = params.selected_message_guid {
            form = form.text("selectedMessageGuid", smg.clone());
        }
        if let Some(pi) = params.part_index {
            form = form.text("partIndex", pi.to_string());
        }
        if let Some(is_audio) = params.is_audio_message {
            form = form.text("isAudioMessage", is_audio.to_string());
        }

        let resp = self.post_multipart("/message/attachment", form).await?;
        let resp: ServerResponse = ApiClient::parse_response(resp).await?;
        resp.data.ok_or_else(|| {
            bb_core::error::BbError::SendFailed("no data in attachment response".into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_attachment_params_serialize() {
        let params = SendAttachmentParams {
            chat_guid: "iMessage;-;+1234".into(),
            temp_guid: "temp-abc".into(),
            file_name: "photo.jpg".into(),
            method: "private-api".into(),
            effect_id: None,
            subject: None,
            selected_message_guid: None,
            part_index: None,
            is_audio_message: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["chat_guid"], "iMessage;-;+1234");
    }
}
