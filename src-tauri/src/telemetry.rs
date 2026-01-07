use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Window};

#[derive(Serialize, Clone)]
pub struct TelemetryEvent {
    pub event_type: String, // e.g., "MIGRATION_STARTED", "ERROR"
    pub timestamp: u64,
    pub session_id: String,
    pub payload: serde_json::Value,
}

impl TelemetryEvent {
    pub fn new(event_type: &str, payload: serde_json::Value) -> Self {
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH).unwrap().as_secs();

        Self {
            event_type: event_type.to_string(),
            timestamp,
            // In a real app, generate a UUID for session
            session_id: "SESSION_ALPHA_1".to_string(),
            payload,
        }
    }
}

pub fn track_event(window: &Window, event: TelemetryEvent) {
    // 1. Log to UI (Flight Recorder)
    let log_msg = format!(
        "TELEMETRY DISPATCH: {} [{}]",
        event.event_type, event.payload
    );
    window.emit("log", log_msg).unwrap();

    // 2. In production, this would POST to an endpoint
    println!("Analytics: {:?}", event.event_type);
}
