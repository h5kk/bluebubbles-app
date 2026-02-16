//! Contact service for managing address book contacts.
//!
//! Handles contact sync from the server, local search, phone suffix matching
//! (7-15 digits), two-pass network fetch, and handle-to-contact resolution.

use tracing::{info, warn};
use bb_core::error::BbResult;
use bb_models::{Database, Contact, Handle};
use bb_models::queries;
use bb_api::ApiClient;

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Service for managing contacts.
///
/// Handles contact sync from the server, local search, and handle-to-contact
/// resolution for display name lookups. Uses phone number suffix matching
/// with 7-15 digits for fuzzy phone lookups.
pub struct ContactService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
}

impl ContactService {
    /// Create a new ContactService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
        }
    }

    /// List all contacts from the local database.
    pub fn list_contacts(&self) -> BbResult<Vec<Contact>> {
        let conn = self.database.conn()?;
        queries::list_contacts(&conn)
    }

    /// Search contacts by display name.
    pub fn search_contacts(&self, query: &str, limit: i64) -> BbResult<Vec<Contact>> {
        let conn = self.database.conn()?;
        queries::search_contacts(&conn, query, limit)
    }

    /// Find a contact by external ID.
    pub fn find_contact(&self, external_id: &str) -> BbResult<Option<Contact>> {
        let conn = self.database.conn()?;
        queries::find_contact_by_external_id(&conn, external_id)
    }

    /// Find a contact by phone number using suffix matching.
    ///
    /// Searches using the last 7-15 digits of the phone number. This handles
    /// international formatting differences (e.g., +1-555-123-4567 matches
    /// 555-123-4567 or 5551234567).
    pub fn find_contact_by_phone(&self, phone: &str) -> BbResult<Option<Contact>> {
        let digits = extract_digits(phone);
        if digits.len() < 7 {
            return Ok(None);
        }

        let conn = self.database.conn()?;

        // Try suffix lengths from 7 to min(15, digit_count) for best match
        let max_suffix = digits.len().min(15);
        for suffix_len in (7..=max_suffix).rev() {
            let suffix = &digits[digits.len() - suffix_len..];
            let results = queries::search_contacts_by_phone_suffix(&conn, suffix, 1)?;
            if let Some(contact) = results.into_iter().next() {
                return Ok(Some(contact));
            }
        }

        Ok(None)
    }

    /// Sync all contacts from the server.
    ///
    /// Two-pass fetch:
    /// 1. First pass: fetch contacts without avatars (fast)
    /// 2. Second pass: fetch with avatars if requested
    ///
    /// Replaces all local contacts with the server's address book.
    pub async fn sync_contacts(
        &self,
        api: &ApiClient,
        include_avatars: bool,
    ) -> BbResult<usize> {
        info!("syncing contacts from server (avatars: {include_avatars})");

        // Pass 1: fetch without avatars for fast initial population
        let contacts_json = api.get_contacts(false).await?;
        let conn = self.database.conn()?;

        // Clear existing contacts
        queries::delete_all_contacts(&conn)?;

        let mut count = 0;
        for contact_json in &contacts_json {
            if let Ok(mut contact) = Contact::from_server_map(contact_json) {
                if contact.save(&conn).is_ok() {
                    count += 1;
                }
            }
        }

        info!("pass 1: synced {count} contacts (no avatars)");

        // Pass 2: fetch with avatars if requested
        if include_avatars {
            match api.get_contacts(true).await {
                Ok(contacts_with_avatars) => {
                    for contact_json in &contacts_with_avatars {
                        if let Ok(mut contact) = Contact::from_server_map(contact_json) {
                            // Update existing contact with avatar data
                            let _ = contact.save(&conn);
                        }
                    }
                    info!("pass 2: updated contacts with avatars");
                }
                Err(e) => {
                    warn!("failed to fetch contacts with avatars: {e}");
                }
            }
        }

        self.event_bus.emit(AppEvent::ContactsUpdated { count });
        Ok(count)
    }

    /// Resolve an address (phone or email) to a contact display name.
    ///
    /// Uses phone suffix matching for phone numbers and exact match for emails.
    pub fn resolve_display_name(&self, address: &str) -> BbResult<Option<String>> {
        // Try email exact match first
        if address.contains('@') {
            let contacts = self.search_contacts(address, 5)?;
            for contact in &contacts {
                if contact.matches_address(address) {
                    return Ok(Some(contact.display_name.clone()));
                }
            }
            return Ok(None);
        }

        // Try phone suffix match
        if let Some(contact) = self.find_contact_by_phone(address)? {
            return Ok(Some(contact.display_name.clone()));
        }

        // Fall back to linear scan for edge cases
        let contacts = self.list_contacts()?;
        for contact in &contacts {
            if contact.matches_address(address) {
                return Ok(Some(contact.display_name.clone()));
            }
        }
        Ok(None)
    }

    /// Resolve a handle to its display name using contacts.
    ///
    /// First checks the handle's own display name, then falls back to
    /// contact resolution.
    pub fn resolve_handle_name(&self, handle: &Handle) -> BbResult<String> {
        // Use the handle's own display name if it has one
        let handle_name = handle.display_name();
        if !handle_name.is_empty() && handle_name != handle.address {
            return Ok(handle_name);
        }

        // Fall back to contact resolution
        if let Some(name) = self.resolve_display_name(&handle.address)? {
            return Ok(name);
        }

        // Last resort: return the raw address
        Ok(handle.address.clone())
    }

    /// Batch-resolve multiple addresses to display names.
    ///
    /// Loads all contacts once and resolves each address against them.
    /// Returns a vec of (address, resolved_name) pairs.
    pub fn resolve_batch(&self, addresses: &[String]) -> BbResult<Vec<(String, String)>> {
        let contacts = self.list_contacts()?;
        let mut results = Vec::with_capacity(addresses.len());

        for address in addresses {
            let mut resolved = address.clone();

            for contact in &contacts {
                if contact.matches_address(address) {
                    resolved = contact.display_name.clone();
                    break;
                }
            }

            // If not found by direct match and it looks like a phone number, try suffix
            if resolved == *address && !address.contains('@') {
                if let Ok(Some(contact)) = self.find_contact_by_phone(address) {
                    resolved = contact.display_name.clone();
                }
            }

            results.push((address.clone(), resolved));
        }

        Ok(results)
    }

    /// Get the total number of contacts.
    pub fn count(&self) -> BbResult<i64> {
        let contacts = self.list_contacts()?;
        Ok(contacts.len() as i64)
    }
}

/// Extract only digit characters from a phone string.
fn extract_digits(phone: &str) -> String {
    phone.chars().filter(|c| c.is_ascii_digit()).collect()
}

impl Service for ContactService {
    fn name(&self) -> &str { "contact" }
    fn state(&self) -> ServiceState { self.state }
    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("contact service initialized");
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

    fn create_test_db() -> Database {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&path, &config).unwrap();
        std::mem::forget(dir);
        db
    }

    #[test]
    fn test_contact_service_name() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = ContactService::new(db, bus);
        assert_eq!(svc.name(), "contact");
    }

    #[test]
    fn test_extract_digits() {
        assert_eq!(extract_digits("+1 (555) 123-4567"), "15551234567");
        assert_eq!(extract_digits("5551234567"), "5551234567");
        assert_eq!(extract_digits("+44 20 7946 0958"), "442079460958");
    }

    #[test]
    fn test_short_phone_returns_none() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = ContactService::new(db, bus);
        // Less than 7 digits should return None
        assert!(svc.find_contact_by_phone("12345").unwrap().is_none());
    }

    #[test]
    fn test_empty_contacts() {
        let db = create_test_db();
        let bus = crate::event_bus::EventBus::new(16);
        let svc = ContactService::new(db, bus);
        assert_eq!(svc.count().unwrap(), 0);
        assert!(svc.resolve_display_name("test@test.com").unwrap().is_none());
    }
}
