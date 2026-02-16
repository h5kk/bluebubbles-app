//! Backup commands - export/import settings and themes from the server.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum BackupAction {
    /// List all backups (themes and settings) stored on the server.
    List,
    /// Get theme backup data from the server.
    GetThemes,
    /// Save a theme backup to the server.
    SaveTheme {
        /// Name for the theme backup.
        name: String,
        /// Path to a JSON file containing the theme data.
        #[arg(short, long)]
        file: String,
    },
    /// Delete a theme backup from the server.
    DeleteTheme {
        /// Name of the theme backup to delete.
        name: String,
    },
    /// Get settings backup data from the server.
    GetSettings,
    /// Save a settings backup to the server.
    SaveSettings {
        /// Name for the settings backup.
        name: String,
        /// Path to a JSON file containing the settings data. If omitted, uses current local settings.
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Delete a settings backup from the server.
    DeleteSettings {
        /// Name of the settings backup to delete.
        name: String,
    },
    /// Export server backups (themes + settings) to a local JSON file.
    Export {
        /// Output file path for the combined backup.
        path: String,
    },
    /// Import backups from a local JSON file to the server.
    Import {
        /// Input file path containing backup data.
        path: String,
    },
}

pub async fn run(config: ConfigHandle, action: BackupAction, format: OutputFormat) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;

    match action {
        BackupAction::List => {
            let themes = api.get_theme_backup().await?;
            let settings = api.get_settings_backup().await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                        "themes": themes,
                        "settings": settings,
                    })).unwrap_or_default());
                }
                OutputFormat::Text => {
                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_content_arrangement(ContentArrangement::Dynamic);

                    table.set_header(vec!["Type", "Name", "Size"]);

                    // Theme entries
                    if let Some(obj) = themes.as_object() {
                        for (name, data) in obj {
                            let size = serde_json::to_string(data)
                                .map(|s| super::format_bytes(s.len() as u64))
                                .unwrap_or_else(|_| "-".to_string());
                            table.add_row(vec![
                                "Theme".to_string(),
                                name.clone(),
                                size,
                            ]);
                        }
                    } else if let Some(arr) = themes.as_array() {
                        for item in arr {
                            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                            let size = serde_json::to_string(item)
                                .map(|s| super::format_bytes(s.len() as u64))
                                .unwrap_or_else(|_| "-".to_string());
                            table.add_row(vec![
                                "Theme".to_string(),
                                name.to_string(),
                                size,
                            ]);
                        }
                    }

                    // Settings entries
                    if let Some(obj) = settings.as_object() {
                        for (name, data) in obj {
                            let size = serde_json::to_string(data)
                                .map(|s| super::format_bytes(s.len() as u64))
                                .unwrap_or_else(|_| "-".to_string());
                            table.add_row(vec![
                                "Settings".to_string(),
                                name.clone(),
                                size,
                            ]);
                        }
                    } else if let Some(arr) = settings.as_array() {
                        for item in arr {
                            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                            let size = serde_json::to_string(item)
                                .map(|s| super::format_bytes(s.len() as u64))
                                .unwrap_or_else(|_| "-".to_string());
                            table.add_row(vec![
                                "Settings".to_string(),
                                name.to_string(),
                                size,
                            ]);
                        }
                    }

                    if table.row_count() == 0 {
                        println!("No backups found on the server.");
                    } else {
                        println!("{table}");
                    }
                }
            }
        }
        BackupAction::GetThemes => {
            let themes = api.get_theme_backup().await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&themes).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if themes.is_null() {
                        println!("No theme backups found on the server.");
                    } else {
                        println!("{}", style("Theme Backups").bold().underlined());
                        println!("{}", serde_json::to_string_pretty(&themes).unwrap_or_default());
                    }
                }
            }
        }
        BackupAction::SaveTheme { name, file } => {
            let content = std::fs::read_to_string(&file)
                .map_err(|e| bb_core::error::BbError::Http(format!("failed to read file {file}: {e}")))?;
            let data: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| bb_core::error::BbError::Serialization(format!("invalid JSON in {file}: {e}")))?;

            println!(
                "  {} Saving theme backup \"{}\"...",
                style("...").dim(),
                style(&name).bold()
            );

            api.save_theme_backup(&name, &data).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "name": name, "saved": true }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Theme backup \"{}\" saved to server.",
                        style("OK").green().bold(),
                        name
                    );
                }
            }
        }
        BackupAction::DeleteTheme { name } => {
            println!(
                "  {} Deleting theme backup \"{}\"...",
                style("...").dim(),
                style(&name).bold()
            );

            api.delete_theme_backup(&name).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "name": name, "deleted": true }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Theme backup \"{}\" deleted.",
                        style("OK").green().bold(),
                        name
                    );
                }
            }
        }
        BackupAction::GetSettings => {
            let settings = api.get_settings_backup().await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&settings).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if settings.is_null() {
                        println!("No settings backups found on the server.");
                    } else {
                        println!("{}", style("Settings Backups").bold().underlined());
                        println!("{}", serde_json::to_string_pretty(&settings).unwrap_or_default());
                    }
                }
            }
        }
        BackupAction::SaveSettings { name, file } => {
            let data = if let Some(ref file_path) = file {
                let content = std::fs::read_to_string(file_path)
                    .map_err(|e| bb_core::error::BbError::Http(format!("failed to read file {file_path}: {e}")))?;
                serde_json::from_str(&content)
                    .map_err(|e| bb_core::error::BbError::Serialization(format!("invalid JSON: {e}")))?
            } else {
                // Use current local config as the settings data
                let cfg = config.read().await;
                serde_json::json!({
                    "server": {
                        "address": cfg.server.address,
                        "api_timeout_ms": cfg.server.api_timeout_ms,
                        "accept_self_signed_certs": cfg.server.accept_self_signed_certs,
                    },
                    "sync": {
                        "messages_per_page": cfg.sync.messages_per_page,
                        "skip_empty_chats": cfg.sync.skip_empty_chats,
                        "sync_contacts_automatically": cfg.sync.sync_contacts_automatically,
                    },
                    "notifications": {
                        "notify_reactions": cfg.notifications.notify_reactions,
                        "notify_on_chat_list": cfg.notifications.notify_on_chat_list,
                        "filter_unknown_senders": cfg.notifications.filter_unknown_senders,
                    },
                    "display": {
                        "user_name": cfg.display.user_name,
                        "use_24hr_format": cfg.display.use_24hr_format,
                        "redacted_mode": cfg.display.redacted_mode,
                    },
                })
            };

            println!(
                "  {} Saving settings backup \"{}\"...",
                style("...").dim(),
                style(&name).bold()
            );

            api.save_settings_backup(&name, &data).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "name": name, "saved": true }));
                }
                OutputFormat::Text => {
                    let source = if file.is_some() { "from file" } else { "from local config" };
                    println!(
                        "  {} Settings backup \"{}\" saved to server ({}).",
                        style("OK").green().bold(),
                        name,
                        source
                    );
                }
            }
        }
        BackupAction::DeleteSettings { name } => {
            println!(
                "  {} Deleting settings backup \"{}\"...",
                style("...").dim(),
                style(&name).bold()
            );

            api.delete_settings_backup(&name).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "name": name, "deleted": true }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Settings backup \"{}\" deleted.",
                        style("OK").green().bold(),
                        name
                    );
                }
            }
        }
        BackupAction::Export { path } => {
            println!(
                "  {} Fetching backups from server...",
                style("...").dim()
            );

            let themes = api.get_theme_backup().await?;
            let settings = api.get_settings_backup().await?;

            let combined = serde_json::json!({
                "themes": themes,
                "settings": settings,
                "exported_at": chrono::Utc::now().to_rfc3339(),
            });

            let content = serde_json::to_string_pretty(&combined)
                .map_err(|e| bb_core::error::BbError::Serialization(e.to_string()))?;
            std::fs::write(&path, &content)
                .map_err(|e| bb_core::error::BbError::Http(format!("failed to write file {path}: {e}")))?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "path": path, "exported": true }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Backups exported to {}",
                        style("OK").green().bold(),
                        path
                    );
                }
            }
        }
        BackupAction::Import { path } => {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| bb_core::error::BbError::Http(format!("failed to read file {path}: {e}")))?;
            let data: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| bb_core::error::BbError::Serialization(format!("invalid JSON in {path}: {e}")))?;

            println!(
                "  {} Importing backups from {}...",
                style("...").dim(),
                style(&path).bold()
            );

            let mut imported_count = 0;

            // Import themes
            if let Some(themes) = data.get("themes") {
                if let Some(obj) = themes.as_object() {
                    for (name, theme_data) in obj {
                        api.save_theme_backup(name, theme_data).await?;
                        imported_count += 1;
                    }
                }
            }

            // Import settings
            if let Some(settings) = data.get("settings") {
                if let Some(obj) = settings.as_object() {
                    for (name, settings_data) in obj {
                        api.save_settings_backup(name, settings_data).await?;
                        imported_count += 1;
                    }
                }
            }

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "path": path,
                        "imported": true,
                        "count": imported_count,
                    }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Imported {} backup(s) from {}",
                        style("OK").green().bold(),
                        imported_count,
                        path
                    );
                }
            }
        }
    }

    Ok(())
}
