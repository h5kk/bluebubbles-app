//! Attributed body parsing for rich text message content.
//!
//! iMessage attributed bodies contain styled text with mentions, links,
//! and formatting information. The server sends this as a JSON array of
//! runs, each with text content and attributes.

use serde::{Deserialize, Serialize};

/// A parsed attributed body consisting of a list of text runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributedBody {
    pub runs: Vec<TextRun>,
}

/// A single run of text with optional attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    /// The text content of this run.
    pub text: String,
    /// Optional attributes applied to this run.
    #[serde(default)]
    pub attributes: TextAttributes,
}

/// Attributes that may be applied to a text run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextAttributes {
    /// Mention handle ID (if this run is an @mention).
    #[serde(rename = "mentionName")]
    pub mention_name: Option<String>,
    /// Whether this run is bold.
    #[serde(default)]
    pub bold: bool,
    /// Whether this run is italic.
    #[serde(default)]
    pub italic: bool,
    /// Whether this run is underlined.
    #[serde(default)]
    pub underline: bool,
    /// Whether this run is strikethrough.
    #[serde(default)]
    pub strikethrough: bool,
    /// Link URL if this run is a hyperlink.
    pub link: Option<String>,
    /// Message part index for this run.
    #[serde(rename = "messagePart")]
    pub message_part: Option<i32>,
}

impl AttributedBody {
    /// Parse an attributed body from the server JSON representation.
    ///
    /// The server format is an array where element 0 contains "runs" entries,
    /// each with a "string" and optional "attributes" map.
    pub fn from_server_json(json: &serde_json::Value) -> Option<Self> {
        // Handle direct array format: [{"runs": [{"string": "...", "attributes": {...}}]}]
        let array = json.as_array()?;
        let first = array.first()?;
        let runs_array = first.get("runs")?.as_array()?;

        let runs = runs_array
            .iter()
            .filter_map(|run| {
                let text = run.get("string")?.as_str()?.to_string();
                let attrs = run.get("attributes");

                let attributes = if let Some(a) = attrs {
                    TextAttributes {
                        mention_name: a.get("__kIMMessagePartAttributeName")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        bold: false,
                        italic: false,
                        underline: false,
                        strikethrough: false,
                        link: a.get("__kIMLinkAttributeName")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        message_part: a.get("__kIMMessagePartAttributeName")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32),
                    }
                } else {
                    TextAttributes::default()
                };

                Some(TextRun { text, attributes })
            })
            .collect();

        Some(AttributedBody { runs })
    }

    /// Get the plain text content by joining all runs.
    pub fn plain_text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }

    /// Get all mentions in the body.
    pub fn mentions(&self) -> Vec<&TextRun> {
        self.runs
            .iter()
            .filter(|r| r.attributes.mention_name.is_some())
            .collect()
    }

    /// Get all links in the body.
    pub fn links(&self) -> Vec<&str> {
        self.runs
            .iter()
            .filter_map(|r| r.attributes.link.as_deref())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attributed_body() {
        let json = serde_json::json!([{
            "runs": [
                {"string": "Hello ", "attributes": {}},
                {"string": "@John", "attributes": {"__kIMMessagePartAttributeName": "john-handle"}},
                {"string": "!"}
            ]
        }]);

        let body = AttributedBody::from_server_json(&json).unwrap();
        assert_eq!(body.runs.len(), 3);
        assert_eq!(body.plain_text(), "Hello @John!");
        assert_eq!(body.mentions().len(), 1);
    }

    #[test]
    fn test_empty_attributed_body() {
        let json = serde_json::json!(null);
        assert!(AttributedBody::from_server_json(&json).is_none());
    }

    #[test]
    fn test_links_extraction() {
        let json = serde_json::json!([{
            "runs": [
                {"string": "Check this: ", "attributes": {}},
                {"string": "example.com", "attributes": {"__kIMLinkAttributeName": "https://example.com"}}
            ]
        }]);
        let body = AttributedBody::from_server_json(&json).unwrap();
        assert_eq!(body.links(), vec!["https://example.com"]);
    }
}
