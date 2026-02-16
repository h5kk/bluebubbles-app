//! Platform detection and OS-specific utilities.

use std::path::PathBuf;
use crate::error::{BbError, BbResult};

/// Detected operating system platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    /// Detect the current platform at compile time.
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        }
    }

    /// Get the platform-specific application data directory.
    ///
    /// - Windows: `%APPDATA%/BlueBubbles`
    /// - macOS: `~/Library/Application Support/BlueBubbles`
    /// - Linux: `~/.local/share/BlueBubbles`
    pub fn data_dir() -> BbResult<PathBuf> {
        let base = dirs::data_dir()
            .ok_or_else(|| BbError::Config("could not determine data directory".into()))?;
        Ok(base.join("BlueBubbles"))
    }

    /// Get the platform-specific configuration directory.
    ///
    /// - Windows: `%APPDATA%/BlueBubbles`
    /// - macOS: `~/Library/Application Support/BlueBubbles`
    /// - Linux: `~/.config/BlueBubbles`
    pub fn config_dir() -> BbResult<PathBuf> {
        let base = dirs::config_dir()
            .ok_or_else(|| BbError::Config("could not determine config directory".into()))?;
        Ok(base.join("BlueBubbles"))
    }

    /// Get the platform-specific cache directory.
    pub fn cache_dir() -> BbResult<PathBuf> {
        let base = dirs::cache_dir()
            .ok_or_else(|| BbError::Config("could not determine cache directory".into()))?;
        Ok(base.join("BlueBubbles"))
    }

    /// Get a human-readable platform name.
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOs => "macOS",
            Platform::Linux => "Linux",
        }
    }

    /// Get the system hostname for device name generation.
    pub fn hostname() -> String {
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "bluebubbles-client".to_string())
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let p = Platform::current();
        // Just verify it returns one of the expected values
        assert!(matches!(p, Platform::Windows | Platform::MacOs | Platform::Linux));
    }

    #[test]
    fn test_platform_name() {
        assert_eq!(Platform::Windows.name(), "Windows");
        assert_eq!(Platform::MacOs.name(), "macOS");
        assert_eq!(Platform::Linux.name(), "Linux");
    }

    #[test]
    fn test_data_dir_exists() {
        // Should succeed on any desktop platform
        let dir = Platform::data_dir();
        assert!(dir.is_ok());
    }
}
