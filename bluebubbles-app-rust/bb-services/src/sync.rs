//! Sync service for full and incremental data synchronization.
//!
//! Handles the complete sync protocol:
//! - Full sync: fetches all chats, messages, handles, and contacts during initial setup
//! - Incremental sync: fetches only changes since last sync timestamp or ROWID
//! - ROWID-based incremental sync for server versions >= 1.6.0

use tracing::{info, warn, debug};

use bb_core::config::ConfigHandle;
use bb_core::constants;
use bb_core::error::BbResult;
use bb_models::Database;
use bb_api::ApiClient;
use bb_api::endpoints::chats::ChatQuery;
use bb_api::endpoints::messages::MessageQuery;

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Progress callback for sync operations.
pub type SyncProgressCallback = Box<dyn Fn(SyncProgress) + Send + Sync>;

/// Sync operation progress information.
#[derive(Debug, Clone)]
pub struct SyncProgress {
    /// Current phase of the sync.
    pub phase: SyncPhase,
    /// Items processed so far.
    pub current: u64,
    /// Total items expected (if known).
    pub total: Option<u64>,
    /// Human-readable status message.
    pub message: String,
}

/// Phases of a full sync operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPhase {
    /// Fetching server info and configuration.
    ServerInfo,
    /// Fetching FCM configuration.
    FcmConfig,
    /// Fetching chat list.
    Chats,
    /// Fetching messages for each chat.
    Messages,
    /// Fetching handles (contacts from iMessage DB).
    Handles,
    /// Fetching contacts from address book.
    Contacts,
    /// Sync complete.
    Complete,
}

/// Service responsible for data synchronization with the server.
pub struct SyncService {
    state: ServiceState,
    config: ConfigHandle,
    database: Database,
    event_bus: EventBus,
}

impl SyncService {
    /// Create a new SyncService.
    pub fn new(config: ConfigHandle, database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            config,
            database,
            event_bus,
        }
    }

    /// Emit a progress event through both the callback and the event bus.
    fn report_progress(
        &self,
        progress: &Option<SyncProgressCallback>,
        phase: SyncPhase,
        current: u64,
        total: Option<u64>,
        msg: &str,
    ) {
        if let Some(ref cb) = progress {
            cb(SyncProgress {
                phase,
                current,
                total,
                message: msg.to_string(),
            });
        }

        self.event_bus.emit(AppEvent::SyncProgress {
            phase: format!("{phase:?}"),
            current,
            total,
            message: msg.to_string(),
        });
    }

    /// Run a full sync (initial setup).
    ///
    /// Fetches all data from the server in the following order:
    /// 1. Server info
    /// 2. FCM configuration
    /// 3. Chats (paginated)
    /// 4. Messages for each chat (paginated)
    /// 5. Handles
    /// 6. Contacts
    pub async fn full_sync(
        &self,
        api: &ApiClient,
        progress: Option<SyncProgressCallback>,
    ) -> BbResult<SyncResult> {
        info!("starting full sync");
        let mut result = SyncResult::default();

        // Phase 1: Server info
        self.report_progress(&progress, SyncPhase::ServerInfo, 0, None, "fetching server info");
        let server_info = api.server_info().await?;
        let server_version = server_info.server_version.clone().unwrap_or_default();
        info!("server version: {}", server_version);

        // Phase 2: FCM config
        self.report_progress(&progress, SyncPhase::FcmConfig, 0, None, "fetching FCM config");
        let fcm_data = api.get_fcm_client().await?;
        if !fcm_data.is_null() {
            let mut fcm = bb_models::FcmData::from_server_map(&fcm_data)?;
            let conn = self.database.conn()?;
            fcm.save(&conn)?;
            debug!("FCM config saved");
        }

        // Phase 3: Chats (paginated)
        self.report_progress(&progress, SyncPhase::Chats, 0, None, "fetching chats");
        let chat_count = api.chat_count().await?;
        self.report_progress(&progress, SyncPhase::Chats, 0, Some(chat_count as u64), "fetching chats");

        let mut offset = 0i64;
        let page_size = constants::DEFAULT_CHAT_PAGE_SIZE as i64;
        let mut chat_guids: Vec<String> = Vec::new();

        loop {
            let query = ChatQuery {
                with: vec![
                    "participants".into(),
                    "lastmessage".into(),
                ],
                offset,
                limit: page_size,
                sort: Some("lastmessage".into()),
            };

            let chats = api.query_chats(&query).await?;
            if chats.is_empty() {
                break;
            }

            let conn = self.database.conn()?;
            for chat_json in &chats {
                if let Ok(mut chat) = bb_models::Chat::from_server_map(chat_json) {
                    // Skip empty chats if configured
                    let config = self.config.read().await;
                    let skip_empty = config.sync.skip_empty_chats;
                    drop(config);

                    if skip_empty {
                        let has_last_msg = chat_json
                            .get("lastMessage")
                            .map_or(false, |v| !v.is_null());
                        if !has_last_msg {
                            continue;
                        }
                    }

                    if let Err(e) = chat.save(&conn) {
                        warn!("failed to save chat {}: {e}", chat.guid);
                    } else {
                        chat_guids.push(chat.guid.clone());

                        // Save participants (handles)
                        for participant_json in chat_json
                            .get("participants")
                            .and_then(|v| v.as_array())
                            .unwrap_or(&vec![])
                        {
                            if let Ok(mut handle) =
                                bb_models::Handle::from_server_map(participant_json)
                            {
                                let _ = handle.save(&conn);
                            }
                        }
                        let _ = chat.save_participants(&conn);
                    }
                }
            }

            result.chats_synced += chats.len() as u64;
            self.report_progress(
                &progress,
                SyncPhase::Chats,
                result.chats_synced,
                Some(chat_count as u64),
                &format!("synced {}/{chat_count} chats", result.chats_synced),
            );

            if (chats.len() as i64) < page_size {
                break;
            }
            offset += page_size;
        }

        info!("synced {} chats", result.chats_synced);

        // Phase 4: Messages per chat
        self.report_progress(
            &progress,
            SyncPhase::Messages,
            0,
            Some(chat_guids.len() as u64),
            "fetching messages",
        );

        let config = self.config.read().await;
        let messages_per_page = config.sync.messages_per_page as i64;
        drop(config);

        for (i, chat_guid) in chat_guids.iter().enumerate() {
            let query = MessageQuery {
                with: vec![
                    "chats".into(),
                    "attachment".into(),
                    "handle".into(),
                    "attributedBody".into(),
                ],
                where_clauses: vec![],
                sort: Some("DESC".into()),
                before: None,
                after: None,
                chat_guid: Some(chat_guid.clone()),
                offset: 0,
                limit: messages_per_page,
                convert_attachments: None,
            };

            match api.query_messages(&query).await {
                Ok((messages, _total)) => {
                    let conn = self.database.conn()?;
                    for msg_json in &messages {
                        if let Ok(mut msg) = bb_models::Message::from_server_map(msg_json) {
                            // Resolve chat_id
                            if let Ok(Some(chat)) =
                                bb_models::queries::find_chat_by_guid(&conn, chat_guid)
                            {
                                msg.chat_id = chat.id;
                            }
                            let _ = msg.save(&conn);

                            // Save attachments
                            if let Some(attachments) =
                                msg_json.get("attachments").and_then(|v| v.as_array())
                            {
                                for att_json in attachments {
                                    if let Ok(mut att) =
                                        bb_models::Attachment::from_server_map(att_json)
                                    {
                                        att.message_id = msg.id;
                                        let _ = att.save(&conn);
                                    }
                                }
                            }

                            // Save handle
                            if let Some(handle_data) = msg_json.get("handle") {
                                if !handle_data.is_null() {
                                    if let Ok(mut handle) =
                                        bb_models::Handle::from_server_map(handle_data)
                                    {
                                        let _ = handle.save(&conn);
                                    }
                                }
                            }
                        }
                    }
                    result.messages_synced += messages.len() as u64;
                }
                Err(e) => {
                    warn!("failed to sync messages for chat {chat_guid}: {e}");
                }
            }

            self.report_progress(
                &progress,
                SyncPhase::Messages,
                (i + 1) as u64,
                Some(chat_guids.len() as u64),
                &format!("synced messages for {}/{} chats", i + 1, chat_guids.len()),
            );
        }

        info!("synced {} messages across {} chats", result.messages_synced, chat_guids.len());

        // Phase 5: Contacts
        self.report_progress(&progress, SyncPhase::Contacts, 0, None, "fetching contacts");
        let contacts_json = api.get_contacts(false).await?;
        {
            let conn = self.database.conn()?;
            for contact_json in &contacts_json {
                if let Ok(mut contact) = bb_models::Contact::from_server_map(contact_json) {
                    let _ = contact.save(&conn);
                    result.contacts_synced += 1;
                }
            }
        }
        info!("synced {} contacts", result.contacts_synced);

        // Update last sync timestamp and ROWID
        let now = chrono::Utc::now().timestamp_millis();
        let mut config = self.config.write().await;
        config.sync.last_incremental_sync = now;

        // Mark sync as complete
        self.report_progress(&progress, SyncPhase::Complete, 0, None, "sync complete");
        self.event_bus.emit(AppEvent::SyncComplete {
            is_full_sync: true,
            messages_synced: result.messages_synced,
        });

        info!("full sync complete: {result}");
        Ok(result)
    }

    /// Run an incremental sync (delta updates since last sync).
    ///
    /// Uses ROWID-based sync when the server supports it (>= 1.6.0),
    /// otherwise falls back to timestamp-based sync.
    pub async fn incremental_sync(&self, api: &ApiClient) -> BbResult<SyncResult> {
        let config = self.config.read().await;
        let last_sync = config.sync.last_incremental_sync;
        let last_row_id = config.sync.last_incremental_sync_row_id;
        drop(config);

        info!("starting incremental sync (since_ts: {last_sync}, since_rowid: {last_row_id})");

        // Try ROWID-based sync first (more reliable), else timestamp fallback
        let result = if last_row_id > 0 {
            self.incremental_sync_by_rowid(api, last_row_id).await?
        } else {
            self.incremental_sync_by_timestamp(api, last_sync).await?
        };

        // Update last sync timestamp
        let now = chrono::Utc::now().timestamp_millis();
        let mut config = self.config.write().await;
        config.sync.last_incremental_sync = now;

        if result.messages_synced > 0 {
            self.event_bus.emit(AppEvent::SyncComplete {
                is_full_sync: false,
                messages_synced: result.messages_synced,
            });
        }

        info!("incremental sync complete: {result}");
        Ok(result)
    }

    /// ROWID-based incremental sync.
    ///
    /// Queries the server for messages with ROWID > last_row_id. This is the
    /// preferred approach for servers >= 1.6.0 as it is deterministic and does
    /// not suffer from clock-skew issues.
    async fn incremental_sync_by_rowid(
        &self,
        api: &ApiClient,
        since_row_id: i64,
    ) -> BbResult<SyncResult> {
        debug!("incremental sync by ROWID (since: {since_row_id})");
        let mut result = SyncResult::default();
        let mut max_row_id = since_row_id;

        let query = MessageQuery {
            with: vec![
                "chats".into(),
                "attachment".into(),
                "handle".into(),
                "attributedBody".into(),
            ],
            where_clauses: vec![],
            sort: Some("ASC".into()),
            before: None,
            after: Some(since_row_id),
            chat_guid: None,
            offset: 0,
            limit: 1000,
            convert_attachments: None,
        };

        let (messages, _total) = api.query_messages(&query).await?;
        let conn = self.database.conn()?;

        for msg_json in &messages {
            if let Ok(mut msg) = bb_models::Message::from_server_map(msg_json) {
                // Track the highest ROWID for the next sync
                if let Some(row_id) = msg_json
                    .get("ROWID")
                    .or_else(|| msg_json.get("rowId"))
                    .and_then(|v| v.as_i64())
                {
                    max_row_id = max_row_id.max(row_id);
                }

                // Resolve chat from the chats array
                if let Some(chat_data) = msg_json
                    .get("chats")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                {
                    if let Ok(mut chat) = bb_models::Chat::from_server_map(chat_data) {
                        let _ = chat.save(&conn);
                        msg.chat_id = chat.id;
                    }
                }

                // Save handle
                if let Some(handle_data) = msg_json.get("handle") {
                    if !handle_data.is_null() {
                        if let Ok(mut handle) = bb_models::Handle::from_server_map(handle_data) {
                            let _ = handle.save(&conn);
                            msg.handle_id = handle.id;
                        }
                    }
                }

                let _ = msg.save(&conn);

                // Save attachments
                if let Some(attachments) = msg_json.get("attachments").and_then(|v| v.as_array()) {
                    for att_json in attachments {
                        if let Ok(mut att) = bb_models::Attachment::from_server_map(att_json) {
                            att.message_id = msg.id;
                            let _ = att.save(&conn);
                        }
                    }
                }

                result.messages_synced += 1;
            }
        }

        // Update the ROWID bookmark
        if max_row_id > since_row_id {
            let mut config = self.config.write().await;
            config.sync.last_incremental_sync_row_id = max_row_id;
            debug!("updated last_incremental_sync_row_id to {max_row_id}");
        }

        Ok(result)
    }

    /// Timestamp-based incremental sync (fallback).
    ///
    /// Queries messages created after the given timestamp. Less reliable
    /// than ROWID-based sync due to potential clock skew, but works with
    /// all server versions.
    async fn incremental_sync_by_timestamp(
        &self,
        api: &ApiClient,
        since_timestamp: i64,
    ) -> BbResult<SyncResult> {
        debug!("incremental sync by timestamp (since: {since_timestamp})");
        let mut result = SyncResult::default();

        let after = if since_timestamp > 0 {
            Some(since_timestamp)
        } else {
            None
        };

        let updated_count = api.message_count(after, None).await?;
        debug!("server reports {updated_count} messages since last sync");

        if updated_count > 0 {
            let query = MessageQuery {
                with: vec![
                    "chats".into(),
                    "attachment".into(),
                    "handle".into(),
                    "attributedBody".into(),
                ],
                where_clauses: vec![],
                sort: Some("ASC".into()),
                before: None,
                after,
                chat_guid: None,
                offset: 0,
                limit: updated_count.min(1000),
                convert_attachments: None,
            };

            let (messages, _total) = api.query_messages(&query).await?;
            let conn = self.database.conn()?;

            for msg_json in &messages {
                if let Ok(mut msg) = bb_models::Message::from_server_map(msg_json) {
                    // Resolve chat
                    if let Some(chat_data) = msg_json
                        .get("chats")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                    {
                        if let Ok(mut chat) = bb_models::Chat::from_server_map(chat_data) {
                            let _ = chat.save(&conn);
                            msg.chat_id = chat.id;
                        }
                    }

                    // Save handle
                    if let Some(handle_data) = msg_json.get("handle") {
                        if !handle_data.is_null() {
                            if let Ok(mut handle) =
                                bb_models::Handle::from_server_map(handle_data)
                            {
                                let _ = handle.save(&conn);
                                msg.handle_id = handle.id;
                            }
                        }
                    }

                    let _ = msg.save(&conn);

                    // Save attachments
                    if let Some(attachments) =
                        msg_json.get("attachments").and_then(|v| v.as_array())
                    {
                        for att_json in attachments {
                            if let Ok(mut att) =
                                bb_models::Attachment::from_server_map(att_json)
                            {
                                att.message_id = msg.id;
                                let _ = att.save(&conn);
                            }
                        }
                    }

                    result.messages_synced += 1;
                }
            }
        }

        Ok(result)
    }
}

/// Result summary from a sync operation.
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub chats_synced: u64,
    pub messages_synced: u64,
    pub contacts_synced: u64,
}

impl std::fmt::Display for SyncResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chats={}, messages={}, contacts={}",
            self.chats_synced, self.messages_synced, self.contacts_synced
        )
    }
}

impl Service for SyncService {
    fn name(&self) -> &str {
        "sync"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("sync service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("sync service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_phase_display() {
        assert_eq!(format!("{:?}", SyncPhase::Chats), "Chats");
        assert_eq!(format!("{:?}", SyncPhase::Complete), "Complete");
    }

    #[test]
    fn test_sync_result_display() {
        let result = SyncResult {
            chats_synced: 10,
            messages_synced: 500,
            contacts_synced: 50,
        };
        let s = format!("{result}");
        assert!(s.contains("chats=10"));
        assert!(s.contains("messages=500"));
    }

    #[test]
    fn test_sync_result_default() {
        let result = SyncResult::default();
        assert_eq!(result.chats_synced, 0);
        assert_eq!(result.messages_synced, 0);
        assert_eq!(result.contacts_synced, 0);
    }
}
