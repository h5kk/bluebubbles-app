//! Sync commands.

use clap::Subcommand;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum SyncAction {
    /// Run a full sync (initial setup).
    Full,
    /// Run an incremental sync (delta updates).
    Incremental,
}

pub async fn run(config: ConfigHandle, action: SyncAction, format: OutputFormat) -> BbResult<()> {
    let db = super::init_database(&config).await?;
    let api = super::create_api_client(&config).await?;

    let event_bus = bb_services::event_bus::EventBus::new(128);
    let sync_service = bb_services::sync::SyncService::new(config.clone(), db.clone(), event_bus);

    match action {
        SyncAction::Full => {
            println!(
                "  {} Starting full sync...\n",
                style("SYNC").cyan().bold()
            );

            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("  {spinner} [{elapsed_precise}] {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let pb_clone = pb.clone();
            let progress_cb = Box::new(move |p: bb_services::sync::SyncProgress| {
                let phase = format!("{:?}", p.phase);
                let progress = if let Some(total) = p.total {
                    format!(" ({}/{})", p.current, total)
                } else if p.current > 0 {
                    format!(" ({})", p.current)
                } else {
                    String::new()
                };
                pb_clone.set_message(format!("[{}]{} {}", phase, progress, p.message));
            });

            sync_service.full_sync(&api, Some(progress_cb)).await?;
            pb.finish_and_clear();

            // Show stats after sync
            let stats = db.stats()?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "status": "complete",
                        "chats": stats.chats,
                        "messages": stats.messages,
                        "handles": stats.handles,
                        "contacts": stats.contacts,
                    }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Full sync complete.\n",
                        style("OK").green().bold()
                    );
                    println!("  Synced data:");
                    println!("    Chats:       {}", stats.chats);
                    println!("    Messages:    {}", stats.messages);
                    println!("    Handles:     {}", stats.handles);
                    println!("    Contacts:    {}", stats.contacts);
                    println!("    Attachments: {}", stats.attachments);
                }
            }
        }
        SyncAction::Incremental => {
            println!(
                "  {} Starting incremental sync...",
                style("SYNC").cyan().bold()
            );

            // Get stats before sync for comparison
            let before = db.stats()?;
            sync_service.incremental_sync(&api).await?;
            let after = db.stats()?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "status": "complete",
                        "new_messages": after.messages - before.messages,
                        "new_chats": after.chats - before.chats,
                        "new_handles": after.handles - before.handles,
                    }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Incremental sync complete.\n",
                        style("OK").green().bold()
                    );

                    let new_msgs = after.messages - before.messages;
                    let new_chats = after.chats - before.chats;
                    let new_handles = after.handles - before.handles;

                    if new_msgs == 0 && new_chats == 0 && new_handles == 0 {
                        println!("  No new data since last sync.");
                    } else {
                        println!("  Changes:");
                        if new_msgs > 0 {
                            println!("    New messages:  +{}", new_msgs);
                        }
                        if new_chats > 0 {
                            println!("    New chats:     +{}", new_chats);
                        }
                        if new_handles > 0 {
                            println!("    New handles:   +{}", new_handles);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
