//! MCP tool definitions and dispatch.
//!
//! Defines the tool catalog exposed via the MCP protocol and routes
//! `tools/call` requests to the appropriate `ApiClient` methods.

use bb_api::ApiClient;
use bb_api::endpoints::chats::ChatQuery;
use bb_api::endpoints::messages::{SendTextParams, SendReactionParams, MessageQuery, WhereClause};
use serde_json::json;
use tracing::{info, debug};

/// Errors that can occur during tool execution.
#[derive(Debug)]
pub enum McpToolError {
    /// The requested tool does not exist.
    ToolNotFound(String),
    /// Invalid or missing parameters.
    InvalidParams(String),
    /// The underlying API call failed.
    Internal(String),
}

impl std::fmt::Display for McpToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToolNotFound(name) => write!(f, "unknown tool: {name}"),
            Self::InvalidParams(msg) => write!(f, "invalid params: {msg}"),
            Self::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

impl McpToolError {
    /// JSON-RPC error code for this error type.
    #[allow(dead_code)]
    pub fn code(&self) -> i64 {
        match self {
            Self::ToolNotFound(_) => -32601,
            Self::InvalidParams(_) => -32602,
            Self::Internal(_) => -32603,
        }
    }
}

/// Return the full list of tool definitions for `tools/list`.
pub fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
        tool_list_chats(),
        tool_get_messages(),
        tool_send_message(),
        tool_send_reaction(),
        tool_search_messages(),
        tool_get_contacts(),
        tool_download_attachment(),
        tool_get_server_info(),
    ]
}

/// Execute a tool by name with the given arguments.
pub async fn execute_tool(
    name: &str,
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    debug!("executing mcp tool: {name}");
    match name {
        "list_chats" => exec_list_chats(args, api).await,
        "get_messages" => exec_get_messages(args, api).await,
        "send_message" => exec_send_message(args, api).await,
        "send_reaction" => exec_send_reaction(args, api).await,
        "search_messages" => exec_search_messages(args, api).await,
        "get_contacts" => exec_get_contacts(args, api).await,
        "download_attachment" => exec_download_attachment(args, api).await,
        "get_server_info" => exec_get_server_info(api).await,
        _ => Err(McpToolError::ToolNotFound(name.to_string())),
    }
}

// ─── Tool Definitions ────────────────────────────────────────────────────────

fn tool_list_chats() -> serde_json::Value {
    json!({
        "name": "list_chats",
        "description": "List iMessage/SMS conversations. Returns chat GUID, display name, participants, and last message preview.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Max chats to return (default 25, max 100)",
                    "default": 25
                },
                "offset": {
                    "type": "integer",
                    "description": "Pagination offset (default 0)",
                    "default": 0
                }
            }
        }
    })
}

fn tool_get_messages() -> serde_json::Value {
    json!({
        "name": "get_messages",
        "description": "Get messages for a specific iMessage/SMS chat. Returns message text, sender, timestamps, and attachments.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "chat_guid": {
                    "type": "string",
                    "description": "Chat GUID (e.g. 'iMessage;-;+15551234567')"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max messages to return (default 25, max 100)",
                    "default": 25
                },
                "offset": {
                    "type": "integer",
                    "description": "Pagination offset (default 0)",
                    "default": 0
                }
            },
            "required": ["chat_guid"]
        }
    })
}

fn tool_send_message() -> serde_json::Value {
    json!({
        "name": "send_message",
        "description": "Send a text message to an iMessage/SMS chat.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "chat_guid": {
                    "type": "string",
                    "description": "Chat GUID to send the message to"
                },
                "message": {
                    "type": "string",
                    "description": "Message text to send"
                }
            },
            "required": ["chat_guid", "message"]
        }
    })
}

fn tool_send_reaction() -> serde_json::Value {
    json!({
        "name": "send_reaction",
        "description": "Send a tapback reaction to a message. Reaction types: love, like, dislike, laugh, emphasize, question.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "chat_guid": {
                    "type": "string",
                    "description": "Chat GUID containing the message"
                },
                "message_guid": {
                    "type": "string",
                    "description": "GUID of the message to react to"
                },
                "reaction": {
                    "type": "string",
                    "description": "Reaction type: love, like, dislike, laugh, emphasize, question"
                },
                "message_text": {
                    "type": "string",
                    "description": "Text of the message being reacted to"
                }
            },
            "required": ["chat_guid", "message_guid", "reaction", "message_text"]
        }
    })
}

fn tool_search_messages() -> serde_json::Value {
    json!({
        "name": "search_messages",
        "description": "Full-text search across all iMessage/SMS messages.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query text"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results to return (default 25, max 100)",
                    "default": 25
                },
                "chat_guid": {
                    "type": "string",
                    "description": "Optional: limit search to a specific chat"
                }
            },
            "required": ["query"]
        }
    })
}

fn tool_get_contacts() -> serde_json::Value {
    json!({
        "name": "get_contacts",
        "description": "List all contacts from the BlueBubbles server.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "include_avatars": {
                    "type": "boolean",
                    "description": "Include avatar data (default false, can be large)",
                    "default": false
                }
            }
        }
    })
}

fn tool_download_attachment() -> serde_json::Value {
    json!({
        "name": "download_attachment",
        "description": "Download an attachment by its GUID. Returns base64-encoded data.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "attachment_guid": {
                    "type": "string",
                    "description": "GUID of the attachment to download"
                }
            },
            "required": ["attachment_guid"]
        }
    })
}

fn tool_get_server_info() -> serde_json::Value {
    json!({
        "name": "get_server_info",
        "description": "Get BlueBubbles server version, macOS version, and capabilities.",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    })
}

// ─── Tool Execution ──────────────────────────────────────────────────────────

fn text_content(text: &str) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    })
}

#[allow(dead_code)]
fn error_content(msg: &str) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": msg
        }],
        "isError": true
    })
}

async fn exec_list_chats(
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(25).min(100);
    let offset = args.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);

    let query = ChatQuery {
        limit,
        offset,
        ..ChatQuery::default()
    };

    let chats = api.query_chats(&query).await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    let pretty = serde_json::to_string_pretty(&chats)
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    info!("mcp list_chats returned {} chats", chats.len());
    Ok(text_content(&pretty))
}

async fn exec_get_messages(
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let chat_guid = args.get("chat_guid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("chat_guid is required".into()))?;

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(25).min(100);
    let offset = args.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);

    // Percent-encode the chat GUID for the URL path
    let encoded_guid = percent_encode_path(chat_guid);

    let messages = api.get_chat_messages(
        &encoded_guid,
        offset,
        limit,
        "DESC",
        &["attachment", "handle"],
        None,
        None,
    ).await.map_err(|e| McpToolError::Internal(e.to_string()))?;

    let pretty = serde_json::to_string_pretty(&messages)
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    info!("mcp get_messages returned {} messages for {}", messages.len(), chat_guid);
    Ok(text_content(&pretty))
}

async fn exec_send_message(
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let chat_guid = args.get("chat_guid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("chat_guid is required".into()))?;

    let message = args.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("message is required".into()))?;

    let temp_guid = format!("temp-{}", uuid::Uuid::new_v4());

    let params = SendTextParams {
        chat_guid: chat_guid.to_string(),
        temp_guid,
        message: message.to_string(),
        method: "private-api".to_string(),
        effect_id: None,
        subject: None,
        selected_message_guid: None,
        part_index: None,
        dd_scan: None,
    };

    let result = api.send_text(&params).await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    let pretty = serde_json::to_string_pretty(&result)
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    info!("mcp send_message to {}", chat_guid);
    Ok(text_content(&pretty))
}

async fn exec_send_reaction(
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let chat_guid = args.get("chat_guid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("chat_guid is required".into()))?;

    let message_guid = args.get("message_guid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("message_guid is required".into()))?;

    let reaction = args.get("reaction")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("reaction is required".into()))?;

    let message_text = args.get("message_text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("message_text is required".into()))?;

    let params = SendReactionParams {
        chat_guid: chat_guid.to_string(),
        selected_message_text: message_text.to_string(),
        selected_message_guid: message_guid.to_string(),
        reaction: reaction.to_string(),
        part_index: None,
    };

    let result = api.send_reaction(&params).await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    let pretty = serde_json::to_string_pretty(&result)
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    info!("mcp send_reaction {} to {}", reaction, message_guid);
    Ok(text_content(&pretty))
}

async fn exec_search_messages(
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let query_text = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("query is required".into()))?;

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(25).min(100);
    let chat_guid = args.get("chat_guid").and_then(|v| v.as_str()).map(String::from);

    let msg_query = MessageQuery {
        with: vec!["handle".into(), "attachment".into()],
        where_clauses: vec![WhereClause {
            statement: "message.text LIKE :query".into(),
            args: json!({ "query": format!("%{query_text}%") }),
        }],
        sort: Some("DESC".into()),
        before: None,
        after: None,
        chat_guid,
        offset: 0,
        limit,
        convert_attachments: None,
    };

    let (messages, _total) = api.query_messages(&msg_query).await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    let pretty = serde_json::to_string_pretty(&messages)
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    info!("mcp search_messages '{}' returned {} results", query_text, messages.len());
    Ok(text_content(&pretty))
}

async fn exec_get_contacts(
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let include_avatars = args.get("include_avatars")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let contacts = api.get_contacts(include_avatars).await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    let pretty = serde_json::to_string_pretty(&contacts)
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    info!("mcp get_contacts returned {} contacts", contacts.len());
    Ok(text_content(&pretty))
}

async fn exec_download_attachment(
    args: serde_json::Value,
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let guid = args.get("attachment_guid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpToolError::InvalidParams("attachment_guid is required".into()))?;

    // Get attachment metadata for mime type
    let meta = api.get_attachment(guid).await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    let mime_type = meta.get("mimeType")
        .and_then(|v| v.as_str())
        .unwrap_or("application/octet-stream");

    // Download the raw bytes
    let bytes = api.download_attachment(guid, false).await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    info!("mcp download_attachment {} ({} bytes)", guid, bytes.len());
    Ok(text_content(&format!(
        "Attachment downloaded: {} bytes, mime: {}\nBase64 data: {}",
        bytes.len(), mime_type, b64
    )))
}

async fn exec_get_server_info(
    api: &ApiClient,
) -> Result<serde_json::Value, McpToolError> {
    let info = api.server_info().await
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    let pretty = serde_json::to_string_pretty(&info)
        .map_err(|e| McpToolError::Internal(e.to_string()))?;

    Ok(text_content(&pretty))
}

/// Simple percent-encoding for URL path segments.
fn percent_encode_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}
