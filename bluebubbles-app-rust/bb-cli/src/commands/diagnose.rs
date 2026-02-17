//! Diagnostic commands for troubleshooting avatar syncing and missing chats.
//!
//! Provides CLI commands to inspect the contact avatar pipeline, detect
//! missing conversations between server and local DB, drill into
//! individual chat discrepancies, and verify contact name resolution.

use std::collections::HashMap;
use clap::Subcommand;
use console::style;

use bb_api::endpoints::chats::ChatQuery;
use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use bb_models::Contact;
use bb_models::models::contact::normalize_address;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum DiagnoseAction {
    /// Test the contact avatar pipeline: fetch contacts with avatars from
    /// the server, inspect avatar data format, and compare with local DB.
    Avatars,
    /// Compare server chats with local DB to find missing conversations
    /// and chats with NULL latest_message_date.
    Chats,
    /// Inspect a specific chat by GUID: compare server vs local DB fields
    /// and fetch its recent messages.
    Chat {
        /// The chat GUID to inspect (e.g. "iMessage;-;+15551234567").
        guid: String,
    },
    /// Test contact name resolution for all handles: simulates the
    /// matching logic from batch_load_participants_with_contacts to find
    /// handles that resolve (or fail to resolve) to a contact name.
    Contacts,
}

pub async fn run(
    config: ConfigHandle,
    action: DiagnoseAction,
    _format: OutputFormat,
) -> BbResult<()> {
    match action {
        DiagnoseAction::Avatars => run_avatars(config).await,
        DiagnoseAction::Chats => run_chats(config).await,
        DiagnoseAction::Chat { guid } => run_chat(config, &guid).await,
        DiagnoseAction::Contacts => run_contacts(config).await,
    }
}

// ─── Avatars ────────────────────────────────────────────────────────────────

async fn run_avatars(config: ConfigHandle) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;
    let db = super::init_database(&config).await?;

    println!("{}", style("Diagnose: Contact Avatars").bold().underlined());
    println!();

    // 1. Fetch contacts WITH avatars from the server
    print!("  Fetching contacts from server (with avatars)... ");
    let server_contacts = api.get_contacts(true).await?;
    println!("{}", style("done").green());
    println!("  Server returned {} contact(s)", server_contacts.len());

    // 2. Count how many have non-null avatar field
    let with_avatar: Vec<_> = server_contacts
        .iter()
        .filter(|c| {
            c.get("avatar")
                .map(|v| !v.is_null() && v.as_str().map_or(false, |s| !s.is_empty()))
                .unwrap_or(false)
        })
        .collect();

    let without_avatar = server_contacts.len() - with_avatar.len();
    println!(
        "  Contacts with avatar data: {}",
        style(with_avatar.len()).cyan()
    );
    println!(
        "  Contacts without avatar:   {}",
        style(without_avatar).dim()
    );
    println!();

    // 3. Inspect each contact with avatar data
    if !with_avatar.is_empty() {
        println!(
            "{}",
            style("  Avatar details (server data):").bold()
        );
        println!();

        for (i, cj) in with_avatar.iter().enumerate() {
            let display_name = cj
                .get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or("(no name)");
            let phones: Vec<String> = cj
                .get("phoneNumbers")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            v.as_str()
                                .map(String::from)
                                .or_else(|| v.get("address").and_then(|a| a.as_str()).map(String::from))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let emails: Vec<String> = cj
                .get("emails")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            v.as_str()
                                .map(String::from)
                                .or_else(|| v.get("address").and_then(|a| a.as_str()).map(String::from))
                        })
                        .collect()
                })
                .unwrap_or_default();

            let avatar_str = cj
                .get("avatar")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let avatar_len = avatar_str.len();
            let preview: String = avatar_str.chars().take(20).collect();

            let format_hint = if avatar_str.starts_with("data:image/") {
                "data URI"
            } else if avatar_str.starts_with('/') || avatar_str.starts_with("iVBOR") || avatar_str.starts_with("/9j/") {
                "raw base64"
            } else {
                "unknown"
            };

            println!("  [{}] {}", i + 1, style(display_name).bold());
            println!("      Phones: {}", if phones.is_empty() { "-".to_string() } else { phones.join(", ") });
            println!("      Emails: {}", if emails.is_empty() { "-".to_string() } else { emails.join(", ") });
            println!(
                "      Avatar: {} bytes, format={}, preview=\"{}...\"",
                style(avatar_len).cyan(),
                style(format_hint).yellow(),
                preview
            );

            // 4. Try parsing via Contact::from_server_map
            match Contact::from_server_map(cj) {
                Ok(parsed) => {
                    let has = parsed.has_avatar();
                    let decoded_len = parsed.avatar.as_ref().map(|a| a.len()).unwrap_or(0);
                    println!(
                        "      Parse:  {} (has_avatar={}, decoded {} bytes)",
                        style("OK").green().bold(),
                        has,
                        decoded_len
                    );
                }
                Err(e) => {
                    println!(
                        "      Parse:  {} - {}",
                        style("FAILED").red().bold(),
                        e
                    );
                }
            }
            println!();
        }
    }

    // 5. Compare with local DB
    println!("{}", style("  Local DB comparison:").bold());
    let conn = db.conn()?;
    let local_contacts = bb_models::queries::list_contacts(&conn)?;
    let local_with_avatar = local_contacts.iter().filter(|c| c.has_avatar()).count();

    println!("  Local contacts total:       {}", local_contacts.len());
    println!("  Local contacts with avatar: {}", local_with_avatar);
    println!(
        "  Server contacts with avatar: {}",
        with_avatar.len()
    );

    if local_with_avatar < with_avatar.len() {
        println!(
            "  {} {} avatar(s) on server are missing from local DB.",
            style("WARNING").yellow().bold(),
            with_avatar.len() - local_with_avatar
        );
    } else if local_with_avatar == with_avatar.len() {
        println!(
            "  {} Avatar counts match.",
            style("OK").green().bold()
        );
    } else {
        println!(
            "  {} Local DB has more avatars ({}) than server ({}). Possibly stale data.",
            style("INFO").blue().bold(),
            local_with_avatar,
            with_avatar.len()
        );
    }

    // Check for contacts that exist on server but not locally
    let mut missing_locally = 0;
    for cj in &server_contacts {
        if let Some(ext_id) = cj.get("id").and_then(|v| v.as_str()) {
            if !local_contacts.iter().any(|lc| lc.external_id.as_deref() == Some(ext_id)) {
                missing_locally += 1;
            }
        }
    }
    if missing_locally > 0 {
        println!(
            "  {} {} contact(s) exist on server but not in local DB. Run `bluebubbles contacts sync`.",
            style("WARNING").yellow().bold(),
            missing_locally
        );
    }

    println!();
    println!("  Diagnosis complete.");

    Ok(())
}

// ─── Chats ──────────────────────────────────────────────────────────────────

async fn run_chats(config: ConfigHandle) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;
    let db = super::init_database(&config).await?;

    println!("{}", style("Diagnose: Chat Sync").bold().underlined());
    println!();

    // 1. Fetch all chats from server (paginated in batches of 1000)
    print!("  Fetching chats from server... ");
    let mut all_server_chats: Vec<serde_json::Value> = Vec::new();
    let mut offset: i64 = 0;
    let batch_size: i64 = 1000;

    loop {
        let query = ChatQuery {
            with: vec!["participants".into()],
            offset,
            limit: batch_size,
            sort: Some("lastmessage".into()),
        };
        let batch = api.query_chats(&query).await?;
        let count = batch.len();
        all_server_chats.extend(batch);
        if (count as i64) < batch_size {
            break;
        }
        offset += batch_size;
    }
    println!("{}", style("done").green());
    println!(
        "  Server chats: {}",
        style(all_server_chats.len()).cyan()
    );

    // 2. Load all local chats
    let conn = db.conn()?;
    let local_count = bb_models::queries::count_chats(&conn)?;
    let local_chats = bb_models::queries::list_chats(&conn, 0, local_count + 1000, true)?;
    println!(
        "  Local DB chats: {}",
        style(local_chats.len()).cyan()
    );
    println!();

    // 3. Build GUID sets for comparison
    let local_guids: std::collections::HashSet<String> = local_chats
        .iter()
        .map(|c| c.guid.clone())
        .collect();

    let server_guids: std::collections::HashSet<String> = all_server_chats
        .iter()
        .filter_map(|c| c.get("guid").and_then(|v| v.as_str()).map(String::from))
        .collect();

    // 4. Chats on server but NOT in local DB
    let missing_locally: Vec<_> = all_server_chats
        .iter()
        .filter(|c| {
            c.get("guid")
                .and_then(|v| v.as_str())
                .map(|g| !local_guids.contains(g))
                .unwrap_or(false)
        })
        .collect();

    if missing_locally.is_empty() {
        println!(
            "  {} No chats are missing from local DB.",
            style("OK").green().bold()
        );
    } else {
        println!(
            "  {} {} chat(s) exist on server but NOT in local DB:",
            style("WARNING").yellow().bold(),
            missing_locally.len()
        );
        for c in &missing_locally {
            let guid = c.get("guid").and_then(|v| v.as_str()).unwrap_or("?");
            let display = c
                .get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let identifier = c
                .get("chatIdentifier")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let participant_count = c
                .get("participants")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            println!(
                "    - {} (identifier={}, display=\"{}\", participants={})",
                style(guid).yellow(),
                identifier,
                display,
                participant_count
            );
        }
    }

    // 5. Chats in local DB but NOT on server
    let only_local: Vec<_> = local_chats
        .iter()
        .filter(|c| !server_guids.contains(&c.guid))
        .collect();

    if !only_local.is_empty() {
        println!();
        println!(
            "  {} {} chat(s) in local DB but NOT on server (possibly deleted):",
            style("INFO").blue().bold(),
            only_local.len()
        );
        for c in only_local.iter().take(20) {
            println!(
                "    - {} (identifier={}, title=\"{}\")",
                style(&c.guid).dim(),
                c.chat_identifier.as_deref().unwrap_or(""),
                c.title()
            );
        }
        if only_local.len() > 20 {
            println!("    ... and {} more", only_local.len() - 20);
        }
    }

    // 6. Chats with NULL latest_message_date in local DB
    let null_date: Vec<_> = local_chats
        .iter()
        .filter(|c| c.latest_message_date.is_none())
        .collect();

    println!();
    if null_date.is_empty() {
        println!(
            "  {} All local chats have a latest_message_date.",
            style("OK").green().bold()
        );
    } else {
        println!(
            "  {} {} local chat(s) have NULL latest_message_date:",
            style("WARNING").yellow().bold(),
            null_date.len()
        );
        for c in null_date.iter().take(20) {
            println!(
                "    - {} (identifier={}, title=\"{}\")",
                style(&c.guid).yellow(),
                c.chat_identifier.as_deref().unwrap_or(""),
                c.title()
            );
        }
        if null_date.len() > 20 {
            println!("    ... and {} more", null_date.len() - 20);
        }
    }

    // 7. Summary
    println!();
    println!("{}", style("  Summary:").bold());
    println!("    Server chats:              {}", all_server_chats.len());
    println!("    Local DB chats:            {}", local_chats.len());
    println!("    Missing from local DB:     {}", missing_locally.len());
    println!("    Only in local DB:          {}", only_local.len());
    println!("    NULL latest_message_date:  {}", null_date.len());

    println!();
    println!("  Diagnosis complete.");

    Ok(())
}

// ─── Single Chat ────────────────────────────────────────────────────────────

async fn run_chat(config: ConfigHandle, guid: &str) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;
    let db = super::init_database(&config).await?;

    println!(
        "{} {}",
        style("Diagnose: Chat").bold().underlined(),
        style(guid).cyan()
    );
    println!();

    // 1. Local DB lookup
    println!("{}", style("  Local DB:").bold());
    let conn = db.conn()?;
    match bb_models::queries::find_chat_by_guid(&conn, guid)? {
        Some(local) => {
            println!("    GUID:                {}", local.guid);
            println!(
                "    Identifier:          {}",
                local.chat_identifier.as_deref().unwrap_or("-")
            );
            println!("    Title:               {}", local.title());
            println!(
                "    Display name:        {}",
                local.display_name.as_deref().unwrap_or("-")
            );
            println!("    Archived:            {}", local.is_archived);
            println!("    Pinned:              {}", local.is_pinned);
            println!("    Has unread:          {}", local.has_unread_message);
            println!(
                "    Latest msg date:     {}",
                local
                    .latest_message_date
                    .as_deref()
                    .unwrap_or("NULL")
            );
            println!(
                "    Style:               {}",
                local.style.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            );
            println!(
                "    Date deleted:        {}",
                local.date_deleted.as_deref().unwrap_or("-")
            );
            println!(
                "    Last read msg GUID:  {}",
                local.last_read_message_guid.as_deref().unwrap_or("-")
            );
        }
        None => {
            println!(
                "    {} Chat not found in local DB!",
                style("NOT FOUND").red().bold()
            );
        }
    }

    // 2. Server lookup
    println!();
    println!("{}", style("  Server:").bold());
    match api.get_chat(guid, &["participants", "lastmessage"]).await {
        Ok(server_chat) => {
            let s_identifier = server_chat
                .get("chatIdentifier")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let s_display = server_chat
                .get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let s_archived = server_chat
                .get("isArchived")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let s_participants = server_chat
                .get("participants")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let s_style = server_chat
                .get("style")
                .and_then(|v| v.as_i64());
            let s_last_msg = server_chat.get("lastMessage");

            println!("    Identifier:          {}", s_identifier);
            println!("    Display name:        {}", s_display);
            println!("    Archived:            {}", s_archived);
            println!("    Participants:        {}", s_participants);
            println!(
                "    Style:               {}",
                s_style.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            );

            if let Some(lm) = s_last_msg {
                let lm_guid = lm.get("guid").and_then(|v| v.as_str()).unwrap_or("-");
                let lm_text = lm.get("text").and_then(|v| v.as_str()).unwrap_or("-");
                let lm_date = lm
                    .get("dateCreated")
                    .and_then(|v| v.as_i64())
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let lm_from_me = lm
                    .get("isFromMe")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                println!();
                println!("{}", style("    Last message (server):").bold());
                println!("      GUID:       {}", lm_guid);
                println!(
                    "      Text:       {}",
                    super::truncate(lm_text, 80)
                );
                println!("      Date:       {}", lm_date);
                println!("      From me:    {}", lm_from_me);
            } else {
                println!("    Last message:        -");
            }

            // List participants
            if let Some(participants) = server_chat.get("participants").and_then(|v| v.as_array()) {
                if !participants.is_empty() {
                    println!();
                    println!("{}", style("    Participants:").bold());
                    for p in participants {
                        let addr = p.get("address").and_then(|v| v.as_str()).unwrap_or("?");
                        let service = p.get("service").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("      - {} ({})", addr, service);
                    }
                }
            }
        }
        Err(e) => {
            println!(
                "    {} Could not fetch from server: {}",
                style("ERROR").red().bold(),
                e
            );
        }
    }

    // 3. Fetch recent messages for this chat from the server
    println!();
    println!("{}", style("  Recent messages (server, last 5):").bold());
    match api
        .get_chat_messages(guid, 0, 5, "DESC", &["attachment"], None, None)
        .await
    {
        Ok(messages) => {
            if messages.is_empty() {
                println!("    (no messages)");
            } else {
                for (i, m) in messages.iter().enumerate() {
                    let m_guid = m.get("guid").and_then(|v| v.as_str()).unwrap_or("?");
                    let m_text = m.get("text").and_then(|v| v.as_str()).unwrap_or("-");
                    let m_date = m
                        .get("dateCreated")
                        .and_then(|v| v.as_i64())
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    let m_from_me = m
                        .get("isFromMe")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let m_has_attach = m
                        .get("hasAttachments")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    println!("    [{}] {}", i + 1, style(m_guid).dim());
                    println!(
                        "        Text: {}",
                        super::truncate(m_text, 60)
                    );
                    println!("        Date: {}  From me: {}  Attachments: {}", m_date, m_from_me, m_has_attach);
                }
            }
        }
        Err(e) => {
            println!(
                "    {} Could not fetch messages: {}",
                style("ERROR").red().bold(),
                e
            );
        }
    }

    println!();
    println!("  Diagnosis complete.");

    Ok(())
}

// ─── Contacts (name resolution) ─────────────────────────────────────────────

async fn run_contacts(config: ConfigHandle) -> BbResult<()> {
    let db = super::init_database(&config).await?;

    println!("{}", style("Diagnose: Contact Name Resolution").bold().underlined());
    println!();

    let conn = db.conn()?;

    // 1. Load all handles and contacts
    let all_handles = bb_models::queries::list_handles(&conn)?;
    let all_contacts = bb_models::queries::list_contacts(&conn)?;

    println!("  Handles in DB:  {}", style(all_handles.len()).cyan());
    println!("  Contacts in DB: {}", style(all_contacts.len()).cyan());
    println!();

    // 2. Build the same contact_by_address lookup map used in queries.rs
    let mut contact_by_address: HashMap<String, &Contact> = HashMap::new();

    for contact in &all_contacts {
        // Index by normalized phone numbers AND digits-only variants
        for phone in contact.phone_list() {
            let normalized = normalize_address(&phone);
            contact_by_address.insert(normalized.clone(), contact);
            // Also index by digits-only (no '+') for cross-format matching
            let digits_only: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
            if !digits_only.is_empty() && digits_only != normalized {
                contact_by_address.insert(digits_only, contact);
            }
        }
        // Index by lowercase trimmed email
        for email in contact.email_list() {
            let lower = email.trim().to_lowercase();
            contact_by_address.insert(lower.clone(), contact);
            if let Some(stripped) = lower.strip_prefix("mailto:") {
                contact_by_address.insert(stripped.to_string(), contact);
            }
        }
    }

    println!(
        "  Contact lookup index entries: {}",
        style(contact_by_address.len()).cyan()
    );
    println!();

    // 3. Categorize handles
    let mut resolved_via_contact_id: Vec<&bb_models::models::handle::Handle> = Vec::new();
    let mut resolved_via_address: Vec<(&bb_models::models::handle::Handle, String, &str)> = Vec::new();
    let mut unresolved: Vec<&bb_models::models::handle::Handle> = Vec::new();

    for handle in &all_handles {
        // Check if handle already has a contact_id link
        if handle.contact_id.is_some() {
            resolved_via_contact_id.push(handle);
            continue;
        }

        let addr = handle.address.trim();
        let is_email = addr.contains('@');

        let matched: Option<(&Contact, &str)> = if is_email {
            let lower = addr.to_lowercase();
            contact_by_address.get(lower.as_str())
                .map(|c| (*c, "email-direct"))
                .or_else(|| {
                    let stripped = lower.strip_prefix("mailto:").unwrap_or(&lower);
                    contact_by_address.get(stripped).map(|c| (*c, "email-stripped"))
                })
        } else {
            let normalized = normalize_address(addr);
            let digits_only: String = addr.chars().filter(|c| c.is_ascii_digit()).collect();

            contact_by_address.get(normalized.as_str())
                .map(|c| (*c, "normalized"))
                .or_else(|| {
                    contact_by_address.get(digits_only.as_str()).map(|c| (*c, "digits-only"))
                })
                .or_else(|| {
                    // Try stripping leading '1' for US numbers (e.g. 15551234567 -> 5551234567)
                    if digits_only.len() == 11 && digits_only.starts_with('1') {
                        contact_by_address.get(&digits_only[1..]).map(|c| (*c, "strip-leading-1"))
                    } else {
                        None
                    }
                })
        };

        match matched {
            Some((contact, method)) => {
                resolved_via_address.push((handle, contact.display_name.clone(), method));
            }
            None => {
                unresolved.push(handle);
            }
        }
    }

    // 4. Print summary
    println!("{}", style("  Resolution Summary:").bold());
    println!(
        "    Resolved via contact_id (DB link):  {}",
        style(resolved_via_contact_id.len()).green()
    );
    println!(
        "    Resolved via address matching:       {}",
        style(resolved_via_address.len()).green()
    );
    println!(
        "    Unresolved (no contact found):       {}",
        style(unresolved.len()).yellow()
    );
    println!(
        "    Total handles:                       {}",
        all_handles.len()
    );
    println!();

    // 5. Show address-match breakdown by method
    if !resolved_via_address.is_empty() {
        let mut method_counts: HashMap<&str, usize> = HashMap::new();
        for (_, _, method) in &resolved_via_address {
            *method_counts.entry(method).or_insert(0) += 1;
        }

        println!("{}", style("  Address Match Breakdown:").bold());
        for (method, count) in &method_counts {
            println!("    {:<20} {}", method, style(count).green());
        }
        println!();

        // Show samples of address-matched handles
        println!("{}", style("  Sample Address Matches (up to 15):").bold());
        for (handle, contact_name, method) in resolved_via_address.iter().take(15) {
            println!(
                "    {} -> {} ({})",
                style(&handle.address).cyan(),
                style(contact_name).green(),
                style(method).dim()
            );
        }
        if resolved_via_address.len() > 15 {
            println!("    ... and {} more", resolved_via_address.len() - 15);
        }
        println!();
    }

    // 6. Show unresolved handles
    if !unresolved.is_empty() {
        println!(
            "{}",
            style("  Unresolved Handles (no contact match):").bold()
        );
        let mut unresolved_phones: Vec<&str> = Vec::new();
        let mut unresolved_emails: Vec<&str> = Vec::new();
        for handle in &unresolved {
            if handle.address.contains('@') {
                unresolved_emails.push(&handle.address);
            } else {
                unresolved_phones.push(&handle.address);
            }
        }

        if !unresolved_phones.is_empty() {
            println!(
                "    Phone handles ({}):",
                style(unresolved_phones.len()).yellow()
            );
            for addr in unresolved_phones.iter().take(20) {
                let normalized = normalize_address(addr);
                let digits: String = addr.chars().filter(|c| c.is_ascii_digit()).collect();
                println!(
                    "      {} (normalized={}, digits={})",
                    style(addr).yellow(),
                    normalized,
                    digits
                );
            }
            if unresolved_phones.len() > 20 {
                println!("      ... and {} more", unresolved_phones.len() - 20);
            }
        }

        if !unresolved_emails.is_empty() {
            println!(
                "    Email handles ({}):",
                style(unresolved_emails.len()).yellow()
            );
            for addr in unresolved_emails.iter().take(20) {
                println!("      {}", style(addr).yellow());
            }
            if unresolved_emails.len() > 20 {
                println!("      ... and {} more", unresolved_emails.len() - 20);
            }
        }
        println!();
    }

    // 7. Cross-check: handles with contact_id but contact missing from DB
    let mut orphaned_contact_ids = 0;
    for handle in &resolved_via_contact_id {
        if let Some(cid) = handle.contact_id {
            if !all_contacts.iter().any(|c| c.id == Some(cid)) {
                orphaned_contact_ids += 1;
                if orphaned_contact_ids <= 5 {
                    println!(
                        "  {} Handle {} has contact_id={} but no matching contact in DB",
                        style("WARNING").yellow().bold(),
                        handle.address,
                        cid
                    );
                }
            }
        }
    }
    if orphaned_contact_ids > 5 {
        println!(
            "  ... and {} more orphaned contact_id references",
            orphaned_contact_ids - 5
        );
    }
    if orphaned_contact_ids > 0 {
        println!();
    }

    // 8. Final verdict
    let total = all_handles.len();
    let resolved_total = resolved_via_contact_id.len() + resolved_via_address.len();
    let pct = if total > 0 {
        (resolved_total as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    println!("{}", style("  Verdict:").bold());
    if unresolved.is_empty() {
        println!(
            "    {} All {} handles resolve to a contact name.",
            style("PASS").green().bold(),
            total
        );
    } else {
        println!(
            "    {} {}/{} handles resolved ({:.1}%). {} unresolved.",
            if pct >= 90.0 { style("OK").green().bold() } else { style("WARNING").yellow().bold() },
            resolved_total,
            total,
            pct,
            unresolved.len()
        );
    }

    println!();
    println!("  Diagnosis complete.");

    Ok(())
}
