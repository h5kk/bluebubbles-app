//! Private API test commands.
//!
//! Provides CLI commands to check Private API status and test
//! Private API features like typing indicators and mark-as-read.

use clap::Subcommand;
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum PrivateApiAction {
    /// Check Private API status on the server.
    Status,
    /// Run a full test of Private API features.
    Test {
        /// Chat GUID to use for testing typing indicators and mark-as-read.
        #[arg(short, long)]
        chat_guid: Option<String>,
    },
    /// Send a typing indicator to a chat.
    Typing {
        /// Chat GUID.
        chat_guid: String,
        /// "start" or "stop".
        #[arg(short, long, default_value = "start")]
        status: String,
    },
    /// Mark a chat as read.
    MarkRead {
        /// Chat GUID.
        chat_guid: String,
    },
}

pub async fn run(
    config: ConfigHandle,
    action: PrivateApiAction,
    format: OutputFormat,
) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;

    match action {
        PrivateApiAction::Status => {
            let info = api.server_info().await?;

            let private_api_enabled = info.private_api.unwrap_or(false);
            let helper_connected = info.helper_connected.unwrap_or(false);

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "private_api_enabled": private_api_enabled,
                        "helper_connected": helper_connected,
                        "server_version": info.server_version,
                        "os_version": info.os_version,
                    }));
                }
                OutputFormat::Text => {
                    println!("{}", style("Private API Status").bold().underlined());
                    println!(
                        "  Private API:      {}",
                        if private_api_enabled {
                            style("enabled").green().to_string()
                        } else {
                            style("disabled").red().to_string()
                        }
                    );
                    println!(
                        "  Helper Connected: {}",
                        if helper_connected {
                            style("yes").green().to_string()
                        } else {
                            style("no").red().to_string()
                        }
                    );
                    println!(
                        "  Server Version:   {}",
                        info.server_version.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "  macOS Version:    {}",
                        info.os_version.as_deref().unwrap_or("unknown")
                    );

                    if !private_api_enabled {
                        println!();
                        println!(
                            "  {} Private API is not enabled on the server.",
                            style("WARNING").yellow().bold()
                        );
                        println!("  Enable it in the BlueBubbles server settings to use");
                        println!("  typing indicators, tapbacks, effects, and other features.");
                    } else if !helper_connected {
                        println!();
                        println!(
                            "  {} Private API is enabled but the helper is not connected.",
                            style("WARNING").yellow().bold()
                        );
                        println!("  The helper process may need to be restarted.");
                    } else {
                        println!();
                        println!(
                            "  {} Private API is fully operational.",
                            style("OK").green().bold()
                        );
                    }
                }
            }
        }

        PrivateApiAction::Test { chat_guid } => {
            println!("{}", style("Private API Test Suite").bold().underlined());
            println!();

            // Test 1: Check status
            print!("  [1/4] Checking Private API status... ");
            let info = api.server_info().await?;
            let private_api_enabled = info.private_api.unwrap_or(false);
            let helper_connected = info.helper_connected.unwrap_or(false);

            if private_api_enabled && helper_connected {
                println!("{}", style("PASS").green().bold());
            } else if private_api_enabled {
                println!("{}", style("WARN - helper not connected").yellow().bold());
            } else {
                println!("{}", style("FAIL - not enabled").red().bold());
                println!();
                println!("  Private API must be enabled on the server to continue.");
                return Ok(());
            }

            // Test 2: Server ping
            print!("  [2/4] Pinging server... ");
            let start = std::time::Instant::now();
            let reachable = api.ping().await?;
            let latency = start.elapsed();
            if reachable {
                println!(
                    "{} ({}ms)",
                    style("PASS").green().bold(),
                    latency.as_millis()
                );
            } else {
                println!("{}", style("FAIL").red().bold());
            }

            if let Some(ref guid) = chat_guid {
                // Test 3: Typing indicator
                print!("  [3/4] Sending typing indicator to {}... ", guid);
                match api.send_typing_indicator(guid, "start").await {
                    Ok(()) => {
                        println!("{}", style("PASS").green().bold());
                        // Stop it after a brief moment
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _ = api.send_typing_indicator(guid, "stop").await;
                    }
                    Err(e) => {
                        println!("{} - {}", style("FAIL").red().bold(), e);
                    }
                }

                // Test 4: Mark as read
                print!("  [4/4] Marking chat as read... ");
                match api.mark_chat_read(guid).await {
                    Ok(()) => {
                        println!("{}", style("PASS").green().bold());
                    }
                    Err(e) => {
                        println!("{} - {}", style("FAIL").red().bold(), e);
                    }
                }
            } else {
                println!("  [3/4] Typing indicator test... {}", style("SKIP (no --chat-guid)").yellow());
                println!("  [4/4] Mark as read test... {}", style("SKIP (no --chat-guid)").yellow());
            }

            println!();
            println!("  Test suite complete.");
            if chat_guid.is_none() {
                println!("  Tip: Use --chat-guid to test typing indicators and mark-as-read.");
                println!("  Example: bluebubbles private-api test --chat-guid \"iMessage;-;+15551234567\"");
            }
        }

        PrivateApiAction::Typing { chat_guid, status } => {
            if status != "start" && status != "stop" {
                println!(
                    "  {} Invalid status '{}'. Use 'start' or 'stop'.",
                    style("ERROR").red().bold(),
                    status
                );
                return Ok(());
            }

            api.send_typing_indicator(&chat_guid, &status).await?;
            println!(
                "  {} Typing indicator '{}' sent to {}",
                style("OK").green().bold(),
                status,
                chat_guid
            );
        }

        PrivateApiAction::MarkRead { chat_guid } => {
            api.mark_chat_read(&chat_guid).await?;
            println!(
                "  {} Chat marked as read: {}",
                style("OK").green().bold(),
                chat_guid
            );
        }
    }

    Ok(())
}
