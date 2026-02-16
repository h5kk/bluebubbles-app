//! Status command - show connection and server status.

use console::style;
use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use bb_core::platform::Platform;
use crate::OutputFormat;

/// Run the status command.
pub async fn run(config: ConfigHandle, format: OutputFormat) -> BbResult<()> {
    let cfg = config.read().await;

    let api = super::create_api_client(&config).await?;

    let start = std::time::Instant::now();
    let reachable = api.ping().await.unwrap_or(false);
    let latency_ms = start.elapsed().as_millis();

    // Try to get local database stats
    let db_path = Platform::data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("bluebubbles.db");
    let db_stats = if db_path.exists() {
        drop(cfg);
        let db = super::init_database(&config).await.ok();
        let cfg = config.read().await;
        let stats = db.and_then(|d| d.stats().ok());
        let file_size = std::fs::metadata(&db_path).ok().map(|m| m.len());
        drop(cfg);
        Some((stats, file_size))
    } else {
        drop(cfg);
        None
    };

    let cfg = config.read().await;

    match format {
        OutputFormat::Json => {
            let mut json = serde_json::json!({
                "server_address": cfg.server.address,
                "server_reachable": reachable,
                "latency_ms": latency_ms,
                "setup_complete": cfg.sync.finished_setup,
                "last_sync": cfg.sync.last_incremental_sync,
            });

            if reachable {
                drop(cfg);
                if let Ok(info) = api.server_info().await {
                    json["server_version"] = serde_json::json!(info.server_version);
                    json["os_version"] = serde_json::json!(info.os_version);
                    json["private_api"] = serde_json::json!(info.private_api);
                }
                if let Ok(totals) = api.server_totals().await {
                    json["server_totals"] = serde_json::json!({
                        "chats": totals.chats,
                        "messages": totals.messages,
                        "handles": totals.handles,
                        "attachments": totals.attachments,
                    });
                }
            } else {
                drop(cfg);
            }

            if let Some((Some(stats), file_size)) = &db_stats {
                json["local_database"] = serde_json::json!({
                    "chats": stats.chats,
                    "messages": stats.messages,
                    "handles": stats.handles,
                    "attachments": stats.attachments,
                    "contacts": stats.contacts,
                    "file_size_bytes": file_size,
                    "path": db_path.display().to_string(),
                });
            }

            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
        }
        OutputFormat::Text => {
            println!("{}", style("Connection").bold().underlined());
            println!("  Server:    {}", cfg.server.address);
            println!(
                "  Status:    {}",
                if reachable {
                    format!("{} ({}ms)", style("reachable").green(), latency_ms)
                } else {
                    style("unreachable").red().to_string()
                }
            );
            println!(
                "  Setup:     {}",
                if cfg.sync.finished_setup {
                    style("complete").green().to_string()
                } else {
                    style("pending").yellow().to_string()
                }
            );

            if cfg.sync.last_incremental_sync > 0 {
                let ts = chrono::DateTime::from_timestamp_millis(cfg.sync.last_incremental_sync);
                if let Some(dt) = ts {
                    println!("  Last sync: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
                }
            }

            if reachable {
                drop(cfg);
                if let Ok(info) = api.server_info().await {
                    println!();
                    println!("{}", style("Server").bold().underlined());
                    println!(
                        "  Version:     {}",
                        info.server_version.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "  OS:          {}",
                        info.os_version.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "  Private API: {}",
                        if info.private_api.unwrap_or(false) {
                            style("enabled").green().to_string()
                        } else {
                            style("disabled").yellow().to_string()
                        }
                    );
                    if let Some(ref proxy) = info.proxy_service {
                        println!("  Proxy:       {proxy}");
                    }
                }

                if let Ok(totals) = api.server_totals().await {
                    println!();
                    println!("{}", style("Server Database").bold().underlined());
                    println!("  Chats:       {}", totals.chats.unwrap_or(0));
                    println!("  Messages:    {}", totals.messages.unwrap_or(0));
                    println!("  Handles:     {}", totals.handles.unwrap_or(0));
                    println!("  Attachments: {}", totals.attachments.unwrap_or(0));
                }
            } else {
                drop(cfg);
            }

            if let Some((Some(stats), file_size)) = &db_stats {
                println!();
                println!("{}", style("Local Database").bold().underlined());
                println!("  Path:        {}", db_path.display());
                println!("  Chats:       {}", stats.chats);
                println!("  Messages:    {}", stats.messages);
                println!("  Handles:     {}", stats.handles);
                println!("  Attachments: {}", stats.attachments);
                println!("  Contacts:    {}", stats.contacts);
                if let Some(size) = file_size {
                    println!("  Disk usage:  {}", super::format_bytes(*size));
                }
            }
        }
    }

    Ok(())
}
