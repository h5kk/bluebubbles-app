//! Socket.IO event types and event dispatcher.
//!
//! Defines all event types streamed from the BlueBubbles server and provides
//! a broadcast-based event dispatcher for decoupled event handling.

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::debug;

/// All socket event types emitted by the BlueBubbles server.
///
/// These map 1:1 to the server's Socket.IO event names as documented
/// in the API reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocketEventType {
    /// A new message was received (`new-message`).
    NewMessage,
    /// An existing message was updated: delivered, read, edited, unsent (`updated-message`).
    UpdatedMessage,
    /// Typing indicator from a chat participant (`typing-indicator`).
    TypingIndicator,
    /// Group chat name was changed (`group-name-change`).
    GroupNameChange,
    /// A participant was added to a group chat (`participant-added`).
    ParticipantAdded,
    /// A participant was removed from a group chat (`participant-removed`).
    ParticipantRemoved,
    /// A participant left a group chat voluntarily (`participant-left`).
    ParticipantLeft,
    /// Chat read status changed on another device (`chat-read-status-changed`).
    ChatReadStatusChanged,
    /// The group chat icon was updated (`group-icon-changed`).
    GroupIconChanged,
    /// An incoming FaceTime call notification (`incoming-facetime`).
    IncomingFaceTime,
    /// FaceTime call status changed (`ft-call-status-changed`).
    FtCallStatusChanged,
    /// iMessage aliases were removed from the account (`imessage-aliases-removed`).
    IMessageAliasesRemoved,
    /// A new server update is available (`server-update`).
    ServerUpdate,
    /// Unknown/unhandled event type.
    Unknown(String),
}

impl SocketEventType {
    /// Parse an event type string from the server.
    pub fn from_str(s: &str) -> Self {
        match s {
            "new-message" => Self::NewMessage,
            "updated-message" => Self::UpdatedMessage,
            "typing-indicator" => Self::TypingIndicator,
            "group-name-change" => Self::GroupNameChange,
            "participant-added" => Self::ParticipantAdded,
            "participant-removed" => Self::ParticipantRemoved,
            "participant-left" => Self::ParticipantLeft,
            "chat-read-status-changed" => Self::ChatReadStatusChanged,
            "group-icon-changed" => Self::GroupIconChanged,
            "incoming-facetime" => Self::IncomingFaceTime,
            "ft-call-status-changed" => Self::FtCallStatusChanged,
            "imessage-aliases-removed" => Self::IMessageAliasesRemoved,
            "server-update" => Self::ServerUpdate,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Convert to the server event string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::NewMessage => "new-message",
            Self::UpdatedMessage => "updated-message",
            Self::TypingIndicator => "typing-indicator",
            Self::GroupNameChange => "group-name-change",
            Self::ParticipantAdded => "participant-added",
            Self::ParticipantRemoved => "participant-removed",
            Self::ParticipantLeft => "participant-left",
            Self::ChatReadStatusChanged => "chat-read-status-changed",
            Self::GroupIconChanged => "group-icon-changed",
            Self::IncomingFaceTime => "incoming-facetime",
            Self::FtCallStatusChanged => "ft-call-status-changed",
            Self::IMessageAliasesRemoved => "imessage-aliases-removed",
            Self::ServerUpdate => "server-update",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// Whether this event type relates to messages.
    pub fn is_message_event(&self) -> bool {
        matches!(self, Self::NewMessage | Self::UpdatedMessage)
    }

    /// Whether this event type relates to group chat membership changes.
    pub fn is_participant_event(&self) -> bool {
        matches!(
            self,
            Self::ParticipantAdded | Self::ParticipantRemoved | Self::ParticipantLeft
        )
    }

    /// Whether this event type relates to FaceTime.
    pub fn is_facetime_event(&self) -> bool {
        matches!(self, Self::IncomingFaceTime | Self::FtCallStatusChanged)
    }

    /// All event type strings that the socket should subscribe to.
    pub fn all_event_names() -> &'static [&'static str] {
        &[
            "new-message",
            "updated-message",
            "typing-indicator",
            "group-name-change",
            "participant-added",
            "participant-removed",
            "participant-left",
            "chat-read-status-changed",
            "group-icon-changed",
            "incoming-facetime",
            "ft-call-status-changed",
            "imessage-aliases-removed",
            "server-update",
        ]
    }
}

/// Typed payload for typing indicator events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicatorPayload {
    /// Chat GUID where typing is occurring.
    pub guid: String,
    /// Whether the indicator should be displayed.
    pub display: bool,
}

/// Typed payload for chat read status events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatReadStatusPayload {
    /// Chat GUID whose read status changed.
    #[serde(rename = "chatGuid")]
    pub chat_guid: String,
    /// Whether the chat is now read.
    pub read: bool,
}

/// Typed payload for FaceTime call status events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtCallStatusPayload {
    /// Call UUID.
    pub uuid: String,
    /// Status ID: 4 = incoming, 6 = ended.
    pub status_id: i32,
    /// Handle information.
    pub handle: Option<FtCallHandle>,
    /// Address of the caller.
    pub address: Option<String>,
    /// Whether this is an audio-only call.
    pub is_audio: Option<bool>,
}

/// Handle info within a FaceTime call status payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtCallHandle {
    pub address: String,
}

/// Typed payload for iMessage aliases removed events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasesRemovedPayload {
    pub aliases: Vec<String>,
}

/// A socket event with type and associated data payload.
#[derive(Debug, Clone)]
pub struct SocketEvent {
    /// The type of event.
    pub event_type: SocketEventType,
    /// The event payload data from the server.
    pub data: serde_json::Value,
}

impl SocketEvent {
    /// Try to parse the data as a TypingIndicatorPayload.
    pub fn as_typing_indicator(&self) -> Option<TypingIndicatorPayload> {
        if self.event_type == SocketEventType::TypingIndicator {
            serde_json::from_value(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Try to parse the data as a ChatReadStatusPayload.
    pub fn as_chat_read_status(&self) -> Option<ChatReadStatusPayload> {
        if self.event_type == SocketEventType::ChatReadStatusChanged {
            serde_json::from_value(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Try to parse the data as a FtCallStatusPayload.
    pub fn as_ft_call_status(&self) -> Option<FtCallStatusPayload> {
        if self.event_type == SocketEventType::FtCallStatusChanged {
            serde_json::from_value(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Try to parse the data as an AliasesRemovedPayload.
    pub fn as_aliases_removed(&self) -> Option<AliasesRemovedPayload> {
        if self.event_type == SocketEventType::IMessageAliasesRemoved {
            serde_json::from_value(self.data.clone()).ok()
        } else {
            None
        }
    }
}

/// Broadcast-based event dispatcher for decoupled event handling.
///
/// Uses tokio::broadcast channels so multiple consumers can independently
/// receive and process events without blocking each other.
#[derive(Clone)]
pub struct EventDispatcher {
    sender: broadcast::Sender<SocketEvent>,
}

impl EventDispatcher {
    /// Create a new EventDispatcher with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to receive socket events.
    ///
    /// Returns a broadcast receiver. Slow consumers that fall behind
    /// will receive a RecvError::Lagged and may miss events.
    pub fn subscribe(&self) -> broadcast::Receiver<SocketEvent> {
        self.sender.subscribe()
    }

    /// Dispatch an event to all active subscribers.
    pub fn dispatch(&self, event: SocketEvent) {
        let event_type = event.event_type.as_str().to_string();
        match self.sender.send(event) {
            Ok(count) => {
                debug!("dispatched {event_type} to {count} subscriber(s)");
            }
            Err(_) => {
                // No active receivers -- this is fine during startup/shutdown
                debug!("no subscribers for event {event_type}");
            }
        }
    }

    /// Get the current number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Connection state for the socket manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected and not trying to connect.
    Disconnected,
    /// Attempting to establish connection.
    Connecting,
    /// Connected and receiving events.
    Connected,
    /// Connection lost, attempting to reconnect.
    Reconnecting,
    /// Fatal error, will not auto-reconnect.
    Failed,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Reconnecting => write!(f, "reconnecting"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_parsing() {
        assert_eq!(
            SocketEventType::from_str("new-message"),
            SocketEventType::NewMessage
        );
        assert_eq!(
            SocketEventType::from_str("updated-message"),
            SocketEventType::UpdatedMessage
        );
        assert_eq!(
            SocketEventType::from_str("chat-read-status-changed"),
            SocketEventType::ChatReadStatusChanged
        );
        assert_eq!(
            SocketEventType::from_str("ft-call-status-changed"),
            SocketEventType::FtCallStatusChanged
        );
        assert_eq!(
            SocketEventType::from_str("imessage-aliases-removed"),
            SocketEventType::IMessageAliasesRemoved
        );
        assert_eq!(
            SocketEventType::from_str("unknown-event"),
            SocketEventType::Unknown("unknown-event".into())
        );
    }

    #[test]
    fn test_event_type_roundtrip() {
        let types = vec![
            SocketEventType::NewMessage,
            SocketEventType::UpdatedMessage,
            SocketEventType::TypingIndicator,
            SocketEventType::GroupNameChange,
            SocketEventType::ParticipantAdded,
            SocketEventType::ParticipantRemoved,
            SocketEventType::ParticipantLeft,
            SocketEventType::ChatReadStatusChanged,
            SocketEventType::GroupIconChanged,
            SocketEventType::IncomingFaceTime,
            SocketEventType::FtCallStatusChanged,
            SocketEventType::IMessageAliasesRemoved,
            SocketEventType::ServerUpdate,
        ];
        for event_type in types {
            let s = event_type.as_str();
            assert_eq!(SocketEventType::from_str(s), event_type);
        }
    }

    #[test]
    fn test_event_type_categories() {
        assert!(SocketEventType::NewMessage.is_message_event());
        assert!(SocketEventType::UpdatedMessage.is_message_event());
        assert!(!SocketEventType::TypingIndicator.is_message_event());

        assert!(SocketEventType::ParticipantAdded.is_participant_event());
        assert!(!SocketEventType::NewMessage.is_participant_event());

        assert!(SocketEventType::IncomingFaceTime.is_facetime_event());
        assert!(SocketEventType::FtCallStatusChanged.is_facetime_event());
    }

    #[test]
    fn test_all_event_names() {
        let names = SocketEventType::all_event_names();
        assert!(names.len() >= 13);
        assert!(names.contains(&"new-message"));
        assert!(names.contains(&"ft-call-status-changed"));
    }

    #[tokio::test]
    async fn test_event_dispatcher() {
        let dispatcher = EventDispatcher::new(16);
        let mut rx = dispatcher.subscribe();

        dispatcher.dispatch(SocketEvent {
            event_type: SocketEventType::NewMessage,
            data: serde_json::json!({"guid": "msg-1"}),
        });

        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type, SocketEventType::NewMessage);
    }

    #[test]
    fn test_connection_state_display() {
        assert_eq!(ConnectionState::Connected.to_string(), "connected");
        assert_eq!(ConnectionState::Reconnecting.to_string(), "reconnecting");
    }

    #[test]
    fn test_typing_indicator_payload() {
        let json = serde_json::json!({"guid": "iMessage;-;+1234", "display": true});
        let payload: TypingIndicatorPayload = serde_json::from_value(json).unwrap();
        assert!(payload.display);
    }

    #[test]
    fn test_chat_read_status_payload() {
        let json = serde_json::json!({"chatGuid": "iMessage;-;+1234", "read": true});
        let payload: ChatReadStatusPayload = serde_json::from_value(json).unwrap();
        assert!(payload.read);
    }

    #[test]
    fn test_ft_call_status_payload() {
        let json = serde_json::json!({
            "uuid": "call-123",
            "status_id": 4,
            "handle": {"address": "+15551234"},
            "address": "+15551234",
            "is_audio": false
        });
        let payload: FtCallStatusPayload = serde_json::from_value(json).unwrap();
        assert_eq!(payload.status_id, 4);
        assert_eq!(payload.handle.unwrap().address, "+15551234");
    }

    #[test]
    fn test_socket_event_typed_access() {
        let event = SocketEvent {
            event_type: SocketEventType::TypingIndicator,
            data: serde_json::json!({"guid": "iMessage;-;+1234", "display": true}),
        };
        let payload = event.as_typing_indicator().unwrap();
        assert!(payload.display);

        // Wrong type should return None
        assert!(event.as_chat_read_status().is_none());
    }
}
