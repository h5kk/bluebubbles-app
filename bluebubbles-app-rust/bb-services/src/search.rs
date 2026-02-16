//! Search service for global search across messages, chats, and contacts.
//!
//! Combines results from multiple data sources (messages, chats, contacts)
//! and ranks them by relevance using text matching heuristics.

use tracing::{info, debug};

use bb_core::error::BbResult;
use bb_models::{Database, Chat, Message, Contact};
use bb_models::queries;

use crate::event_bus::EventBus;
use crate::service::{Service, ServiceState};

/// A unified search result that can be a message, chat, or contact match.
#[derive(Debug, Clone)]
pub enum SearchResult {
    /// A matching message.
    MessageResult {
        message: Message,
        /// Relevance score (higher is better).
        score: f64,
    },
    /// A matching chat.
    ChatResult {
        chat: Chat,
        score: f64,
    },
    /// A matching contact.
    ContactResult {
        contact: Contact,
        score: f64,
    },
}

impl SearchResult {
    /// Get the relevance score.
    pub fn score(&self) -> f64 {
        match self {
            SearchResult::MessageResult { score, .. } => *score,
            SearchResult::ChatResult { score, .. } => *score,
            SearchResult::ContactResult { score, .. } => *score,
        }
    }
}

/// Service for global search across messages, chats, and contacts.
///
/// Searches are performed locally against the SQLite database. Results from
/// each domain are scored individually and then merged into a unified result
/// list sorted by relevance.
pub struct SearchService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
}

impl SearchService {
    /// Create a new SearchService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
        }
    }

    /// Perform a global search across all data sources.
    ///
    /// Returns a merged list of results sorted by relevance score (descending).
    /// The `limit` parameter caps the total number of results returned.
    pub fn search_all(&self, query: &str, limit: usize) -> BbResult<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Search messages
        let messages = self.search_messages(&query_lower, limit)?;
        results.extend(messages);

        // Search chats
        let chats = self.search_chats(&query_lower, limit)?;
        results.extend(chats);

        // Search contacts
        let contacts = self.search_contacts(&query_lower, limit)?;
        results.extend(contacts);

        // Sort by score descending
        results.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap_or(std::cmp::Ordering::Equal));

        // Truncate to the requested limit
        results.truncate(limit);

        debug!("search '{}' returned {} results", query, results.len());
        Ok(results)
    }

    /// Search messages by text content.
    pub fn search_messages(&self, query: &str, limit: usize) -> BbResult<Vec<SearchResult>> {
        let conn = self.database.conn()?;
        let messages = queries::search_messages(&conn, query, limit as i64)?;

        Ok(messages
            .into_iter()
            .map(|msg| {
                let score = Self::score_message_match(&msg, query);
                SearchResult::MessageResult {
                    message: msg,
                    score,
                }
            })
            .collect())
    }

    /// Search chats by display name or identifier.
    pub fn search_chats(&self, query: &str, limit: usize) -> BbResult<Vec<SearchResult>> {
        let conn = self.database.conn()?;
        let chats = queries::search_chats(&conn, query, limit as i64)?;

        Ok(chats
            .into_iter()
            .map(|chat| {
                let score = Self::score_chat_match(&chat, query);
                SearchResult::ChatResult { chat, score }
            })
            .collect())
    }

    /// Search contacts by display name, phone, or email.
    pub fn search_contacts(&self, query: &str, limit: usize) -> BbResult<Vec<SearchResult>> {
        let conn = self.database.conn()?;
        let contacts = queries::search_contacts(&conn, query, limit as i64)?;

        Ok(contacts
            .into_iter()
            .map(|contact| {
                let score = Self::score_contact_match(&contact, query);
                SearchResult::ContactResult { contact, score }
            })
            .collect())
    }

    /// Score a message match. Exact matches and shorter messages score higher.
    fn score_message_match(msg: &Message, query: &str) -> f64 {
        let text = msg.text.as_deref().unwrap_or("").to_lowercase();
        if text.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;

        // Exact match bonus
        if text == query {
            score += 10.0;
        }

        // Starts-with bonus
        if text.starts_with(query) {
            score += 5.0;
        }

        // Contains bonus (base score)
        if text.contains(query) {
            score += 3.0;
        }

        // Length penalty: prefer shorter matches (more focused)
        let length_factor = 1.0 / (1.0 + (text.len() as f64 / 100.0));
        score *= 1.0 + length_factor;

        // Recency bonus based on the from_me flag (outgoing scored slightly less)
        if !msg.is_from_me {
            score += 0.5;
        }

        score
    }

    /// Score a chat match. Display name matches rank highest.
    fn score_chat_match(chat: &Chat, query: &str) -> f64 {
        let mut score = 0.0;

        if let Some(ref name) = chat.display_name {
            let name_lower = name.to_lowercase();
            if name_lower == query {
                score += 15.0;
            } else if name_lower.starts_with(query) {
                score += 10.0;
            } else if name_lower.contains(query) {
                score += 5.0;
            }
        }

        if let Some(ref identifier) = chat.chat_identifier {
            let id_lower = identifier.to_lowercase();
            if id_lower.contains(query) {
                score += 3.0;
            }
        }

        score
    }

    /// Score a contact match. Name matches rank highest, then addresses.
    fn score_contact_match(contact: &Contact, query: &str) -> f64 {
        let mut score = 0.0;

        let name = contact.display_name.to_lowercase();
        if name == query {
            score += 15.0;
        } else if name.starts_with(query) {
            score += 10.0;
        } else if name.contains(query) {
            score += 5.0;
        }

        // Check phone numbers (stored as JSON array string)
        if let Ok(phones) = serde_json::from_str::<Vec<String>>(&contact.phones) {
            for phone in &phones {
                let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
                let query_digits: String = query.chars().filter(|c| c.is_ascii_digit()).collect();
                if !query_digits.is_empty() && digits.contains(&query_digits) {
                    score += 4.0;
                }
            }
        }

        // Check emails (stored as JSON array string)
        if let Ok(emails) = serde_json::from_str::<Vec<String>>(&contact.emails) {
            for email in &emails {
                if email.to_lowercase().contains(query) {
                    score += 4.0;
                }
            }
        }

        score
    }
}

impl Service for SearchService {
    fn name(&self) -> &str {
        "search"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("search service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("search service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&path, &config).unwrap();
        std::mem::forget(dir);
        db
    }

    #[test]
    fn test_search_service_name() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = SearchService::new(db, bus);
        assert_eq!(svc.name(), "search");
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = SearchService::new(db, bus);
        let results = svc.search_all("", 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_no_results() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = SearchService::new(db, bus);
        let results = svc.search_all("nonexistent query xyz", 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_result_score_accessor() {
        let json = serde_json::json!({"guid": "test-msg", "text": "hello world"});
        let msg = Message::from_server_map(&json).unwrap();
        let result = SearchResult::MessageResult {
            message: msg,
            score: 7.5,
        };
        assert!((result.score() - 7.5).abs() < f64::EPSILON);
    }
}
