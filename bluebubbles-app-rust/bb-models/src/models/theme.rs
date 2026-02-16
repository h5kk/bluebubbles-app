//! Theme entity model with full Material 3 color scheme and text theme support.

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Row};
use bb_core::error::{BbError, BbResult};
use std::collections::HashMap;

/// Represents a UI theme definition.
///
/// Themes are stored as JSON blobs containing color scheme data,
/// font settings, and other visual configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeStruct {
    pub id: Option<i64>,
    pub name: String,
    pub gradient_bg: bool,
    pub google_font: String,
    /// JSON-encoded theme data (colors, fonts, etc).
    pub theme_data: String,
}

/// Full parsed theme color scheme with all 23 customizable color slots + 2 SMS slots.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ColorScheme {
    /// 0 = dark, 1 = light
    #[serde(default)]
    pub brightness: i32,
    #[serde(default)]
    pub primary: u32,
    #[serde(default)]
    pub on_primary: u32,
    #[serde(default)]
    pub primary_container: u32,
    #[serde(default)]
    pub on_primary_container: u32,
    #[serde(default)]
    pub secondary: u32,
    #[serde(default)]
    pub on_secondary: u32,
    #[serde(default)]
    pub secondary_container: u32,
    #[serde(default)]
    pub on_secondary_container: u32,
    #[serde(default)]
    pub tertiary: u32,
    #[serde(default)]
    pub on_tertiary: u32,
    #[serde(default)]
    pub tertiary_container: u32,
    #[serde(default)]
    pub on_tertiary_container: u32,
    #[serde(default)]
    pub error: u32,
    #[serde(default)]
    pub on_error: u32,
    #[serde(default)]
    pub error_container: u32,
    #[serde(default)]
    pub on_error_container: u32,
    #[serde(default)]
    pub background: u32,
    #[serde(default)]
    pub on_background: u32,
    #[serde(default)]
    pub surface: u32,
    #[serde(default)]
    pub on_surface: u32,
    #[serde(default)]
    pub surface_variant: u32,
    #[serde(default)]
    pub on_surface_variant: u32,
    #[serde(default)]
    pub outline: u32,
    #[serde(default)]
    pub shadow: u32,
    #[serde(default)]
    pub inverse_surface: u32,
    #[serde(default)]
    pub on_inverse_surface: u32,
    #[serde(default)]
    pub inverse_primary: u32,
    /// SMS bubble color.
    #[serde(default)]
    pub sms_bubble: u32,
    /// Text/icons on SMS bubble.
    #[serde(default)]
    pub on_sms_bubble: u32,
}

/// A single text style entry in the theme.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TextStyleEntry {
    #[serde(default)]
    pub color: Option<u32>,
    #[serde(default)]
    pub font_weight: Option<i32>,
    #[serde(default)]
    pub font_size: Option<f64>,
}

/// Text theme configuration with all 7 text size slots.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TextTheme {
    pub font: Option<String>,
    pub title_large: Option<TextStyleEntry>,
    pub body_large: Option<TextStyleEntry>,
    pub body_medium: Option<TextStyleEntry>,
    pub body_small: Option<TextStyleEntry>,
    pub label_large: Option<TextStyleEntry>,
    pub label_small: Option<TextStyleEntry>,
    pub bubble_text: Option<TextStyleEntry>,
}

/// Parsed theme data containing all visual properties.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThemeData {
    pub color_scheme: Option<ColorScheme>,
    pub text_theme: Option<TextTheme>,
}

/// Default text sizes for the 7 slots.
pub fn default_text_sizes() -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();
    m.insert("titleLarge", 22.0);
    m.insert("bodyLarge", 16.0);
    m.insert("bodyMedium", 14.0);
    m.insert("bodySmall", 12.0);
    m.insert("labelLarge", 14.0);
    m.insert("labelSmall", 11.0);
    m.insert("bubbleText", 15.0);
    m
}

/// Preset theme names.
pub const PRESET_OLED_DARK: &str = "OLED Dark";
pub const PRESET_BRIGHT_WHITE: &str = "Bright White";

impl ThemeStruct {
    /// Construct a ThemeStruct from a database row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            gradient_bg: row.get::<_, i32>("gradient_bg")? != 0,
            google_font: row.get("google_font")?,
            theme_data: row.get("theme_data")?,
        })
    }

    // ─── Static finders ──────────────────────────────────────────────────

    /// Find a theme by its local database ID.
    pub fn find_by_id(conn: &Connection, id: i64) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM themes WHERE id = ?1", [id], Self::from_row) {
            Ok(t) => Ok(Some(t)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Find a theme by name.
    pub fn find_by_name(conn: &Connection, name: &str) -> BbResult<Option<Self>> {
        match conn.query_row("SELECT * FROM themes WHERE name = ?1", [name], Self::from_row) {
            Ok(t) => Ok(Some(t)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BbError::Database(e.to_string())),
        }
    }

    /// Delete a theme by its local database ID.
    pub fn delete(conn: &Connection, id: i64) -> BbResult<bool> {
        let changed = conn
            .execute("DELETE FROM themes WHERE id = ?1", [id])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }

    // ─── Parsing ─────────────────────────────────────────────────────────

    /// Parse the JSON theme data.
    pub fn parsed_data(&self) -> Option<ThemeData> {
        serde_json::from_str(&self.theme_data).ok()
    }

    /// Whether this is a dark theme.
    pub fn is_dark(&self) -> bool {
        self.parsed_data()
            .and_then(|d| d.color_scheme)
            .map_or(false, |cs| cs.brightness == 0)
    }

    /// Whether this is a preset theme.
    pub fn is_preset(&self) -> bool {
        self.name == PRESET_OLED_DARK || self.name == PRESET_BRIGHT_WHITE
    }

    /// Get a map of all customizable color slots from this theme.
    pub fn colors(&self) -> HashMap<String, u32> {
        let mut map = HashMap::new();
        if let Some(data) = self.parsed_data() {
            if let Some(cs) = data.color_scheme {
                map.insert("primary".to_string(), cs.primary);
                map.insert("onPrimary".to_string(), cs.on_primary);
                map.insert("primaryContainer".to_string(), cs.primary_container);
                map.insert("onPrimaryContainer".to_string(), cs.on_primary_container);
                map.insert("secondary".to_string(), cs.secondary);
                map.insert("onSecondary".to_string(), cs.on_secondary);
                map.insert("tertiaryContainer".to_string(), cs.tertiary_container);
                map.insert("onTertiaryContainer".to_string(), cs.on_tertiary_container);
                map.insert("error".to_string(), cs.error);
                map.insert("onError".to_string(), cs.on_error);
                map.insert("errorContainer".to_string(), cs.error_container);
                map.insert("onErrorContainer".to_string(), cs.on_error_container);
                map.insert("background".to_string(), cs.background);
                map.insert("onBackground".to_string(), cs.on_background);
                map.insert("surface".to_string(), cs.surface);
                map.insert("onSurface".to_string(), cs.on_surface);
                map.insert("surfaceVariant".to_string(), cs.surface_variant);
                map.insert("onSurfaceVariant".to_string(), cs.on_surface_variant);
                map.insert("inverseSurface".to_string(), cs.inverse_surface);
                map.insert("onInverseSurface".to_string(), cs.on_inverse_surface);
                map.insert("outline".to_string(), cs.outline);
                map.insert("smsBubble".to_string(), cs.sms_bubble);
                map.insert("onSmsBubble".to_string(), cs.on_sms_bubble);
            }
        }
        map
    }

    /// Get a map of current text sizes.
    pub fn text_sizes(&self) -> HashMap<String, f64> {
        let defaults = default_text_sizes();
        let mut map: HashMap<String, f64> = defaults.iter().map(|(k, v)| (k.to_string(), *v)).collect();

        if let Some(data) = self.parsed_data() {
            if let Some(tt) = data.text_theme {
                if let Some(ref e) = tt.title_large { if let Some(s) = e.font_size { map.insert("titleLarge".to_string(), s); } }
                if let Some(ref e) = tt.body_large { if let Some(s) = e.font_size { map.insert("bodyLarge".to_string(), s); } }
                if let Some(ref e) = tt.body_medium { if let Some(s) = e.font_size { map.insert("bodyMedium".to_string(), s); } }
                if let Some(ref e) = tt.body_small { if let Some(s) = e.font_size { map.insert("bodySmall".to_string(), s); } }
                if let Some(ref e) = tt.label_large { if let Some(s) = e.font_size { map.insert("labelLarge".to_string(), s); } }
                if let Some(ref e) = tt.label_small { if let Some(s) = e.font_size { map.insert("labelSmall".to_string(), s); } }
                if let Some(ref e) = tt.bubble_text { if let Some(s) = e.font_size { map.insert("bubbleText".to_string(), s); } }
            }
        }
        map
    }

    // ─── Persistence ─────────────────────────────────────────────────────

    /// Save or update this theme in the database. Returns the local database ID.
    pub fn save(&mut self, conn: &Connection) -> BbResult<i64> {
        conn.execute(
            "INSERT INTO themes (name, gradient_bg, google_font, theme_data)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(name) DO UPDATE SET
                gradient_bg = excluded.gradient_bg,
                google_font = excluded.google_font,
                theme_data = excluded.theme_data",
            params![
                self.name,
                self.gradient_bg as i32,
                self.google_font,
                self.theme_data,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        if id > 0 {
            self.id = Some(id);
        } else {
            let existing_id: i64 = conn
                .query_row("SELECT id FROM themes WHERE name = ?1", [&self.name], |row| {
                    row.get(0)
                })
                .map_err(|e| BbError::Database(e.to_string()))?;
            self.id = Some(existing_id);
        }

        Ok(self.id.unwrap_or(0))
    }

    /// Update this theme in the database.
    pub fn update(&self, conn: &Connection) -> BbResult<()> {
        let id = self.id.ok_or_else(|| BbError::Database("theme has no id for update".into()))?;
        conn.execute(
            "UPDATE themes SET
                gradient_bg = ?1, google_font = ?2, theme_data = ?3
            WHERE id = ?4",
            params![
                self.gradient_bg as i32,
                self.google_font,
                self.theme_data,
                id,
            ],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(())
    }

    /// Load all themes from the database.
    pub fn load_all(conn: &Connection) -> BbResult<Vec<Self>> {
        let mut stmt = conn
            .prepare("SELECT * FROM themes ORDER BY name")
            .map_err(|e| BbError::Database(e.to_string()))?;

        let themes = stmt
            .query_map([], Self::from_row)
            .map_err(|e| BbError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(themes)
    }

    /// Delete a theme by name. Blocks deletion of preset themes.
    pub fn delete_by_name(conn: &Connection, name: &str) -> BbResult<bool> {
        if name == PRESET_OLED_DARK || name == PRESET_BRIGHT_WHITE {
            return Err(BbError::Database("cannot delete preset themes".into()));
        }
        let changed = conn
            .execute("DELETE FROM themes WHERE name = ?1", [name])
            .map_err(|e| BbError::Database(e.to_string()))?;
        Ok(changed > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_parse_data() {
        let theme = ThemeStruct {
            id: None,
            name: "Test Dark".into(),
            gradient_bg: false,
            google_font: "Default".into(),
            theme_data: r#"{"colorScheme":{"brightness":0,"primary":4278221567,"onPrimary":4294967295,"background":4278190080,"onBackground":4294967295,"surface":4278190080,"onSurface":4294967295},"textTheme":{"font":"Default"}}"#.into(),
        };
        assert!(theme.is_dark());
        let data = theme.parsed_data().unwrap();
        assert_eq!(data.color_scheme.unwrap().brightness, 0);
    }

    #[test]
    fn test_theme_light() {
        let theme = ThemeStruct {
            id: None,
            name: "Light".into(),
            gradient_bg: false,
            google_font: "Default".into(),
            theme_data: r#"{"colorScheme":{"brightness":1,"primary":4278221567,"onPrimary":4294967295,"background":4294967295,"onBackground":4278190080,"surface":4294967295,"onSurface":4278190080}}"#.into(),
        };
        assert!(!theme.is_dark());
    }

    #[test]
    fn test_is_preset() {
        let theme = ThemeStruct {
            id: None,
            name: "OLED Dark".into(),
            gradient_bg: false,
            google_font: "Default".into(),
            theme_data: "{}".into(),
        };
        assert!(theme.is_preset());

        let custom = ThemeStruct {
            id: None,
            name: "My Theme".into(),
            gradient_bg: false,
            google_font: "Default".into(),
            theme_data: "{}".into(),
        };
        assert!(!custom.is_preset());
    }

    #[test]
    fn test_default_text_sizes() {
        let sizes = default_text_sizes();
        assert_eq!(sizes.len(), 7);
        assert_eq!(sizes["bubbleText"], 15.0);
    }

    #[test]
    fn test_theme_colors() {
        let theme = ThemeStruct {
            id: None,
            name: "Test".into(),
            gradient_bg: false,
            google_font: "Default".into(),
            theme_data: r#"{"colorScheme":{"brightness":0,"primary":100,"onPrimary":200,"smsBubble":300,"onSmsBubble":400}}"#.into(),
        };
        let colors = theme.colors();
        assert_eq!(colors.get("primary"), Some(&100));
        assert_eq!(colors.get("smsBubble"), Some(&300));
    }

    #[test]
    fn test_text_sizes_override() {
        let theme = ThemeStruct {
            id: None,
            name: "Test".into(),
            gradient_bg: false,
            google_font: "Default".into(),
            theme_data: r#"{"textTheme":{"font":"Default","bubbleText":{"fontSize":18.0}}}"#.into(),
        };
        let sizes = theme.text_sizes();
        assert_eq!(sizes["bubbleText"], 18.0);
        assert_eq!(sizes["titleLarge"], 22.0); // default
    }
}
