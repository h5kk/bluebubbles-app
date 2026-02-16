//! Application-wide constants.

/// Application name.
pub const APP_NAME: &str = "BlueBubbles";

/// Application version.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// REST API version prefix.
pub const API_VERSION: &str = "v1";

/// Default server API timeout in milliseconds.
pub const DEFAULT_API_TIMEOUT_MS: u64 = 30_000;

/// Extended timeout multiplier for large transfers.
pub const EXTENDED_TIMEOUT_MULTIPLIER: u64 = 12;

/// Maximum concurrent attachment downloads.
pub const MAX_CONCURRENT_DOWNLOADS: usize = 2;

/// Socket reconnection delay in seconds.
pub const SOCKET_RECONNECT_DELAY_SECS: u64 = 5;

/// Maximum number of handled GUIDs for deduplication.
pub const MAX_HANDLED_GUID_HISTORY: usize = 100;

/// Default number of messages per page for sync.
pub const DEFAULT_MESSAGES_PER_PAGE: u32 = 25;

/// Default chat page size for sync.
pub const DEFAULT_CHAT_PAGE_SIZE: u32 = 200;

/// Database schema version.
pub const DB_SCHEMA_VERSION: i32 = 1;

/// Reaction type string constants matching iMessage values.
pub mod reactions {
    pub const LOVE: &str = "love";
    pub const LIKE: &str = "like";
    pub const DISLIKE: &str = "dislike";
    pub const LAUGH: &str = "laugh";
    pub const EMPHASIZE: &str = "emphasize";
    pub const QUESTION: &str = "question";

    /// All valid reaction types.
    pub const ALL: &[&str] = &[LOVE, LIKE, DISLIKE, LAUGH, EMPHASIZE, QUESTION];
}

/// Message effect identifiers matching Apple's internal IDs.
pub mod effects {
    pub const SLAM: &str = "com.apple.MobileSMS.expressivesend.impact";
    pub const LOUD: &str = "com.apple.MobileSMS.expressivesend.loud";
    pub const GENTLE: &str = "com.apple.MobileSMS.expressivesend.gentle";
    pub const INVISIBLE_INK: &str = "com.apple.MobileSMS.expressivesend.invisibleink";
    pub const ECHO: &str = "com.apple.messages.effect.CKEchoEffect";
    pub const SPOTLIGHT: &str = "com.apple.messages.effect.CKSpotlightEffect";
    pub const BALLOONS: &str = "com.apple.messages.effect.CKHappyBirthdayEffect";
    pub const CONFETTI: &str = "com.apple.messages.effect.CKConfettiEffect";
    pub const LOVE: &str = "com.apple.messages.effect.CKHeartEffect";
    pub const LASERS: &str = "com.apple.messages.effect.CKLasersEffect";
    pub const FIREWORKS: &str = "com.apple.messages.effect.CKFireworksEffect";
    pub const CELEBRATION: &str = "com.apple.messages.effect.CKSparklesEffect";
}

/// Chat style constants.
pub mod chat_style {
    /// Group chat style identifier.
    pub const GROUP: i32 = 43;
}

/// Known balloon bundle IDs mapped to human-readable names.
pub fn balloon_bundle_name(bundle_id: &str) -> &str {
    match bundle_id {
        "com.apple.Handwriting.HandwritingProvider" => "Handwritten Message",
        "com.apple.PassbookUIService.PeerPaymentMessagesExtension" => "Apple Pay",
        "com.apple.mobileslideshow.PhotosMessagesApp" => "Photo Slideshow",
        "com.apple.icloud.apps.messages.business.extension" => "Business Chat",
        _ => "Interactive Message",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_constants() {
        assert_eq!(reactions::ALL.len(), 6);
        assert!(reactions::ALL.contains(&"love"));
    }

    #[test]
    fn test_balloon_bundle_name() {
        assert_eq!(
            balloon_bundle_name("com.apple.Handwriting.HandwritingProvider"),
            "Handwritten Message"
        );
        assert_eq!(balloon_bundle_name("unknown.bundle"), "Interactive Message");
    }
}
