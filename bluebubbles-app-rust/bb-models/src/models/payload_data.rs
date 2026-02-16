//! Payload data parsing for URL previews and iMessage app data.
//!
//! When a message contains a URL or uses an iMessage app, the server may
//! provide payload data with rich metadata such as titles, descriptions,
//! icons, and app-specific fields.

use serde::{Deserialize, Serialize};

/// Parsed payload data from a message containing URL preview or app data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadData {
    /// URL preview metadata.
    pub url_data: Option<UrlPreviewData>,
    /// iMessage app-specific data.
    pub app_data: Option<AppData>,
    /// Raw JSON for fields not covered by structured types.
    pub raw: Option<serde_json::Value>,
}

/// URL preview metadata extracted from a link in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPreviewData {
    /// The original URL.
    pub url: Option<String>,
    /// The URL after redirection (if different).
    pub original_url: Option<String>,
    /// Page title.
    pub title: Option<String>,
    /// Page description / summary.
    pub summary: Option<String>,
    /// Site name (e.g. "YouTube", "Twitter").
    pub site_name: Option<String>,
    /// Icon image URL or data.
    pub icon: Option<String>,
    /// Preview image URL or data.
    pub image: Option<String>,
}

/// iMessage app-specific payload data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    /// The bundle ID of the iMessage app.
    pub bundle_id: Option<String>,
    /// App name.
    pub app_name: Option<String>,
    /// App-specific URL.
    pub url: Option<String>,
    /// Caption text.
    pub caption: Option<String>,
    /// Sub-caption text.
    pub subcaption: Option<String>,
    /// Trailing caption.
    pub trailing_caption: Option<String>,
    /// User info dictionary (app-specific key-value pairs).
    pub user_info: Option<serde_json::Value>,
}

impl PayloadData {
    /// Parse payload data from the server JSON representation.
    pub fn from_server_json(json: &serde_json::Value) -> Option<Self> {
        if json.is_null() {
            return None;
        }

        let url_data = Self::parse_url_data(json);
        let app_data = Self::parse_app_data(json);

        Some(PayloadData {
            url_data,
            app_data,
            raw: Some(json.clone()),
        })
    }

    /// Extract URL preview data from the JSON.
    fn parse_url_data(json: &serde_json::Value) -> Option<UrlPreviewData> {
        // Check if there is URL metadata
        let url = json.get("url").and_then(|v| v.as_str()).map(String::from)
            .or_else(|| json.get("URL").and_then(|v| v.as_str()).map(String::from));

        let title = json.get("title").and_then(|v| v.as_str()).map(String::from)
            .or_else(|| json.get("richLinkMetadata").and_then(|m| m.get("title")).and_then(|v| v.as_str()).map(String::from));

        if url.is_none() && title.is_none() {
            return None;
        }

        Some(UrlPreviewData {
            url,
            original_url: json.get("originalURL").and_then(|v| v.as_str()).map(String::from),
            title,
            summary: json.get("summary").and_then(|v| v.as_str()).map(String::from)
                .or_else(|| json.get("richLinkMetadata").and_then(|m| m.get("summary")).and_then(|v| v.as_str()).map(String::from)),
            site_name: json.get("siteName").and_then(|v| v.as_str()).map(String::from),
            icon: json.get("icon").and_then(|v| v.as_str()).map(String::from),
            image: json.get("image").and_then(|v| v.as_str()).map(String::from)
                .or_else(|| json.get("richLinkMetadata").and_then(|m| m.get("image")).and_then(|v| v.as_str()).map(String::from)),
        })
    }

    /// Extract iMessage app data from the JSON.
    fn parse_app_data(json: &serde_json::Value) -> Option<AppData> {
        let bundle_id = json.get("appBundleId").and_then(|v| v.as_str()).map(String::from)
            .or_else(|| json.get("bundleId").and_then(|v| v.as_str()).map(String::from));

        if bundle_id.is_none() {
            return None;
        }

        Some(AppData {
            bundle_id,
            app_name: json.get("appName").and_then(|v| v.as_str()).map(String::from),
            url: json.get("url").and_then(|v| v.as_str()).map(String::from),
            caption: json.get("caption").and_then(|v| v.as_str()).map(String::from),
            subcaption: json.get("subcaption").and_then(|v| v.as_str()).map(String::from),
            trailing_caption: json.get("trailingCaption").and_then(|v| v.as_str()).map(String::from),
            user_info: json.get("userInfo").cloned(),
        })
    }

    /// Whether this payload contains a URL preview.
    pub fn has_url_preview(&self) -> bool {
        self.url_data.is_some()
    }

    /// Whether this payload contains iMessage app data.
    pub fn has_app_data(&self) -> bool {
        self.app_data.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_preview_parsing() {
        let json = serde_json::json!({
            "url": "https://example.com",
            "title": "Example",
            "summary": "An example page",
            "siteName": "Example.com"
        });
        let payload = PayloadData::from_server_json(&json).unwrap();
        assert!(payload.has_url_preview());
        let url = payload.url_data.unwrap();
        assert_eq!(url.title.as_deref(), Some("Example"));
    }

    #[test]
    fn test_app_data_parsing() {
        let json = serde_json::json!({
            "appBundleId": "com.apple.pay",
            "appName": "Apple Pay",
            "caption": "$5.00"
        });
        let payload = PayloadData::from_server_json(&json).unwrap();
        assert!(payload.has_app_data());
        let app = payload.app_data.unwrap();
        assert_eq!(app.app_name.as_deref(), Some("Apple Pay"));
    }

    #[test]
    fn test_null_payload() {
        let json = serde_json::json!(null);
        assert!(PayloadData::from_server_json(&json).is_none());
    }
}
