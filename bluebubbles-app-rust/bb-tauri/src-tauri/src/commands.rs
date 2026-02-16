//! Tauri IPC commands exposed to the frontend.
//!
//! Each command bridges a frontend invocation to the Rust backend services.
//! Commands are async and return `Result<T, String>` for Tauri serialization.

use std::collections::HashMap;
use tauri::State;
use tracing::{info, debug};

use bb_core::config::ServerConfig;
use bb_api::ApiClient;
use bb_models::{Chat, Message, Contact, ThemeStruct, Settings};
use bb_models::queries;

use crate::state::AppState;

// ─── Serializable response types for the frontend ───────────────────────────

/// Server info returned after connection.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ServerInfo {
    pub os_version: Option<String>,
    pub server_version: Option<String>,
    pub private_api: bool,
    pub proxy_service: Option<String>,
    pub helper_connected: bool,
    pub detected_imessage: Option<String>,
}

/// Chat with latest message and participant details for the frontend.
#[derive(serde::Serialize, Clone, Debug)]
pub struct ChatWithPreview {
    pub chat: Chat,
    pub latest_message_text: Option<String>,
    pub latest_message_date: Option<String>,
    pub latest_message_is_from_me: bool,
    pub participant_names: Vec<String>,
}

/// Sync result returned from full sync.
#[derive(serde::Serialize, Clone, Debug)]
pub struct SyncResult {
    pub chats_synced: u32,
    pub messages_synced: u32,
    pub handles_synced: u32,
    pub contacts_synced: u32,
}

// ─── Helper: parse ServerInfo from JSON data ────────────────────────────────

fn parse_server_info(data: Option<&serde_json::Value>) -> ServerInfo {
    ServerInfo {
        os_version: data.and_then(|d| d.get("osVersion")).and_then(|v| v.as_str()).map(String::from),
        server_version: data.and_then(|d| d.get("serverVersion")).and_then(|v| v.as_str()).map(String::from),
        private_api: data.and_then(|d| d.get("privateAPI")).and_then(|v| v.as_bool()).unwrap_or(false),
        proxy_service: data.and_then(|d| d.get("proxyService")).and_then(|v| v.as_str()).map(String::from),
        helper_connected: data.and_then(|d| d.get("helperConnected")).and_then(|v| v.as_bool()).unwrap_or(false),
        detected_imessage: data.and_then(|d| d.get("detectediMessage")).and_then(|v| v.as_str()).map(String::from),
    }
}

// ─── Connection commands ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn connect(
    state: State<'_, AppState>,
    address: String,
    password: String,
) -> Result<ServerInfo, String> {
    info!("connecting to server: {address}");

    // Build server config
    let server_config = ServerConfig {
        address: address.clone(),
        guid_auth_key: password.clone(),
        custom_headers: HashMap::new(),
        api_timeout_ms: 30000,
        accept_self_signed_certs: false,
    };

    // Create API client
    let api_client = ApiClient::new(&server_config).map_err(|e| e.to_string())?;

    // Test the connection by fetching server info
    let response = api_client
        .get_json::<serde_json::Value>("/server/info")
        .await
        .map_err(|e| format!("connection failed: {e}"))?;

    // Save the API client to the registry
    state.set_api_client(api_client).await;

    // Update config
    {
        let config = state.config.clone();
        let mut cfg = config.write().await;
        cfg.server.address = address;
        cfg.server.guid_auth_key = password;
    }

    // Parse server info from response
    let info = parse_server_info(response.data.as_ref());

    info!("connected to server v{}", info.server_version.as_deref().unwrap_or("unknown"));
    Ok(info)
}

#[tauri::command]
pub async fn get_server_info(state: State<'_, AppState>) -> Result<ServerInfo, String> {
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let response = api
        .get_json::<serde_json::Value>("/server/info")
        .await
        .map_err(|e| e.to_string())?;

    Ok(parse_server_info(response.data.as_ref()))
}

// ─── Chat commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_chats(
    state: State<'_, AppState>,
    page: u32,
    limit: u32,
) -> Result<Vec<ChatWithPreview>, String> {
    debug!("get_chats page={page} limit={limit}");

    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let offset = (page * limit) as i64;

    let chats = queries::list_chats_with_details(
        &conn,
        offset,
        limit as i64,
        false,
    )
    .map_err(|e| e.to_string())?;

    let previews = chats
        .into_iter()
        .map(|detail| {
            let participant_names: Vec<String> = detail
                .chat
                .participants
                .iter()
                .map(|h| h.display_name())
                .collect();

            ChatWithPreview {
                chat: detail.chat,
                latest_message_text: detail.last_message_text,
                latest_message_date: detail.last_message_date,
                latest_message_is_from_me: detail.last_message_is_from_me,
                participant_names,
            }
        })
        .collect();

    Ok(previews)
}

// ─── Message commands ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    chat_guid: String,
    offset: Option<u32>,
    limit: u32,
) -> Result<Vec<Message>, String> {
    debug!("get_messages chat={chat_guid} offset={offset:?} limit={limit}");

    let conn = state.database.conn().map_err(|e| e.to_string())?;

    // Find the chat by guid to get the local ID
    let chat = Chat::find_by_guid(&conn, &chat_guid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("chat not found: {chat_guid}"))?;

    let chat_id = chat.id.ok_or_else(|| "chat has no local id".to_string())?;
    let off = offset.unwrap_or(0) as i64;

    let messages = queries::list_messages_for_chat(
        &conn,
        chat_id,
        off,
        limit as i64,
        queries::SortDirection::Desc,
    )
    .map_err(|e| e.to_string())?;

    Ok(messages)
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    chat_guid: String,
    text: String,
    effect: Option<String>,
) -> Result<Message, String> {
    info!("send_message to={chat_guid} text_len={}", text.len());

    let api = state.api_client().await.map_err(|e| e.to_string())?;

    let mut body = serde_json::json!({
        "chatGuid": chat_guid,
        "message": text,
        "method": "private-api",
    });

    if let Some(eff) = effect {
        body["effectId"] = serde_json::Value::String(eff);
    }

    let response = api
        .post_json::<serde_json::Value>("/message/text", &body)
        .await
        .map_err(|e| format!("send failed: {e}"))?;

    let data = response
        .data
        .as_ref()
        .ok_or_else(|| "no data in send response".to_string())?;

    let msg = Message::from_server_map(data).map_err(|e| e.to_string())?;
    Ok(msg)
}

#[tauri::command]
pub async fn search_messages(
    state: State<'_, AppState>,
    query: String,
    chat_guid: Option<String>,
) -> Result<Vec<Message>, String> {
    debug!("search_messages query={query} chat={chat_guid:?}");

    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let messages = queries::search_messages(&conn, &query, 50)
        .map_err(|e| e.to_string())?;

    // If a chat_guid filter is provided, filter results
    if let Some(ref guid) = chat_guid {
        if let Some(chat) = Chat::find_by_guid(&conn, guid).map_err(|e| e.to_string())? {
            let chat_id = chat.id;
            let filtered: Vec<Message> = messages
                .into_iter()
                .filter(|m| m.chat_id == chat_id)
                .collect();
            return Ok(filtered);
        }
    }

    Ok(messages)
}

// ─── Contact commands ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<Contact>, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    queries::list_contacts(&conn).map_err(|e| e.to_string())
}

// ─── Attachment commands ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn download_attachment(
    state: State<'_, AppState>,
    guid: String,
) -> Result<String, String> {
    info!("download_attachment guid={guid}");

    let api = state.api_client().await.map_err(|e| e.to_string())?;

    // Request the attachment download URL
    let response = api
        .get_json::<serde_json::Value>(&format!("/attachment/{guid}/download"))
        .await
        .map_err(|e| format!("download failed: {e}"))?;

    // Return the path or URL to the downloaded file
    let path = response
        .data
        .as_ref()
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();

    Ok(path)
}

// ─── Settings commands ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    Settings::get_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    debug!("update_setting key={key}");
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    Settings::set(&conn, &key, &value).map_err(|e| e.to_string())
}

// ─── Sync commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn sync_full(state: State<'_, AppState>) -> Result<SyncResult, String> {
    info!("starting full sync");

    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let conn = state.database.conn().map_err(|e| e.to_string())?;

    // Sync chats
    let chats_response = api
        .get_json::<serde_json::Value>("/chat?with=lastMessage&limit=1000&offset=0&sort=lastmessage")
        .await
        .map_err(|e| format!("chat sync failed: {e}"))?;

    let mut chats_synced = 0u32;
    let mut handles_synced = 0u32;

    if let Some(data) = chats_response.data.as_ref().and_then(|d| d.as_array()) {
        for chat_json in data {
            if let Ok(mut chat) = Chat::from_server_map(chat_json) {
                // Save participants (handles) first
                for handle in &mut chat.participants {
                    if handle.save(&conn).is_ok() {
                        handles_synced += 1;
                    }
                }
                if chat.save(&conn).is_ok() {
                    let _ = chat.save_participants(&conn);
                    chats_synced += 1;
                }
            }
        }
    }

    // Sync contacts
    let contacts_response = api
        .get_json::<serde_json::Value>("/contact")
        .await
        .map_err(|e| format!("contact sync failed: {e}"))?;

    let mut contacts_synced = 0u32;
    if let Some(data) = contacts_response.data.as_ref().and_then(|d| d.as_array()) {
        for contact_json in data {
            if let Ok(mut contact) = Contact::from_server_map(contact_json) {
                if contact.save(&conn).is_ok() {
                    contacts_synced += 1;
                }
            }
        }
    }

    let result = SyncResult {
        chats_synced,
        messages_synced: 0, // Messages are loaded per-chat on demand
        handles_synced,
        contacts_synced,
    };

    info!("sync complete: {chats_synced} chats, {handles_synced} handles, {contacts_synced} contacts");
    Ok(result)
}

// ─── Theme commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_themes(state: State<'_, AppState>) -> Result<Vec<ThemeStruct>, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    ThemeStruct::load_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_theme(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    debug!("set_theme name={name}");
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    Settings::set(&conn, "selected-dark", &name).map_err(|e| e.to_string())?;
    Settings::set(&conn, "selected-light", &name).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Setup commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn check_setup_complete(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let finished = Settings::get_bool(&conn, bb_models::models::settings::keys::FINISHED_SETUP)
        .map_err(|e| e.to_string())?
        .unwrap_or(false);
    Ok(finished)
}

#[tauri::command]
pub async fn complete_setup(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    Settings::set_bool(&conn, bb_models::models::settings::keys::FINISHED_SETUP, true)
        .map_err(|e| e.to_string())?;
    state.mark_setup_complete().await;
    Ok(())
}
