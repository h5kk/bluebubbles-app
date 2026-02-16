//! Message commands.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum MessagesAction {
    /// List messages in a chat.
    List {
        /// Chat GUID.
        #[arg(short, long)]
        chat: String,
        /// Number of messages to show.
        #[arg(short = 'n', long, default_value = "25")]
        limit: i64,
        /// Page number (1-based).
        #[arg(short, long, default_value = "1")]
        page: i64,
        /// Only show messages before this date (ISO 8601 or epoch ms).
        #[arg(long)]
        before: Option<String>,
        /// Only show messages after this date (ISO 8601 or epoch ms).
        #[arg(long)]
        after: Option<String>,
    },
    /// Get a specific message.
    Get {
        /// Message GUID.
        guid: String,
    },
    /// Search messages by text.
    Search {
        /// Search query.
        query: String,
        /// Max results.
        #[arg(short = 'n', long, default_value = "20")]
        limit: i64,
        /// Limit search to a specific chat GUID.
        #[arg(short, long)]
        chat: Option<String>,
    },
    /// Send a text message.
    Send {
        /// Chat GUID to send to.
        #[arg(short, long)]
        chat: String,
        /// Message text.
        text: String,
        /// iMessage effect ID (e.g., slam, loud, gentle, invisible-ink).
        #[arg(short, long)]
        effect: Option<String>,
    },
    /// Send a reaction / tapback to a message.
    React {
        /// Chat GUID the message belongs to.
        #[arg(short, long)]
        chat: String,
        /// GUID of the message to react to.
        #[arg(short, long)]
        message: String,
        /// Reaction type (love, like, dislike, laugh, emphasize, question).
        reaction: String,
        /// Part index within the message (default: 0).
        #[arg(short, long, default_value = "0")]
        part_index: i32,
    },
    /// Edit a previously sent message.
    Edit {
        /// GUID of the message to edit.
        guid: String,
        /// New message text.
        text: String,
        /// Part index within the message (default: 0).
        #[arg(short, long, default_value = "0")]
        part_index: i32,
        /// Backwards-compatibility text shown to older clients.
        #[arg(long)]
        compat_text: Option<String>,
    },
    /// Unsend (retract) a previously sent message.
    Unsend {
        /// GUID of the message to unsend.
        guid: String,
        /// Part index within the message (default: 0).
        #[arg(short, long, default_value = "0")]
        part_index: i32,
    },
}

/// Map a human-readable effect name to an iMessage effect ID.
fn resolve_effect_id(effect: &str) -> String {
    match effect.to_lowercase().as_str() {
        "slam" => "com.apple.MobileSMS.expressivesend.impact".to_string(),
        "loud" => "com.apple.MobileSMS.expressivesend.loud".to_string(),
        "gentle" => "com.apple.MobileSMS.expressivesend.gentle".to_string(),
        "invisible-ink" | "invisibleink" | "invisible" => {
            "com.apple.MobileSMS.expressivesend.invisibleink".to_string()
        }
        "echo" => "com.apple.messages.effect.CKEchoEffect".to_string(),
        "spotlight" => "com.apple.messages.effect.CKSpotlightEffect".to_string(),
        "balloons" => "com.apple.messages.effect.CKHappyBirthdayEffect".to_string(),
        "confetti" => "com.apple.messages.effect.CKConfettiEffect".to_string(),
        "love" => "com.apple.messages.effect.CKHeartEffect".to_string(),
        "lasers" => "com.apple.messages.effect.CKLasersEffect".to_string(),
        "fireworks" => "com.apple.messages.effect.CKFireworksEffect".to_string(),
        "celebration" => "com.apple.messages.effect.CKSparklesEffect".to_string(),
        other => other.to_string(), // Pass through raw effect IDs
    }
}

pub async fn run(config: ConfigHandle, action: MessagesAction, format: OutputFormat) -> BbResult<()> {
    match action {
        MessagesAction::List { chat, limit, page, before, after } => {
            let db = super::init_database(&config).await?;
            let conn = db.conn()?;
            let chat_obj = bb_models::queries::find_chat_by_guid(&conn, &chat)?;
            let chat_id = chat_obj
                .and_then(|c| c.id)
                .ok_or_else(|| bb_core::error::BbError::ChatNotFound(chat.clone()))?;

            // If before/after are specified, use the API for server-side filtering
            if before.is_some() || after.is_some() {
                let api = super::create_api_client(&config).await?;
                let before_ts = before.as_deref().and_then(|s| s.parse::<i64>().ok());
                let after_ts = after.as_deref().and_then(|s| s.parse::<i64>().ok());
                let offset = (page.max(1) - 1) * limit;

                let messages = api.get_chat_messages(
                    &chat,
                    offset,
                    limit,
                    "DESC",
                    &["chats"],
                    before_ts,
                    after_ts,
                ).await?;

                match format {
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&messages).unwrap_or_default());
                    }
                    OutputFormat::Text => {
                        if messages.is_empty() {
                            println!("No messages found in the specified date range.");
                        } else {
                            for msg in messages.iter().rev() {
                                let from_me = msg.get("isFromMe").and_then(|v| v.as_bool()).unwrap_or(false);
                                let sender = if from_me { "You" } else { "Them" };
                                let text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("[no text]");
                                let date = msg.get("dateCreated").and_then(|v| v.as_str()).unwrap_or("");
                                let date_short = if date.len() > 19 { &date[..19] } else { date };
                                println!(
                                    "  {} {}: {}",
                                    style(date_short).dim(),
                                    style(sender).bold(),
                                    text
                                );
                            }
                        }
                    }
                }
            } else {
                // Use local database for regular listing
                let offset = (page.max(1) - 1) * limit;
                let messages = bb_models::queries::list_messages_for_chat(
                    &conn,
                    chat_id,
                    offset,
                    limit,
                    bb_models::queries::SortDirection::Desc,
                )?;

                let total = bb_models::queries::count_messages_for_chat(&conn, chat_id).unwrap_or(0);

                match format {
                    OutputFormat::Json => {
                        let json: Vec<_> = messages.iter().map(|m| {
                            serde_json::json!({
                                "guid": m.guid,
                                "text": m.text,
                                "from_me": m.is_from_me,
                                "date_created": m.date_created,
                                "date_read": m.date_read,
                                "date_delivered": m.date_delivered,
                                "is_delivered": m.is_delivered,
                                "status": m.indicator_to_show(),
                                "has_attachments": m.has_attachments,
                            })
                        }).collect();
                        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                    }
                    OutputFormat::Text => {
                        if messages.is_empty() {
                            println!("No messages found.");
                        } else {
                            let mut table = Table::new();
                            table
                                .load_preset(UTF8_FULL)
                                .apply_modifier(UTF8_ROUND_CORNERS)
                                .set_content_arrangement(ContentArrangement::Dynamic);

                            table.set_header(vec!["Sender", "Message", "Date", "Status"]);

                            // Show in chronological order
                            for msg in messages.iter().rev() {
                                let sender = if msg.is_from_me { "You" } else { "Them" };
                                let text = msg.text.as_deref().unwrap_or("[no text]");
                                let text = super::truncate(text, 50);
                                let date = msg.date_created.as_deref().unwrap_or("-");
                                let date_short = if date.len() > 19 { &date[..19] } else { date };
                                let status = msg.indicator_to_show();

                                table.add_row(vec![
                                    sender.to_string(),
                                    text,
                                    date_short.to_string(),
                                    status.to_string(),
                                ]);
                            }

                            println!("{table}");
                            println!(
                                "\nPage {}/{} ({} total messages)",
                                page,
                                ((total as f64) / (limit as f64)).ceil() as i64,
                                total
                            );
                        }
                    }
                }
            }
        }
        MessagesAction::Get { guid } => {
            let db = super::init_database(&config).await?;
            let conn = db.conn()?;
            match bb_models::queries::find_message_by_guid(&conn, &guid)? {
                Some(msg) => {
                    // Load attachments if any
                    let attachments = if msg.has_attachments {
                        if let Some(msg_id) = msg.id {
                            bb_models::queries::load_attachments_for_message(&conn, msg_id)
                                .unwrap_or_default()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    };

                    match format {
                        OutputFormat::Json => {
                            let mut json = serde_json::to_value(&msg).unwrap_or_default();
                            if !attachments.is_empty() {
                                json["attachments_detail"] = serde_json::json!(
                                    attachments.iter().map(|a| {
                                        serde_json::json!({
                                            "guid": a.guid,
                                            "transfer_name": a.transfer_name,
                                            "mime_type": a.mime_type,
                                            "total_bytes": a.total_bytes,
                                        })
                                    }).collect::<Vec<_>>()
                                );
                            }
                            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                        }
                        OutputFormat::Text => {
                            println!("{}", style("Message Details").bold().underlined());
                            println!("  GUID:       {}", msg.guid.as_deref().unwrap_or("?"));
                            println!("  Text:       {}", msg.text.as_deref().unwrap_or("[no text]"));
                            if let Some(ref subj) = msg.subject {
                                println!("  Subject:    {subj}");
                            }
                            println!("  From me:    {}", msg.is_from_me);
                            println!("  Date:       {}", msg.date_created.as_deref().unwrap_or("-"));
                            println!("  Delivered:  {}", msg.is_delivered);
                            println!("  Read:       {}", msg.date_read.as_deref().unwrap_or("-"));
                            println!("  Status:     {}", msg.indicator_to_show());
                            if msg.error != 0 {
                                println!(
                                    "  Error:      {} (code {})",
                                    style("yes").red(),
                                    msg.error
                                );
                            }
                            if let Some(ref effect) = msg.expressive_send_style_id {
                                println!("  Effect:     {effect}");
                            }
                            if msg.has_attachments && !attachments.is_empty() {
                                println!();
                                println!("{}", style("Attachments").bold().underlined());
                                for a in &attachments {
                                    let name = a.transfer_name.as_deref().unwrap_or("?");
                                    let mime = a.mime_type.as_deref().unwrap_or("?");
                                    let size = a.total_bytes.map(|b| super::format_bytes(b as u64))
                                        .unwrap_or_else(|| "?".to_string());
                                    println!("  - {name} ({mime}, {size})");
                                    println!("    GUID: {}", a.guid.as_deref().unwrap_or("?"));
                                }
                            }
                        }
                    }
                }
                None => {
                    println!("{} Message not found: {guid}", style("ERROR").red().bold());
                }
            }
        }
        MessagesAction::Search { query, limit, chat } => {
            let db = super::init_database(&config).await?;
            let conn = db.conn()?;

            let messages = if let Some(ref chat_guid) = chat {
                // Search within a specific chat
                let chat_obj = bb_models::queries::find_chat_by_guid(&conn, chat_guid)?;
                let chat_id = chat_obj
                    .and_then(|c| c.id)
                    .ok_or_else(|| bb_core::error::BbError::ChatNotFound(chat_guid.clone()))?;
                bb_models::queries::search_messages_in_chat(&conn, chat_id, &query, limit)?
            } else {
                bb_models::queries::search_messages(&conn, &query, limit)?
            };

            match format {
                OutputFormat::Json => {
                    let json: Vec<_> = messages.iter().map(|m| {
                        serde_json::json!({
                            "guid": m.guid,
                            "text": m.text,
                            "chat_id": m.chat_id,
                            "date_created": m.date_created,
                            "from_me": m.is_from_me,
                        })
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if messages.is_empty() {
                        println!("No messages matching \"{}\".", query);
                    } else {
                        let scope = if let Some(ref cg) = chat {
                            format!(" in chat {cg}")
                        } else {
                            String::new()
                        };
                        println!(
                            "{} result(s) for \"{}\"{}\n",
                            messages.len(),
                            query,
                            scope
                        );
                        for msg in &messages {
                            let sender = if msg.is_from_me { "You" } else { "Them" };
                            let text = msg.text.as_deref().unwrap_or("[no text]");
                            let date = msg.date_created.as_deref().unwrap_or("");
                            let date_short = if date.len() > 19 { &date[..19] } else { date };
                            println!(
                                "  {} {} {}: {}",
                                style(msg.guid.as_deref().unwrap_or("?")).dim(),
                                style(date_short).dim(),
                                style(sender).bold(),
                                text
                            );
                        }
                    }
                }
            }
        }
        MessagesAction::Send { chat, text, effect } => {
            let api = super::create_api_client(&config).await?;
            let temp_guid = format!("temp-{}", uuid::Uuid::new_v4());

            let effect_id = effect.map(|e| resolve_effect_id(&e));

            println!(
                "  {} Sending message to {}...",
                style("...").dim(),
                style(&chat).bold()
            );

            let params = bb_api::endpoints::messages::SendTextParams {
                chat_guid: chat.clone(),
                temp_guid,
                message: text.clone(),
                method: "private-api".into(),
                effect_id,
                subject: None,
                selected_message_guid: None,
                part_index: None,
                dd_scan: None,
            };
            let result = api.send_text(&params).await?;
            let guid = result.get("guid").and_then(|v| v.as_str()).unwrap_or("unknown");

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Message sent (guid: {})",
                        style("OK").green().bold(),
                        guid
                    );
                }
            }
        }
        MessagesAction::React { chat, message, reaction, part_index } => {
            let api = super::create_api_client(&config).await?;

            // Resolve reaction name to the numeric tapback value
            let reaction_value = resolve_reaction(&reaction);

            // We need the message text for the reaction API; fetch it from the server
            let msg_data = api.get_message(&message, &[]).await?;
            let selected_text = msg_data
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            println!(
                "  {} Sending {} reaction to {}...",
                style("...").dim(),
                style(&reaction).bold(),
                style(&message).dim()
            );

            let params = bb_api::endpoints::messages::SendReactionParams {
                chat_guid: chat.clone(),
                selected_message_text: selected_text,
                selected_message_guid: message.clone(),
                reaction: reaction_value,
                part_index: if part_index != 0 { Some(part_index) } else { None },
            };
            let result = api.send_reaction(&params).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Reaction \"{}\" sent to message {}",
                        style("OK").green().bold(),
                        reaction,
                        message
                    );
                }
            }
        }
        MessagesAction::Edit { guid, text, part_index, compat_text } => {
            let api = super::create_api_client(&config).await?;

            let compat = compat_text.unwrap_or_else(|| format!("Edited to: {text}"));

            println!(
                "  {} Editing message {}...",
                style("...").dim(),
                style(&guid).dim()
            );

            let params = bb_api::endpoints::messages::EditMessageParams {
                edited_message: text.clone(),
                backwards_compatibility_message: compat,
                part_index,
            };
            let result = api.edit_message(&guid, &params).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Message {} edited successfully.",
                        style("OK").green().bold(),
                        guid
                    );
                }
            }
        }
        MessagesAction::Unsend { guid, part_index } => {
            let api = super::create_api_client(&config).await?;

            println!(
                "  {} Unsending message {}...",
                style("...").dim(),
                style(&guid).dim()
            );

            let result = api.unsend_message(&guid, part_index).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Message {} unsent successfully.",
                        style("OK").green().bold(),
                        guid
                    );
                }
            }
        }
    }

    Ok(())
}

/// Resolve a human-readable reaction name to the iMessage tapback value.
fn resolve_reaction(name: &str) -> String {
    match name.to_lowercase().as_str() {
        "love" | "heart" => "2000".to_string(),
        "like" | "thumbsup" => "2001".to_string(),
        "dislike" | "thumbsdown" => "2002".to_string(),
        "laugh" | "haha" => "2003".to_string(),
        "emphasize" | "exclamation" | "!!" => "2004".to_string(),
        "question" | "?" => "2005".to_string(),
        // Negative (remove) variants
        "-love" | "-heart" => "3000".to_string(),
        "-like" | "-thumbsup" => "3001".to_string(),
        "-dislike" | "-thumbsdown" => "3002".to_string(),
        "-laugh" | "-haha" => "3003".to_string(),
        "-emphasize" | "-exclamation" => "3004".to_string(),
        "-question" => "3005".to_string(),
        other => other.to_string(), // Pass through raw numeric values
    }
}
