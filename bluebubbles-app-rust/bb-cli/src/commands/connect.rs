//! Connect command - establish a persistent connection to the server.

use console::style;
use dialoguer::{Input, Password};
use tracing::error;

use bb_core::config::ConfigHandle;
use bb_core::error::BbResult;
use bb_core::platform::Platform;
use bb_socket::{EventDispatcher, SocketManager};

/// Run the connect command.
pub async fn run(
    config: ConfigHandle,
    address: Option<String>,
    password: Option<String>,
    save_config: bool,
) -> BbResult<()> {
    // Determine address: arg > config > interactive prompt
    let addr = if let Some(a) = address {
        a
    } else {
        let current = config.read().await.server.address.clone();
        if current.is_empty() {
            Input::new()
                .with_prompt("Server address")
                .interact_text()
                .map_err(|e| bb_core::error::BbError::Internal(e.to_string()))?
        } else {
            current
        }
    };

    // Determine password: arg > config > interactive prompt
    let pass = if let Some(p) = password {
        p
    } else {
        let current = config.read().await.server.guid_auth_key.clone();
        if current.is_empty() {
            Password::new()
                .with_prompt("Server password")
                .interact()
                .map_err(|e| bb_core::error::BbError::Internal(e.to_string()))?
        } else {
            current
        }
    };

    // Apply to config
    {
        let mut cfg = config.write().await;
        cfg.server.address = bb_core::config::AppConfig::sanitize_server_address(&addr);
        cfg.server.guid_auth_key = pass;
    }

    let server_config = config.read().await.server.clone();

    if server_config.address.is_empty() {
        error!("no server address configured. Use --address or set it in config.");
        return Err(bb_core::error::BbError::MissingConfig("server address".into()));
    }

    println!(
        "{} Connecting to {}...",
        style("[1/3]").bold().dim(),
        server_config.address
    );

    // Create API client and test connection
    let api = super::create_api_client(&config).await?;
    match api.ping().await {
        Ok(true) => println!("  {} Server is reachable.", style("OK").green().bold()),
        Ok(false) => {
            error!("server returned unexpected response");
            return Err(bb_core::error::BbError::Http("ping failed".into()));
        }
        Err(e) => {
            println!("  {} Failed to reach server: {e}", style("FAIL").red().bold());
            return Err(e);
        }
    }

    // Get server info
    println!(
        "{} Fetching server info...",
        style("[2/3]").bold().dim(),
    );
    let info = api.server_info().await?;
    println!(
        "  Version:     {}",
        info.server_version.as_deref().unwrap_or("unknown")
    );
    println!(
        "  OS:          {}",
        info.os_version.as_deref().unwrap_or("unknown")
    );
    println!(
        "  Private API: {}",
        if info.private_api.unwrap_or(false) {
            style("enabled").green()
        } else {
            style("disabled").yellow()
        }
    );
    if let Some(ref ips) = info.local_ipv4s {
        println!("  Local IPs:   {}", ips.join(", "));
    }

    // Optionally save config to disk
    if save_config {
        let cfg = config.read().await;
        let path = Platform::config_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("config.toml");
        cfg.save_to_file(&path)?;
        println!("  {} Config saved to {}", style("OK").green(), path.display());
    }

    // Set up socket connection
    println!(
        "{} Establishing socket connection...",
        style("[3/3]").bold().dim(),
    );
    let dispatcher = EventDispatcher::new(256);
    let mut rx = dispatcher.subscribe();
    let manager = SocketManager::new(server_config, dispatcher, None);

    manager.connect().await?;
    println!(
        "  {} Connected. Listening for events... (Ctrl+C to stop)",
        style("OK").green().bold()
    );
    println!();

    // Listen for events
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        println!(
                            "  {} {}",
                            style(format!("[{}]", ev.event_type.as_str())).cyan(),
                            ev.data
                        );
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        println!(
                            "  {} Missed {n} events (slow consumer)",
                            style("WARN").yellow()
                        );
                    }
                    Err(_) => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n  Disconnecting...");
                manager.disconnect().await;
                break;
            }
        }
    }

    Ok(())
}
