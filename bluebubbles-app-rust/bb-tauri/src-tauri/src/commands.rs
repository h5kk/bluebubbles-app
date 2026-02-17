//! Tauri IPC commands exposed to the frontend.
//!
//! Each command bridges a frontend invocation to the Rust backend services.
//! Commands are async and return `Result<T, String>` for Tauri serialization.

use std::collections::HashMap;
use tauri::State;
use tauri::Emitter;
use tracing::{info, debug};

use bb_core::config::ServerConfig;
use bb_api::ApiClient;
use bb_models::{Chat, Message, Contact, ThemeStruct, Settings};
use bb_models::queries;

use crate::state::AppState;
use crate::otp_detector::{detect_otp, OtpDetection};

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
    /// Base API URL for constructing asset URLs (e.g. avatar URLs).
    pub api_root: Option<String>,
    /// Auth key for constructing asset URLs.
    pub auth_key: Option<String>,
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

fn parse_server_info(data: Option<&serde_json::Value>, api_root: Option<String>, auth_key: Option<String>) -> ServerInfo {
    ServerInfo {
        os_version: data.and_then(|d| d.get("osVersion")).and_then(|v| v.as_str()).map(String::from),
        server_version: data.and_then(|d| d.get("serverVersion")).and_then(|v| v.as_str()).map(String::from),
        private_api: data
            .and_then(|d| d.get("private_api").or_else(|| d.get("privateAPI")))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        proxy_service: data.and_then(|d| d.get("proxyService")).and_then(|v| v.as_str()).map(String::from),
        helper_connected: data.and_then(|d| d.get("helperConnected")).and_then(|v| v.as_bool()).unwrap_or(false),
        detected_imessage: data.and_then(|d| d.get("detectediMessage")).and_then(|v| v.as_str()).map(String::from),
        api_root,
        auth_key,
    }
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

    // Capture the api_root before moving the client
    let api_root_str = api_client.api_root().to_string();

    // Save the API client to the registry
    state.set_api_client(api_client).await;

    // Persist credentials to SQLite so we can auto-reconnect on restart
    {
        let conn = state.database.conn().map_err(|e| e.to_string())?;
        Settings::set(&conn, bb_models::models::settings::keys::SERVER_ADDRESS, &address)
            .map_err(|e| e.to_string())?;
        Settings::set(&conn, bb_models::models::settings::keys::GUID_AUTH_KEY, &password)
            .map_err(|e| e.to_string())?;
    }

    // Update in-memory config
    {
        let config = state.config.clone();
        let mut cfg = config.write().await;
        cfg.server.address = address;
        cfg.server.guid_auth_key = password.clone();
    }

    // Parse server info from response
    let info = parse_server_info(response.data.as_ref(), Some(api_root_str), Some(password));

    info!("connected to server v{}", info.server_version.as_deref().unwrap_or("unknown"));
    Ok(info)
}

#[tauri::command]
pub async fn get_server_info(state: State<'_, AppState>) -> Result<ServerInfo, String> {
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let api_root_str = api.api_root().to_string();
    let response = api
        .get_json::<serde_json::Value>("/server/info")
        .await
        .map_err(|e| e.to_string())?;

    let config = state.config.clone();
    let cfg = config.read().await;
    let auth_key = cfg.server.guid_auth_key.clone();

    Ok(parse_server_info(response.data.as_ref(), Some(api_root_str), Some(auth_key)))
}

/// Try to auto-reconnect using saved credentials from the database.
/// Returns ServerInfo if successful, or null if no saved credentials.
#[tauri::command]
pub async fn try_auto_connect(state: State<'_, AppState>) -> Result<Option<ServerInfo>, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;

    let address = Settings::get(&conn, bb_models::models::settings::keys::SERVER_ADDRESS)
        .map_err(|e| e.to_string())?;
    let password = Settings::get(&conn, bb_models::models::settings::keys::GUID_AUTH_KEY)
        .map_err(|e| e.to_string())?;

    let (address, password) = match (address, password) {
        (Some(a), Some(p)) if !a.is_empty() && !p.is_empty() => (a, p),
        _ => {
            debug!("no saved credentials found, skipping auto-connect");
            return Ok(None);
        }
    };

    info!("auto-reconnecting to saved server: {address}");

    let server_config = ServerConfig {
        address: address.clone(),
        guid_auth_key: password.clone(),
        custom_headers: HashMap::new(),
        api_timeout_ms: 30000,
        accept_self_signed_certs: false,
    };

    let api_client = ApiClient::new(&server_config).map_err(|e| e.to_string())?;
    let api_root_str = api_client.api_root().to_string();

    let response = api_client
        .get_json::<serde_json::Value>("/server/info")
        .await
        .map_err(|e| format!("auto-connect failed: {e}"))?;

    state.set_api_client(api_client).await;

    // Update in-memory config
    {
        let config = state.config.clone();
        let mut cfg = config.write().await;
        cfg.server.address = address;
        cfg.server.guid_auth_key = password.clone();
    }

    let info = parse_server_info(response.data.as_ref(), Some(api_root_str), Some(password));
    info!("auto-connected to server v{}", info.server_version.as_deref().unwrap_or("unknown"));
    Ok(Some(info))
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

/// Refresh chats from the server API, save to local DB, and return the updated list.
/// This is used for background polling to pick up new messages and unread states.
#[tauri::command]
pub async fn refresh_chats(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<ChatWithPreview>, String> {
    debug!("refresh_chats limit={limit}");

    let api = state.api_client().await.map_err(|e| e.to_string())?;

    // Query the server for chats with last message and participants.
    // Fetch in pages of 500 to ensure we capture ALL chats, not just the most recent.
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let server_page_size = 500u32;
    let mut server_offset = 0u32;
    let mut total_saved = 0u32;

    loop {
        let chat_query = serde_json::json!({
            "with": ["lastMessage", "participants"],
            "limit": server_page_size,
            "offset": server_offset,
            "sort": "lastmessage"
        });

        let chats_response = api
            .post_json::<serde_json::Value>("/chat/query", &chat_query)
            .await
            .map_err(|e| format!("refresh chats failed: {e}"))?;

        let batch_count = if let Some(data) = chats_response.data.as_ref().and_then(|d| d.as_array()) {
            let count = data.len() as u32;
            for chat_json in data {
                if let Ok(mut chat) = Chat::from_server_map(chat_json) {
                    // Save participants (handles) first so they get valid IDs
                    for handle in &mut chat.participants {
                        let _ = handle.save(&conn);
                    }
                    if chat.save(&conn).is_ok() {
                        let _ = chat.save_participants(&conn);
                        total_saved += 1;
                    }

                    // If the server response includes the latest message, save it too
                    if let Some(last_msg_json) = chat_json.get("lastMessage") {
                        if !last_msg_json.is_null() {
                            if let Ok(mut msg) = Message::from_server_map(last_msg_json) {
                                msg.chat_id = chat.id;
                                let _ = msg.save(&conn);
                            }
                        }
                    }
                }
            }
            count
        } else {
            0
        };

        server_offset += batch_count;

        // Stop when we get fewer results than page size (no more pages)
        if batch_count < server_page_size {
            break;
        }
    }

    debug!("refresh_chats: saved {total_saved} chats from server (fetched up to offset {server_offset})");

    // Now read back from local DB to get consistent ChatWithPreview format
    let chats = queries::list_chats_with_details(&conn, 0, limit as i64, false)
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

/// Mark a chat as read both on the server and in the local DB.
#[tauri::command]
pub async fn mark_chat_read(
    state: State<'_, AppState>,
    chat_guid: String,
) -> Result<(), String> {
    debug!("mark_chat_read chat={chat_guid}");

    // Update local DB first
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE chats SET has_unread_message = 0 WHERE guid = ?1",
        [&chat_guid],
    )
    .map_err(|e| e.to_string())?;

    // Call the server API to mark as read
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let encoded_guid = percent_encode_path(&chat_guid);
    let path = format!("/chat/{}/read", encoded_guid);
    let body = serde_json::json!({});
    // Fire and forget - don't fail the command if the server call fails
    if let Err(e) = api.post_json::<serde_json::Value>(&path, &body).await {
        debug!("mark_chat_read server call failed (non-fatal): {e}");
    }

    Ok(())
}

/// Update a chat's properties on the server (pin, archive, mute).
/// The body is a JSON object with the fields to update.
#[tauri::command]
pub async fn update_chat(
    state: State<'_, AppState>,
    chat_guid: String,
    updates: serde_json::Value,
) -> Result<(), String> {
    debug!("update_chat chat={chat_guid} updates={updates}");

    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let encoded_guid = percent_encode_path(&chat_guid);
    let path = format!("/chat/{}", encoded_guid);

    api.put_json::<serde_json::Value>(&path, &updates)
        .await
        .map_err(|e| format!("update chat failed: {e}"))?;

    // Update local DB to reflect changes
    let conn = state.database.conn().map_err(|e| e.to_string())?;

    if let Some(pinned) = updates.get("pinned").and_then(|v| v.as_bool()) {
        let val = if pinned { "1" } else { "0" };
        conn.execute(
            "UPDATE chats SET is_pinned = ?1 WHERE guid = ?2",
            [val, &chat_guid],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(archived) = updates.get("isArchived").and_then(|v| v.as_bool()) {
        let val = if archived { "1" } else { "0" };
        conn.execute(
            "UPDATE chats SET is_archived = ?1 WHERE guid = ?2",
            [val, &chat_guid],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(mute_type) = updates.get("muteType") {
        if mute_type.is_null() {
            conn.execute(
                "UPDATE chats SET mute_type = NULL WHERE guid = ?1",
                [&chat_guid],
            ).map_err(|e| e.to_string())?;
        } else if let Some(mt) = mute_type.as_str() {
            conn.execute(
                "UPDATE chats SET mute_type = ?1 WHERE guid = ?2",
                [mt, &chat_guid],
            ).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Mark a chat as unread in the local DB and on the server.
#[tauri::command]
pub async fn mark_chat_unread(
    state: State<'_, AppState>,
    chat_guid: String,
) -> Result<(), String> {
    debug!("mark_chat_unread chat={chat_guid}");

    let conn = state.database.conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE chats SET has_unread_message = 1 WHERE guid = ?1",
        [&chat_guid],
    ).map_err(|e| e.to_string())?;

    // Call server API to mark as unread
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let encoded_guid = percent_encode_path(&chat_guid);
    let path = format!("/chat/{}/unread", encoded_guid);
    let body = serde_json::json!({});
    if let Err(e) = api.post_json::<serde_json::Value>(&path, &body).await {
        debug!("mark_chat_unread server call failed (non-fatal): {e}");
    }

    Ok(())
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
    let off = offset.unwrap_or(0) as i64;

    // Try local DB first (messages are synced via sync_messages)
    let mut chat_id_for_save: Option<i64> = None;
    if let Some(chat) = Chat::find_by_guid(&conn, &chat_guid).map_err(|e| e.to_string())? {
        if let Some(chat_id) = chat.id {
            chat_id_for_save = Some(chat_id as i64);
            let local_msgs = queries::list_messages_for_chat(
                &conn, chat_id, off, limit as i64, queries::SortDirection::Desc,
            ).map_err(|e| e.to_string())?;

            // If we have enough local messages, return them directly.
            // If we have very few (< 5) and this is the first page (offset 0),
            // it likely means only the lastMessage was saved during refresh_chats.
            // In that case, fall through to fetch from server and backfill.
            let too_few = local_msgs.len() < 5 && off == 0;
            if !local_msgs.is_empty() && !too_few {
                return Ok(local_msgs);
            }

            if too_few && !local_msgs.is_empty() {
                debug!("chat {} has only {} local messages, fetching from server to backfill", chat_guid, local_msgs.len());
            }
        }
    }

    // Fetch from server API if no local messages or too few local messages
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let encoded_guid = percent_encode_path(&chat_guid);
    let path = format!(
        "/chat/{}/message?offset={}&limit={}&sort=DESC&with%5B%5D=attachment&with%5B%5D=handle",
        encoded_guid, offset.unwrap_or(0), limit
    );

    let response = api
        .get_json::<serde_json::Value>(&path)
        .await
        .map_err(|e| format!("get messages failed: {e}"))?;

    let mut messages = Vec::new();
    if let Some(data) = response.data.as_ref().and_then(|d| d.as_array()) {
        for msg_json in data {
            if let Ok(mut msg) = Message::from_server_map(msg_json) {
                // Save fetched messages to local DB for future use
                if let Some(cid) = chat_id_for_save {
                    msg.chat_id = Some(cid);
                    let _ = msg.save(&conn);
                }
                messages.push(msg);
            }
        }
    }

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

    let mut msg = Message::from_server_map(data).map_err(|e| e.to_string())?;

    // Save the sent message to local DB so it persists across view reloads.
    // Look up the chat's local ID to associate the message correctly.
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    if let Some(chat) = Chat::find_by_guid(&conn, &chat_guid).map_err(|e| e.to_string())? {
        msg.chat_id = chat.id;
    }
    // Best-effort save; don't fail the send if DB write fails
    if let Err(e) = msg.save(&conn) {
        debug!("failed to save sent message to local DB (non-fatal): {e}");
    }

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

/// Get the base64 avatar for a contact matching the given address.
/// Returns null if no avatar is found.
#[tauri::command]
pub async fn get_contact_avatar(
    state: State<'_, AppState>,
    address: String,
) -> Result<Option<String>, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let contacts = queries::list_contacts(&conn).map_err(|e| e.to_string())?;

    for contact in contacts {
        if contact.matches_address(&address) {
            if let Some(ref avatar_data) = contact.avatar {
                if !avatar_data.is_empty() {
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(avatar_data);
                    return Ok(Some(format!("data:image/jpeg;base64,{b64}")));
                }
            }
        }
    }

    Ok(None)
}

/// Get all contact avatars as a map of address -> data URI.
/// Returns avatars for every address (phone/email) that each contact owns.
/// This allows the frontend to load all avatars in a single IPC call.
#[tauri::command]
pub async fn get_all_contact_avatars(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let contacts = queries::list_contacts(&conn).map_err(|e| e.to_string())?;
    let mut avatars: HashMap<String, String> = HashMap::new();

    use base64::Engine;

    for contact in &contacts {
        if let Some(ref avatar_data) = contact.avatar {
            if !avatar_data.is_empty() {
                let b64 = base64::engine::general_purpose::STANDARD.encode(avatar_data);
                let data_uri = format!("data:image/jpeg;base64,{b64}");

                // Index by every phone number the contact has
                for phone in contact.phone_list() {
                    avatars.insert(phone.clone(), data_uri.clone());
                    // Also insert the normalized form for matching
                    let normalized = bb_models::models::contact::normalize_address(&phone);
                    if normalized != phone {
                        avatars.insert(normalized, data_uri.clone());
                    }
                }

                // Index by every email the contact has
                for email in contact.email_list() {
                    avatars.insert(email.to_lowercase(), data_uri.clone());
                    avatars.insert(email.clone(), data_uri.clone());
                }
            }
        }
    }

    debug!("returning {} avatar entries for {} contacts", avatars.len(), contacts.len());
    Ok(avatars)
}

/// Sync contact avatars from the server.
/// Fetches contacts with avatar data and stores them in the local database.
/// Call this after initial connection to ensure avatars are available.
#[tauri::command]
pub async fn sync_contact_avatars(
    state: State<'_, AppState>,
) -> Result<u32, String> {
    info!("syncing contact avatars from server");

    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let conn = state.database.conn().map_err(|e| e.to_string())?;

    // Fetch contacts with avatar data
    let contacts_response = api
        .get_json::<serde_json::Value>("/contact?extraProperties=avatar")
        .await
        .map_err(|e| format!("contact avatar sync failed: {e}"))?;

    let mut avatars_synced = 0u32;
    let mut total_contacts = 0u32;
    let mut contacts_with_avatar_field = 0u32;
    if let Some(data) = contacts_response.data.as_ref().and_then(|d| d.as_array()) {
        total_contacts = data.len() as u32;
        for contact_json in data {
            // Check if the server returned an avatar field at all
            if contact_json.get("avatar").is_some() && !contact_json["avatar"].is_null() {
                contacts_with_avatar_field += 1;
            }
            if let Ok(mut contact) = Contact::from_server_map(contact_json) {
                if contact.has_avatar() {
                    if contact.save(&conn).is_ok() {
                        avatars_synced += 1;
                    }
                } else {
                    // Save even without avatar to get display names
                    let _ = contact.save(&conn);
                }
            }
        }
    }

    // After saving contacts, link them to handles by setting handle.contact_id.
    // This ensures batch_load_participants_with_contacts resolves names via the
    // fast first-pass JOIN rather than relying solely on address matching.
    let linked = queries::link_contacts_to_handles(&conn).unwrap_or(0);
    info!("avatar sync: {total_contacts} contacts from server, {contacts_with_avatar_field} had avatar field, {avatars_synced} saved with avatar data, {linked} handles linked to contacts");
    Ok(avatars_synced)
}

// ─── Attachment commands ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn download_attachment(
    state: State<'_, AppState>,
    guid: String,
) -> Result<String, String> {
    info!("download_attachment guid={guid}");

    let api = state.api_client().await.map_err(|e| e.to_string())?;

    // First get attachment metadata to determine the mime type
    let meta = api
        .get_attachment(&guid)
        .await
        .map_err(|e| format!("attachment metadata failed: {e}"))?;

    let mime_type = meta
        .get("mimeType")
        .and_then(|v| v.as_str())
        .unwrap_or("application/octet-stream")
        .to_string();

    // Download the raw binary data
    let bytes = api
        .download_attachment(&guid, false)
        .await
        .map_err(|e| format!("download failed: {e}"))?;

    // Encode as base64 data URI
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_uri = format!("data:{mime_type};base64,{b64}");

    info!("download_attachment complete: {} bytes", bytes.len());
    Ok(data_uri)
}

// ─── Private API commands ────────────────────────────────────────────────────

/// Check the Private API status from the server.
/// Returns detailed info about private API availability.
#[derive(serde::Serialize, Clone, Debug)]
pub struct PrivateApiStatus {
    pub enabled: bool,
    pub helper_connected: bool,
    pub server_version: Option<String>,
    pub os_version: Option<String>,
}

#[tauri::command]
pub async fn check_private_api_status(
    state: State<'_, AppState>,
) -> Result<PrivateApiStatus, String> {
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let response = api
        .get_json::<serde_json::Value>("/server/info")
        .await
        .map_err(|e| format!("failed to check private api status: {e}"))?;

    let data = response.data.as_ref();
    Ok(PrivateApiStatus {
        enabled: data
            .and_then(|d| d.get("private_api").or_else(|| d.get("privateAPI")))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        helper_connected: data
            .and_then(|d| d.get("helperConnected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        server_version: data
            .and_then(|d| d.get("serverVersion"))
            .and_then(|v| v.as_str())
            .map(String::from),
        os_version: data
            .and_then(|d| d.get("osVersion"))
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}

/// Send a typing indicator for a chat.
/// `status` should be "start" or "stop".
#[tauri::command]
pub async fn send_typing_indicator(
    state: State<'_, AppState>,
    chat_guid: String,
    status: String,
) -> Result<(), String> {
    debug!("send_typing_indicator chat={chat_guid} status={status}");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let encoded_guid = percent_encode_path(&chat_guid);
    api.send_typing_indicator(&encoded_guid, &status)
        .await
        .map_err(|e| format!("typing indicator failed: {e}"))
}

/// Send a reaction (tapback) to a message.
#[tauri::command]
pub async fn send_reaction(
    state: State<'_, AppState>,
    chat_guid: String,
    selected_message_text: String,
    selected_message_guid: String,
    reaction: String,
    part_index: Option<i32>,
) -> Result<serde_json::Value, String> {
    info!("send_reaction chat={chat_guid} reaction={reaction}");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let params = bb_api::endpoints::messages::SendReactionParams {
        chat_guid,
        selected_message_text,
        selected_message_guid,
        reaction,
        part_index,
    };
    api.send_reaction(&params)
        .await
        .map_err(|e| format!("reaction failed: {e}"))
}

/// Edit a sent message.
#[tauri::command]
pub async fn edit_message(
    state: State<'_, AppState>,
    message_guid: String,
    edited_message: String,
    backwards_compatibility_message: String,
    part_index: i32,
) -> Result<serde_json::Value, String> {
    info!("edit_message guid={message_guid}");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let params = bb_api::endpoints::messages::EditMessageParams {
        edited_message,
        backwards_compatibility_message,
        part_index,
    };
    api.edit_message(&message_guid, &params)
        .await
        .map_err(|e| format!("edit failed: {e}"))
}

/// Unsend a sent message.
#[tauri::command]
pub async fn unsend_message(
    state: State<'_, AppState>,
    message_guid: String,
    part_index: i32,
) -> Result<serde_json::Value, String> {
    info!("unsend_message guid={message_guid}");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    api.unsend_message(&message_guid, part_index)
        .await
        .map_err(|e| format!("unsend failed: {e}"))
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

    // Sync chats (POST to /chat/query with JSON body)
    let chat_query = serde_json::json!({
        "with": ["lastMessage", "participants"],
        "limit": 1000,
        "offset": 0,
        "sort": "lastmessage"
    });
    let chats_response = api
        .post_json::<serde_json::Value>("/chat/query", &chat_query)
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

    // Sync contacts (include avatars via extraProperties)
    let contacts_response = api
        .get_json::<serde_json::Value>("/contact?extraProperties=avatar")
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

    // Link contacts to handles so name resolution works via the fast JOIN path
    let linked = queries::link_contacts_to_handles(&conn).unwrap_or(0);

    let result = SyncResult {
        chats_synced,
        messages_synced: 0,
        handles_synced,
        contacts_synced,
    };

    info!("sync complete: {chats_synced} chats, {handles_synced} handles, {contacts_synced} contacts, {linked} handles linked");
    Ok(result)
}

/// Progress event payload emitted during message sync.
#[derive(serde::Serialize, Clone, Debug)]
pub struct SyncProgress {
    pub current: u32,
    pub total: u32,
    pub chat_name: String,
    pub messages_saved: u32,
}

/// Sync messages for all chats from the server, storing them locally.
/// Emits "sync-progress" events so the frontend can show a progress bar.
#[tauri::command]
pub async fn sync_messages(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    messages_per_chat: u32,
) -> Result<SyncResult, String> {
    info!("starting message sync: {messages_per_chat} messages per chat");

    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let conn = state.database.conn().map_err(|e| e.to_string())?;

    // Get all synced chats from local DB (ordered by latest message)
    let chats = queries::list_chats_with_details(&conn, 0, 10000, false)
        .map_err(|e| e.to_string())?;

    let total = chats.len() as u32;
    let mut total_messages = 0u32;

    for (i, detail) in chats.iter().enumerate() {
        let chat_name = detail.chat.display_name.clone()
            .or_else(|| detail.chat.chat_identifier.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Emit progress
        let _ = app.emit("sync-progress", SyncProgress {
            current: i as u32 + 1,
            total,
            chat_name: chat_name.clone(),
            messages_saved: total_messages,
        });

        let guid = &detail.chat.guid;
        let chat_id = detail.chat.id;

        // Fetch messages from server API
        let encoded_guid = percent_encode_path(guid);
        let path = format!(
            "/chat/{}/message?offset=0&limit={}&sort=DESC&with%5B%5D=attachment&with%5B%5D=handle",
            encoded_guid, messages_per_chat
        );

        match api.get_json::<serde_json::Value>(&path).await {
            Ok(response) => {
                if let Some(data) = response.data.as_ref().and_then(|d| d.as_array()) {
                    for msg_json in data {
                        if let Ok(mut msg) = Message::from_server_map(msg_json) {
                            msg.chat_id = chat_id;
                            if msg.save(&conn).is_ok() {
                                total_messages += 1;
                            }
                        }
                    }
                }
                debug!("synced messages for {} ({}/{})", chat_name, i + 1, total);
            }
            Err(e) => {
                debug!("failed to sync messages for {}: {e}", guid);
            }
        }
    }

    // Mark messages as synced
    Settings::set(&conn, "messagesSynced", "true").map_err(|e| e.to_string())?;

    let _ = app.emit("sync-complete", total_messages);

    info!("message sync complete: {total_messages} messages across {total} chats");
    Ok(SyncResult {
        chats_synced: total,
        messages_synced: total_messages,
        handles_synced: 0,
        contacts_synced: 0,
    })
}

/// Check if messages have been synced before.
#[tauri::command]
pub async fn check_messages_synced(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let synced = Settings::get(&conn, "messagesSynced")
        .map_err(|e| e.to_string())?
        .map(|v| v == "true")
        .unwrap_or(false);
    Ok(synced)
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

// ─── FindMy commands ─────────────────────────────────────────────────────────

/// Serializable FindMy device for the frontend.
#[derive(serde::Serialize, Clone, Debug)]
pub struct FindMyDeviceInfo {
    pub id: String,
    pub name: String,
    pub model: String,
    pub device_class: Option<String>,
    pub raw_device_model: Option<String>,
    pub battery_level: Option<f64>,
    pub battery_status: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub location_timestamp: Option<u64>,
    pub location_type: Option<String>,
    pub address: Option<String>,
    pub is_old_location: bool,
    pub is_online: bool,
    pub is_mac: bool,
    pub this_device: bool,
    pub lost_mode_enabled: bool,
}

/// Helper: extract a location from a FindMy device JSON object.
/// Falls back to crowdSourcedLocation if primary location has no coordinates.
fn extract_findmy_location(d: &serde_json::Value) -> Option<&serde_json::Value> {
    let primary = d.get("location").filter(|l| !l.is_null());
    let has_coords = primary
        .and_then(|l| l.get("latitude"))
        .and_then(|v| v.as_f64())
        .is_some();

    if has_coords {
        primary
    } else {
        // Fall back to crowdSourcedLocation if primary location has no coordinates
        d.get("crowdSourcedLocation").filter(|l| !l.is_null())
    }
}

/// Helper: extract formatted address string from the address object.
fn extract_findmy_address(d: &serde_json::Value) -> Option<String> {
    let addr = d.get("address").filter(|a| !a.is_null())?;

    // Try formattedAddressLines first (array of strings)
    if let Some(lines) = addr.get("formattedAddressLines").and_then(|v| v.as_array()) {
        let parts: Vec<&str> = lines.iter().filter_map(|l| l.as_str()).collect();
        if !parts.is_empty() {
            return Some(parts.join(", "));
        }
    }

    // Fall back to mapItemFullAddress
    if let Some(full) = addr.get("mapItemFullAddress").and_then(|v| v.as_str()) {
        return Some(full.to_string());
    }

    // Fall back to locality + country
    let locality = addr.get("locality").and_then(|v| v.as_str());
    let country = addr.get("country").and_then(|v| v.as_str());
    match (locality, country) {
        (Some(loc), Some(ctry)) => Some(format!("{}, {}", loc, ctry)),
        (Some(loc), None) => Some(loc.to_string()),
        (None, Some(ctry)) => Some(ctry.to_string()),
        _ => None,
    }
}

/// Get FindMy devices from the server.
#[tauri::command]
pub async fn get_findmy_devices(
    state: State<'_, AppState>,
) -> Result<Vec<FindMyDeviceInfo>, String> {
    info!("get_findmy_devices");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let devices = api
        .get_findmy_devices()
        .await
        .map_err(|e| format!("findmy failed: {e}"))?;

    tracing::debug!("FindMy devices raw response: {} devices", devices.len());
    tracing::debug!("First device (if any): {:?}", devices.first());

    let result: Vec<FindMyDeviceInfo> = devices
        .iter()
        .map(|d| {
            let location = extract_findmy_location(d);

            // deviceStatus can be a string like "200" or a number like 200
            let is_online = d.get("deviceStatus")
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s == "200" || s == "203"
                    } else if let Some(n) = v.as_u64() {
                        n == 200 || n == 203
                    } else {
                        false
                    }
                })
                .unwrap_or(false);

            // batteryStatus can be a string or a number (for Items)
            let battery_status = d.get("batteryStatus").and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(n) = v.as_u64() {
                    // FindMyItem uses numeric batteryStatus
                    Some(n.to_string())
                } else {
                    None
                }
            });

            FindMyDeviceInfo {
                id: d.get("id")
                    .or_else(|| d.get("identifier"))
                    .or_else(|| d.get("deviceDiscoveryId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: d.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                model: d.get("deviceDisplayName")
                    .or_else(|| d.get("modelDisplayName"))
                    .or_else(|| d.get("deviceModel"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                device_class: d.get("deviceClass")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                raw_device_model: d.get("rawDeviceModel")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                battery_level: d.get("batteryLevel").and_then(|v| v.as_f64()),
                battery_status,
                latitude: location.and_then(|l| l.get("latitude")).and_then(|v| v.as_f64()),
                longitude: location.and_then(|l| l.get("longitude")).and_then(|v| v.as_f64()),
                // timeStamp is a number (epoch milliseconds), not a string
                location_timestamp: location
                    .and_then(|l| l.get("timeStamp"))
                    .and_then(|v| v.as_u64()),
                location_type: location
                    .and_then(|l| l.get("positionType"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                address: extract_findmy_address(d),
                is_old_location: location
                    .and_then(|l| l.get("isOld"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                is_online,
                is_mac: d.get("isMac")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                this_device: d.get("thisDevice")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                lost_mode_enabled: d.get("lostModeEnabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            }
        })
        .collect();

    info!("get_findmy_devices: {} devices", result.len());
    Ok(result)
}

/// Refresh FindMy device locations by triggering an iCloud refresh.
/// This can take 30+ seconds as it contacts Apple's servers.
#[tauri::command]
pub async fn refresh_findmy_devices(
    state: State<'_, AppState>,
) -> Result<Vec<FindMyDeviceInfo>, String> {
    info!("refresh_findmy_devices");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let devices = api
        .refresh_findmy_devices()
        .await
        .map_err(|e| format!("findmy refresh failed: {e}"))?;

    let result: Vec<FindMyDeviceInfo> = devices
        .iter()
        .map(|d| {
            let location = extract_findmy_location(d);

            let is_online = d.get("deviceStatus")
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s == "200" || s == "203"
                    } else if let Some(n) = v.as_u64() {
                        n == 200 || n == 203
                    } else {
                        false
                    }
                })
                .unwrap_or(false);

            let battery_status = d.get("batteryStatus").and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(n) = v.as_u64() {
                    Some(n.to_string())
                } else {
                    None
                }
            });

            FindMyDeviceInfo {
                id: d.get("id")
                    .or_else(|| d.get("identifier"))
                    .or_else(|| d.get("deviceDiscoveryId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: d.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                model: d.get("deviceDisplayName")
                    .or_else(|| d.get("modelDisplayName"))
                    .or_else(|| d.get("deviceModel"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                device_class: d.get("deviceClass")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                raw_device_model: d.get("rawDeviceModel")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                battery_level: d.get("batteryLevel").and_then(|v| v.as_f64()),
                battery_status,
                latitude: location.and_then(|l| l.get("latitude")).and_then(|v| v.as_f64()),
                longitude: location.and_then(|l| l.get("longitude")).and_then(|v| v.as_f64()),
                location_timestamp: location
                    .and_then(|l| l.get("timeStamp"))
                    .and_then(|v| v.as_u64()),
                location_type: location
                    .and_then(|l| l.get("positionType"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                address: extract_findmy_address(d),
                is_old_location: location
                    .and_then(|l| l.get("isOld"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                is_online,
                is_mac: d.get("isMac")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                this_device: d.get("thisDevice")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                lost_mode_enabled: d.get("lostModeEnabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            }
        })
        .collect();

    info!("refresh_findmy_devices: {} devices", result.len());
    Ok(result)
}

// ─── FindMy Friends commands ────────────────────────────────────────────────

/// Serializable FindMy friend for the frontend.
#[derive(serde::Serialize, Clone, Debug)]
pub struct FindMyFriendInfo {
    pub id: String,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub last_updated: Option<u64>,
    pub status: Option<String>,
    pub locating_in_progress: bool,
}

/// Helper: parse a list of friend JSON values into FindMyFriendInfo structs.
fn parse_findmy_friends(friends: &[serde_json::Value]) -> Vec<FindMyFriendInfo> {
    friends
        .iter()
        .map(|f| {
            let location = f.get("location").filter(|l| !l.is_null())
                .or_else(|| f.get("locationInfo").filter(|l| !l.is_null()));

            let first = f.get("firstName").and_then(|v| v.as_str()).unwrap_or("");
            let last = f.get("lastName").and_then(|v| v.as_str()).unwrap_or("");
            let name = if first.is_empty() && last.is_empty() {
                "Unknown".to_string()
            } else {
                format!("{first} {last}").trim().to_string()
            };

            // Try shortAddress first, then longAddress from the address sub-object
            let address = f.get("address").and_then(|a| {
                a.get("shortAddress")
                    .and_then(|v| v.as_str())
                    .or_else(|| a.get("longAddress").and_then(|v| v.as_str()))
                    .map(String::from)
            }).or_else(|| {
                // Some responses have address fields at location level
                location.and_then(|l| {
                    l.get("shortAddress")
                        .and_then(|v| v.as_str())
                        .or_else(|| l.get("longAddress").and_then(|v| v.as_str()))
                        .map(String::from)
                })
            });

            FindMyFriendInfo {
                id: f.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name,
                latitude: location.and_then(|l| l.get("latitude")).and_then(|v| v.as_f64()),
                longitude: location.and_then(|l| l.get("longitude")).and_then(|v| v.as_f64()),
                address,
                last_updated: location
                    .and_then(|l| l.get("timeStamp"))
                    .and_then(|v| v.as_u64()),
                status: f.get("status")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                locating_in_progress: f.get("locatingInProgress")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            }
        })
        .collect()
}

/// Get FindMy friends from the server.
#[tauri::command]
pub async fn get_findmy_friends(
    state: State<'_, AppState>,
) -> Result<Vec<FindMyFriendInfo>, String> {
    info!("get_findmy_friends");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let friends = api
        .get_findmy_friends()
        .await
        .map_err(|e| format!("findmy friends failed: {e}"))?;

    let result = parse_findmy_friends(&friends);
    info!("get_findmy_friends: {} friends", result.len());
    Ok(result)
}

/// Refresh FindMy friend locations by triggering an iCloud refresh.
/// This can take 30+ seconds as it contacts Apple's servers.
#[tauri::command]
pub async fn refresh_findmy_friends(
    state: State<'_, AppState>,
) -> Result<Vec<FindMyFriendInfo>, String> {
    info!("refresh_findmy_friends");
    let api = state.api_client().await.map_err(|e| e.to_string())?;
    let friends = api
        .refresh_findmy_friends()
        .await
        .map_err(|e| format!("findmy friends refresh failed: {e}"))?;

    let result = parse_findmy_friends(&friends);
    info!("refresh_findmy_friends: {} friends", result.len());
    Ok(result)
}

// ─── OTP Detection commands ──────────────────────────────────────────────────

/// Detect OTP in a message by its GUID.
/// Checks settings to see if OTP detection is enabled.
/// Returns the detected OTP if found, or null if disabled/not found.
#[tauri::command]
pub async fn detect_otp_in_message(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    message_guid: String,
) -> Result<Option<OtpDetection>, String> {
    debug!("detect_otp_in_message guid={message_guid}");

    // Check if OTP detection is enabled
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let enabled = Settings::get_bool(&conn, bb_models::models::settings::keys::OTP_DETECTION_ENABLED)
        .map_err(|e| e.to_string())?
        .unwrap_or(true); // Default to enabled

    if !enabled {
        debug!("otp detection is disabled");
        return Ok(None);
    }

    // Find the message
    let message = Message::find_by_guid(&conn, &message_guid)
        .map_err(|e| e.to_string())?;

    let message = match message {
        Some(m) => m,
        None => {
            debug!("message not found: {message_guid}");
            return Ok(None);
        }
    };

    // Get the message text
    let text = match message.text {
        Some(ref t) if !t.is_empty() => t,
        _ => {
            debug!("message has no text");
            return Ok(None);
        }
    };

    // Detect OTP
    let detection = detect_otp(text);

    if let Some(ref otp) = detection {
        info!("otp detected in message {message_guid}: code={}, pattern={:?}", otp.code, otp.pattern);

        // Emit event to frontend
        let _ = app.emit("otp-detected", serde_json::json!({
            "messageGuid": message_guid,
            "code": otp.code,
            "pattern": format!("{:?}", otp.pattern),
            "chatId": message.chat_id,
        }));

        // Check if auto-copy is enabled
        let auto_copy = Settings::get_bool(&conn, bb_models::models::settings::keys::OTP_AUTO_COPY)
            .map_err(|e| e.to_string())?
            .unwrap_or(false);

        if auto_copy {
            debug!("auto-copy enabled for otp: {}", otp.code);
            // Note: Actual clipboard copy would be handled by frontend
            // We just emit an additional event
            let _ = app.emit("otp-auto-copy", serde_json::json!({
                "code": otp.code,
            }));
        }
    }

    Ok(detection)
}

/// Detect OTP in arbitrary text.
/// Does not check settings or emit events - just performs detection.
/// Useful for testing or manual detection.
#[tauri::command]
pub async fn detect_otp_in_text(text: String) -> Result<Option<OtpDetection>, String> {
    debug!("detect_otp_in_text text_len={}", text.len());
    Ok(detect_otp(&text))
}

// ─── Helper function for message processing ─────────────────────────────────

/// Process a newly received message for OTP detection.
/// This is called internally when new messages arrive from the server.
/// Can also be exposed as a command if needed for manual triggering.
pub async fn process_message_for_otp(
    state: &AppState,
    app: &tauri::AppHandle,
    message: &Message,
) -> Result<Option<OtpDetection>, String> {
    // Only process messages that are not from the user
    if message.is_from_me {
        return Ok(None);
    }

    // Check if OTP detection is enabled
    let conn = state.database.conn().map_err(|e| e.to_string())?;
    let enabled = Settings::get_bool(&conn, bb_models::models::settings::keys::OTP_DETECTION_ENABLED)
        .map_err(|e| e.to_string())?
        .unwrap_or(true);

    if !enabled {
        return Ok(None);
    }

    // Get message text
    let text = match &message.text {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(None),
    };

    // Detect OTP
    let detection = detect_otp(text);

    if let Some(ref otp) = detection {
        info!("otp detected in new message: code={}, pattern={:?}", otp.code, otp.pattern);

        // Emit event to frontend
        let _ = app.emit("otp-detected", serde_json::json!({
            "messageGuid": message.guid,
            "code": otp.code,
            "pattern": format!("{:?}", otp.pattern),
            "chatId": message.chat_id,
        }));

        // Check auto-copy setting
        let auto_copy = Settings::get_bool(&conn, bb_models::models::settings::keys::OTP_AUTO_COPY)
            .map_err(|e| e.to_string())?
            .unwrap_or(false);

        if auto_copy {
            debug!("auto-copy enabled, emitting otp-auto-copy event");
            let _ = app.emit("otp-auto-copy", serde_json::json!({
                "code": otp.code,
            }));
        }
    }

    Ok(detection)
}
