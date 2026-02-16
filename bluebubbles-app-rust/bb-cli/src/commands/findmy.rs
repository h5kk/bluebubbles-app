//! FindMy commands - show FindMy devices and friends locations.

use clap::Subcommand;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum FindMyAction {
    /// List FindMy devices with their locations.
    Devices {
        /// Refresh device locations from iCloud before listing (slower).
        #[arg(short, long)]
        refresh: bool,
    },
    /// List FindMy friends with their locations.
    Friends {
        /// Refresh friend locations from iCloud before listing (slower).
        #[arg(short, long)]
        refresh: bool,
    },
}

pub async fn run(config: ConfigHandle, action: FindMyAction, format: OutputFormat) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;

    match action {
        FindMyAction::Devices { refresh } => {
            let devices = if refresh {
                println!(
                    "  {} Refreshing device locations from iCloud (this may take a moment)...",
                    style("...").dim()
                );
                api.refresh_findmy_devices().await?
            } else {
                api.get_findmy_devices().await?
            };

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&devices).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if devices.is_empty() {
                        println!("No FindMy devices found.");
                    } else {
                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS)
                            .set_content_arrangement(ContentArrangement::Dynamic);

                        table.set_header(vec!["Name", "Model", "Battery", "Latitude", "Longitude", "Address"]);

                        for device in &devices {
                            let name = device.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown");
                            let model = device.get("deviceDisplayName")
                                .or_else(|| device.get("modelDisplayName"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("-");

                            let battery_level = device.get("batteryLevel")
                                .and_then(|v| v.as_f64())
                                .map(|b| format!("{:.0}%", b * 100.0))
                                .unwrap_or_else(|| "-".to_string());

                            let location = device.get("location");
                            let lat = location
                                .and_then(|l| l.get("latitude"))
                                .and_then(|v| v.as_f64())
                                .map(|v| format!("{:.6}", v))
                                .unwrap_or_else(|| "-".to_string());
                            let lon = location
                                .and_then(|l| l.get("longitude"))
                                .and_then(|v| v.as_f64())
                                .map(|v| format!("{:.6}", v))
                                .unwrap_or_else(|| "-".to_string());

                            let address = device.get("address")
                                .and_then(|a| {
                                    // Address can be a string or object with fields
                                    if let Some(s) = a.as_str() {
                                        Some(s.to_string())
                                    } else {
                                        // Try to build from mapItemFullAddress or streetAddress
                                        a.get("mapItemFullAddress")
                                            .or_else(|| a.get("streetAddress"))
                                            .and_then(|v| v.as_str())
                                            .map(String::from)
                                    }
                                })
                                .unwrap_or_else(|| {
                                    // Some responses put address fields at the location level
                                    location
                                        .and_then(|l| l.get("address"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string()
                                });

                            table.add_row(vec![
                                name.to_string(),
                                model.to_string(),
                                battery_level,
                                lat,
                                lon,
                                super::truncate(&address, 40),
                            ]);
                        }

                        println!("{table}");
                        println!("\n{} device(s) found.", devices.len());
                    }
                }
            }
        }
        FindMyAction::Friends { refresh } => {
            let friends = if refresh {
                println!(
                    "  {} Refreshing friend locations from iCloud (this may take a moment)...",
                    style("...").dim()
                );
                api.refresh_findmy_friends().await?
            } else {
                api.get_findmy_friends().await?
            };

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&friends).unwrap_or_default());
                }
                OutputFormat::Text => {
                    if friends.is_empty() {
                        println!("No FindMy friends found.");
                    } else {
                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS)
                            .set_content_arrangement(ContentArrangement::Dynamic);

                        table.set_header(vec!["Name", "Handle", "Latitude", "Longitude", "Address", "Status"]);

                        for friend in &friends {
                            let name = friend.get("firstName")
                                .and_then(|v| v.as_str())
                                .map(|first| {
                                    let last = friend.get("lastName")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    if last.is_empty() {
                                        first.to_string()
                                    } else {
                                        format!("{first} {last}")
                                    }
                                })
                                .unwrap_or_else(|| "Unknown".to_string());

                            let handle = friend.get("id")
                                .or_else(|| friend.get("handle"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("-");

                            let location = friend.get("location");
                            let lat = location
                                .and_then(|l| l.get("latitude"))
                                .and_then(|v| v.as_f64())
                                .map(|v| format!("{:.6}", v))
                                .unwrap_or_else(|| "-".to_string());
                            let lon = location
                                .and_then(|l| l.get("longitude"))
                                .and_then(|v| v.as_f64())
                                .map(|v| format!("{:.6}", v))
                                .unwrap_or_else(|| "-".to_string());

                            let address = location
                                .and_then(|l| l.get("address"))
                                .and_then(|a| {
                                    if let Some(s) = a.as_str() {
                                        Some(s.to_string())
                                    } else {
                                        a.get("mapItemFullAddress")
                                            .or_else(|| a.get("streetAddress"))
                                            .and_then(|v| v.as_str())
                                            .map(String::from)
                                    }
                                })
                                .unwrap_or_else(|| "-".to_string());

                            let status = friend.get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("-");

                            table.add_row(vec![
                                name,
                                handle.to_string(),
                                lat,
                                lon,
                                super::truncate(&address, 40),
                                status.to_string(),
                            ]);
                        }

                        println!("{table}");
                        println!("\n{} friend(s) found.", friends.len());
                    }
                }
            }
        }
    }

    Ok(())
}
