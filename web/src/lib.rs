//! neoland-web — Leptos WASM Web Console for the Neoland control plane.
//!
//! This crate contains the browser-side SPA that communicates with the
//! Neoland REST API and SSE event stream.

pub mod api;
pub mod components;
pub mod model;

#[cfg(test)]
mod tests {
    use crate::api::SseEvent;
    use crate::model::Message;

    // -----------------------------------------------------------------------
    // model::Message
    // -----------------------------------------------------------------------

    #[test]
    fn message_user_role() {
        let msg = Message { role: "user", body: "hello".into(), code: None };
        assert_eq!(msg.role, "user");
        assert_eq!(msg.body, "hello");
        assert!(msg.code.is_none());
    }

    #[test]
    fn message_agent_role_with_code() {
        let msg =
            Message { role: "agent", body: "fn main() {}".into(), code: Some("fn main() {}") };
        assert_eq!(msg.role, "agent");
        assert_eq!(msg.body, "fn main() {}");
        assert_eq!(msg.code, Some("fn main() {}"));
    }

    // -----------------------------------------------------------------------
    // api::SseEvent — deserialization from JSON
    // -----------------------------------------------------------------------

    #[test]
    fn sse_event_stage_output() {
        let json = r#"{"type":"stage_output","stage":"junior","content":"Hypothesis: use Arc<AtomicU32>"}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid stage_output JSON");
        assert_eq!(ev.event_type, "stage_output");
        assert_eq!(ev.stage(), Some("junior"));
        assert_eq!(ev.content(), Some("Hypothesis: use Arc<AtomicU32>"));
        assert!(!ev.is_terminal());
    }

    #[test]
    fn sse_event_pipeline_done() {
        let json = r#"{"type":"pipeline_done","session_id":"550e8400-e29b-41d4-a716-446655440000","latency_ms":1234}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid pipeline_done JSON");
        assert_eq!(ev.event_type, "pipeline_done");
        assert!(ev.content().is_none());
        assert!(ev.is_terminal());
    }

    #[test]
    fn sse_event_pipeline_error() {
        let json = r#"{"type":"pipeline_error","session_id":"00000000-0000-0000-0000-000000000000","error":"timeout"}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid pipeline_error JSON");
        assert_eq!(ev.event_type, "pipeline_error");
        assert!(ev.is_terminal());
    }

    #[test]
    fn sse_event_pipeline_started() {
        let json = r#"{"type":"pipeline_started","session_id":"00000000-0000-0000-0000-000000000000","task_preview":"test task"}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid pipeline_started JSON");
        assert_eq!(ev.event_type, "pipeline_started");
        assert!(!ev.is_terminal());
        assert!(ev.content().is_none());
        assert!(ev.stage().is_none());
    }

    #[test]
    fn sse_event_stage_done_with_confidence() {
        let json = r#"{"type":"stage_done","session_id":"550e8400-e29b-41d4-a716-446655440000","stage":"senior","confidence":0.87,"risk_level":2,"latency_ms":450}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid stage_done JSON");
        assert_eq!(ev.event_type, "stage_done");
        assert_eq!(ev.stage(), Some("senior"));
        assert!(!ev.is_terminal());
    }

    #[test]
    fn sse_event_adr_checkpoint() {
        let json = r#"{"type":"adr_checkpoint","session_id":"00000000-0000-0000-0000-000000000000","adr_id":"ADR-016","status":"accepted","title":"Test ADR"}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid adr_checkpoint JSON");
        assert_eq!(ev.event_type, "adr_checkpoint");
        assert!(!ev.is_terminal());
    }

    #[test]
    fn sse_event_unknown_type_is_not_terminal() {
        let json = r#"{"type":"heartbeat","ts":12345}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid heartbeat JSON");
        assert_eq!(ev.event_type, "heartbeat");
        assert!(!ev.is_terminal());
    }

    #[test]
    fn sse_event_missing_type_field_defaults_to_empty() {
        let json = r#"{"stage":"junior","content":"no type tag"}"#;
        let ev: SseEvent = serde_json::from_str(json).expect("valid JSON without type");
        assert_eq!(ev.event_type, "");
        assert_eq!(ev.content(), Some("no type tag"));
        assert_eq!(ev.stage(), Some("junior"));
    }

    #[test]
    fn sse_event_invalid_json_is_rejected() {
        let json = r#"not json at all"#;
        let result: Result<SseEvent, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // api::Session — deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn session_deserialization() {
        let json = r#"{
            "session_id": "abc-123",
            "session_name": "Test Session",
            "task_count": 5,
            "last_activity": "2026-07-15T10:30:00Z",
            "active": true
        }"#;
        let session: crate::api::Session = serde_json::from_str(json).expect("valid session JSON");
        assert_eq!(session.session_id, "abc-123");
        assert_eq!(session.session_name, "Test Session");
        assert_eq!(session.task_count, 5);
        assert!(session.active);
    }

    #[test]
    fn session_list_empty() {
        let json = r#"[]"#;
        let sessions: Vec<crate::api::Session> =
            serde_json::from_str(json).expect("valid empty list");
        assert!(sessions.is_empty());
    }

    #[test]
    fn api_message_deserialization() {
        let json = r#"{"role":"user","content":"hello world","timestamp":"2026-07-15T10:30:00Z"}"#;
        let msg: crate::api::ApiMessage = serde_json::from_str(json).expect("valid message JSON");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "hello world");
        assert_eq!(msg.timestamp.as_deref(), Some("2026-07-15T10:30:00Z"));
    }

    #[test]
    fn api_message_without_timestamp() {
        let json = r#"{"role":"agent","content":"ok"}"#;
        let msg: crate::api::ApiMessage =
            serde_json::from_str(json).expect("valid message without timestamp");
        assert_eq!(msg.role, "agent");
        assert_eq!(msg.content, "ok");
        assert!(msg.timestamp.is_none());
    }

    #[test]
    fn task_submit_response_deserialization() {
        let json = r#"{"task_id":"task-1","session_id":"sess-1"}"#;
        let resp: crate::api::TaskSubmitResponse =
            serde_json::from_str(json).expect("valid task submit response");
        assert_eq!(resp.task_id, "task-1");
        assert_eq!(resp.session_id, "sess-1");
    }
}
