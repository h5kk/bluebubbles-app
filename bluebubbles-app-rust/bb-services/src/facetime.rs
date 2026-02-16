//! FaceTime service for managing FaceTime call interactions.
//!
//! Handles answering and leaving FaceTime calls through the server API,
//! tracks active call state, and emits events for UI consumption.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, debug};

use bb_core::error::{BbError, BbResult};
use bb_api::ApiClient;

use crate::event_bus::{AppEvent, EventBus};
use crate::service::{Service, ServiceState};

/// Known FaceTime call statuses.
pub mod call_status {
    /// Call is ringing / incoming.
    pub const RINGING: i32 = 1;
    /// Call is active / connected.
    pub const ACTIVE: i32 = 2;
    /// Call has ended.
    pub const ENDED: i32 = 3;
    /// Call was declined.
    pub const DECLINED: i32 = 4;
    /// Call failed to connect.
    pub const FAILED: i32 = 5;
}

/// Information about an active or recent FaceTime call.
#[derive(Debug, Clone)]
pub struct FaceTimeCall {
    /// Unique call identifier.
    pub uuid: String,
    /// Address of the caller.
    pub caller: String,
    /// Whether this is an audio-only call.
    pub is_audio: bool,
    /// Current call status.
    pub status: i32,
    /// FaceTime link for joining the call (returned after answering).
    pub link: Option<String>,
}

/// Service for FaceTime call management.
///
/// Tracks active calls, provides answer/leave operations via the server API,
/// and emits FaceTime-related events through the event bus.
pub struct FaceTimeService {
    state: ServiceState,
    event_bus: EventBus,
    /// Active calls indexed by UUID.
    active_calls: Arc<Mutex<HashMap<String, FaceTimeCall>>>,
}

impl FaceTimeService {
    /// Create a new FaceTimeService.
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            state: ServiceState::Created,
            event_bus,
            active_calls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record an incoming FaceTime call.
    ///
    /// Called by the action handler when an incoming-facetime socket event
    /// is received. Stores the call in the active calls map.
    pub async fn track_incoming_call(
        &self,
        uuid: &str,
        caller: &str,
        is_audio: bool,
    ) {
        let call = FaceTimeCall {
            uuid: uuid.to_string(),
            caller: caller.to_string(),
            is_audio,
            status: call_status::RINGING,
            link: None,
        };

        let mut calls = self.active_calls.lock().await;
        calls.insert(uuid.to_string(), call);
        info!("tracking incoming FaceTime call from {caller} (uuid: {uuid})");
    }

    /// Answer a FaceTime call via the server API.
    ///
    /// Returns the FaceTime link URL that can be opened in a browser to
    /// join the call.
    pub async fn answer_call(
        &self,
        api: &ApiClient,
        call_uuid: &str,
    ) -> BbResult<String> {
        let link = api.answer_facetime(call_uuid).await?;

        let link_str = link.ok_or_else(|| {
            BbError::Http("server did not return a FaceTime link".into())
        })?;

        // Update call state
        {
            let mut calls = self.active_calls.lock().await;
            if let Some(call) = calls.get_mut(call_uuid) {
                call.status = call_status::ACTIVE;
                call.link = Some(link_str.clone());
            }
        }

        info!("answered FaceTime call {call_uuid}");

        self.event_bus.emit(AppEvent::FaceTimeStatusChanged {
            call_uuid: call_uuid.to_string(),
            status: call_status::ACTIVE,
        });

        Ok(link_str)
    }

    /// Leave / end a FaceTime call via the server API.
    pub async fn leave_call(
        &self,
        api: &ApiClient,
        call_uuid: &str,
    ) -> BbResult<()> {
        api.leave_facetime(call_uuid).await?;

        {
            let mut calls = self.active_calls.lock().await;
            if let Some(call) = calls.get_mut(call_uuid) {
                call.status = call_status::ENDED;
            }
        }

        info!("left FaceTime call {call_uuid}");

        self.event_bus.emit(AppEvent::FaceTimeStatusChanged {
            call_uuid: call_uuid.to_string(),
            status: call_status::ENDED,
        });

        Ok(())
    }

    /// Update the status of a tracked call.
    ///
    /// Called by the action handler when a ft-call-status-changed event is
    /// received from the socket.
    pub async fn update_call_status(&self, call_uuid: &str, status: i32) {
        let mut calls = self.active_calls.lock().await;
        if let Some(call) = calls.get_mut(call_uuid) {
            call.status = status;
            debug!("updated FaceTime call {call_uuid} status to {status}");
        }

        // Clean up ended calls
        if status == call_status::ENDED || status == call_status::DECLINED || status == call_status::FAILED {
            calls.remove(call_uuid);
            debug!("removed ended FaceTime call {call_uuid}");
        }
    }

    /// Get an active call by UUID.
    pub async fn get_call(&self, uuid: &str) -> Option<FaceTimeCall> {
        let calls = self.active_calls.lock().await;
        calls.get(uuid).cloned()
    }

    /// Get all currently active calls.
    pub async fn active_calls(&self) -> Vec<FaceTimeCall> {
        let calls = self.active_calls.lock().await;
        calls.values().cloned().collect()
    }

    /// Check if there are any active calls.
    pub async fn has_active_calls(&self) -> bool {
        let calls = self.active_calls.lock().await;
        !calls.is_empty()
    }
}

impl Service for FaceTimeService {
    fn name(&self) -> &str {
        "facetime"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Running;
        info!("FaceTime service initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("FaceTime service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facetime_service_name() {
        let bus = EventBus::new(16);
        let svc = FaceTimeService::new(bus);
        assert_eq!(svc.name(), "facetime");
    }

    #[tokio::test]
    async fn test_track_incoming_call() {
        let bus = EventBus::new(16);
        let svc = FaceTimeService::new(bus);

        svc.track_incoming_call("call-1", "+15551234567", false).await;

        let call = svc.get_call("call-1").await.unwrap();
        assert_eq!(call.caller, "+15551234567");
        assert!(!call.is_audio);
        assert_eq!(call.status, call_status::RINGING);
        assert!(call.link.is_none());
    }

    #[tokio::test]
    async fn test_update_call_status_removes_ended() {
        let bus = EventBus::new(16);
        let svc = FaceTimeService::new(bus);

        svc.track_incoming_call("call-2", "+15559876543", true).await;
        assert!(svc.has_active_calls().await);

        svc.update_call_status("call-2", call_status::ENDED).await;
        assert!(!svc.has_active_calls().await);
        assert!(svc.get_call("call-2").await.is_none());
    }

    #[tokio::test]
    async fn test_no_active_calls_initially() {
        let bus = EventBus::new(16);
        let svc = FaceTimeService::new(bus);
        assert!(!svc.has_active_calls().await);
        assert!(svc.active_calls().await.is_empty());
    }
}
