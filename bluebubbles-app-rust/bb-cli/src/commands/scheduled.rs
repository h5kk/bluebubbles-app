//! Scheduled message commands - list, create, update, and delete scheduled messages.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum ScheduledAction {
    /// List all scheduled messages.
    List,
    /// Create a new scheduled message.
    Create {
        /// Chat GUID to send the message to.
        #[arg(short, long)]
        chat: String,
        /// Message text to send.
        #[arg(short, long)]
        message: String,
        /// When to send (epoch timestamp in milliseconds).
        #[arg(short, long)]
        scheduled_for: i64,
        /// Schedule type (send-message, remind).
        #[arg(short = 't', long, default_value = "send-message")]
        schedule_type: String,
    },
    /// Update an existing scheduled message.
    Update {
        /// ID of the scheduled message to update.
        id: i64,
        /// Chat GUID to send the message to.
        #[arg(short, long)]
        chat: Option<String>,
        /// New message text.
        #[arg(short, long)]
        message: Option<String>,
        /// New scheduled time (epoch timestamp in milliseconds).
        #[arg(short, long)]
        scheduled_for: Option<i64>,
        /// Schedule type (send-message, remind).
        #[arg(short = 't', long)]
        schedule_type: Option<String>,
    },
    /// Delete a scheduled message.
    Delete {
        /// ID of the scheduled message to delete.
        id: i64,
    },
}

pub async fn run(config: ConfigHandle, action: ScheduledAction, format: OutputFormat) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;

    match action {
        ScheduledAction::List => {
            let messages = api.get_scheduled_messages().await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&messages).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if messages.is_empty() {
                        println!("No scheduled messages.");
                    } else {
                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS)
                            .set_content_arrangement(ContentArrangement::Dynamic);

                        table.set_header(vec!["ID", "Type", "Chat", "Message", "Scheduled For", "Status"]);

                        for msg in &messages {
                            let id = msg.get("id")
                                .and_then(|v| v.as_i64())
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "-".to_string());
                            let stype = msg.get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("-");
                            let chat = msg.get("payload")
                                .and_then(|p| p.get("chatGuid"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("-");
                            let message_text = msg.get("payload")
                                .and_then(|p| p.get("message"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("[no text]");
                            let scheduled_for = msg.get("scheduledFor")
                                .and_then(|v| v.as_i64())
                                .map(|ts| {
                                    chrono::DateTime::from_timestamp_millis(ts)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .unwrap_or_else(|| ts.to_string())
                                })
                                .unwrap_or_else(|| "-".to_string());
                            let status = msg.get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("pending");

                            table.add_row(vec![
                                id,
                                stype.to_string(),
                                super::truncate(chat, 30),
                                super::truncate(message_text, 40),
                                scheduled_for,
                                status.to_string(),
                            ]);
                        }

                        println!("{table}");
                        println!("\n{} scheduled message(s).", messages.len());
                    }
                }
            }
        }
        ScheduledAction::Create { chat, message, scheduled_for, schedule_type } => {
            let temp_guid = format!("temp-{}", uuid::Uuid::new_v4());

            let payload = serde_json::json!({
                "chatGuid": chat,
                "message": message,
                "tempGuid": temp_guid,
                "method": "private-api",
            });

            let params = bb_api::endpoints::messages::ScheduleMessageParams {
                schedule_type,
                payload,
                scheduled_for,
                schedule: None,
            };

            println!(
                "  {} Creating scheduled message for {}...",
                style("...").dim(),
                style(&chat).bold()
            );

            let result = api.create_scheduled_message(&params).await?;
            let id = result.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Scheduled message created (id: {}).",
                        style("OK").green().bold(),
                        id
                    );
                }
            }
        }
        ScheduledAction::Update { id, chat, message, scheduled_for, schedule_type } => {
            // Fetch the existing scheduled message to merge changes
            let existing_messages = api.get_scheduled_messages().await?;
            let existing = existing_messages.iter()
                .find(|m| m.get("id").and_then(|v| v.as_i64()) == Some(id));

            let existing = match existing {
                Some(e) => e,
                None => {
                    println!(
                        "{} Scheduled message with id {} not found.",
                        style("ERROR").red().bold(),
                        id
                    );
                    return Ok(());
                }
            };

            let existing_payload = existing.get("payload").cloned().unwrap_or(serde_json::json!({}));
            let existing_type = existing.get("type").and_then(|v| v.as_str()).unwrap_or("send-message").to_string();
            let existing_scheduled = existing.get("scheduledFor").and_then(|v| v.as_i64()).unwrap_or(0);

            let mut new_payload = existing_payload.clone();
            if let Some(ref c) = chat {
                new_payload["chatGuid"] = serde_json::Value::String(c.clone());
            }
            if let Some(ref m) = message {
                new_payload["message"] = serde_json::Value::String(m.clone());
            }

            let params = bb_api::endpoints::messages::ScheduleMessageParams {
                schedule_type: schedule_type.unwrap_or(existing_type),
                payload: new_payload,
                scheduled_for: scheduled_for.unwrap_or(existing_scheduled),
                schedule: None,
            };

            println!(
                "  {} Updating scheduled message {}...",
                style("...").dim(),
                id
            );

            let result = api.update_scheduled_message(id, &params).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Scheduled message {} updated.",
                        style("OK").green().bold(),
                        id
                    );
                }
            }
        }
        ScheduledAction::Delete { id } => {
            println!(
                "  {} Deleting scheduled message {}...",
                style("...").dim(),
                id
            );

            api.delete_scheduled_message(id).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "id": id, "deleted": true }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Scheduled message {} deleted.",
                        style("OK").green().bold(),
                        id
                    );
                }
            }
        }
    }

    Ok(())
}
