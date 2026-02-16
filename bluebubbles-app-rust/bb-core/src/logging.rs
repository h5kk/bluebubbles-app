//! Structured logging setup using the `tracing` ecosystem.
//!
//! Provides file rotation, configurable log levels, and both
//! human-readable and JSON output formats.

use std::path::Path;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling;

use crate::error::BbResult;

/// Initialize the global tracing subscriber with the given settings.
///
/// Sets up:
/// - Console output (stderr) with colors
/// - File output with daily rotation
/// - Configurable log level via the `level` parameter
///
/// # Arguments
/// * `level` - Log level string: "trace", "debug", "info", "warn", "error"
/// * `log_dir` - Directory for log files
/// * `json_output` - If true, use JSON format for file output
pub fn init_logging(level: &str, log_dir: &Path, json_output: bool) -> BbResult<LogGuard> {
    std::fs::create_dir_all(log_dir)?;

    let file_appender = rolling::daily(log_dir, "bluebubbles.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_new(level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact();

    if json_output {
        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .init();
    } else {
        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .init();
    }

    tracing::info!("logging initialized at level={level}, dir={}", log_dir.display());

    Ok(LogGuard { _guard: guard })
}

/// Guard that keeps the non-blocking log writer alive.
/// Drop this to flush and close the log file.
pub struct LogGuard {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

/// Initialize a minimal console-only logger for testing or simple CLI usage.
pub fn init_console_logging(level: &str) {
    let env_filter = EnvFilter::try_new(level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .compact(),
        )
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_logging_does_not_panic() {
        // Just verify it doesn't panic. Subsequent calls are no-ops.
        init_console_logging("debug");
    }
}
