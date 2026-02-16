//! FaceTime commands - answer and leave FaceTime calls.

use clap::Subcommand;
use console::style;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use crate::OutputFormat;

#[derive(Subcommand)]
pub enum FaceTimeAction {
    /// Answer an incoming FaceTime call and get the join link.
    Answer {
        /// UUID of the FaceTime call to answer.
        call_uuid: String,
    },
    /// Leave (hang up) a FaceTime call.
    Leave {
        /// UUID of the FaceTime call to leave.
        call_uuid: String,
    },
}

pub async fn run(config: ConfigHandle, action: FaceTimeAction, format: OutputFormat) -> BbResult<()> {
    let api = super::create_api_client(&config).await?;

    match action {
        FaceTimeAction::Answer { call_uuid } => {
            println!(
                "  {} Answering FaceTime call {}...",
                style("...").dim(),
                style(&call_uuid).dim()
            );

            let link = api.answer_facetime(&call_uuid).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "call_uuid": call_uuid,
                        "answered": true,
                        "link": link,
                    }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} FaceTime call answered.",
                        style("OK").green().bold()
                    );
                    if let Some(ref join_link) = link {
                        println!("  Join link: {join_link}");
                    }
                }
            }
        }
        FaceTimeAction::Leave { call_uuid } => {
            println!(
                "  {} Leaving FaceTime call {}...",
                style("...").dim(),
                style(&call_uuid).dim()
            );

            api.leave_facetime(&call_uuid).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "call_uuid": call_uuid,
                        "left": true,
                    }));
                }
                OutputFormat::Text => {
                    println!(
                        "  {} Left FaceTime call {}.",
                        style("OK").green().bold(),
                        call_uuid
                    );
                }
            }
        }
    }

    Ok(())
}
