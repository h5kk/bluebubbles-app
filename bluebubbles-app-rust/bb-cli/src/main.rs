//! BlueBubbles CLI - Command-line interface for the BlueBubbles client.
//!
//! Provides a fully functional CLI for interacting with a BlueBubbles server
//! from the terminal. Useful for headless operation, scripting, debugging,
//! and as the first phase of the Rust rewrite (before the Tauri UI).

mod commands;

use clap::{Parser, Subcommand};
use tracing::info;

use bb_core::config::{AppConfig, ConfigHandle};
use bb_core::error::BbResult;
use bb_core::logging;
use bb_core::platform::Platform;

/// BlueBubbles - iMessage client for Windows, Linux, and macOS.
#[derive(Parser)]
#[command(
    name = "bluebubbles",
    version,
    about = "BlueBubbles iMessage client CLI",
    long_about = "A command-line interface for the BlueBubbles iMessage client.\n\
                   Connect to a BlueBubbles server to send and receive iMessages from any platform."
)]
struct Cli {
    /// Path to the configuration file.
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Enable verbose logging (debug level).
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (text, json).
    #[arg(short = 'f', long, global = true, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

/// Output format for CLI responses.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output.
    Text,
    /// JSON output for scripting.
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to the BlueBubbles server and start receiving events.
    Connect {
        /// Server address (overrides config).
        #[arg(short, long)]
        address: Option<String>,
        /// Server password / GUID auth key (overrides config).
        #[arg(short, long)]
        password: Option<String>,
        /// Save connection settings to config file after successful connect.
        #[arg(long)]
        save: bool,
    },
    /// Show the current connection and server status.
    Status,
    /// List and manage chats.
    Chats {
        #[command(subcommand)]
        action: commands::chats::ChatsAction,
    },
    /// List and manage messages.
    Messages {
        #[command(subcommand)]
        action: commands::messages::MessagesAction,
    },
    /// List and search contacts.
    Contacts {
        #[command(subcommand)]
        action: commands::contacts::ContactsAction,
    },
    /// Manage attachments.
    Attachments {
        #[command(subcommand)]
        action: commands::attachments::AttachmentsAction,
    },
    /// Run data synchronization.
    Sync {
        #[command(subcommand)]
        action: commands::sync::SyncAction,
    },
    /// View and modify settings.
    Settings {
        #[command(subcommand)]
        action: commands::settings::SettingsAction,
    },
    /// Server management commands.
    Server {
        #[command(subcommand)]
        action: commands::server::ServerAction,
    },
    /// View application logs.
    Logs {
        /// Number of log lines to show.
        #[arg(short = 'n', long, default_value = "50")]
        count: u32,
        /// Show server logs instead of client logs.
        #[arg(long)]
        server: bool,
        /// Follow log output in real-time (tail -f style).
        #[arg(short = 'F', long)]
        follow: bool,
        /// Filter log level (trace, debug, info, warn, error).
        #[arg(short, long)]
        level: Option<String>,
    },
    /// Database management commands.
    Db {
        #[command(subcommand)]
        action: commands::db::DbAction,
    },
    /// FindMy device and friend location commands.
    #[command(name = "findmy")]
    FindMy {
        #[command(subcommand)]
        action: commands::findmy::FindMyAction,
    },
    /// FaceTime call management commands.
    #[command(name = "facetime")]
    FaceTime {
        #[command(subcommand)]
        action: commands::facetime::FaceTimeAction,
    },
    /// Manage scheduled messages.
    Scheduled {
        #[command(subcommand)]
        action: commands::scheduled::ScheduledAction,
    },
    /// Server backup management (themes and settings).
    Backup {
        #[command(subcommand)]
        action: commands::backup::BackupAction,
    },
}

#[tokio::main]
async fn main() -> BbResult<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    let log_dir = Platform::data_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("logs");
    let _guard = logging::init_logging(log_level, &log_dir, false)?;

    // Load configuration
    let config_path = cli.config.as_deref().map(std::path::Path::new);
    let config = if let Some(path) = config_path {
        AppConfig::load_from_file(path)?
    } else {
        let default_path = Platform::config_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("config.toml");
        if default_path.exists() {
            AppConfig::load_from_file(&default_path)?
        } else {
            AppConfig::default()
        }
    };

    let config_handle = ConfigHandle::new(config);

    info!("BlueBubbles CLI v{}", bb_core::constants::APP_VERSION);

    // Dispatch to command handlers
    match cli.command {
        Commands::Connect { address, password, save } => {
            commands::connect::run(config_handle, address, password, save).await
        }
        Commands::Status => {
            commands::status::run(config_handle, cli.format).await
        }
        Commands::Chats { action } => {
            commands::chats::run(config_handle, action, cli.format).await
        }
        Commands::Messages { action } => {
            commands::messages::run(config_handle, action, cli.format).await
        }
        Commands::Contacts { action } => {
            commands::contacts::run(config_handle, action, cli.format).await
        }
        Commands::Attachments { action } => {
            commands::attachments::run(config_handle, action, cli.format).await
        }
        Commands::Sync { action } => {
            commands::sync::run(config_handle, action, cli.format).await
        }
        Commands::Settings { action } => {
            commands::settings::run(config_handle, action, cli.format).await
        }
        Commands::Server { action } => {
            commands::server::run(config_handle, action, cli.format).await
        }
        Commands::Logs { count, server, follow, level } => {
            commands::logs::run(config_handle, count, server, follow, level, cli.format).await
        }
        Commands::Db { action } => {
            commands::db::run(config_handle, action, cli.format).await
        }
        Commands::FindMy { action } => {
            commands::findmy::run(config_handle, action, cli.format).await
        }
        Commands::FaceTime { action } => {
            commands::facetime::run(config_handle, action, cli.format).await
        }
        Commands::Scheduled { action } => {
            commands::scheduled::run(config_handle, action, cli.format).await
        }
        Commands::Backup { action } => {
            commands::backup::run(config_handle, action, cli.format).await
        }
    }
}
