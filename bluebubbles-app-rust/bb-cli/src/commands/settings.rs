//! Settings commands.

use clap::Subcommand;
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum SettingsAction {
    /// List/show all settings (alias for show).
    List,
    /// Show all settings.
    Show,
    /// Get a specific setting value by key path.
    Get {
        /// Setting key path (e.g., "server.address", "sync.messages_per_page").
        key: String,
    },
    /// Set a specific setting value by key path.
    Set {
        /// Setting key path (e.g., "server.address", "sync.messages_per_page").
        key: String,
        /// New value.
        value: String,
    },
    /// Set the server address.
    SetAddress {
        /// Server address URL.
        address: String,
    },
    /// Set the server password (GUID auth key).
    SetPassword {
        /// Server password.
        password: String,
    },
    /// Export settings to a file.
    Export {
        /// Output file path.
        path: String,
    },
    /// Import settings from a file.
    Import {
        /// Input file path.
        path: String,
    },
}

/// Resolve a dot-separated key path to a value from the config.
fn get_setting_value(cfg: &bb_core::config::AppConfig, key: &str) -> Option<String> {
    match key {
        "server.address" => Some(cfg.server.address.clone()),
        "server.guid_auth_key" | "server.password" => Some("********".to_string()),
        "server.api_timeout_ms" | "server.timeout" => Some(cfg.server.api_timeout_ms.to_string()),
        "server.accept_self_signed_certs" => Some(cfg.server.accept_self_signed_certs.to_string()),
        "database.wal_mode" => Some(cfg.database.wal_mode.to_string()),
        "database.pool_size" => Some(cfg.database.pool_size.to_string()),
        "database.integrity_check_on_startup" => Some(cfg.database.integrity_check_on_startup.to_string()),
        "logging.level" | "log.level" => Some(cfg.logging.level.clone()),
        "logging.json_output" => Some(cfg.logging.json_output.to_string()),
        "logging.max_file_size_bytes" => Some(cfg.logging.max_file_size_bytes.to_string()),
        "logging.max_rotated_files" => Some(cfg.logging.max_rotated_files.to_string()),
        "sync.finished_setup" => Some(cfg.sync.finished_setup.to_string()),
        "sync.last_incremental_sync" => Some(cfg.sync.last_incremental_sync.to_string()),
        "sync.messages_per_page" => Some(cfg.sync.messages_per_page.to_string()),
        "sync.skip_empty_chats" => Some(cfg.sync.skip_empty_chats.to_string()),
        "sync.sync_contacts_automatically" => Some(cfg.sync.sync_contacts_automatically.to_string()),
        "notifications.notify_reactions" => Some(cfg.notifications.notify_reactions.to_string()),
        "notifications.notify_on_chat_list" => Some(cfg.notifications.notify_on_chat_list.to_string()),
        "notifications.filter_unknown_senders" => Some(cfg.notifications.filter_unknown_senders.to_string()),
        "display.user_name" => Some(cfg.display.user_name.clone()),
        "display.use_24hr_format" => Some(cfg.display.use_24hr_format.to_string()),
        "display.redacted_mode" => Some(cfg.display.redacted_mode.to_string()),
        _ => None,
    }
}

/// Apply a value to a dot-separated key path on the config.
fn set_setting_value(cfg: &mut bb_core::config::AppConfig, key: &str, value: &str) -> Result<(), String> {
    match key {
        "server.address" => {
            cfg.server.address = bb_core::config::AppConfig::sanitize_server_address(value);
        }
        "server.guid_auth_key" | "server.password" => {
            cfg.server.guid_auth_key = value.to_string();
        }
        "server.api_timeout_ms" | "server.timeout" => {
            cfg.server.api_timeout_ms = value.parse().map_err(|_| "invalid integer".to_string())?;
        }
        "server.accept_self_signed_certs" => {
            cfg.server.accept_self_signed_certs = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "database.wal_mode" => {
            cfg.database.wal_mode = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "database.pool_size" => {
            cfg.database.pool_size = value.parse().map_err(|_| "invalid integer".to_string())?;
        }
        "logging.level" | "log.level" => {
            let v = value.to_lowercase();
            if !["trace", "debug", "info", "warn", "error"].contains(&v.as_str()) {
                return Err("expected one of: trace, debug, info, warn, error".to_string());
            }
            cfg.logging.level = v;
        }
        "logging.json_output" => {
            cfg.logging.json_output = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "sync.messages_per_page" => {
            cfg.sync.messages_per_page = value.parse().map_err(|_| "invalid integer".to_string())?;
        }
        "sync.skip_empty_chats" => {
            cfg.sync.skip_empty_chats = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "sync.sync_contacts_automatically" => {
            cfg.sync.sync_contacts_automatically = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "notifications.notify_reactions" => {
            cfg.notifications.notify_reactions = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "notifications.filter_unknown_senders" => {
            cfg.notifications.filter_unknown_senders = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "display.user_name" => {
            cfg.display.user_name = value.to_string();
        }
        "display.use_24hr_format" => {
            cfg.display.use_24hr_format = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        "display.redacted_mode" => {
            cfg.display.redacted_mode = value.parse().map_err(|_| "expected true/false".to_string())?;
        }
        _ => {
            return Err(format!("unknown setting key: {key}"));
        }
    }
    Ok(())
}

fn print_settings_text(cfg: &bb_core::config::AppConfig) {
    println!("{}", style("Server").bold().underlined());
    println!("  server.address                    {}", cfg.server.address);
    println!("  server.api_timeout_ms             {}", cfg.server.api_timeout_ms);
    println!("  server.accept_self_signed_certs   {}", cfg.server.accept_self_signed_certs);

    println!();
    println!("{}", style("Database").bold().underlined());
    println!("  database.wal_mode                 {}", cfg.database.wal_mode);
    println!("  database.pool_size                {}", cfg.database.pool_size);
    println!("  database.integrity_check_on_startup {}", cfg.database.integrity_check_on_startup);

    println!();
    println!("{}", style("Sync").bold().underlined());
    println!("  sync.finished_setup               {}", cfg.sync.finished_setup);
    println!("  sync.messages_per_page            {}", cfg.sync.messages_per_page);
    println!("  sync.skip_empty_chats             {}", cfg.sync.skip_empty_chats);
    println!("  sync.sync_contacts_automatically  {}", cfg.sync.sync_contacts_automatically);

    println!();
    println!("{}", style("Logging").bold().underlined());
    println!("  logging.level                     {}", cfg.logging.level);
    println!("  logging.json_output               {}", cfg.logging.json_output);
    println!("  logging.max_file_size_bytes       {}", cfg.logging.max_file_size_bytes);
    println!("  logging.max_rotated_files         {}", cfg.logging.max_rotated_files);

    println!();
    println!("{}", style("Notifications").bold().underlined());
    println!("  notifications.notify_reactions     {}", cfg.notifications.notify_reactions);
    println!("  notifications.notify_on_chat_list  {}", cfg.notifications.notify_on_chat_list);
    println!("  notifications.filter_unknown_senders {}", cfg.notifications.filter_unknown_senders);

    println!();
    println!("{}", style("Display").bold().underlined());
    println!("  display.user_name                 {}", cfg.display.user_name);
    println!("  display.use_24hr_format           {}", cfg.display.use_24hr_format);
    println!("  display.redacted_mode             {}", cfg.display.redacted_mode);
}

fn settings_json(cfg: &bb_core::config::AppConfig) -> serde_json::Value {
    serde_json::json!({
        "server": {
            "address": cfg.server.address,
            "api_timeout_ms": cfg.server.api_timeout_ms,
            "accept_self_signed_certs": cfg.server.accept_self_signed_certs,
        },
        "database": {
            "wal_mode": cfg.database.wal_mode,
            "pool_size": cfg.database.pool_size,
            "integrity_check_on_startup": cfg.database.integrity_check_on_startup,
        },
        "sync": {
            "finished_setup": cfg.sync.finished_setup,
            "messages_per_page": cfg.sync.messages_per_page,
            "skip_empty_chats": cfg.sync.skip_empty_chats,
            "sync_contacts_automatically": cfg.sync.sync_contacts_automatically,
        },
        "logging": {
            "level": cfg.logging.level,
            "json_output": cfg.logging.json_output,
            "max_file_size_bytes": cfg.logging.max_file_size_bytes,
            "max_rotated_files": cfg.logging.max_rotated_files,
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
}

pub async fn run(config: ConfigHandle, action: SettingsAction, format: OutputFormat) -> BbResult<()> {
    match action {
        SettingsAction::Show | SettingsAction::List => {
            let cfg = config.read().await;
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&settings_json(&cfg)).unwrap_or_default());
                }
                OutputFormat::Text => {
                    print_settings_text(&cfg);
                }
            }
        }
        SettingsAction::Get { key } => {
            let cfg = config.read().await;
            match get_setting_value(&cfg, &key) {
                Some(value) => {
                    match format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::json!({ "key": key, "value": value }));
                        }
                        OutputFormat::Text => {
                            println!("{} = {}", key, value);
                        }
                    }
                }
                None => {
                    println!(
                        "{} Unknown setting key: {}",
                        style("ERROR").red().bold(),
                        key
                    );
                    println!("  Use `bluebubbles settings list` to see available keys.");
                }
            }
        }
        SettingsAction::Set { key, value } => {
            {
                let mut cfg = config.write().await;
                match set_setting_value(&mut cfg, &key, &value) {
                    Ok(()) => {}
                    Err(e) => {
                        println!(
                            "{} Failed to set {}: {}",
                            style("ERROR").red().bold(),
                            key,
                            e
                        );
                        return Ok(());
                    }
                }
            }
            // Save to disk
            let cfg = config.read().await;
            let path = bb_core::platform::Platform::config_dir()?.join("config.toml");
            cfg.save_to_file(&path)?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "key": key, "value": value, "saved": true }));
                }
                OutputFormat::Text => {
                    println!(
                        "{} {} = {}",
                        style("SET").green().bold(),
                        key,
                        value
                    );
                }
            }
        }
        SettingsAction::SetAddress { address } => {
            let sanitized = bb_core::config::AppConfig::sanitize_server_address(&address);
            let mut cfg = config.write().await;
            cfg.server.address = sanitized.clone();
            drop(cfg);

            let cfg = config.read().await;
            let path = bb_core::platform::Platform::config_dir()?.join("config.toml");
            cfg.save_to_file(&path)?;
            println!(
                "{} Server address set to: {}",
                style("SET").green().bold(),
                sanitized
            );
        }
        SettingsAction::SetPassword { password } => {
            {
                let mut cfg = config.write().await;
                cfg.server.guid_auth_key = password;
            }
            let cfg = config.read().await;
            let path = bb_core::platform::Platform::config_dir()?.join("config.toml");
            cfg.save_to_file(&path)?;
            println!(
                "{} Server password updated.",
                style("SET").green().bold()
            );
        }
        SettingsAction::Export { path } => {
            let cfg = config.read().await;
            cfg.save_to_file(std::path::Path::new(&path))?;
            println!(
                "{} Settings exported to {}",
                style("OK").green().bold(),
                path
            );
        }
        SettingsAction::Import { path } => {
            let imported = bb_core::config::AppConfig::load_from_file(std::path::Path::new(&path))?;
            let mut cfg = config.write().await;
            *cfg = imported;
            drop(cfg);

            let cfg = config.read().await;
            let save_path = bb_core::platform::Platform::config_dir()?.join("config.toml");
            cfg.save_to_file(&save_path)?;
            println!(
                "{} Settings imported from {} and saved.",
                style("OK").green().bold(),
                path
            );
        }
    }

    Ok(())
}
