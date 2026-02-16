//! Chat commands.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

/// Sort options for chat listing.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ChatSort {
    /// Sort by most recent message date.
    Date,
    /// Sort by unread count (most unread first).
    Unread,
    /// Sort by chat name alphabetically.
    Name,
}

#[derive(Subcommand)]
pub enum ChatsAction {
    /// List all chats.
    List {
        /// Number of chats to show per page.
        #[arg(short = 'n', long, default_value = "25")]
        limit: i64,
        /// Page number (1-based).
        #[arg(short, long, default_value = "1")]
        page: i64,
        /// Sort order.
        #[arg(short, long, default_value = "date")]
        sort: ChatSort,
        /// Include archived chats.
        #[arg(long)]
        archived: bool,
    },
    /// Get details for a specific chat.
    Get {
        /// Chat GUID.
        guid: String,
    },
    /// Search chats by name.
    Search {
        /// Search query.
        query: String,
    },
    /// Mark a chat as read.
    Read {
        /// Chat GUID.
        guid: String,
    },
}

pub async fn run(config: ConfigHandle, action: ChatsAction, format: OutputFormat) -> BbResult<()> {
    let db = super::init_database(&config).await?;

    match action {
        ChatsAction::List { limit, page, sort, archived } => {
            let offset = (page.max(1) - 1) * limit;
            let conn = db.conn()?;

            // Use the detailed query for table output
            let details = bb_models::queries::list_chats_with_details(
                &conn, offset, limit, archived,
            )?;

            // Sort in-memory for non-date sorts
            let mut details = details;
            match sort {
                ChatSort::Unread => {
                    details.sort_by(|a, b| b.unread_count.cmp(&a.unread_count));
                }
                ChatSort::Name => {
                    details.sort_by(|a, b| {
                        a.chat.title().to_lowercase().cmp(&b.chat.title().to_lowercase())
                    });
                }
                ChatSort::Date => {
                    // Already sorted by date from the query
                }
            }

            match format {
                OutputFormat::Json => {
                    let json: Vec<_> = details.iter().map(|d| {
                        serde_json::json!({
                            "guid": d.chat.guid,
                            "title": d.chat.title(),
                            "pinned": d.chat.is_pinned,
                            "unread_count": d.unread_count,
                            "participant_count": d.participant_count,
                            "last_message_text": d.last_message_text,
                            "last_message_date": d.last_message_date,
                            "last_message_is_from_me": d.last_message_is_from_me,
                        })
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if details.is_empty() {
                        println!("No chats found.");
                    } else {
                        let total_count = bb_models::queries::count_chats(&conn).unwrap_or(0);

                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS)
                            .set_content_arrangement(ContentArrangement::Dynamic);

                        table.set_header(vec!["Chat", "Last Message", "Date", "Unread"]);

                        for d in &details {
                            let title = {
                                let t = d.chat.title();
                                let pin = if d.chat.is_pinned { " [P]" } else { "" };
                                format!("{}{}", super::truncate(&t, 30), pin)
                            };

                            let last_msg = d.last_message_text.as_deref().unwrap_or("-");
                            let last_msg = super::truncate(last_msg, 40);

                            let date = d.last_message_date.as_deref().unwrap_or("-");
                            let date_short = if date.len() > 10 { &date[..10] } else { date };

                            let unread = if d.unread_count > 0 {
                                format!("{}", d.unread_count)
                            } else {
                                "-".to_string()
                            };

                            table.add_row(vec![title, last_msg, date_short.to_string(), unread]);
                        }

                        println!("{table}");
                        println!(
                            "\nPage {}/{} ({} total chats)",
                            page,
                            ((total_count as f64) / (limit as f64)).ceil() as i64,
                            total_count
                        );
                    }
                }
            }
        }
        ChatsAction::Get { guid } => {
            let conn = db.conn()?;
            match bb_models::queries::find_chat_by_guid(&conn, &guid)? {
                Some(chat) => {
                    let participants = bb_models::queries::load_chat_participants(
                        &conn,
                        chat.id.unwrap_or(0),
                    )?;

                    let msg_count = bb_models::queries::count_messages_for_chat(
                        &conn,
                        chat.id.unwrap_or(0),
                    ).unwrap_or(0);

                    let unread = bb_models::queries::unread_count_for_chat(
                        &conn,
                        chat.id.unwrap_or(0),
                    ).unwrap_or(0);

                    match format {
                        OutputFormat::Json => {
                            let json = serde_json::json!({
                                "guid": chat.guid,
                                "title": chat.title(),
                                "type": if chat.is_group() { "group" } else { "direct" },
                                "is_imessage": chat.is_imessage(),
                                "is_pinned": chat.is_pinned,
                                "is_archived": chat.is_archived,
                                "message_count": msg_count,
                                "unread_count": unread,
                                "latest_message_date": chat.latest_message_date,
                                "participants": participants.iter().map(|p| {
                                    serde_json::json!({
                                        "address": p.address,
                                        "service": p.service,
                                        "display_name": p.display_name(),
                                    })
                                }).collect::<Vec<_>>(),
                            });
                            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                        }
                        OutputFormat::Text => {
                            println!("{}", style("Chat Details").bold().underlined());
                            println!("  Title:      {}", chat.title());
                            println!("  GUID:       {}", chat.guid);
                            println!(
                                "  Type:       {}",
                                if chat.is_group() { "group" } else { "direct" }
                            );
                            println!(
                                "  Service:    {}",
                                if chat.is_imessage() { "iMessage" } else { "SMS" }
                            );
                            println!("  Pinned:     {}", if chat.is_pinned { "yes" } else { "no" });
                            println!("  Archived:   {}", if chat.is_archived { "yes" } else { "no" });
                            println!("  Messages:   {}", msg_count);
                            println!("  Unread:     {}", unread);
                            if let Some(ref date) = chat.latest_message_date {
                                println!("  Last msg:   {date}");
                            }

                            if !participants.is_empty() {
                                println!();
                                println!("{}", style("Participants").bold().underlined());
                                for p in &participants {
                                    println!("  - {} ({}, {})", p.display_name(), p.address, p.service);
                                }
                            }
                        }
                    }
                }
                None => {
                    println!("{} Chat not found: {guid}", style("ERROR").red().bold());
                }
            }
        }
        ChatsAction::Search { query } => {
            let conn = db.conn()?;
            let chats = bb_models::queries::search_chats(&conn, &query, 20)?;

            match format {
                OutputFormat::Json => {
                    let json: Vec<_> = chats.iter().map(|c| {
                        serde_json::json!({"guid": c.guid, "title": c.title()})
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if chats.is_empty() {
                        println!("No chats matching \"{}\".", query);
                    } else {
                        println!("{} result(s) for \"{}\":\n", chats.len(), query);
                        for chat in &chats {
                            println!("  {} ({})", chat.title(), style(&chat.guid).dim());
                        }
                    }
                }
            }
        }
        ChatsAction::Read { guid } => {
            let api = super::create_api_client(&config).await?;
            api.mark_chat_read(&guid).await?;
            println!("{} Chat marked as read: {guid}", style("OK").green().bold());
        }
    }

    Ok(())
}
