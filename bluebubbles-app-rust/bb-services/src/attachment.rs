//! Attachment service for managing file downloads and uploads.
//!
//! Handles attachment download with a concurrency-limited queue (max 2),
//! active chat prioritization, live photo handling, post-download processing,
//! and local file caching with cleanup.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::{Mutex, Semaphore};
use tracing::{info, debug};
use bb_core::constants;
use bb_core::error::{BbError, BbResult};
use bb_models::{Database, Attachment};
use bb_models::queries;
use bb_api::ApiClient;

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Priority level for download queue items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DownloadPriority {
    /// Low priority: background pre-fetch.
    Low = 0,
    /// Normal priority: user scrolled to a message with an attachment.
    Normal = 1,
    /// High priority: attachment in the currently active chat.
    High = 2,
}

/// A queued attachment download request.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    /// Attachment GUID.
    pub attachment_guid: String,
    /// Whether to download the original quality.
    pub original: bool,
    /// Download priority.
    pub priority: DownloadPriority,
    /// The chat this attachment belongs to (for prioritization).
    pub chat_guid: Option<String>,
}

/// Service for managing message attachments.
///
/// Handles attachment download with concurrency limiting (max 2 simultaneous),
/// priority queue (active chat first), upload for outgoing messages, local
/// file caching, live photo support, and cache cleanup.
pub struct AttachmentService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
    /// Local directory for cached attachment files.
    cache_dir: PathBuf,
    /// Download concurrency limiter.
    download_semaphore: Arc<Semaphore>,
    /// Pending download queue (priority-sorted).
    download_queue: Arc<Mutex<VecDeque<DownloadRequest>>>,
    /// GUID of the currently active chat for priority boosting.
    active_chat_guid: Arc<Mutex<Option<String>>>,
}

impl AttachmentService {
    /// Create a new AttachmentService.
    pub fn new(database: Database, event_bus: EventBus, cache_dir: PathBuf) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
            cache_dir,
            download_semaphore: Arc::new(Semaphore::new(constants::MAX_CONCURRENT_DOWNLOADS)),
            download_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_chat_guid: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the local cache path for an attachment GUID.
    pub fn cache_path(&self, guid: &str, extension: Option<&str>) -> PathBuf {
        let safe_name = guid.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let filename = if let Some(ext) = extension {
            format!("{safe_name}.{ext}")
        } else {
            safe_name
        };
        self.cache_dir.join(filename)
    }

    /// Whether a cached file exists for the given attachment GUID.
    pub fn is_cached(&self, guid: &str, extension: Option<&str>) -> bool {
        self.cache_path(guid, extension).exists()
    }

    /// Set the currently active chat GUID for download prioritization.
    pub async fn set_active_chat(&self, chat_guid: Option<String>) {
        let mut active = self.active_chat_guid.lock().await;
        *active = chat_guid;
    }

    /// Enqueue an attachment for download with priority handling.
    ///
    /// Inserts the request at the correct position based on priority.
    /// High-priority requests (active chat) are moved to the front.
    pub async fn enqueue_download(&self, mut request: DownloadRequest) {
        // Boost priority if this attachment belongs to the active chat
        let active = self.active_chat_guid.lock().await;
        if let Some(ref active_guid) = *active {
            if request.chat_guid.as_deref() == Some(active_guid.as_str()) {
                request.priority = DownloadPriority::High;
            }
        }
        drop(active);

        let mut queue = self.download_queue.lock().await;

        // Remove any existing request for the same GUID
        queue.retain(|r| r.attachment_guid != request.attachment_guid);

        // Insert at the correct position based on priority
        let pos = queue
            .iter()
            .position(|r| r.priority < request.priority)
            .unwrap_or(queue.len());

        queue.insert(pos, request);
        debug!("download queue size: {}", queue.len());
    }

    /// Get the next download request from the queue.
    pub async fn dequeue_download(&self) -> Option<DownloadRequest> {
        let mut queue = self.download_queue.lock().await;
        queue.pop_front()
    }

    /// Get the current download queue length.
    pub async fn queue_len(&self) -> usize {
        self.download_queue.lock().await.len()
    }

    /// Download an attachment from the server and cache it locally.
    ///
    /// Respects the concurrency limit (max 2 simultaneous downloads).
    pub async fn download(
        &self,
        api: &ApiClient,
        guid: &str,
        original: bool,
    ) -> BbResult<PathBuf> {
        // Acquire a permit from the semaphore
        let _permit = self
            .download_semaphore
            .acquire()
            .await
            .map_err(|_| BbError::Internal("download semaphore closed".into()))?;

        debug!("downloading attachment: {guid}");

        let bytes = api.download_attachment(guid, original).await?;

        // Determine file extension from attachment metadata
        let conn = self.database.conn()?;
        let extension = queries::find_attachment_by_guid(&conn, guid)?
            .and_then(|a| a.file_extension().map(String::from));

        let path = self.cache_path(guid, extension.as_deref());

        // Ensure cache directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, &bytes)?;
        info!("attachment cached: {} ({} bytes)", path.display(), bytes.len());

        self.event_bus.emit(AppEvent::AttachmentDownloaded {
            attachment_guid: guid.to_string(),
            local_path: path.to_string_lossy().to_string(),
        });

        Ok(path)
    }

    /// Download and cache a live photo movie component.
    ///
    /// Live photos consist of a still image + a short video. This downloads
    /// the video part.
    pub async fn download_live_photo(
        &self,
        api: &ApiClient,
        guid: &str,
    ) -> BbResult<PathBuf> {
        let _permit = self
            .download_semaphore
            .acquire()
            .await
            .map_err(|_| BbError::Internal("download semaphore closed".into()))?;

        debug!("downloading live photo: {guid}");

        let bytes = api.download_live_photo(guid).await?;
        let path = self.cache_path(&format!("{guid}-live"), Some("mov"));

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, &bytes)?;
        info!(
            "live photo cached: {} ({} bytes)",
            path.display(),
            bytes.len()
        );

        Ok(path)
    }

    /// Upload an attachment and send it in a chat.
    pub async fn upload_and_send(
        &self,
        api: &ApiClient,
        chat_guid: &str,
        file_path: &Path,
        mime_type: &str,
    ) -> BbResult<Attachment> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        let file_bytes = std::fs::read(file_path)?;
        let temp_guid = format!("temp-{}", uuid::Uuid::new_v4());

        let msg_json = api
            .send_attachment(chat_guid, &temp_guid, file_name, file_bytes, mime_type, "private-api")
            .await?;

        // Parse attachment from the response
        let attachments = msg_json
            .get("attachments")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if let Some(att_json) = attachments.first() {
            let mut att = Attachment::from_server_map(att_json)?;
            let conn = self.database.conn()?;
            att.save(&conn)?;
            info!("attachment uploaded: {:?}", att.guid);
            Ok(att)
        } else {
            Err(BbError::Http("no attachment in response".into()))
        }
    }

    /// Load attachments for a message from the local database.
    pub fn attachments_for_message(&self, message_id: i64) -> BbResult<Vec<Attachment>> {
        let conn = self.database.conn()?;
        queries::load_attachments_for_message(&conn, message_id)
    }

    /// Find an attachment by GUID.
    pub fn find_attachment(&self, guid: &str) -> BbResult<Option<Attachment>> {
        let conn = self.database.conn()?;
        queries::find_attachment_by_guid(&conn, guid)
    }

    /// Check if an attachment is a live photo (has a paired video component).
    pub fn is_live_photo(&self, attachment: &Attachment) -> bool {
        // Live photos are HEIC/JPEG images with a paired .MOV file
        if let Some(mime) = &attachment.mime_type {
            return mime.starts_with("image/")
                && attachment
                    .transfer_name
                    .as_deref()
                    .map_or(false, |n| {
                        n.to_lowercase().ends_with(".heic")
                            || n.to_lowercase().ends_with(".jpeg")
                            || n.to_lowercase().ends_with(".jpg")
                    });
        }
        false
    }

    /// Get the file size of a cached attachment (if cached).
    pub fn cached_file_size(&self, guid: &str) -> Option<u64> {
        let conn = self.database.conn().ok()?;
        let ext = queries::find_attachment_by_guid(&conn, guid)
            .ok()?
            .and_then(|a| a.file_extension().map(String::from));
        let path = self.cache_path(guid, ext.as_deref());
        std::fs::metadata(&path).ok().map(|m| m.len())
    }

    /// Clean up cached files older than the given number of days.
    pub fn cleanup_cache(&self, max_age_days: u32) -> BbResult<u32> {
        let mut removed = 0u32;
        let max_age = std::time::Duration::from_secs(max_age_days as u64 * 86400);

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = modified.elapsed() {
                            if age > max_age {
                                if std::fs::remove_file(entry.path()).is_ok() {
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if removed > 0 {
            info!("cleaned up {removed} cached attachments");
        }
        Ok(removed)
    }

    /// Get the total size of the attachment cache in bytes.
    pub fn cache_size_bytes(&self) -> u64 {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            }
        }
        total
    }

    /// Clear the entire attachment cache.
    pub fn clear_cache(&self) -> BbResult<u32> {
        let mut removed = 0u32;
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if std::fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }
        if removed > 0 {
            info!("cleared {removed} cached attachments");
        }
        Ok(removed)
    }
}

impl Service for AttachmentService {
    fn name(&self) -> &str { "attachment" }
    fn state(&self) -> ServiceState { self.state }
    fn init(&mut self) -> BbResult<()> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;
        self.state = ServiceState::Running;
        info!("attachment service initialized (cache: {})", self.cache_dir.display());
        Ok(())
    }
    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_svc() -> (AttachmentService, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&db_path, &config).unwrap();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = AttachmentService::new(db, bus, dir.path().join("cache"));
        (svc, dir)
    }

    #[test]
    fn test_cache_path() {
        let (svc, _dir) = create_test_svc();
        let path = svc.cache_path("att-123", Some("jpg"));
        assert!(path.to_string_lossy().contains("att-123.jpg"));
    }

    #[test]
    fn test_cache_path_sanitize() {
        let (svc, _dir) = create_test_svc();
        let path = svc.cache_path("att/bad:name", Some("png"));
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(!name.contains('/'));
        assert!(!name.contains(':'));
    }

    #[tokio::test]
    async fn test_download_queue_priority() {
        let (svc, _dir) = create_test_svc();

        // Enqueue normal priority
        svc.enqueue_download(DownloadRequest {
            attachment_guid: "att-1".into(),
            original: false,
            priority: DownloadPriority::Normal,
            chat_guid: None,
        })
        .await;

        // Enqueue high priority - should go to front
        svc.enqueue_download(DownloadRequest {
            attachment_guid: "att-2".into(),
            original: false,
            priority: DownloadPriority::High,
            chat_guid: None,
        })
        .await;

        assert_eq!(svc.queue_len().await, 2);

        let first = svc.dequeue_download().await.unwrap();
        assert_eq!(first.attachment_guid, "att-2");
        assert_eq!(first.priority, DownloadPriority::High);
    }

    #[tokio::test]
    async fn test_active_chat_boost() {
        let (svc, _dir) = create_test_svc();

        svc.set_active_chat(Some("chat-active".into())).await;

        // This should get boosted to High priority
        svc.enqueue_download(DownloadRequest {
            attachment_guid: "att-active".into(),
            original: false,
            priority: DownloadPriority::Normal,
            chat_guid: Some("chat-active".into()),
        })
        .await;

        let req = svc.dequeue_download().await.unwrap();
        assert_eq!(req.priority, DownloadPriority::High);
    }

    #[test]
    fn test_service_name() {
        let (svc, _dir) = create_test_svc();
        assert_eq!(svc.name(), "attachment");
    }

    #[test]
    fn test_cleanup_empty_cache() {
        let (mut svc, _dir) = create_test_svc();
        svc.init().unwrap();
        let removed = svc.cleanup_cache(30).unwrap();
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_cache_size_empty() {
        let (mut svc, _dir) = create_test_svc();
        svc.init().unwrap();
        assert_eq!(svc.cache_size_bytes(), 0);
    }
}
