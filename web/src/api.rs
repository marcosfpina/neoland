//! API client for the Neoland REST API — compiled to WASM via `gloo-net`.
//!
//! All functions return `Result<T, String>` to keep error handling simple
//! in the Leptos signal layer.  For production use you may prefer a typed
//! error enum.

// Several public types are consumed only by the binary crate (`main.rs`), not
// yet by `lib.rs`.  They are part of the public API surface and will be used
// as the Web Console matures (e.g. `SessionDetail` for session detail view,
// `steer_task` for interactive steering, etc.).
#![allow(dead_code)]

use gloo_net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::EventSource;

// ---------------------------------------------------------------------------
// Public types — mirror the backend JSON shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Session {
    pub session_id: String,
    pub session_name: String,
    pub task_count: i32,
    pub last_activity: String,
    pub active: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionDetail {
    pub session_id: String,
    pub session_name: String,
    pub task_count: i32,
    pub last_activity: String,
    pub active: bool,
    pub messages: Vec<ApiMessage>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TaskSubmitResponse {
    pub task_id: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// SSE event types — parsed from the JSON payloads the backend emits
// ---------------------------------------------------------------------------

/// A structured event received via SSE from `/v1/agents/events/{session_id}`.
///
/// The backend serialises `AgentEvent` with `#[serde(tag = "type")]`, so every
/// JSON payload has a `"type"` field.  We parse that field and expose the rest
/// as raw JSON so the caller can further destructure only what it needs.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: String,
    pub payload: serde_json::Value,
}

impl<'de> serde::Deserialize<'de> for SseEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let event_type =
            value.get("type").and_then(|v| v.as_str()).map(String::from).unwrap_or_default();
        Ok(SseEvent { event_type, payload: value })
    }
}

impl SseEvent {
    /// Convenience: returns the `content` field for `stage_output` events.
    pub fn stage_output(&self) -> Option<&str> {
        if self.event_type == "stage_output" {
            self.payload.get("content")?.as_str()
        } else {
            None
        }
    }

    /// Convenience: returns the `content` field for any stage-related event.
    pub fn content(&self) -> Option<&str> {
        self.payload.get("content")?.as_str()
    }

    /// Convenience: returns the `stage` field if present.
    pub fn stage(&self) -> Option<&str> {
        self.payload.get("stage")?.as_str()
    }

    /// Whether this event signals the end of a pipeline run.
    pub fn is_terminal(&self) -> bool {
        matches!(self.event_type.as_str(), "pipeline_done" | "pipeline_error")
    }
}

// ---------------------------------------------------------------------------
// REST helpers
// ---------------------------------------------------------------------------

/// GET `/v1/agents/sessions` — list all sessions.
pub async fn list_sessions(base_url: &str) -> Result<Vec<Session>, String> {
    let url = format!("{}/v1/agents/sessions", base_url);
    let resp = Request::get(&url).send().await.map_err(|e| format!("request failed: {e}"))?;

    if !resp.ok() {
        return Err(format!("HTTP {} ({})", resp.status(), resp.status_text()));
    }

    let body = resp.text().await.map_err(|e| format!("failed to read body: {e}"))?;

    serde_json::from_str::<Vec<Session>>(&body)
        .map_err(|e| format!("failed to parse response: {e}"))
}

/// GET `/v1/agents/session/{session_id}` — fetch a single session with messages.
pub async fn get_session(base_url: &str, session_id: &str) -> Result<SessionDetail, String> {
    let url = format!("{}/v1/agents/session/{}", base_url, session_id);
    let resp = Request::get(&url).send().await.map_err(|e| format!("request failed: {e}"))?;

    if !resp.ok() {
        return Err(format!("HTTP {} ({})", resp.status(), resp.status_text()));
    }

    let body = resp.text().await.map_err(|e| format!("failed to read body: {e}"))?;

    serde_json::from_str::<SessionDetail>(&body)
        .map_err(|e| format!("failed to parse response: {e}"))
}

/// POST `/v1/agents/task` — submit a new task (optionally in an existing session).
pub async fn submit_task(
    base_url: &str,
    description: &str,
    session_id: Option<&str>,
) -> Result<TaskSubmitResponse, String> {
    let url = format!("{}/v1/agents/task", base_url);

    let body = match session_id {
        Some(sid) => serde_json::json!({
            "task": description,
            "session_id": sid,
        }),
        None => serde_json::json!({
            "task": description,
        }),
    };

    let resp = Request::post(&url)
        .json(&body)
        .map_err(|e| format!("failed to serialise body: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.ok() {
        return Err(format!("HTTP {} ({})", resp.status(), resp.status_text()));
    }

    let body = resp.text().await.map_err(|e| format!("failed to read body: {e}"))?;

    serde_json::from_str::<TaskSubmitResponse>(&body)
        .map_err(|e| format!("failed to parse response: {e}"))
}

/// POST `/v1/agents/session/{session_id}/steer` — send a steering message.
pub async fn steer_task(
    base_url: &str,
    session_id: &str,
    message: &str,
) -> Result<serde_json::Value, String> {
    let url = format!("{}/v1/agents/session/{}/steer", base_url, session_id);

    let body = serde_json::json!({ "message": message });

    let resp = Request::post(&url)
        .json(&body)
        .map_err(|e| format!("failed to serialise body: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.ok() {
        return Err(format!("HTTP {} ({})", resp.status(), resp.status_text()));
    }

    let body = resp.text().await.map_err(|e| format!("failed to read body: {e}"))?;

    serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|e| format!("failed to parse response: {e}"))
}

// ---------------------------------------------------------------------------
/// PATCH `/v1/agents/session/{session_id}/name` — set the session name.
pub async fn set_session_name(base_url: &str, session_id: &str, name: &str) -> Result<(), String> {
    let url = format!("{}/v1/agents/session/{}/name", base_url, session_id);
    let body = serde_json::json!({"name": name});
    let resp = Request::patch(&url)
        .json(&body)
        .map_err(|e| format!("failed to serialise body: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if !resp.ok() {
        return Err(format!("HTTP {} ({})", resp.status(), resp.status_text()));
    }
    Ok(())
}

/// GET `/v1/agents/session/{session_id}/messages` — fetch messages for a session.
pub async fn fetch_messages(base_url: &str, session_id: &str) -> Result<Vec<ApiMessage>, String> {
    let url = format!("{}/v1/agents/session/{}/messages", base_url, session_id);
    let resp = Request::get(&url).send().await.map_err(|e| format!("request failed: {e}"))?;
    if !resp.ok() {
        return Err(format!("HTTP {} ({})", resp.status(), resp.status_text()));
    }
    let body = resp.text().await.map_err(|e| format!("failed to read body: {e}"))?;
    serde_json::from_str::<Vec<ApiMessage>>(&body)
        .map_err(|e| format!("failed to parse response: {e}"))
}

// ---------------------------------------------------------------------------
// SSE streaming via browser-native `EventSource`
// ---------------------------------------------------------------------------

/// Opens a persistent SSE connection to `/v1/agents/events/{session_id}`.
///
/// Each `data:` payload is parsed as JSON into an `SseEvent`.  The `on_event`
/// callback receives the structured event.  The connection lives as long as the
/// returned `EventSource` handle does — drop it or call `.close()` to
/// disconnect.
pub fn connect_sse(
    base_url: &str,
    session_id: &str,
    on_event: impl Fn(SseEvent) + 'static,
) -> EventSource {
    let url = format!("{}/v1/agents/events/{}", base_url, session_id);
    let source =
        EventSource::new(&url).expect("failed to create EventSource — check the URL and CORS");

    let onmessage_callback = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        if let Some(data) = event.data().as_string() {
            // The backend sends JSON with a "type" field — parse it and forward
            // structured events to the caller.
            if let Ok(sse) = serde_json::from_str::<SseEvent>(&data) {
                on_event(sse);
            }
        }
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);

    source.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget(); // leak — lives for the lifetime of the EventSource

    source
}
