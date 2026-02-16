//! Theme service for managing UI themes.
//!
//! Handles theme CRUD operations, built-in preset themes, applying themes,
//! and import/export via server backups.

use tracing::{info, debug, warn};
use bb_core::error::{BbError, BbResult};
use bb_models::{Database, ThemeStruct};
use bb_models::models::theme::{ThemeData, PRESET_OLED_DARK, PRESET_BRIGHT_WHITE};
use bb_api::ApiClient;

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Service for managing UI themes.
///
/// Provides CRUD for custom themes, built-in preset access, and server
/// backup import/export. Emits ThemeChanged events through the event bus
/// when the active theme is changed.
pub struct ThemeService {
    state: ServiceState,
    database: Database,
    event_bus: EventBus,
    /// Name of the currently active theme.
    active_theme: String,
}

impl ThemeService {
    /// Create a new ThemeService.
    pub fn new(database: Database, event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            database,
            event_bus,
            active_theme: PRESET_OLED_DARK.to_string(),
        }
    }

    /// List all themes (built-in and custom) from the local database.
    pub fn list_themes(&self) -> BbResult<Vec<ThemeStruct>> {
        let conn = self.database.conn()?;
        ThemeStruct::load_all(&conn)
    }

    /// Find a theme by name.
    pub fn find_theme(&self, name: &str) -> BbResult<Option<ThemeStruct>> {
        let conn = self.database.conn()?;
        ThemeStruct::find_by_name(&conn, name)
    }

    /// Find a theme by database ID.
    pub fn find_theme_by_id(&self, id: i64) -> BbResult<Option<ThemeStruct>> {
        let conn = self.database.conn()?;
        ThemeStruct::find_by_id(&conn, id)
    }

    /// Create or update a custom theme.
    ///
    /// Saves the theme to the database and returns its ID.
    pub fn save_theme(&self, mut theme: ThemeStruct) -> BbResult<i64> {
        let conn = self.database.conn()?;
        let id = theme.save(&conn)?;
        info!("saved theme: {} (id={})", theme.name, id);
        Ok(id)
    }

    /// Update an existing theme.
    pub fn update_theme(&self, theme: &ThemeStruct) -> BbResult<()> {
        let conn = self.database.conn()?;
        theme.update(&conn)?;
        debug!("updated theme: {}", theme.name);

        // If this is the active theme, notify listeners
        if theme.name == self.active_theme {
            self.event_bus.emit(AppEvent::ThemeChanged {
                theme_name: theme.name.clone(),
            });
        }
        Ok(())
    }

    /// Delete a custom theme by name.
    ///
    /// Preset themes cannot be deleted.
    pub fn delete_theme(&self, name: &str) -> BbResult<bool> {
        let conn = self.database.conn()?;
        let deleted = ThemeStruct::delete_by_name(&conn, name)?;
        if deleted {
            info!("deleted theme: {name}");
        }
        Ok(deleted)
    }

    /// Delete a theme by its database ID.
    pub fn delete_theme_by_id(&self, id: i64) -> BbResult<bool> {
        // Look up the theme to check if it is a preset
        let conn = self.database.conn()?;
        if let Some(theme) = ThemeStruct::find_by_id(&conn, id)? {
            if theme.is_preset() {
                return Err(BbError::Database("cannot delete preset themes".into()));
            }
        }
        let deleted = ThemeStruct::delete(&conn, id)?;
        if deleted {
            info!("deleted theme id={id}");
        }
        Ok(deleted)
    }

    /// Apply a theme by name, making it the active theme.
    ///
    /// Verifies the theme exists, then sets it as active and emits a
    /// ThemeChanged event for the UI to pick up.
    pub fn apply_theme(&mut self, name: &str) -> BbResult<ThemeStruct> {
        let conn = self.database.conn()?;
        let theme = ThemeStruct::find_by_name(&conn, name)?
            .ok_or_else(|| BbError::Database(format!("theme not found: {name}")))?;

        self.active_theme = name.to_string();
        info!("applied theme: {name}");

        self.event_bus.emit(AppEvent::ThemeChanged {
            theme_name: name.to_string(),
        });

        Ok(theme)
    }

    /// Get the currently active theme name.
    pub fn active_theme_name(&self) -> &str {
        &self.active_theme
    }

    /// Get the currently active theme data.
    pub fn active_theme(&self) -> BbResult<Option<ThemeStruct>> {
        self.find_theme(&self.active_theme)
    }

    /// Ensure the built-in preset themes exist in the database.
    ///
    /// Called during initialization. Creates OLED Dark and Bright White
    /// presets if they do not already exist.
    pub fn ensure_presets(&self) -> BbResult<()> {
        let conn = self.database.conn()?;

        // OLED Dark preset
        if ThemeStruct::find_by_name(&conn, PRESET_OLED_DARK)?.is_none() {
            let mut oled = ThemeStruct {
                id: None,
                name: PRESET_OLED_DARK.to_string(),
                gradient_bg: false,
                google_font: "Default".to_string(),
                theme_data: serde_json::json!({
                    "colorScheme": {
                        "brightness": 0,
                        "primary": 4278221567_u32,
                        "onPrimary": 4294967295_u32,
                        "background": 4278190080_u32,
                        "onBackground": 4294967295_u32,
                        "surface": 4278190080_u32,
                        "onSurface": 4294967295_u32
                    },
                    "textTheme": { "font": "Default" }
                }).to_string(),
            };
            oled.save(&conn)?;
            debug!("created preset: {PRESET_OLED_DARK}");
        }

        // Bright White preset
        if ThemeStruct::find_by_name(&conn, PRESET_BRIGHT_WHITE)?.is_none() {
            let mut white = ThemeStruct {
                id: None,
                name: PRESET_BRIGHT_WHITE.to_string(),
                gradient_bg: false,
                google_font: "Default".to_string(),
                theme_data: serde_json::json!({
                    "colorScheme": {
                        "brightness": 1,
                        "primary": 4278221567_u32,
                        "onPrimary": 4294967295_u32,
                        "background": 4294967295_u32,
                        "onBackground": 4278190080_u32,
                        "surface": 4294967295_u32,
                        "onSurface": 4278190080_u32
                    },
                    "textTheme": { "font": "Default" }
                }).to_string(),
            };
            white.save(&conn)?;
            debug!("created preset: {PRESET_BRIGHT_WHITE}");
        }

        Ok(())
    }

    /// Export all custom themes to the server as a backup.
    ///
    /// Only exports non-preset themes.
    pub async fn export_to_server(&self, api: &ApiClient) -> BbResult<usize> {
        let themes = self.list_themes()?;
        let mut exported = 0;

        for theme in &themes {
            if theme.is_preset() {
                continue;
            }

            let data = serde_json::json!({
                "name": theme.name,
                "gradientBg": theme.gradient_bg,
                "googleFont": theme.google_font,
                "themeData": theme.theme_data,
            });

            match api.save_theme_backup(&theme.name, &data).await {
                Ok(_) => {
                    exported += 1;
                    debug!("exported theme to server: {}", theme.name);
                }
                Err(e) => {
                    warn!("failed to export theme {}: {e}", theme.name);
                }
            }
        }

        info!("exported {exported} themes to server");
        Ok(exported)
    }

    /// Import themes from a server backup.
    ///
    /// Fetches the theme backup from the server and saves any themes
    /// that do not already exist locally.
    pub async fn import_from_server(&self, api: &ApiClient) -> BbResult<usize> {
        let backup = api.get_theme_backup().await?;
        let mut imported = 0;

        if let Some(themes_arr) = backup.as_array() {
            let conn = self.database.conn()?;

            for theme_json in themes_arr {
                let name = theme_json
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if name.is_empty() {
                    continue;
                }

                // Skip if already exists locally
                if ThemeStruct::find_by_name(&conn, &name)?.is_some() {
                    debug!("skipping existing theme from backup: {name}");
                    continue;
                }

                let gradient_bg = theme_json
                    .get("gradientBg")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let google_font = theme_json
                    .get("googleFont")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Default")
                    .to_string();
                let theme_data = theme_json
                    .get("themeData")
                    .cloned()
                    .map(|v| {
                        if v.is_string() {
                            v.as_str().unwrap_or("{}").to_string()
                        } else {
                            v.to_string()
                        }
                    })
                    .unwrap_or_else(|| "{}".to_string());

                let mut theme = ThemeStruct {
                    id: None,
                    name: name.clone(),
                    gradient_bg,
                    google_font,
                    theme_data,
                };

                match theme.save(&conn) {
                    Ok(_) => {
                        imported += 1;
                        debug!("imported theme from server: {name}");
                    }
                    Err(e) => {
                        warn!("failed to import theme {name}: {e}");
                    }
                }
            }
        } else if backup.is_object() {
            // Single theme object format
            let conn = self.database.conn()?;
            let name = backup
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !name.is_empty() && ThemeStruct::find_by_name(&conn, &name)?.is_none() {
                let gradient_bg = backup
                    .get("gradientBg")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let google_font = backup
                    .get("googleFont")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Default")
                    .to_string();
                let theme_data = backup
                    .get("themeData")
                    .cloned()
                    .map(|v| {
                        if v.is_string() {
                            v.as_str().unwrap_or("{}").to_string()
                        } else {
                            v.to_string()
                        }
                    })
                    .unwrap_or_else(|| "{}".to_string());

                let mut theme = ThemeStruct {
                    id: None,
                    name: name.clone(),
                    gradient_bg,
                    google_font,
                    theme_data,
                };

                if theme.save(&conn).is_ok() {
                    imported += 1;
                    debug!("imported single theme from server: {name}");
                }
            }
        }

        info!("imported {imported} themes from server");
        Ok(imported)
    }

    /// Delete a theme backup from the server.
    pub async fn delete_server_backup(&self, api: &ApiClient, name: &str) -> BbResult<()> {
        api.delete_theme_backup(name).await?;
        info!("deleted theme backup from server: {name}");
        Ok(())
    }

    /// Parse theme data from a ThemeStruct.
    pub fn parse_theme_data(theme: &ThemeStruct) -> Option<ThemeData> {
        theme.parsed_data()
    }
}

impl Service for ThemeService {
    fn name(&self) -> &str {
        "theme"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Initializing;
        self.ensure_presets()?;
        self.state = ServiceState::Running;
        info!("theme service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("theme service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventBus;

    fn create_test_db() -> Database {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let config = bb_core::config::DatabaseConfig::default();
        let db = Database::init(&path, &config).unwrap();
        std::mem::forget(dir);
        db
    }

    #[test]
    fn test_theme_service_name() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ThemeService::new(db, bus);
        assert_eq!(svc.name(), "theme");
    }

    #[test]
    fn test_ensure_presets() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ThemeService::new(db.clone(), bus);
        svc.ensure_presets().unwrap();

        let themes = svc.list_themes().unwrap();
        let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&PRESET_BRIGHT_WHITE));
        assert!(names.contains(&PRESET_OLED_DARK));
    }

    #[test]
    fn test_save_and_find_theme() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ThemeService::new(db, bus);

        let theme = ThemeStruct {
            id: None,
            name: "Custom Blue".to_string(),
            gradient_bg: true,
            google_font: "Roboto".to_string(),
            theme_data: r#"{"colorScheme":{"brightness":0,"primary":255}}"#.to_string(),
        };

        let id = svc.save_theme(theme).unwrap();
        assert!(id > 0);

        let found = svc.find_theme("Custom Blue").unwrap().unwrap();
        assert_eq!(found.name, "Custom Blue");
        assert!(found.gradient_bg);
    }

    #[test]
    fn test_delete_custom_theme() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ThemeService::new(db, bus);

        let theme = ThemeStruct {
            id: None,
            name: "Temp Theme".to_string(),
            gradient_bg: false,
            google_font: "Default".to_string(),
            theme_data: "{}".to_string(),
        };

        svc.save_theme(theme).unwrap();
        let deleted = svc.delete_theme("Temp Theme").unwrap();
        assert!(deleted);
        assert!(svc.find_theme("Temp Theme").unwrap().is_none());
    }

    #[test]
    fn test_cannot_delete_preset() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let svc = ThemeService::new(db, bus);
        svc.ensure_presets().unwrap();

        let result = svc.delete_theme(PRESET_OLED_DARK);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_theme() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        let mut svc = ThemeService::new(db, bus);
        svc.ensure_presets().unwrap();

        let theme = svc.apply_theme(PRESET_BRIGHT_WHITE).unwrap();
        assert_eq!(theme.name, PRESET_BRIGHT_WHITE);
        assert_eq!(svc.active_theme_name(), PRESET_BRIGHT_WHITE);

        // Check that ThemeChanged event was emitted
        let event = rx.try_recv().unwrap();
        match event {
            AppEvent::ThemeChanged { theme_name } => {
                assert_eq!(theme_name, PRESET_BRIGHT_WHITE);
            }
            _ => panic!("expected ThemeChanged event"),
        }
    }

    #[test]
    fn test_apply_nonexistent_theme() {
        let db = create_test_db();
        let bus = EventBus::new(16);
        let mut svc = ThemeService::new(db, bus);
        let result = svc.apply_theme("Nonexistent");
        assert!(result.is_err());
    }
}
