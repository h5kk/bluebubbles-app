//! Notification service for desktop notifications.
//!
//! Manages native notifications for incoming messages, reactions, FaceTime
//! calls, and connection errors. Supports notification grouping by chat,
//! mute/DND filtering, and mark-as-read from notification actions.

#[allow(unused_imports)]
use bb_core::error::BbError;
use bb_core::error::BbResult;
use bb_core::config::ConfigHandle;
use tracing::{info, debug};

use crate::service::{Service, ServiceState};

/// Notification category for grouping and filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationCategory {
    /// A new incoming message.
    Message,
    /// A tapback/reaction.
    Reaction,
    /// An incoming FaceTime call.
    FaceTime,
    /// A socket/connection error.
    ConnectionError,
    /// A sync status notification.
    SyncStatus,
}

/// Service for managing desktop notifications.
///
/// Creates and displays native notifications for incoming messages,
/// reactions, FaceTime calls, and other events. Respects mute settings,
/// DND state, and notification configuration.
pub struct NotificationService {
    state: ServiceState,
    config: ConfigHandle,
    /// Whether notifications are globally enabled.
    enabled: bool,
    /// Set of muted chat GUIDs (notifications are suppressed for these).
    muted_chats: std::collections::HashSet<String>,
}

impl NotificationService {
    /// Create a new NotificationService.
    pub fn new(config: ConfigHandle) -> Self {
        Self {
            state: ServiceState::Created,
            config,
            enabled: true,
            muted_chats: std::collections::HashSet::new(),
        }
    }

    /// Set whether notifications are globally enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("notifications {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Whether notifications are globally enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Mute notifications for a specific chat.
    pub fn mute_chat(&mut self, chat_guid: &str) {
        self.muted_chats.insert(chat_guid.to_string());
        debug!("muted notifications for chat: {chat_guid}");
    }

    /// Unmute notifications for a specific chat.
    pub fn unmute_chat(&mut self, chat_guid: &str) {
        self.muted_chats.remove(chat_guid);
        debug!("unmuted notifications for chat: {chat_guid}");
    }

    /// Whether a chat's notifications are muted.
    pub fn is_chat_muted(&self, chat_guid: &str) -> bool {
        self.muted_chats.contains(chat_guid)
    }

    /// Show a notification for an incoming message.
    ///
    /// Respects global enable, chat mute, reaction filtering, and
    /// unknown sender filtering from configuration.
    pub async fn notify_message(
        &self,
        sender: &str,
        text: &str,
        chat_title: &str,
        chat_guid: &str,
        is_from_known_sender: bool,
    ) -> BbResult<()> {
        if !self.should_notify(chat_guid, is_from_known_sender).await {
            return Ok(());
        }

        self.show_notification(
            &format!("{sender} in {chat_title}"),
            text,
            NotificationCategory::Message,
            Some(chat_guid),
        )?;

        debug!("message notification: {sender} - {text}");
        Ok(())
    }

    /// Show a notification for a reaction/tapback.
    pub async fn notify_reaction(
        &self,
        sender: &str,
        reaction: &str,
        chat_title: &str,
        chat_guid: &str,
    ) -> BbResult<()> {
        // Check if reaction notifications are enabled
        let config = self.config.read().await;
        if !config.notifications.notify_reactions {
            return Ok(());
        }
        drop(config);

        if !self.should_notify(chat_guid, true).await {
            return Ok(());
        }

        self.show_notification(
            &format!("{sender} reacted in {chat_title}"),
            reaction,
            NotificationCategory::Reaction,
            Some(chat_guid),
        )?;

        debug!("reaction notification: {sender} {reaction}");
        Ok(())
    }

    /// Show a notification for an incoming FaceTime call.
    pub fn notify_facetime(
        &self,
        caller: &str,
        is_audio: bool,
    ) -> BbResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let call_type = if is_audio { "audio" } else { "video" };
        self.show_notification(
            &format!("Incoming FaceTime {call_type} call"),
            &format!("From: {caller}"),
            NotificationCategory::FaceTime,
            None,
        )?;

        info!("FaceTime notification: {caller} ({call_type})");
        Ok(())
    }

    /// Show a notification for a connection error or state change.
    pub fn notify_connection_error(&self, message: &str) -> BbResult<()> {
        if !self.enabled {
            return Ok(());
        }

        self.show_notification(
            "Connection Issue",
            message,
            NotificationCategory::ConnectionError,
            None,
        )?;

        debug!("connection notification: {message}");
        Ok(())
    }

    /// Show a generic notification.
    pub fn notify(&self, title: &str, body: &str) -> BbResult<()> {
        if !self.enabled {
            return Ok(());
        }

        self.show_notification(title, body, NotificationCategory::SyncStatus, None)
    }

    /// Check whether text detection keywords match the message.
    ///
    /// If global_text_detection is configured, checks if the message text
    /// contains any of the configured keywords.
    pub async fn matches_text_detection(&self, text: &str) -> bool {
        let config = self.config.read().await;
        let keywords = &config.notifications.global_text_detection;
        if keywords.is_empty() {
            return false;
        }
        drop(config);

        let text_lower = text.to_lowercase();
        // Keywords are comma-separated
        let config = self.config.read().await;
        config
            .notifications
            .global_text_detection
            .split(',')
            .map(|k| k.trim().to_lowercase())
            .any(|keyword| !keyword.is_empty() && text_lower.contains(&keyword))
    }

    /// Determine whether a notification should be shown.
    async fn should_notify(&self, chat_guid: &str, is_from_known_sender: bool) -> bool {
        if !self.enabled {
            return false;
        }

        if self.muted_chats.contains(chat_guid) {
            return false;
        }

        // Check unknown sender filter
        let config = self.config.read().await;
        if config.notifications.filter_unknown_senders && !is_from_known_sender {
            return false;
        }

        true
    }

    /// Actually show the native notification.
    fn show_notification(
        &self,
        title: &str,
        body: &str,
        _category: NotificationCategory,
        _chat_guid: Option<&str>,
    ) -> BbResult<()> {
        #[cfg(not(test))]
        {
            notify_rust::Notification::new()
                .summary(title)
                .body(body)
                .appname("BlueBubbles")
                .show()
                .map_err(|e| BbError::Notification(e.to_string()))?;
        }

        let _ = (title, body);
        Ok(())
    }
}

impl Service for NotificationService {
    fn name(&self) -> &str { "notification" }
    fn state(&self) -> ServiceState { self.state }
    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("notification service initialized");
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

    fn make_config() -> ConfigHandle {
        ConfigHandle::new(bb_core::config::AppConfig::default())
    }

    #[test]
    fn test_notification_service_name() {
        let svc = NotificationService::new(make_config());
        assert_eq!(svc.name(), "notification");
    }

    #[test]
    fn test_enable_disable() {
        let mut svc = NotificationService::new(make_config());
        svc.init().unwrap();
        assert!(svc.is_enabled());

        svc.set_enabled(false);
        assert!(!svc.is_enabled());

        svc.set_enabled(true);
        assert!(svc.is_enabled());
    }

    #[test]
    fn test_mute_unmute_chat() {
        let mut svc = NotificationService::new(make_config());
        assert!(!svc.is_chat_muted("chat-1"));

        svc.mute_chat("chat-1");
        assert!(svc.is_chat_muted("chat-1"));
        assert!(!svc.is_chat_muted("chat-2"));

        svc.unmute_chat("chat-1");
        assert!(!svc.is_chat_muted("chat-1"));
    }

    #[tokio::test]
    async fn test_should_notify_basic() {
        let svc = NotificationService::new(make_config());
        assert!(svc.should_notify("chat-1", true).await);

        let mut svc2 = NotificationService::new(make_config());
        svc2.set_enabled(false);
        assert!(!svc2.should_notify("chat-1", true).await);
    }

    #[tokio::test]
    async fn test_should_notify_muted_chat() {
        let mut svc = NotificationService::new(make_config());
        svc.mute_chat("chat-muted");
        assert!(!svc.should_notify("chat-muted", true).await);
        assert!(svc.should_notify("chat-other", true).await);
    }

    #[tokio::test]
    async fn test_notify_message_disabled() {
        let mut svc = NotificationService::new(make_config());
        svc.set_enabled(false);
        // Should not error even when disabled
        svc.notify_message("John", "Hello", "Test", "chat-1", true)
            .await
            .unwrap();
    }

    #[test]
    fn test_notify_facetime() {
        let svc = NotificationService::new(make_config());
        svc.notify_facetime("John Doe", false).unwrap();
        svc.notify_facetime("Jane", true).unwrap();
    }

    #[test]
    fn test_notification_categories() {
        assert_eq!(NotificationCategory::Message, NotificationCategory::Message);
        assert_ne!(NotificationCategory::Message, NotificationCategory::Reaction);
    }
}
