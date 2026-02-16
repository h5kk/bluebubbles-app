//! Database management commands.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;
use dialoguer::Confirm;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum DbAction {
    /// Show database statistics.
    Stats,
    /// Run an integrity check.
    Check,
    /// Reset the database (WARNING: destroys all data).
    Reset,
    /// Show the database file path.
    Path,
}

pub async fn run(config: ConfigHandle, action: DbAction, format: OutputFormat) -> BbResult<()> {
    let db_path = bb_core::platform::Platform::data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("bluebubbles.db");

    match action {
        DbAction::Stats => {
            let db = super::init_database(&config).await?;
            let stats = db.stats()?;

            let file_size = std::fs::metadata(&db_path).ok().map(|m| m.len());

            // Get WAL file size if it exists
            let wal_path = db_path.with_extension("db-wal");
            let wal_size = std::fs::metadata(&wal_path).ok().map(|m| m.len());

            // Get SHM file size if it exists
            let shm_path = db_path.with_extension("db-shm");
            let shm_size = std::fs::metadata(&shm_path).ok().map(|m| m.len());

            // Check WAL mode
            let conn = db.conn()?;
            let journal_mode: String = conn
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .unwrap_or_else(|_| "unknown".to_string());

            let page_size: i64 = conn
                .query_row("PRAGMA page_size", [], |row| row.get(0))
                .unwrap_or(0);

            let page_count: i64 = conn
                .query_row("PRAGMA page_count", [], |row| row.get(0))
                .unwrap_or(0);

            let freelist_count: i64 = conn
                .query_row("PRAGMA freelist_count", [], |row| row.get(0))
                .unwrap_or(0);

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "path": db_path.display().to_string(),
                        "tables": {
                            "chats": stats.chats,
                            "messages": stats.messages,
                            "handles": stats.handles,
                            "attachments": stats.attachments,
                            "contacts": stats.contacts,
                            "themes": stats.themes,
                            "settings": stats.settings,
                        },
                        "file_size_bytes": file_size,
                        "wal_size_bytes": wal_size,
                        "journal_mode": journal_mode,
                        "page_size": page_size,
                        "page_count": page_count,
                        "freelist_count": freelist_count,
                    }));
                }
                OutputFormat::Text => {
                    println!("{}", style("Database Statistics").bold().underlined());
                    println!("  Path:          {}", db_path.display());
                    println!("  Journal mode:  {}", journal_mode);
                    println!();

                    // Table counts
                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_content_arrangement(ContentArrangement::Dynamic);

                    table.set_header(vec!["Table", "Row Count"]);
                    table.add_row(vec!["chats".to_string(), stats.chats.to_string()]);
                    table.add_row(vec!["messages".to_string(), stats.messages.to_string()]);
                    table.add_row(vec!["handles".to_string(), stats.handles.to_string()]);
                    table.add_row(vec!["attachments".to_string(), stats.attachments.to_string()]);
                    table.add_row(vec!["contacts".to_string(), stats.contacts.to_string()]);
                    table.add_row(vec!["themes".to_string(), stats.themes.to_string()]);
                    table.add_row(vec!["settings".to_string(), stats.settings.to_string()]);

                    println!("{table}");

                    println!();
                    println!("{}", style("Storage").bold().underlined());
                    if let Some(size) = file_size {
                        println!("  Database:      {}", super::format_bytes(size));
                    }
                    if let Some(size) = wal_size {
                        println!("  WAL file:      {}", super::format_bytes(size));
                    }
                    if let Some(size) = shm_size {
                        println!("  SHM file:      {}", super::format_bytes(size));
                    }
                    let total_size = file_size.unwrap_or(0)
                        + wal_size.unwrap_or(0)
                        + shm_size.unwrap_or(0);
                    if total_size > 0 {
                        println!("  Total:         {}", super::format_bytes(total_size));
                    }

                    println!();
                    println!("{}", style("Internals").bold().underlined());
                    println!("  Page size:     {} bytes", page_size);
                    println!("  Page count:    {}", page_count);
                    println!("  Free pages:    {}", freelist_count);
                    if freelist_count > 0 {
                        let wasted = freelist_count * page_size;
                        println!(
                            "  Reclaimable:   {} (run VACUUM to reclaim)",
                            super::format_bytes(wasted as u64)
                        );
                    }
                }
            }
        }
        DbAction::Check => {
            println!(
                "  {} Running integrity check...",
                style("...").dim()
            );
            let db = super::init_database(&config).await?;

            // Run quick check first
            let conn = db.conn()?;
            let quick_result: String = conn
                .query_row("PRAGMA quick_check", [], |row| row.get(0))
                .unwrap_or_else(|_| "error".to_string());

            if quick_result == "ok" {
                println!(
                    "  {} Quick check passed.",
                    style("OK").green().bold()
                );
            } else {
                println!(
                    "  {} Quick check issue: {}",
                    style("WARN").yellow().bold(),
                    quick_result
                );
            }

            // Run full integrity check
            match db.run_integrity_check() {
                Ok(()) => {
                    println!(
                        "  {} Full integrity check passed.",
                        style("OK").green().bold()
                    );
                }
                Err(e) => {
                    println!(
                        "  {} Integrity check failed: {}",
                        style("FAIL").red().bold(),
                        e
                    );
                }
            }

            // Check foreign key constraints
            let fk_violations: Vec<String> = {
                let mut stmt = conn
                    .prepare("PRAGMA foreign_key_check")
                    .unwrap();
                let rows: Vec<String> = stmt
                    .query_map([], |row| {
                        let table: String = row.get(0)?;
                        let rowid: i64 = row.get(1)?;
                        let parent: String = row.get(2)?;
                        Ok(format!("{table} row {rowid} -> {parent}"))
                    })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect();
                rows
            };

            if fk_violations.is_empty() {
                println!(
                    "  {} Foreign key constraints OK.",
                    style("OK").green().bold()
                );
            } else {
                println!(
                    "  {} {} foreign key violation(s):",
                    style("WARN").yellow().bold(),
                    fk_violations.len()
                );
                for v in fk_violations.iter().take(10) {
                    println!("    - {v}");
                }
                if fk_violations.len() > 10 {
                    println!("    ... and {} more", fk_violations.len() - 10);
                }
            }
        }
        DbAction::Reset => {
            println!(
                "  {} This will delete ALL local data.",
                style("WARNING").red().bold()
            );
            println!("  Database: {}", db_path.display());

            let confirmed = Confirm::new()
                .with_prompt("  Are you sure you want to reset the database?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if !confirmed {
                println!("  Reset cancelled.");
                return Ok(());
            }

            let db = super::init_database(&config).await?;
            db.reset()?;
            println!(
                "  {} Database reset complete.",
                style("OK").green().bold()
            );
        }
        DbAction::Path => {
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({"path": db_path.display().to_string()}));
                }
                OutputFormat::Text => {
                    println!("{}", db_path.display());
                }
            }
        }
    }

    Ok(())
}
