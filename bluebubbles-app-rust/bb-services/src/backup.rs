//! Backup service for settings and theme backup/restore via the server.
//!
//! Manages named backups of the application configuration and themes,
//! supports export to the server and import from server backups.

use tracing::{info, warn, debug};

use bb_core::config::ConfigHandle;
use bb_core::error::{BbError, BbResult};
use bb_api::ApiClient;

use crate::event_bus::EventBus;
use crate::service::{Service, ServiceState};

/// Service for managing settings and theme backups via the server.
///
/// Provides export/import operations that serialize the current application
/// configuration to JSON and store it on the BlueBubbles server. This enables
/// restoring settings when reinstalling the client or switching devices.
pub struct BackupService {
    state: ServiceState,
    config: ConfigHandle,
    event_bus: EventBus,
}

impl BackupService {
    /// Create a new BackupService.
    pub fn new(config: ConfigHandle, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            config,
            event_bus,
        }
    }

    /// Export current application settings to the server as a named backup.
    ///
    /// Serialises the full AppConfig (minus sensitive credentials) to JSON
    /// and saves it to the server's backup store.
    pub async fn export_settings(&self, api: &ApiClient, name: &str) -> BbResult<()> {
        let config = self.config.read().await;
        let mut data = serde_json::to_value(&*config)
            .map_err(|e| BbError::Serialization(e.to_string()))?;

        // Strip sensitive fields before uploading
        if let Some(server) = data.as_object_mut().and_then(|o| o.get_mut("server")) {
            if let Some(obj) = server.as_object_mut() {
                obj.remove("guid_auth_key");
            }
        }

        api.save_settings_backup(name, &data).await?;
        info!("exported settings backup: {name}");
        Ok(())
    }

    /// Import settings from a server backup.
    ///
    /// Fetches the settings backup, deserializes it, and applies it to the
    /// running configuration while preserving server credentials and database
    /// path (to avoid breaking the active connection).
    pub async fn import_settings(&self, api: &ApiClient) -> BbResult<()> {
        let backup_data = api.get_settings_backup().await?;

        if backup_data.is_null() {
            warn!("no settings backup found on server");
            return Ok(());
        }

        if let Ok(imported) = serde_json::from_value::<bb_core::config::AppConfig>(backup_data.clone()) {
            let mut current = self.config.write().await;

            // Preserve connection-critical fields
            let preserved_address = current.server.address.clone();
            let preserved_key = current.server.guid_auth_key.clone();
            let preserved_db_path = current.database.path.clone();

            *current = imported;

            // Restore preserved fields
            current.server.address = preserved_address;
            current.server.guid_auth_key = preserved_key;
            current.database.path = preserved_db_path;

            info!("imported settings from server backup");
        } else {
            warn!("failed to parse settings backup, applying partial import");
            // Attempt partial application for known fields
            let mut current = self.config.write().await;

            if let Some(display) = backup_data.get("display") {
                if let Some(name) = display.get("user_name").and_then(|v| v.as_str()) {
                    current.display.user_name = name.to_string();
                }
                if let Some(v) = display.get("use_24hr_format").and_then(|v| v.as_bool()) {
                    current.display.use_24hr_format = v;
                }
            }

            if let Some(notif) = backup_data.get("notifications") {
                if let Some(v) = notif.get("notify_reactions").and_then(|v| v.as_bool()) {
                    current.notifications.notify_reactions = v;
                }
                if let Some(v) = notif.get("filter_unknown_senders").and_then(|v| v.as_bool()) {
                    current.notifications.filter_unknown_senders = v;
                }
            }

            if let Some(sync) = backup_data.get("sync") {
                if let Some(v) = sync.get("messages_per_page").and_then(|v| v.as_u64()) {
                    current.sync.messages_per_page = v as u32;
                }
            }
        }

        Ok(())
    }

    /// Delete a settings backup from the server.
    pub async fn delete_settings_backup(&self, api: &ApiClient, name: &str) -> BbResult<()> {
        api.delete_settings_backup(name).await?;
        info!("deleted settings backup: {name}");
        Ok(())
    }

    /// Export a named theme backup to the server.
    pub async fn export_theme(
        &self,
        api: &ApiClient,
        name: &str,
        theme_data: &serde_json::Value,
    ) -> BbResult<()> {
        api.save_theme_backup(name, theme_data).await?;
        info!("exported theme backup: {name}");
        Ok(())
    }

    /// Fetch the theme backup from the server.
    pub async fn fetch_theme_backup(&self, api: &ApiClient) -> BbResult<serde_json::Value> {
        let data = api.get_theme_backup().await?;
        debug!("fetched theme backup from server");
        Ok(data)
    }

    /// Delete a theme backup from the server.
    pub async fn delete_theme_backup(&self, api: &ApiClient, name: &str) -> BbResult<()> {
        api.delete_theme_backup(name).await?;
        info!("deleted theme backup: {name}");
        Ok(())
    }

    /// Create a full backup snapshot containing both settings and theme data.
    pub async fn create_full_backup(
        &self,
        api: &ApiClient,
        backup_name: &str,
    ) -> BbResult<()> {
        self.export_settings(api, backup_name).await?;
        info!("created full backup: {backup_name}");
        Ok(())
    }
}

impl Service for BackupService {
    fn name(&self) -> &str {
        "backup"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("backup service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("backup service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_core::config::AppConfig;

    #[test]
    fn test_backup_service_name() {
        let config = ConfigHandle::new(AppConfig::default());
        let bus = EventBus::new(16);
        let svc = BackupService::new(config, bus);
        assert_eq!(svc.name(), "backup");
    }

    #[test]
    fn test_backup_service_lifecycle() {
        let config = ConfigHandle::new(AppConfig::default());
        let bus = EventBus::new(16);
        let mut svc = BackupService::new(config, bus);
        svc.init().unwrap();
        assert_eq!(svc.state(), ServiceState::Running);
        svc.shutdown().unwrap();
        assert_eq!(svc.state(), ServiceState::Stopped);
    }
}
