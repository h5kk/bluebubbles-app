//! Query builders for common database access patterns.
//!
//! Provides paginated, filtered, and sorted queries for messages, chats,
//! handles, contacts, and attachments. All queries use parameterized SQL
//! to prevent injection and return domain model types.
//!
//! **Performance key**: Message pagination uses cursor-based pagination
//! (keyset pagination on date_created) rather than OFFSET-based pagination,
//! which avoids the O(n) skip cost that degraded Flutter performance.

use rusqlite::{params, Connection};
use bb_core::error::{BbError, BbResult};

use crate::models::chat::Chat;
use crate::models::message::Message;
use crate::models::handle::Handle;
use crate::models::attachment::Attachment;
use crate::models::contact::Contact;

/// Sort direction for query results.
#[derive(Debug, Clone, Copy)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn as_sql(&self) -> &str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

// ─── Chat Queries ───────────────────────────────────────────────────────────

/// Chat with additional computed fields from a join query.
#[derive(Debug, Clone)]
pub struct ChatWithDetails {
    pub chat: Chat,
    pub unread_count: i64,
    pub last_message_text: Option<String>,
    pub last_message_date: Option<String>,
    pub last_message_is_from_me: bool,
    pub participant_count: i64,
}

/// List chats ordered by pinned-first then latest message date.
///
/// Joins with messages to get last message info and unread count.
pub fn list_chats_with_details(
    conn: &Connection,
    offset: i64,
    limit: i64,
    include_archived: bool,
) -> BbResult<Vec<ChatWithDetails>> {
    let archive_filter = if include_archived { "" } else { "WHERE c.is_archived = 0 AND c.date_deleted IS NULL" };

    let sql = format!(
        "SELECT c.*,
            COALESCE((SELECT COUNT(*) FROM messages m2 WHERE m2.chat_id = c.id AND m2.date_read IS NULL AND m2.is_from_me = 0 AND m2.date_deleted IS NULL), 0) AS unread_count,
            lm.text AS last_message_text,
            lm.date_created AS last_message_date_computed,
            COALESCE(lm.is_from_me, 0) AS last_message_is_from_me,
            COALESCE((SELECT COUNT(*) FROM chat_handle_join chj WHERE chj.chat_id = c.id), 0) AS participant_count
        FROM chats c
        LEFT JOIN messages lm ON lm.id = (
            SELECT m.id FROM messages m WHERE m.chat_id = c.id AND m.date_deleted IS NULL
            ORDER BY m.date_created DESC LIMIT 1
        )
        {archive_filter}
        ORDER BY c.is_pinned DESC, c.pin_index ASC, c.latest_message_date DESC
        LIMIT ?1 OFFSET ?2"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| BbError::Database(e.to_string()))?;
    let results = stmt
        .query_map(params![limit, offset], |row| {
            let chat = Chat::from_row(row)?;
            Ok(ChatWithDetails {
                chat,
                unread_count: row.get("unread_count")?,
                last_message_text: row.get("last_message_text")?,
                last_message_date: row.get("last_message_date_computed")?,
                last_message_is_from_me: row.get::<_, i32>("last_message_is_from_me")? != 0,
                participant_count: row.get("participant_count")?,
            })
        })
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// List chats ordered by latest message date (simple version without join).
pub fn list_chats(
    conn: &Connection,
    offset: i64,
    limit: i64,
    include_archived: bool,
) -> BbResult<Vec<Chat>> {
    let sql = if include_archived {
        "SELECT * FROM chats ORDER BY is_pinned DESC, latest_message_date DESC LIMIT ?1 OFFSET ?2"
    } else {
        "SELECT * FROM chats WHERE is_archived = 0 AND date_deleted IS NULL ORDER BY is_pinned DESC, latest_message_date DESC LIMIT ?1 OFFSET ?2"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| BbError::Database(e.to_string()))?;
    let chats = stmt
        .query_map(params![limit, offset], Chat::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(chats)
}

/// Find a chat by its GUID.
pub fn find_chat_by_guid(conn: &Connection, guid: &str) -> BbResult<Option<Chat>> {
    Chat::find_by_guid(conn, guid)
}

/// Find a chat by its database ID.
pub fn find_chat_by_id(conn: &Connection, id: i64) -> BbResult<Option<Chat>> {
    Chat::find_by_id(conn, id)
}

/// Get the total count of chats.
pub fn count_chats(conn: &Connection) -> BbResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM chats", [], |row| row.get(0))
        .map_err(|e| BbError::Database(e.to_string()))
}

/// Get unread message count for a chat.
pub fn unread_count_for_chat(conn: &Connection, chat_id: i64) -> BbResult<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM messages
         WHERE chat_id = ?1 AND date_read IS NULL AND is_from_me = 0 AND date_deleted IS NULL",
        [chat_id],
        |row| row.get(0),
    )
    .map_err(|e| BbError::Database(e.to_string()))
}

/// Get total unread count across all chats.
pub fn total_unread_count(conn: &Connection) -> BbResult<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM messages
         WHERE date_read IS NULL AND is_from_me = 0 AND date_deleted IS NULL",
        [],
        |row| row.get(0),
    )
    .map_err(|e| BbError::Database(e.to_string()))
}

/// Load participants (handles) for a chat.
pub fn load_chat_participants(conn: &Connection, chat_id: i64) -> BbResult<Vec<Handle>> {
    let mut stmt = conn
        .prepare(
            "SELECT h.* FROM handles h
             INNER JOIN chat_handle_join chj ON chj.handle_id = h.id
             WHERE chj.chat_id = ?1
             ORDER BY h.address",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let handles = stmt
        .query_map([chat_id], Handle::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(handles)
}

/// Search chats by display name or chat identifier.
pub fn search_chats(conn: &Connection, query: &str, limit: i64) -> BbResult<Vec<Chat>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT * FROM chats
             WHERE display_name LIKE ?1 OR chat_identifier LIKE ?1
             ORDER BY latest_message_date DESC
             LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let chats = stmt
        .query_map(params![pattern, limit], Chat::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(chats)
}

// ─── Message Queries ────────────────────────────────────────────────────────

/// Cursor-based paginated message loading for a chat.
///
/// Uses keyset pagination on `date_created` instead of OFFSET, which gives
/// O(1) performance regardless of how deep into the conversation we are.
///
/// - `cursor`: The date_created value of the last loaded message (exclusive boundary).
///   Pass `None` for the initial load (gets the newest messages).
/// - `limit`: Maximum number of messages to return.
/// - `direction`: Desc = newer messages first (default chat view), Asc = older first.
///
/// Returns messages sorted by date_created in the requested direction.
pub fn messages_for_chat_cursor(
    conn: &Connection,
    chat_id: i64,
    cursor: Option<&str>,
    limit: i64,
    direction: SortDirection,
) -> BbResult<Vec<Message>> {
    let (comparator, order) = match direction {
        SortDirection::Desc => ("<", "DESC"),
        SortDirection::Asc => (">", "ASC"),
    };

    let (sql, use_cursor) = match cursor {
        Some(_) => (
            format!(
                "SELECT * FROM messages WHERE chat_id = ?1 AND date_created {comparator} ?2 AND date_deleted IS NULL
                 ORDER BY date_created {order} LIMIT ?3"
            ),
            true,
        ),
        None => (
            format!(
                "SELECT * FROM messages WHERE chat_id = ?1 AND date_deleted IS NULL
                 ORDER BY date_created {order} LIMIT ?2"
            ),
            false,
        ),
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| BbError::Database(e.to_string()))?;

    let messages = if use_cursor {
        stmt.query_map(
            params![chat_id, cursor.unwrap_or(""), limit],
            Message::from_row,
        )
    } else {
        stmt.query_map(params![chat_id, limit], Message::from_row)
    }
    .map_err(|e| BbError::Database(e.to_string()))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(messages)
}

/// List messages for a chat with offset-based pagination (legacy compatibility).
pub fn list_messages_for_chat(
    conn: &Connection,
    chat_id: i64,
    offset: i64,
    limit: i64,
    direction: SortDirection,
) -> BbResult<Vec<Message>> {
    let sql = format!(
        "SELECT * FROM messages WHERE chat_id = ?1 AND date_deleted IS NULL
         ORDER BY date_created {} LIMIT ?2 OFFSET ?3",
        direction.as_sql()
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| BbError::Database(e.to_string()))?;
    let messages = stmt
        .query_map(params![chat_id, limit, offset], Message::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

/// Search around a specific date (bidirectional pagination).
///
/// Returns messages both before and after the given timestamp, centered on it.
pub fn messages_around_date(
    conn: &Connection,
    chat_id: i64,
    date: &str,
    count_per_side: i64,
) -> BbResult<Vec<Message>> {
    // Messages before (inclusive of the target date)
    let mut before: Vec<Message> = {
        let mut stmt = conn
            .prepare(
                "SELECT * FROM messages WHERE chat_id = ?1 AND date_created <= ?2 AND date_deleted IS NULL
                 ORDER BY date_created DESC LIMIT ?3",
            )
            .map_err(|e| BbError::Database(e.to_string()))?;
        let rows = stmt.query_map(params![chat_id, date, count_per_side], Message::from_row)
            .map_err(|e| BbError::Database(e.to_string()))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    before.reverse(); // Oldest first

    // Messages after
    let after: Vec<Message> = {
        let mut stmt = conn
            .prepare(
                "SELECT * FROM messages WHERE chat_id = ?1 AND date_created > ?2 AND date_deleted IS NULL
                 ORDER BY date_created ASC LIMIT ?3",
            )
            .map_err(|e| BbError::Database(e.to_string()))?;
        let rows = stmt.query_map(params![chat_id, date, count_per_side], Message::from_row)
            .map_err(|e| BbError::Database(e.to_string()))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    before.extend(after);
    Ok(before)
}

/// Find a message by its GUID.
pub fn find_message_by_guid(conn: &Connection, guid: &str) -> BbResult<Option<Message>> {
    Message::find_by_guid(conn, guid)
}

/// Get the total count of messages in a chat.
pub fn count_messages_for_chat(conn: &Connection, chat_id: i64) -> BbResult<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE chat_id = ?1",
        [chat_id],
        |row| row.get(0),
    )
    .map_err(|e| BbError::Database(e.to_string()))
}

/// Get the latest message for a chat.
pub fn latest_message_for_chat(conn: &Connection, chat_id: i64) -> BbResult<Option<Message>> {
    match conn.query_row(
        "SELECT * FROM messages WHERE chat_id = ?1 AND date_deleted IS NULL ORDER BY date_created DESC LIMIT 1",
        [chat_id],
        Message::from_row,
    ) {
        Ok(msg) => Ok(Some(msg)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(BbError::Database(e.to_string())),
    }
}

/// Load reactions (associated messages) for a message.
pub fn load_reactions_for_message(conn: &Connection, message_guid: &str) -> BbResult<Vec<Message>> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM messages WHERE associated_message_guid = ?1
             ORDER BY date_created ASC",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let messages = stmt
        .query_map([message_guid], Message::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

/// Load thread replies for a thread originator GUID.
pub fn load_thread_replies(conn: &Connection, originator_guid: &str) -> BbResult<Vec<Message>> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM messages WHERE thread_originator_guid = ?1
             ORDER BY date_created ASC",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let messages = stmt
        .query_map([originator_guid], Message::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

/// Search messages by text content.
pub fn search_messages(conn: &Connection, query: &str, limit: i64) -> BbResult<Vec<Message>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT * FROM messages WHERE (text LIKE ?1 OR subject LIKE ?1) AND date_deleted IS NULL
             ORDER BY date_created DESC LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let messages = stmt
        .query_map(params![pattern, limit], Message::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

/// Search messages within a specific chat.
pub fn search_messages_in_chat(
    conn: &Connection,
    chat_id: i64,
    query: &str,
    limit: i64,
) -> BbResult<Vec<Message>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT * FROM messages WHERE chat_id = ?1 AND (text LIKE ?2 OR subject LIKE ?2) AND date_deleted IS NULL
             ORDER BY date_created DESC LIMIT ?3",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let messages = stmt
        .query_map(params![chat_id, pattern, limit], Message::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

/// Get messages created after a given timestamp (for incremental sync).
pub fn messages_after(conn: &Connection, after_date: &str, limit: i64) -> BbResult<Vec<Message>> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM messages WHERE date_created > ?1
             ORDER BY date_created ASC LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let messages = stmt
        .query_map(params![after_date, limit], Message::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

/// Get bookmarked messages.
pub fn bookmarked_messages(conn: &Connection, limit: i64) -> BbResult<Vec<Message>> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM messages WHERE is_bookmarked = 1 AND date_deleted IS NULL
             ORDER BY date_created DESC LIMIT ?1",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let messages = stmt
        .query_map([limit], Message::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

// ─── Handle Queries ─────────────────────────────────────────────────────────

/// Find a handle by address and service.
pub fn find_handle(conn: &Connection, address: &str, service: &str) -> BbResult<Option<Handle>> {
    Handle::find_by_address(conn, address, service)
}

/// Find a handle by its database ID.
pub fn find_handle_by_id(conn: &Connection, id: i64) -> BbResult<Option<Handle>> {
    Handle::find_by_id(conn, id)
}

/// List all handles.
pub fn list_handles(conn: &Connection) -> BbResult<Vec<Handle>> {
    let mut stmt = conn
        .prepare("SELECT * FROM handles ORDER BY address")
        .map_err(|e| BbError::Database(e.to_string()))?;

    let handles = stmt
        .query_map([], Handle::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(handles)
}

/// Search handles by address (partial match).
pub fn search_handles(conn: &Connection, query: &str, limit: i64) -> BbResult<Vec<Handle>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT * FROM handles WHERE address LIKE ?1 OR formatted_address LIKE ?1
             ORDER BY address LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let handles = stmt
        .query_map(params![pattern, limit], Handle::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(handles)
}

// ─── Attachment Queries ─────────────────────────────────────────────────────

/// Load attachments for a message.
pub fn load_attachments_for_message(conn: &Connection, message_id: i64) -> BbResult<Vec<Attachment>> {
    let mut stmt = conn
        .prepare("SELECT * FROM attachments WHERE message_id = ?1 ORDER BY id")
        .map_err(|e| BbError::Database(e.to_string()))?;

    let attachments = stmt
        .query_map([message_id], Attachment::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(attachments)
}

/// Find an attachment by GUID.
pub fn find_attachment_by_guid(conn: &Connection, guid: &str) -> BbResult<Option<Attachment>> {
    Attachment::find_by_guid(conn, guid)
}

/// Load all attachments for a chat (via message join).
pub fn load_attachments_for_chat(
    conn: &Connection,
    chat_id: i64,
    limit: i64,
) -> BbResult<Vec<Attachment>> {
    let mut stmt = conn
        .prepare(
            "SELECT a.* FROM attachments a
             INNER JOIN messages m ON a.message_id = m.id
             WHERE m.chat_id = ?1 AND a.mime_type IS NOT NULL
             ORDER BY m.date_created DESC
             LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let attachments = stmt
        .query_map(params![chat_id, limit], Attachment::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(attachments)
}

/// Load attachments filtered by MIME type prefix (e.g., "image/", "video/").
pub fn load_attachments_by_mime(
    conn: &Connection,
    mime_prefix: &str,
    limit: i64,
) -> BbResult<Vec<Attachment>> {
    let pattern = format!("{mime_prefix}%");
    let mut stmt = conn
        .prepare(
            "SELECT * FROM attachments WHERE mime_type LIKE ?1
             ORDER BY id DESC LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let attachments = stmt
        .query_map(params![pattern, limit], Attachment::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(attachments)
}

/// Load attachments for a chat filtered by MIME type prefix.
pub fn load_chat_attachments_by_mime(
    conn: &Connection,
    chat_id: i64,
    mime_prefix: &str,
    limit: i64,
) -> BbResult<Vec<Attachment>> {
    let pattern = format!("{mime_prefix}%");
    let mut stmt = conn
        .prepare(
            "SELECT a.* FROM attachments a
             INNER JOIN messages m ON a.message_id = m.id
             WHERE m.chat_id = ?1 AND a.mime_type LIKE ?2
             ORDER BY m.date_created DESC
             LIMIT ?3",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let attachments = stmt
        .query_map(params![chat_id, pattern, limit], Attachment::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(attachments)
}

// ─── Contact Queries ────────────────────────────────────────────────────────

/// Search contacts by display name (fuzzy LIKE match).
pub fn search_contacts(conn: &Connection, query: &str, limit: i64) -> BbResult<Vec<Contact>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT * FROM contacts WHERE display_name LIKE ?1
             ORDER BY display_name LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let contacts = stmt
        .query_map(params![pattern, limit], Contact::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(contacts)
}

/// Search contacts by phone number suffix (last N digits).
///
/// This is the key pattern from doc 03: phone numbers are matched by their
/// trailing digits to handle country codes and formatting differences.
pub fn search_contacts_by_phone_suffix(
    conn: &Connection,
    suffix: &str,
    limit: i64,
) -> BbResult<Vec<Contact>> {
    // Strip non-digit characters from the suffix for comparison
    let digits: String = suffix.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Ok(vec![]);
    }
    let pattern = format!("%{digits}%");

    let mut stmt = conn
        .prepare(
            "SELECT * FROM contacts WHERE phones LIKE ?1
             ORDER BY display_name LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let contacts = stmt
        .query_map(params![pattern, limit], Contact::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(contacts)
}

/// Search contacts by email.
pub fn search_contacts_by_email(
    conn: &Connection,
    email: &str,
    limit: i64,
) -> BbResult<Vec<Contact>> {
    let pattern = format!("%{email}%");
    let mut stmt = conn
        .prepare(
            "SELECT * FROM contacts WHERE emails LIKE ?1
             ORDER BY display_name LIMIT ?2",
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

    let contacts = stmt
        .query_map(params![pattern, limit], Contact::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(contacts)
}

/// Load all contacts.
pub fn list_contacts(conn: &Connection) -> BbResult<Vec<Contact>> {
    let mut stmt = conn
        .prepare("SELECT * FROM contacts ORDER BY display_name")
        .map_err(|e| BbError::Database(e.to_string()))?;

    let contacts = stmt
        .query_map([], Contact::from_row)
        .map_err(|e| BbError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(contacts)
}

/// Find a contact by external ID.
pub fn find_contact_by_external_id(conn: &Connection, external_id: &str) -> BbResult<Option<Contact>> {
    Contact::find_by_external_id(conn, external_id)
}

/// Delete all contacts (used during full contact sync).
pub fn delete_all_contacts(conn: &Connection) -> BbResult<usize> {
    conn.execute("DELETE FROM contacts", [])
        .map_err(|e| BbError::Database(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;
    use crate::migrations;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::create_tables(&conn).unwrap();
        migrations::run_migrations(&conn).unwrap();
        conn
    }

    fn insert_chat(conn: &Connection, guid: &str) -> i64 {
        let mut chat = Chat::from_server_map(&serde_json::json!({"guid": guid})).unwrap();
        chat.save(conn).unwrap()
    }

    fn insert_message(conn: &Connection, guid: &str, chat_id: i64, date: &str, from_me: bool) -> i64 {
        let mut msg = Message::from_server_map(&serde_json::json!({
            "guid": guid,
            "dateCreated": date,
            "isFromMe": from_me,
        })).unwrap();
        msg.chat_id = Some(chat_id);
        msg.save(conn).unwrap()
    }

    #[test]
    fn test_list_chats_empty() {
        let conn = setup_db();
        let chats = list_chats(&conn, 0, 10, false).unwrap();
        assert!(chats.is_empty());
    }

    #[test]
    fn test_count_chats() {
        let conn = setup_db();
        assert_eq!(count_chats(&conn).unwrap(), 0);
    }

    #[test]
    fn test_search_messages_empty() {
        let conn = setup_db();
        let results = search_messages(&conn, "hello", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_nonexistent_chat() {
        let conn = setup_db();
        assert!(find_chat_by_guid(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_list_contacts_empty() {
        let conn = setup_db();
        let contacts = list_contacts(&conn).unwrap();
        assert!(contacts.is_empty());
    }

    #[test]
    fn test_cursor_based_pagination() {
        let conn = setup_db();
        let chat_id = insert_chat(&conn, "chat-1");

        // Insert messages with sequential dates
        for i in 1..=10 {
            insert_message(&conn, &format!("msg-{i}"), chat_id, &format!("2024-01-{i:02}T00:00:00Z"), true);
        }

        // First page (newest first, no cursor)
        let page1 = messages_for_chat_cursor(&conn, chat_id, None, 3, SortDirection::Desc).unwrap();
        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].guid.as_deref(), Some("msg-10"));

        // Second page (use the last date from page1 as cursor)
        let cursor = page1.last().unwrap().date_created.as_deref();
        let page2 = messages_for_chat_cursor(&conn, chat_id, cursor, 3, SortDirection::Desc).unwrap();
        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].guid.as_deref(), Some("msg-7"));
    }

    #[test]
    fn test_messages_around_date() {
        let conn = setup_db();
        let chat_id = insert_chat(&conn, "chat-1");

        for i in 1..=10 {
            insert_message(&conn, &format!("msg-{i}"), chat_id, &format!("2024-01-{i:02}T00:00:00Z"), true);
        }

        let result = messages_around_date(&conn, chat_id, "2024-01-05T00:00:00Z", 3).unwrap();
        assert!(result.len() >= 3); // At least 3 messages around the target
    }

    #[test]
    fn test_unread_count() {
        let conn = setup_db();
        let chat_id = insert_chat(&conn, "chat-1");

        // Insert a received message (not from me, no date_read)
        insert_message(&conn, "msg-1", chat_id, "2024-01-01T00:00:00Z", false);
        // Insert a sent message
        insert_message(&conn, "msg-2", chat_id, "2024-01-02T00:00:00Z", true);

        assert_eq!(unread_count_for_chat(&conn, chat_id).unwrap(), 1);
    }

    #[test]
    fn test_search_messages_in_chat() {
        let conn = setup_db();
        let chat_id = insert_chat(&conn, "chat-1");

        let mut msg = Message::from_server_map(&serde_json::json!({
            "guid": "msg-1",
            "text": "hello world",
            "dateCreated": "2024-01-01T00:00:00Z",
        })).unwrap();
        msg.chat_id = Some(chat_id);
        msg.save(&conn).unwrap();

        let results = search_messages_in_chat(&conn, chat_id, "hello", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_attachment_queries() {
        let conn = setup_db();
        let chat_id = insert_chat(&conn, "chat-1");
        let msg_id = insert_message(&conn, "msg-1", chat_id, "2024-01-01T00:00:00Z", true);

        let mut att = Attachment::from_server_map(&serde_json::json!({
            "guid": "att-1",
            "mimeType": "image/jpeg",
            "transferName": "photo.jpg"
        })).unwrap();
        att.message_id = Some(msg_id);
        att.save(&conn).unwrap();

        // By message
        let atts = load_attachments_for_message(&conn, msg_id).unwrap();
        assert_eq!(atts.len(), 1);

        // By chat
        let atts = load_attachments_for_chat(&conn, chat_id, 10).unwrap();
        assert_eq!(atts.len(), 1);

        // By MIME type
        let atts = load_attachments_by_mime(&conn, "image/", 10).unwrap();
        assert_eq!(atts.len(), 1);

        // By chat + MIME type
        let atts = load_chat_attachments_by_mime(&conn, chat_id, "video/", 10).unwrap();
        assert_eq!(atts.len(), 0);
    }

    #[test]
    fn test_contact_phone_suffix_search() {
        let conn = setup_db();
        let mut contact = Contact::from_server_map(&serde_json::json!({
            "id": "c1",
            "displayName": "Test User",
            "phoneNumbers": ["+15551234567"],
            "emails": ["test@example.com"]
        })).unwrap();
        contact.save(&conn).unwrap();

        // Search by last 4 digits
        let results = search_contacts_by_phone_suffix(&conn, "4567", 10).unwrap();
        assert_eq!(results.len(), 1);

        // Search by full number
        let results = search_contacts_by_phone_suffix(&conn, "+1 (555) 123-4567", 10).unwrap();
        assert_eq!(results.len(), 1);

        // Search miss
        let results = search_contacts_by_phone_suffix(&conn, "9999", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_contact_email_search() {
        let conn = setup_db();
        let mut contact = Contact::from_server_map(&serde_json::json!({
            "id": "c1",
            "displayName": "Test User",
            "emails": ["test@example.com"]
        })).unwrap();
        contact.save(&conn).unwrap();

        let results = search_contacts_by_email(&conn, "test@example", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_chat_with_details() {
        let conn = setup_db();
        let chat_id = insert_chat(&conn, "chat-1");
        insert_message(&conn, "msg-1", chat_id, "2024-01-01T00:00:00Z", false);

        let details = list_chats_with_details(&conn, 0, 10, true).unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].unread_count, 1);
    }

    #[test]
    fn test_bookmarked_messages() {
        let conn = setup_db();
        let chat_id = insert_chat(&conn, "chat-1");

        let mut msg = Message::from_server_map(&serde_json::json!({
            "guid": "msg-1",
            "text": "bookmarked",
            "dateCreated": "2024-01-01T00:00:00Z",
            "isBookmarked": true,
        })).unwrap();
        msg.chat_id = Some(chat_id);
        msg.save(&conn).unwrap();

        let results = bookmarked_messages(&conn, 10).unwrap();
        assert_eq!(results.len(), 1);
    }
}
