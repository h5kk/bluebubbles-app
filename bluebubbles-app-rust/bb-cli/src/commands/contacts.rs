//! Contact commands.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum ContactsAction {
    /// List all contacts.
    List {
        /// Maximum number of contacts to display.
        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },
    /// Search contacts by name, phone number, or email.
    Search {
        /// Search query (matches name, phone, or email).
        query: String,
        /// Maximum number of results.
        #[arg(short = 'n', long, default_value = "20")]
        limit: i64,
    },
    /// Sync contacts from the server.
    Sync,
}

pub async fn run(config: ConfigHandle, action: ContactsAction, format: OutputFormat) -> BbResult<()> {
    let db = super::init_database(&config).await?;

    match action {
        ContactsAction::List { limit } => {
            let conn = db.conn()?;
            let contacts = bb_models::queries::list_contacts(&conn)?;

            let contacts_to_show: Vec<_> = if let Some(lim) = limit {
                contacts.into_iter().take(lim).collect()
            } else {
                contacts
            };

            match format {
                OutputFormat::Json => {
                    let json: Vec<_> = contacts_to_show.iter().map(|c| {
                        serde_json::json!({
                            "name": c.display_name,
                            "phones": c.phone_list(),
                            "emails": c.email_list(),
                            "initials": c.initials(),
                        })
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if contacts_to_show.is_empty() {
                        println!("No contacts. Run `bluebubbles contacts sync` to fetch from server.");
                    } else {
                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS)
                            .set_content_arrangement(ContentArrangement::Dynamic);

                        table.set_header(vec!["Name", "Phone(s)", "Email(s)"]);

                        for c in &contacts_to_show {
                            let phones = c.phone_list().join(", ");
                            let emails = c.email_list().join(", ");
                            table.add_row(vec![
                                c.display_name.clone(),
                                if phones.is_empty() { "-".to_string() } else { phones },
                                if emails.is_empty() { "-".to_string() } else { emails },
                            ]);
                        }

                        println!("{table}");
                        println!("\n{} contact(s) shown", contacts_to_show.len());
                    }
                }
            }
        }
        ContactsAction::Search { query, limit } => {
            let conn = db.conn()?;

            // Search by name first
            let mut results = bb_models::queries::search_contacts(&conn, &query, limit)?;

            // Also search by phone if the query looks like a number
            if query.chars().any(|c| c.is_ascii_digit()) {
                let phone_results = bb_models::queries::search_contacts_by_phone_suffix(
                    &conn, &query, limit,
                )?;
                for contact in phone_results {
                    if !results.iter().any(|r| r.id == contact.id) {
                        results.push(contact);
                    }
                }
            }

            // Also search by email if the query contains @ or looks like an email
            if query.contains('@') || query.contains('.') {
                let email_results = bb_models::queries::search_contacts_by_email(
                    &conn, &query, limit,
                )?;
                for contact in email_results {
                    if !results.iter().any(|r| r.id == contact.id) {
                        results.push(contact);
                    }
                }
            }

            // Limit total results
            results.truncate(limit as usize);

            match format {
                OutputFormat::Json => {
                    let json: Vec<_> = results.iter().map(|c| {
                        serde_json::json!({
                            "name": c.display_name,
                            "phones": c.phone_list(),
                            "emails": c.email_list(),
                        })
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if results.is_empty() {
                        println!("No contacts matching \"{}\".", query);
                    } else {
                        println!(
                            "{} result(s) for \"{}\":\n",
                            results.len(),
                            query
                        );
                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS)
                            .set_content_arrangement(ContentArrangement::Dynamic);

                        table.set_header(vec!["Name", "Phone(s)", "Email(s)"]);

                        for c in &results {
                            let phones = c.phone_list().join(", ");
                            let emails = c.email_list().join(", ");
                            table.add_row(vec![
                                c.display_name.clone(),
                                if phones.is_empty() { "-".to_string() } else { phones },
                                if emails.is_empty() { "-".to_string() } else { emails },
                            ]);
                        }

                        println!("{table}");
                    }
                }
            }
        }
        ContactsAction::Sync => {
            let api = super::create_api_client(&config).await?;

            println!("  {} Fetching contacts from server...", style("...").dim());
            let contacts_json = api.get_contacts(false).await?;
            let conn = db.conn()?;
            bb_models::queries::delete_all_contacts(&conn)?;
            let mut count = 0;
            for cj in &contacts_json {
                if let Ok(mut c) = bb_models::Contact::from_server_map(cj) {
                    if c.save(&conn).is_ok() {
                        count += 1;
                    }
                }
            }
            println!(
                "  {} Synced {} contacts.",
                style("OK").green().bold(),
                count
            );
        }
    }

    Ok(())
}
