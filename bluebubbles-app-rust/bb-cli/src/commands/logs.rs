//! Log viewing commands.

use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use bb_core::platform::Platform;
use crate::OutputFormat;

/// Run the logs command.
pub async fn run(
    config: ConfigHandle,
    count: u32,
    server_logs: bool,
    follow: bool,
    level_filter: Option<String>,
    format: OutputFormat,
) -> BbResult<()> {
    if server_logs {
        // Fetch server logs via API
        let api = super::create_api_client(&config).await?;
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
                        let msg = if let Some(s) = entry.as_str() {
                            s.to_string()
                        } else {
                            entry.to_string()
                        };

                        // Apply level filter
                        if let Some(ref filter) = level_filter {
                            let filter_upper = filter.to_uppercase();
                            let msg_upper = msg.to_uppercase();
                            if !msg_upper.contains(&filter_upper) {
                                continue;
                            }
                        }

                        // Colorize log levels
                        let colored = colorize_log_line(&msg);
                        println!("{colored}");
                    }
                }
            }
        }
    } else {
        // Show local log files
        let log_dir = {
            let cfg = config.read().await;
            if cfg.logging.directory.is_empty() {
                Platform::data_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("logs")
            } else {
                std::path::PathBuf::from(&cfg.logging.directory)
            }
        };

        if !log_dir.exists() {
            println!("No log directory found at: {}", log_dir.display());
            return Ok(());
        }

        // Find the most recent log file
        let mut log_files: Vec<_> = std::fs::read_dir(&log_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "log" || ext == "json")
            })
            .collect();

        log_files.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

        if let Some(latest) = log_files.first() {
            let log_path = latest.path();

            if follow {
                // Tail -f mode: show last N lines then follow
                println!(
                    "  {} Following {} (Ctrl+C to stop)\n",
                    style("TAIL").cyan().bold(),
                    log_path.display()
                );

                let content = std::fs::read_to_string(&log_path)?;
                let lines: Vec<&str> = content.lines().collect();
                let start = if lines.len() > count as usize {
                    lines.len() - count as usize
                } else {
                    0
                };

                // Print initial lines
                for line in &lines[start..] {
                    if should_show_line(line, &level_filter) {
                        println!("{}", colorize_log_line(line));
                    }
                }

                // Track file position and poll for new content
                let mut last_size = content.len();
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                            if let Ok(new_content) = std::fs::read_to_string(&log_path) {
                                if new_content.len() > last_size {
                                    let new_part = &new_content[last_size..];
                                    for line in new_part.lines() {
                                        if !line.is_empty() && should_show_line(line, &level_filter) {
                                            println!("{}", colorize_log_line(line));
                                        }
                                    }
                                    last_size = new_content.len();
                                }
                            }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            println!("\n  Stopped tailing.");
                            break;
                        }
                    }
                }
            } else {
                // Regular mode: show last N lines
                let content = std::fs::read_to_string(&log_path)?;
                let lines: Vec<&str> = content.lines().collect();
                let start = if lines.len() > count as usize {
                    lines.len() - count as usize
                } else {
                    0
                };

                let mut shown = 0;
                for line in &lines[start..] {
                    if should_show_line(line, &level_filter) {
                        match format {
                            OutputFormat::Text => println!("{}", colorize_log_line(line)),
                            OutputFormat::Json => println!("{line}"),
                        }
                        shown += 1;
                    }
                }

                println!(
                    "\n  --- {} ({} shown / {} total lines) ---",
                    log_path.display(),
                    shown,
                    lines.len()
                );
            }
        } else {
            println!("No log files found in {}", log_dir.display());
        }
    }

    Ok(())
}

/// Check if a log line should be shown based on the level filter.
fn should_show_line(line: &str, level_filter: &Option<String>) -> bool {
    if let Some(ref filter) = level_filter {
        let filter_upper = filter.to_uppercase();
        let line_upper = line.to_uppercase();

        // Log level severity ordering
        let levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        let filter_idx = levels.iter().position(|l| *l == filter_upper);

        if let Some(idx) = filter_idx {
            // Show this level and above (more severe)
            for level in &levels[..=idx] {
                if line_upper.contains(level) {
                    return true;
                }
            }
            return false;
        }

        // Fallback to simple string match
        line_upper.contains(&filter_upper)
    } else {
        true
    }
}

/// Apply color to a log line based on its log level.
fn colorize_log_line(line: &str) -> String {
    let upper = line.to_uppercase();
    if upper.contains("ERROR") {
        style(line).red().to_string()
    } else if upper.contains("WARN") {
        style(line).yellow().to_string()
    } else if upper.contains("DEBUG") {
        style(line).dim().to_string()
    } else if upper.contains("TRACE") {
        style(line).dim().to_string()
    } else {
        line.to_string()
    }
}
