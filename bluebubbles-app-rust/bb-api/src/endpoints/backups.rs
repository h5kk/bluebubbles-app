//! Backup (theme and settings) endpoints.

use bb_core::error::BbResult;
use crate::client::ApiClient;
use crate::response::ServerResponse;

impl ApiClient {
    /// Get theme backup from server.
    pub async fn get_theme_backup(&self) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json("/backup/theme").await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Save a theme backup to the server.
    pub async fn save_theme_backup(&self, name: &str, data: &serde_json::Value) -> BbResult<()> {
        let body = serde_json::json!({ "name": name, "data": data });
        self.post("/backup/theme", &body).await?;
        Ok(())
    }

    /// Delete a theme backup from the server.
    pub async fn delete_theme_backup(&self, name: &str) -> BbResult<()> {
        let body = serde_json::json!({ "name": name });
        self.delete_with_body("/backup/theme", &body).await?;
        Ok(())
    }

    /// Get settings backup from server.
    pub async fn get_settings_backup(&self) -> BbResult<serde_json::Value> {
        let resp: ServerResponse = self.get_json("/backup/settings").await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Save a settings backup to the server.
    pub async fn save_settings_backup(&self, name: &str, data: &serde_json::Value) -> BbResult<()> {
        let body = serde_json::json!({ "name": name, "data": data });
        self.post("/backup/settings", &body).await?;
        Ok(())
    }

    /// Delete a settings backup from the server.
    pub async fn delete_settings_backup(&self, name: &str) -> BbResult<()> {
        let body = serde_json::json!({ "name": name });
        self.delete_with_body("/backup/settings", &body).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_backup_endpoints_exist() {
        // Compile-time verification
    }
}
