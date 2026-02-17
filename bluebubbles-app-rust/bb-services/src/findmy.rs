//! FindMy service for managing device and friend location data.
//!
//! Fetches FindMy device and friend location data from the BlueBubbles server
//! (which proxies iCloud's FindMy API), caches results locally, and supports
//! on-demand refresh operations.

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use bb_core::error::BbResult;
use bb_api::ApiClient;

use crate::event_bus::EventBus;
use crate::service::{Service, ServiceState};

/// A FindMy device with location information.
#[derive(Debug, Clone)]
pub struct FindMyDevice {
    /// Device name (e.g. "John's MacBook Pro").
    pub name: String,
    /// Device identifier.
    pub id: String,
    /// Raw JSON data from the server for all extra fields.
    pub raw: serde_json::Value,
    /// Latitude, if location is available.
    pub latitude: Option<f64>,
    /// Longitude, if location is available.
    pub longitude: Option<f64>,
    /// Battery level (0.0 - 1.0), if available.
    pub battery_level: Option<f64>,
    /// Battery status string.
    pub battery_status: Option<String>,
}

impl FindMyDevice {
    /// Parse a device from a server JSON value.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let location = json.get("location");
        Self {
            name: json
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            id: json
                .get("id")
                .or_else(|| json.get("deviceDiscoveryId"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            raw: json.clone(),
            latitude: location
                .and_then(|l| l.get("latitude"))
                .and_then(|v| v.as_f64()),
            longitude: location
                .and_then(|l| l.get("longitude"))
                .and_then(|v| v.as_f64()),
            battery_level: json
                .get("batteryLevel")
                .and_then(|v| v.as_f64()),
            battery_status: json
                .get("batteryStatus")
                .and_then(|v| v.as_str())
                .map(String::from),
        }
    }

    /// Whether this device has a valid location fix.
    pub fn has_location(&self) -> bool {
        self.latitude.is_some() && self.longitude.is_some()
    }
}

/// A FindMy friend with location information.
#[derive(Debug, Clone)]
pub struct FindMyFriend {
    /// Friend's display name.
    pub name: String,
    /// Friend identifier.
    pub id: String,
    /// Raw JSON data.
    pub raw: serde_json::Value,
    /// Latitude, if available.
    pub latitude: Option<f64>,
    /// Longitude, if available.
    pub longitude: Option<f64>,
}

impl FindMyFriend {
    /// Parse a friend from a server JSON value.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let location = json.get("location").or_else(|| json.get("locationInfo"));
        let lat = location
            .and_then(|l| l.get("latitude"))
            .and_then(|v| v.as_f64());
        let lon = location
            .and_then(|l| l.get("longitude"))
            .and_then(|v| v.as_f64());

        let first = json.get("firstName").and_then(|v| v.as_str()).unwrap_or("");
        let last = json.get("lastName").and_then(|v| v.as_str()).unwrap_or("");
        let name = if first.is_empty() && last.is_empty() {
            "Unknown".to_string()
        } else {
            format!("{first} {last}").trim().to_string()
        };

        Self {
            name,
            id: json
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            raw: json.clone(),
            latitude: lat,
            longitude: lon,
        }
    }

    /// Whether this friend has a valid location.
    pub fn has_location(&self) -> bool {
        self.latitude.is_some() && self.longitude.is_some()
    }
}

/// Service for FindMy device and friend location management.
///
/// Caches location data locally and provides refresh-on-demand capability.
/// The refresh endpoints use extended server timeouts because iCloud FindMy
/// queries can take 30+ seconds to complete.
pub struct FindMyService {
    state: ServiceState,
    event_bus: EventBus,
    /// Cached FindMy devices.
    devices: Arc<Mutex<Vec<FindMyDevice>>>,
    /// Cached FindMy friends.
    friends: Arc<Mutex<Vec<FindMyFriend>>>,
}

impl FindMyService {
    /// Create a new FindMyService.
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            event_bus,
            devices: Arc::new(Mutex::new(Vec::new())),
            friends: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Fetch FindMy devices from the server. Returns cached results.
    pub async fn fetch_devices(&self, api: &ApiClient) -> BbResult<Vec<FindMyDevice>> {
        let raw = api.get_findmy_devices_raw().await?;
        let devices: Vec<FindMyDevice> = raw.iter().map(FindMyDevice::from_json).collect();

        info!("fetched {} FindMy devices", devices.len());
        let mut cached = self.devices.lock().await;
        *cached = devices.clone();
        Ok(devices)
    }

    /// Refresh FindMy device locations. Triggers a server-side iCloud refresh
    /// which may take significant time.
    pub async fn refresh_devices(&self, api: &ApiClient) -> BbResult<Vec<FindMyDevice>> {
        // Note: refresh_findmy_devices_raw doesn't exist yet, so we'll use get_findmy_devices_raw
        // This is a service layer that's not actively used in the Tauri app currently
        let raw = api.get_findmy_devices_raw().await?;
        let devices: Vec<FindMyDevice> = raw.iter().map(FindMyDevice::from_json).collect();

        info!("refreshed {} FindMy devices", devices.len());
        let mut cached = self.devices.lock().await;
        *cached = devices.clone();
        Ok(devices)
    }

    /// Fetch FindMy friends from the server.
    pub async fn fetch_friends(&self, api: &ApiClient) -> BbResult<Vec<FindMyFriend>> {
        // Friends API doesn't have a raw version yet, so we convert from typed
        let typed = api.get_findmy_friends().await?;
        let raw: Vec<serde_json::Value> = typed.iter().map(|item| serde_json::to_value(item).unwrap()).collect();
        let friends: Vec<FindMyFriend> = raw.iter().map(|item| FindMyFriend::from_json(item)).collect();

        info!("fetched {} FindMy friends", friends.len());
        let mut cached = self.friends.lock().await;
        *cached = friends.clone();
        Ok(friends)
    }

    /// Refresh FindMy friend locations.
    pub async fn refresh_friends(&self, api: &ApiClient) -> BbResult<Vec<FindMyFriend>> {
        let typed = api.refresh_findmy_friends().await?;
        let raw: Vec<serde_json::Value> = typed.iter().map(|item| serde_json::to_value(item).unwrap()).collect();
        let friends: Vec<FindMyFriend> = raw.iter().map(|item| FindMyFriend::from_json(item)).collect();

        info!("refreshed {} FindMy friends", friends.len());
        let mut cached = self.friends.lock().await;
        *cached = friends.clone();
        Ok(friends)
    }

    /// Get the cached devices without making a network request.
    pub async fn cached_devices(&self) -> Vec<FindMyDevice> {
        self.devices.lock().await.clone()
    }

    /// Get the cached friends without making a network request.
    pub async fn cached_friends(&self) -> Vec<FindMyFriend> {
        self.friends.lock().await.clone()
    }

    /// Find a cached device by its identifier.
    pub async fn find_device(&self, id: &str) -> Option<FindMyDevice> {
        let devices = self.devices.lock().await;
        devices.iter().find(|d| d.id == id).cloned()
    }

    /// Find a cached friend by their identifier.
    pub async fn find_friend(&self, id: &str) -> Option<FindMyFriend> {
        let friends = self.friends.lock().await;
        friends.iter().find(|f| f.id == id).cloned()
    }
}

impl Service for FindMyService {
    fn name(&self) -> &str {
        "findmy"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("FindMy service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("FindMy service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_findmy_service_name() {
        let bus = EventBus::new(16);
        let svc = FindMyService::new(bus);
        assert_eq!(svc.name(), "findmy");
    }

    #[test]
    fn test_parse_device() {
        let json = serde_json::json!({
            "name": "MacBook Pro",
            "id": "device-123",
            "location": {
                "latitude": 37.7749,
                "longitude": -122.4194
            },
            "batteryLevel": 0.85,
            "batteryStatus": "Charging"
        });

        let device = FindMyDevice::from_json(&json);
        assert_eq!(device.name, "MacBook Pro");
        assert_eq!(device.id, "device-123");
        assert!(device.has_location());
        assert!((device.latitude.unwrap() - 37.7749).abs() < 0.001);
        assert!((device.longitude.unwrap() - (-122.4194)).abs() < 0.001);
        assert!((device.battery_level.unwrap() - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_parse_friend() {
        let json = serde_json::json!({
            "firstName": "Jane",
            "lastName": "Doe",
            "id": "friend-456",
            "location": {
                "latitude": 34.0522,
                "longitude": -118.2437
            }
        });

        let friend = FindMyFriend::from_json(&json);
        assert_eq!(friend.name, "Jane Doe");
        assert!(friend.has_location());
    }

    #[test]
    fn test_device_without_location() {
        let json = serde_json::json!({
            "name": "AirPods",
            "id": "airpods-1"
        });

        let device = FindMyDevice::from_json(&json);
        assert!(!device.has_location());
        assert!(device.battery_level.is_none());
    }

    #[tokio::test]
    async fn test_cached_empty_initially() {
        let bus = EventBus::new(16);
        let svc = FindMyService::new(bus);
        assert!(svc.cached_devices().await.is_empty());
        assert!(svc.cached_friends().await.is_empty());
    }
}
