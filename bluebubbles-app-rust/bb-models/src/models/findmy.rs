//! Find My location models.
//!
//! Represents devices, friends, and location data from iCloud Find My service.

use serde::{Deserialize, Serialize};

/// A Find My location item (device or friend) from the iCloud API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindMyLocationItem {
    /// Unique identifier.
    #[serde(alias = "identifier", alias = "deviceDiscoveryId")]
    pub id: String,

    /// Display name (person or device name).
    pub name: Option<String>,

    /// Location information.
    pub location: Option<FindMyLocation>,

    /// Alternative location source (e.g., crowd-sourced).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crowd_sourced_location: Option<FindMyLocation>,

    /// Address information.
    pub address: Option<FindMyAddress>,

    /// Device status code (200 = online, 203 = locating, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_status: Option<serde_json::Value>,

    /// Battery level (0.0 - 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_level: Option<f64>,

    /// Battery status string or number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_status: Option<serde_json::Value>,

    /// Friend-specific fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// Device-specific fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_device_model: Option<String>,

    /// Whether this is a Mac device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mac: Option<bool>,

    /// Whether this is the current device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub this_device: Option<bool>,

    /// Whether lost mode is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lost_mode_enabled: Option<bool>,

    /// Friend-specific status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Whether friend location is currently being refreshed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locating_in_progress: Option<bool>,
}

impl FindMyLocationItem {
    /// Get the best available location (primary or crowd-sourced).
    pub fn best_location(&self) -> Option<&FindMyLocation> {
        self.location
            .as_ref()
            .filter(|loc| loc.latitude.is_some() && loc.longitude.is_some())
            .or_else(|| {
                self.crowd_sourced_location
                    .as_ref()
                    .filter(|loc| loc.latitude.is_some() && loc.longitude.is_some())
            })
    }

    /// Get the display name for this item.
    pub fn display_name(&self) -> String {
        if let Some(ref name) = self.name {
            return name.clone();
        }

        // For friends, combine first and last name
        if let (Some(ref first), Some(ref last)) = (&self.first_name, &self.last_name) {
            let combined = format!("{} {}", first, last).trim().to_string();
            if !combined.is_empty() {
                return combined;
            }
        }

        "Unknown".to_string()
    }

    /// Check if this item has a recent location.
    pub fn has_recent_location(&self) -> bool {
        if let Some(loc) = self.best_location() {
            if let Some(timestamp) = loc.time_stamp {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let age_ms = now.saturating_sub(timestamp);
                // Consider location recent if < 1 hour old
                return age_ms < 3600_000;
            }
        }
        false
    }

    /// Check if the device is online.
    pub fn is_online(&self) -> bool {
        if let Some(ref status) = self.device_status {
            if let Some(s) = status.as_str() {
                return s == "200" || s == "203";
            } else if let Some(n) = status.as_u64() {
                return n == 200 || n == 203;
            }
        }
        false
    }
}

/// A Find My device from the iCloud API.
/// This is a specialized view of FindMyLocationItem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindMyDevice {
    pub id: String,
    pub name: String,
    pub model: String,
    pub device_class: Option<String>,
    pub raw_device_model: Option<String>,
    pub battery_level: Option<f64>,
    pub battery_status: Option<String>,
    pub location: Option<FindMyLocation>,
    pub address: Option<FindMyAddress>,
    pub is_old_location: bool,
    pub is_online: bool,
    pub is_mac: bool,
    pub this_device: bool,
    pub lost_mode_enabled: bool,
}

impl From<FindMyLocationItem> for FindMyDevice {
    fn from(item: FindMyLocationItem) -> Self {
        // Extract values we need before moving any fields
        let location = item.best_location().cloned();
        let is_old_location = location
            .as_ref()
            .and_then(|l| l.is_old)
            .unwrap_or(false);

        let display_name = item.display_name();
        let is_online = item.is_online();

        let battery_status = item.battery_status.and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(n) = v.as_u64() {
                Some(n.to_string())
            } else {
                None
            }
        });

        Self {
            id: item.id,
            name: display_name,
            model: item
                .device_display_name
                .or(item.model_display_name)
                .or(item.device_model)
                .unwrap_or_else(|| "Unknown".to_string()),
            device_class: item.device_class,
            raw_device_model: item.raw_device_model,
            battery_level: item.battery_level,
            battery_status,
            location,
            address: item.address,
            is_old_location,
            is_online,
            is_mac: item.is_mac.unwrap_or(false),
            this_device: item.this_device.unwrap_or(false),
            lost_mode_enabled: item.lost_mode_enabled.unwrap_or(false),
        }
    }
}

/// Location information from Find My.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindMyLocation {
    /// Latitude in degrees.
    pub latitude: Option<f64>,

    /// Longitude in degrees.
    pub longitude: Option<f64>,

    /// Location timestamp (epoch milliseconds).
    pub time_stamp: Option<u64>,

    /// Position type (e.g., "GPS", "WiFi", "Cell").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_type: Option<String>,

    /// Whether the location is old/stale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_old: Option<bool>,

    /// Horizontal accuracy in meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,

    /// Vertical accuracy in meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_accuracy: Option<f64>,

    /// Altitude in meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,

    /// Whether the location is inaccurate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_inaccurate: Option<bool>,
}

/// Address information from Find My.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindMyAddress {
    /// Formatted address lines (array of strings).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_address_lines: Option<Vec<String>>,

    /// Full address string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_item_full_address: Option<String>,

    /// Short address string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_address: Option<String>,

    /// Long address string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_address: Option<String>,

    /// Street name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_name: Option<String>,

    /// Street address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_address: Option<String>,

    /// City/locality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality: Option<String>,

    /// State/province/region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_code: Option<String>,

    /// Country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Country code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

impl FindMyAddress {
    /// Get a formatted address string.
    pub fn formatted(&self) -> Option<String> {
        // Try formatted address lines first
        if let Some(ref lines) = self.formatted_address_lines {
            if !lines.is_empty() {
                return Some(lines.join(", "));
            }
        }

        // Try full address
        if let Some(ref full) = self.map_item_full_address {
            return Some(full.clone());
        }

        // Try short/long address
        if let Some(ref short) = self.short_address {
            return Some(short.clone());
        }
        if let Some(ref long) = self.long_address {
            return Some(long.clone());
        }

        // Build from components
        let mut parts = Vec::new();
        if let Some(ref street) = self.street_address.as_ref().or(self.street_name.as_ref()) {
            parts.push(street.as_str());
        }
        if let Some(ref locality) = self.locality {
            parts.push(locality);
        }
        if let Some(ref country) = self.country {
            parts.push(country);
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_findmy_device_display_name() {
        let item = FindMyLocationItem {
            id: "test123".to_string(),
            name: Some("iPhone".to_string()),
            location: None,
            crowd_sourced_location: None,
            address: None,
            device_status: None,
            battery_level: None,
            battery_status: None,
            first_name: None,
            last_name: None,
            device_display_name: None,
            model_display_name: None,
            device_model: None,
            device_class: None,
            raw_device_model: None,
            is_mac: None,
            this_device: None,
            lost_mode_enabled: None,
            status: None,
            locating_in_progress: None,
        };

        assert_eq!(item.display_name(), "iPhone");
    }

    #[test]
    fn test_findmy_friend_display_name() {
        let item = FindMyLocationItem {
            id: "friend123".to_string(),
            name: None,
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            location: None,
            crowd_sourced_location: None,
            address: None,
            device_status: None,
            battery_level: None,
            battery_status: None,
            device_display_name: None,
            model_display_name: None,
            device_model: None,
            device_class: None,
            raw_device_model: None,
            is_mac: None,
            this_device: None,
            lost_mode_enabled: None,
            status: None,
            locating_in_progress: None,
        };

        assert_eq!(item.display_name(), "John Doe");
    }

    #[test]
    fn test_is_online() {
        let mut item = FindMyLocationItem {
            id: "test".to_string(),
            name: None,
            device_status: Some(serde_json::json!("200")),
            location: None,
            crowd_sourced_location: None,
            address: None,
            battery_level: None,
            battery_status: None,
            first_name: None,
            last_name: None,
            device_display_name: None,
            model_display_name: None,
            device_model: None,
            device_class: None,
            raw_device_model: None,
            is_mac: None,
            this_device: None,
            lost_mode_enabled: None,
            status: None,
            locating_in_progress: None,
        };

        assert!(item.is_online());

        item.device_status = Some(serde_json::json!(200));
        assert!(item.is_online());

        item.device_status = Some(serde_json::json!("404"));
        assert!(!item.is_online());
    }

    #[test]
    fn test_address_formatting() {
        let addr = FindMyAddress {
            formatted_address_lines: Some(vec![
                "123 Main St".to_string(),
                "Springfield, IL".to_string(),
            ]),
            map_item_full_address: None,
            short_address: None,
            long_address: None,
            street_name: None,
            street_address: None,
            locality: None,
            state_code: None,
            country: None,
            country_code: None,
        };

        assert_eq!(addr.formatted(), Some("123 Main St, Springfield, IL".to_string()));
    }
}
