//! Typed event bus for intra-service communication.
//!
//! Uses tokio broadcast channels to decouple services from one another.
//! Any service can emit events without knowing who is listening, and any
//! number of subscribers can independently consume events.

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

/// All application-level event types that flow through the event bus.
///
/// These are distinct from raw socket events -- they represent processed,
/// application-meaningful state changes that other services care about.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A new message was saved to the local database.
    MessageReceived {
        message_guid: String,
        chat_guid: String,
        is_from_me: bool,
    },
    /// An existing message was updated (delivered, read, edited, unsent).
    MessageUpdated {
        message_guid: String,
        chat_guid: String,
    },
    /// A message was successfully sent (temp GUID replaced with real GUID).
    MessageSent {
        temp_guid: String,
        real_guid: String,
        chat_guid: String,
    },
    /// A message send failed permanently.
    MessageFailed {
        temp_guid: String,
        chat_guid: String,
        error: String,
    },
    /// A chat was created or updated in the local database.
    ChatUpdated {
        chat_guid: String,
    },
    /// A chat was deleted or soft-deleted.
    ChatDeleted {
        chat_guid: String,
    },
    /// Typing indicator state changed for a chat.
    TypingChanged {
        chat_guid: String,
        is_typing: bool,
    },
    /// Socket connection state changed.
    ConnectionStateChanged {
        connected: bool,
        message: String,
    },
    /// Sync progress update.
    SyncProgress {
        phase: String,
        current: u64,
        total: Option<u64>,
        message: String,
    },
    /// Sync completed.
    SyncComplete {
        is_full_sync: bool,
        messages_synced: u64,
    },
    /// Contacts were refreshed.
    ContactsUpdated {
        count: usize,
    },
    /// Theme was changed.
    ThemeChanged {
        theme_name: String,
    },
    /// A participant was added to a group chat.
    ParticipantAdded {
        chat_guid: String,
        address: String,
    },
    /// A participant was removed from a group chat.
    ParticipantRemoved {
        chat_guid: String,
        address: String,
    },
    /// Group chat name was changed.
    GroupNameChanged {
        chat_guid: String,
        new_name: String,
    },
    /// Incoming FaceTime call.
    IncomingFaceTime {
        call_uuid: String,
        caller: String,
        is_audio: bool,
    },
    /// FaceTime call status changed.
    FaceTimeStatusChanged {
        call_uuid: String,
        status: i32,
    },
    /// An attachment download completed.
    AttachmentDownloaded {
        attachment_guid: String,
        local_path: String,
    },
    /// An attachment download failed.
    AttachmentDownloadFailed {
        attachment_guid: String,
        error: String,
    },
    /// iMessage aliases were removed from the account.
    AliasesRemoved {
        aliases: Vec<String>,
    },
}

/// Application-wide event bus backed by a tokio broadcast channel.
///
/// Designed for fan-out delivery: every subscriber gets every event.
/// Slow subscribers that fall behind will receive a `Lagged` error
/// and may miss events, which is acceptable for UI-driven consumers.
#[derive(Clone)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<AppEvent>>,
}

impl EventBus {
    /// Create a new EventBus with the given channel capacity.
    ///
    /// A capacity of 256 is recommended. Events beyond this limit will
    /// cause slow subscribers to lag and miss events.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Subscribe to receive application events.
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, event: AppEvent) {
        let label = event_label(&event);
        match self.sender.send(event) {
            Ok(count) => {
                debug!("event_bus: emitted {label} to {count} subscriber(s)");
            }
            Err(_) => {
                debug!("event_bus: no subscribers for {label}");
            }
        }
    }

    /// Get the current number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Human-readable label for an event (for logging).
fn event_label(event: &AppEvent) -> &'static str {
    match event {
        AppEvent::MessageReceived { .. } => "MessageReceived",
        AppEvent::MessageUpdated { .. } => "MessageUpdated",
        AppEvent::MessageSent { .. } => "MessageSent",
        AppEvent::MessageFailed { .. } => "MessageFailed",
        AppEvent::ChatUpdated { .. } => "ChatUpdated",
        AppEvent::ChatDeleted { .. } => "ChatDeleted",
        AppEvent::TypingChanged { .. } => "TypingChanged",
        AppEvent::ConnectionStateChanged { .. } => "ConnectionStateChanged",
        AppEvent::SyncProgress { .. } => "SyncProgress",
        AppEvent::SyncComplete { .. } => "SyncComplete",
        AppEvent::ContactsUpdated { .. } => "ContactsUpdated",
        AppEvent::ThemeChanged { .. } => "ThemeChanged",
        AppEvent::ParticipantAdded { .. } => "ParticipantAdded",
        AppEvent::ParticipantRemoved { .. } => "ParticipantRemoved",
        AppEvent::GroupNameChanged { .. } => "GroupNameChanged",
        AppEvent::IncomingFaceTime { .. } => "IncomingFaceTime",
        AppEvent::FaceTimeStatusChanged { .. } => "FaceTimeStatusChanged",
        AppEvent::AttachmentDownloaded { .. } => "AttachmentDownloaded",
        AppEvent::AttachmentDownloadFailed { .. } => "AttachmentDownloadFailed",
        AppEvent::AliasesRemoved { .. } => "AliasesRemoved",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_emit_receive() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.emit(AppEvent::ChatUpdated {
            chat_guid: "chat-1".into(),
        });

        let event = rx.recv().await.unwrap();
        match event {
            AppEvent::ChatUpdated { chat_guid } => assert_eq!(chat_guid, "chat-1"),
            _ => panic!("unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        bus.emit(AppEvent::ContactsUpdated { count: 42 });

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();

        match (e1, e2) {
            (AppEvent::ContactsUpdated { count: c1 }, AppEvent::ContactsUpdated { count: c2 }) => {
                assert_eq!(c1, 42);
                assert_eq!(c2, 42);
            }
            _ => panic!("unexpected event types"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_no_subscribers() {
        let bus = EventBus::new(16);
        // Should not panic even with no subscribers
        bus.emit(AppEvent::SyncComplete {
            is_full_sync: true,
            messages_synced: 100,
        });
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_event_labels() {
        assert_eq!(
            event_label(&AppEvent::MessageReceived {
                message_guid: String::new(),
                chat_guid: String::new(),
                is_from_me: false,
            }),
            "MessageReceived"
        );
        assert_eq!(
            event_label(&AppEvent::ThemeChanged {
                theme_name: String::new()
            }),
            "ThemeChanged"
        );
    }
}
