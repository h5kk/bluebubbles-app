//! Mac control endpoints.

use bb_core::error::BbResult;
use crate::client::ApiClient;

impl ApiClient {
    /// Lock the Mac.
    pub async fn lock_mac(&self) -> BbResult<()> {
        self.post("/mac/lock", &serde_json::json!({})).await?;
        Ok(())
    }

    /// Restart the iMessage application on the Mac.
    pub async fn restart_imessage(&self) -> BbResult<()> {
        self.post("/mac/imessage/restart", &serde_json::json!({})).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mac_endpoints_exist() {
        // Compile-time verification
    }
}
