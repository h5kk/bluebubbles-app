//! Attachment commands.

use clap::Subcommand;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum AttachmentsAction {
    /// Get attachment info.
    Info {
        /// Attachment GUID.
        guid: String,
    },
    /// Download an attachment.
    Download {
        /// Attachment GUID.
        guid: String,
        /// Output file path.
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub async fn run(config: ConfigHandle, action: AttachmentsAction, format: OutputFormat) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;

    match action {
        AttachmentsAction::Info { guid } => {
            let info = api.get_attachment(&guid).await?;
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&info).unwrap_or_default());
                }
                OutputFormat::Text => {
                    println!("{}", style("Attachment Info").bold().underlined());
                    println!("  GUID:      {guid}");
                    if let Some(name) = info.get("transferName").and_then(|v| v.as_str()) {
                        println!("  Name:      {name}");
                    }
                    if let Some(mime) = info.get("mimeType").and_then(|v| v.as_str()) {
                        println!("  Type:      {mime}");
                    }
                    if let Some(bytes) = info.get("totalBytes").and_then(|v| v.as_i64()) {
                        println!("  Size:      {}", super::format_bytes(bytes as u64));
                    }
                    if let Some(created) = info.get("created").and_then(|v| v.as_str()) {
                        println!("  Created:   {created}");
                    }
                    if let Some(width) = info.get("width").and_then(|v| v.as_i64()) {
                        if let Some(height) = info.get("height").and_then(|v| v.as_i64()) {
                            println!("  Dims:      {}x{}", width, height);
                        }
                    }
                }
            }
        }
        AttachmentsAction::Download { guid, output } => {
            // Get attachment info first for filename and size
            let info = api.get_attachment(&guid).await.ok();
            let default_name = info.as_ref()
                .and_then(|i| i.get("transferName").and_then(|v| v.as_str()))
                .unwrap_or("attachment");
            let total_bytes = info.as_ref()
                .and_then(|i| i.get("totalBytes").and_then(|v| v.as_u64()))
                .unwrap_or(0);

            let path = output.unwrap_or_else(|| default_name.to_string());

            // Set up progress bar
            let pb = if total_bytes > 0 {
                let pb = ProgressBar::new(total_bytes);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("  Downloading [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .unwrap_or_else(|_| ProgressStyle::default_bar())
                        .progress_chars("=>-"),
                );
                pb
            } else {
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template("  Downloading {spinner} {bytes}")
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                );
                pb
            };

            let pb_clone = pb.clone();
            let bytes = api.download_attachment_with_progress(
                &guid,
                true,
                move |downloaded, _total| {
                    pb_clone.set_position(downloaded);
                },
            ).await?;

            pb.finish_and_clear();

            std::fs::write(&path, &bytes)?;
            println!(
                "  {} Saved to {} ({})",
                style("OK").green().bold(),
                path,
                super::format_bytes(bytes.len() as u64)
            );
        }
    }

    Ok(())
}
