//! Message queue service for outgoing message retry.
//!
//! Provides a priority queue for outgoing messages that handles retry logic
//! with exponential backoff, temp GUID -> real GUID mutation tracking,
//! error classification for retry decisions, and a background processing loop.

use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::{info, debug, warn};

use bb_core::error::{BbResult, MessageError};
use crate::service::{Service, ServiceState};

/// A queued outgoing message awaiting send.
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    /// Unique identifier for this queue entry.
    pub id: String,
    /// Target chat GUID.
    pub chat_guid: String,
    /// Message text (for text messages).
    pub text: Option<String>,
    /// File path (for attachment messages).
    pub file_path: Option<String>,
    /// Number of send attempts so far.
    pub attempts: u32,
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Timestamp of the last attempt.
    pub last_attempt: Option<std::time::Instant>,
}

impl QueuedMessage {
    /// Whether this message should be retried.
    pub fn should_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    /// Calculate the delay before the next retry attempt.
    ///
    /// Uses exponential backoff: 2^attempts seconds, capped at 256 seconds.
    pub fn retry_delay(&self) -> Duration {
        let base = 2u64.pow(self.attempts.min(8));
        Duration::from_secs(base)
    }

    /// Whether this message is ready for its next retry attempt.
    pub fn is_ready(&self) -> bool {
        self.last_attempt.map_or(true, |last| {
            last.elapsed() >= self.retry_delay()
        })
    }
}

/// Status of a sent message (for tracking temp -> real GUID).
#[derive(Debug, Clone)]
pub enum SendStatus {
    /// Message is pending in the queue.
    Pending,
    /// Message was sent successfully. Contains the real server GUID.
    Sent { real_guid: String },
    /// Message send failed permanently.
    Failed { error: String, error_code: MessageError },
}

/// Service that manages outgoing message retry queue.
///
/// Provides:
/// - Priority queue with exponential backoff retry
/// - Temp GUID -> real GUID mutation tracking
/// - Error classification for retry decisions
/// - Queue statistics and monitoring
pub struct QueueService {
    state: ServiceState,
    /// The message queue (priority sorted: older messages first).
    queue: Arc<Mutex<VecDeque<QueuedMessage>>>,
    /// Mapping of temp GUIDs to their send status.
    guid_status: Arc<Mutex<HashMap<String, SendStatus>>>,
}

impl QueueService {
    /// Create a new QueueService.
    pub fn new() -> Self {
        Self {
            state: ServiceState::Created,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            guid_status: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a message to the retry queue.
    pub async fn enqueue(&self, msg: QueuedMessage) {
        let id = msg.id.clone();
        let mut queue = self.queue.lock().await;
        info!("queued message for retry: {} (attempt {})", msg.id, msg.attempts);
        queue.push_back(msg);

        // Track status
        let mut status = self.guid_status.lock().await;
        status.insert(id, SendStatus::Pending);
    }

    /// Remove and return the next message ready for retry.
    pub async fn dequeue(&self) -> Option<QueuedMessage> {
        let mut queue = self.queue.lock().await;

        // Find the first message whose retry delay has elapsed
        for i in 0..queue.len() {
            if queue[i].is_ready() {
                return queue.remove(i);
            }
        }

        None
    }

    /// Get the current queue length.
    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Whether the queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }

    /// Remove a message from the queue by ID.
    pub async fn remove(&self, id: &str) -> bool {
        let mut queue = self.queue.lock().await;
        if let Some(pos) = queue.iter().position(|m| m.id == id) {
            queue.remove(pos);
            debug!("removed message from queue: {id}");
            true
        } else {
            false
        }
    }

    /// Clear all messages from the queue.
    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        let count = queue.len();
        queue.clear();
        if count > 0 {
            info!("cleared {count} messages from queue");
        }
    }

    /// Record that a message was sent successfully.
    ///
    /// Maps the temp GUID to the real server GUID for UI updates.
    pub async fn mark_sent(&self, temp_guid: &str, real_guid: &str) {
        let mut status = self.guid_status.lock().await;
        status.insert(
            temp_guid.to_string(),
            SendStatus::Sent {
                real_guid: real_guid.to_string(),
            },
        );
        debug!("marked sent: {temp_guid} -> {real_guid}");
    }

    /// Record that a message send failed permanently.
    pub async fn mark_failed(&self, temp_guid: &str, error: &str, error_code: MessageError) {
        let mut status = self.guid_status.lock().await;
        status.insert(
            temp_guid.to_string(),
            SendStatus::Failed {
                error: error.to_string(),
                error_code,
            },
        );
        warn!("marked failed: {temp_guid} ({error})");
    }

    /// Look up the send status of a message by its temp GUID.
    pub async fn get_status(&self, temp_guid: &str) -> Option<SendStatus> {
        let status = self.guid_status.lock().await;
        status.get(temp_guid).cloned()
    }

    /// Resolve a temp GUID to its real GUID if the message was sent.
    pub async fn resolve_guid(&self, temp_guid: &str) -> Option<String> {
        let status = self.guid_status.lock().await;
        match status.get(temp_guid) {
            Some(SendStatus::Sent { real_guid }) => Some(real_guid.clone()),
            _ => None,
        }
    }

    /// Clean up old status entries to prevent unbounded growth.
    ///
    /// Removes entries for messages that have been sent or failed and are
    /// no longer pending.
    pub async fn cleanup_status(&self, max_entries: usize) {
        let mut status = self.guid_status.lock().await;
        if status.len() <= max_entries {
            return;
        }

        // Keep only the most recent entries
        let to_remove = status.len() - max_entries;
        let keys_to_remove: Vec<String> = status
            .iter()
            .filter(|(_, v)| !matches!(v, SendStatus::Pending))
            .take(to_remove)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            status.remove(&key);
        }
    }

    /// Get queue statistics.
    pub async fn stats(&self) -> QueueStats {
        let queue = self.queue.lock().await;
        let status = self.guid_status.lock().await;

        let pending = queue.len();
        let ready = queue.iter().filter(|m| m.is_ready()).count();
        let total_tracked = status.len();
        let sent_count = status
            .values()
            .filter(|s| matches!(s, SendStatus::Sent { .. }))
            .count();
        let failed_count = status
            .values()
            .filter(|s| matches!(s, SendStatus::Failed { .. }))
            .count();

        QueueStats {
            pending,
            ready,
            total_tracked,
            sent_count,
            failed_count,
        }
    }
}

/// Queue statistics for monitoring.
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Messages waiting in queue.
    pub pending: usize,
    /// Messages ready for immediate retry.
    pub ready: usize,
    /// Total tracked message statuses.
    pub total_tracked: usize,
    /// Successfully sent messages.
    pub sent_count: usize,
    /// Permanently failed messages.
    pub failed_count: usize,
}

impl std::fmt::Display for QueueStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pending={}, ready={}, sent={}, failed={}",
            self.pending, self.ready, self.sent_count, self.failed_count
        )
    }
}

impl Service for QueueService {
    fn name(&self) -> &str { "queue" }
    fn state(&self) -> ServiceState { self.state }
    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("queue service initialized");
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

    #[tokio::test]
    async fn test_queue_enqueue_dequeue() {
        let svc = QueueService::new();

        let msg = QueuedMessage {
            id: "q-1".into(),
            chat_guid: "chat-1".into(),
            text: Some("Hello".into()),
            file_path: None,
            attempts: 0,
            max_attempts: 3,
            last_attempt: None,
        };

        svc.enqueue(msg).await;
        assert_eq!(svc.len().await, 1);

        let dequeued = svc.dequeue().await.unwrap();
        assert_eq!(dequeued.id, "q-1");
        assert!(svc.is_empty().await);
    }

    #[test]
    fn test_retry_delay() {
        let msg = QueuedMessage {
            id: "q-1".into(),
            chat_guid: "chat-1".into(),
            text: None,
            file_path: None,
            attempts: 3,
            max_attempts: 5,
            last_attempt: None,
        };
        assert_eq!(msg.retry_delay(), Duration::from_secs(8));
        assert!(msg.should_retry());
    }

    #[test]
    fn test_max_attempts_reached() {
        let msg = QueuedMessage {
            id: "q-1".into(),
            chat_guid: "chat-1".into(),
            text: None,
            file_path: None,
            attempts: 5,
            max_attempts: 5,
            last_attempt: None,
        };
        assert!(!msg.should_retry());
    }

    #[test]
    fn test_is_ready_no_last_attempt() {
        let msg = QueuedMessage {
            id: "q-1".into(),
            chat_guid: "chat-1".into(),
            text: None,
            file_path: None,
            attempts: 0,
            max_attempts: 3,
            last_attempt: None,
        };
        assert!(msg.is_ready());
    }

    #[tokio::test]
    async fn test_guid_tracking() {
        let svc = QueueService::new();

        // Initially no status
        assert!(svc.get_status("temp-1").await.is_none());

        // Mark sent
        svc.mark_sent("temp-1", "real-1").await;
        let status = svc.get_status("temp-1").await.unwrap();
        match status {
            SendStatus::Sent { real_guid } => assert_eq!(real_guid, "real-1"),
            _ => panic!("expected Sent status"),
        }

        // Resolve GUID
        assert_eq!(svc.resolve_guid("temp-1").await, Some("real-1".to_string()));
        assert_eq!(svc.resolve_guid("temp-nonexistent").await, None);
    }

    #[tokio::test]
    async fn test_mark_failed() {
        let svc = QueueService::new();
        svc.mark_failed("temp-2", "timeout", MessageError::Timeout).await;

        let status = svc.get_status("temp-2").await.unwrap();
        match status {
            SendStatus::Failed { error, error_code } => {
                assert_eq!(error, "timeout");
                assert_eq!(error_code, MessageError::Timeout);
            }
            _ => panic!("expected Failed status"),
        }
    }

    #[tokio::test]
    async fn test_stats() {
        let svc = QueueService::new();

        svc.enqueue(QueuedMessage {
            id: "q-1".into(),
            chat_guid: "chat-1".into(),
            text: Some("test".into()),
            file_path: None,
            attempts: 0,
            max_attempts: 3,
            last_attempt: None,
        })
        .await;

        svc.mark_sent("temp-a", "real-a").await;
        svc.mark_failed("temp-b", "err", MessageError::Unknown).await;

        let stats = svc.stats().await;
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.sent_count, 1);
        assert_eq!(stats.failed_count, 1);
        assert!(stats.to_string().contains("pending=1"));
    }
}
