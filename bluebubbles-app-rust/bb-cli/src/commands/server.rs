//! Server management commands.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum ServerAction {
    /// Ping the server.
    Ping,
    /// Get server info.
    Info,
    /// Get server statistics.
    Stats,
    /// Restart the server (soft).
    Restart,
    /// Restart the server (hard).
    RestartHard,
    /// Check for server updates.
    CheckUpdate,
    /// Get server logs.
    Logs {
        /// Number of log entries.
        #[arg(short = 'n', long, default_value = "100")]
        count: u32,
    },
}

pub async fn run(config: ConfigHandle, action: ServerAction, format: OutputFormat) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;

    match action {
        ServerAction::Ping => {
            let start = std::time::Instant::now();
            let reachable = api.ping().await?;
            let elapsed = start.elapsed();

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "reachable": reachable,
                        "latency_ms": elapsed.as_millis(),
                    }));
                }
                OutputFormat::Text => {
                    if reachable {
                        println!(
                            "  {} ({}ms)",
                            style("Pong!").green().bold(),
                            elapsed.as_millis()
                        );
                    } else {
                        println!(
                            "  {} Server unreachable.",
                            style("FAIL").red().bold()
                        );
                    }
                }
            }
        }
        ServerAction::Info => {
            let info = api.server_info().await?;
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&info).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!("{}", style("Server Information").bold().underlined());
                    println!(
                        "  Version:      {}",
                        info.server_version.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "  OS:           {}",
                        info.os_version.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "  Private API:  {}",
                        if info.private_api.unwrap_or(false) {
                            style("enabled").green().to_string()
                        } else {
                            style("disabled").yellow().to_string()
                        }
                    );
                    println!(
                        "  Helper:       {}",
                        if info.helper_connected.unwrap_or(false) {
                            style("connected").green().to_string()
                        } else {
                            style("disconnected").red().to_string()
                        }
                    );
                    if let Some(ref proxy) = info.proxy_service {
                        println!("  Proxy:        {proxy}");
                    }
                    if let Some(ref icloud) = info.detected_icloud {
                        println!("  iCloud:       {icloud}");
                    }
                    if let Some(ref ips) = info.local_ipv4s {
                        println!("  IPv4:         {}", ips.join(", "));
                    }
                    if let Some(ref ips) = info.local_ipv6s {
                        if !ips.is_empty() {
                            println!("  IPv6:         {}", ips.join(", "));
                        }
                    }
                }
            }
        }
        ServerAction::Stats => {
            let totals = api.server_totals().await?;
            let media = api.server_media_totals().await.ok();

            match format {
                OutputFormat::Json => {
                    let mut json = serde_json::to_value(&totals).unwrap_or_default();
                    if let Some(ref m) = media {
                        json["media"] = serde_json::to_value(m).unwrap_or_default();
                    }
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                }
                OutputFormat::Text => {
                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_content_arrangement(ContentArrangement::Dynamic);

                    table.set_header(vec!["Category", "Count"]);
                    table.add_row(vec![
                        "Chats".to_string(),
                        totals.chats.unwrap_or(0).to_string(),
                    ]);
                    table.add_row(vec![
                        "Messages".to_string(),
                        totals.messages.unwrap_or(0).to_string(),
                    ]);
                    table.add_row(vec![
                        "Handles".to_string(),
                        totals.handles.unwrap_or(0).to_string(),
                    ]);
                    table.add_row(vec![
                        "Attachments".to_string(),
                        totals.attachments.unwrap_or(0).to_string(),
                    ]);

                    if let Some(ref m) = media {
                        table.add_row(vec![
                            "Images".to_string(),
                            m.images.unwrap_or(0).to_string(),
                        ]);
                        table.add_row(vec![
                            "Videos".to_string(),
                            m.videos.unwrap_or(0).to_string(),
                        ]);
                        table.add_row(vec![
                            "Locations".to_string(),
                            m.locations.unwrap_or(0).to_string(),
                        ]);
                    }

                    println!("{}", style("Server Statistics").bold().underlined());
                    println!("{table}");
                }
            }
        }
        ServerAction::Restart => {
            api.server_restart_soft().await?;
            println!(
                "  {} Server soft restart initiated.",
                style("OK").green().bold()
            );
        }
        ServerAction::RestartHard => {
            api.server_restart_hard().await?;
            println!(
                "  {} Server hard restart initiated.",
                style("OK").green().bold()
            );
        }
        ServerAction::CheckUpdate => {
            let result = api.server_check_update().await?;
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if result.is_null() {
                        println!("  No update information available.");
                    } else if let Some(available) = result.get("available").and_then(|v| v.as_bool()) {
                        if available {
                            let version = result.get("metadata")
                                .and_then(|m| m.get("version"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            println!(
                                "  {} Update available: v{}",
                                style("UPDATE").yellow().bold(),
                                version
                            );
                        } else {
                            println!(
                                "  {} Server is up to date.",
                                style("OK").green().bold()
                            );
                        }
                    } else {
                        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                    }
                }
            }
        }
        ServerAction::Logs { count } => {
            let logs = api.server_logs(count).await?;
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&logs).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if logs.is_empty() {
                        println!("  No server logs available.");
                    } else {
                        for entry in &logs {
                            if let Some(msg) = entry.as_str() {
                                println!("{msg}");
                            } else {
                                println!("{entry}");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
