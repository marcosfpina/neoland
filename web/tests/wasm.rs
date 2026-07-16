//! WASM integration tests for neoland-web.
//!
//! These tests run in a headless browser via `wasm-bindgen-test`.
//! They validate that the crate's public API works correctly when
//! compiled to `wasm32-unknown-unknown`.
//!
//! Run with:
//!   wasm-pack test --headless --chrome web/
//!   cargo test --target wasm32-unknown-unknown -p neoland-web

use prost::Message;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Type system: SseEvent deserialization in WASM context
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn wasm_sse_event_stage_output() {
    let json = r#"{"type":"stage_output","stage":"junior","content":"WASM test output"}"#;
    let ev: neoland_web::api::SseEvent =
        serde_json::from_str(json).expect("valid stage_output JSON");
    assert_eq!(ev.event_type, "stage_output");
    assert_eq!(ev.stage(), Some("junior"));
    assert_eq!(ev.content(), Some("WASM test output"));
    assert!(!ev.is_terminal());
}

#[wasm_bindgen_test]
fn wasm_sse_event_pipeline_done() {
    let json = r#"{"type":"pipeline_done","session_id":"550e8400-e29b-41d4-a716-446655440000","latency_ms":1234}"#;
    let ev: neoland_web::api::SseEvent =
        serde_json::from_str(json).expect("valid pipeline_done JSON");
    assert_eq!(ev.event_type, "pipeline_done");
    assert!(ev.is_terminal());
}

#[wasm_bindgen_test]
fn wasm_sse_event_pipeline_error() {
    let json = r#"{"type":"pipeline_error","session_id":"00000000-0000-0000-0000-000000000000","error":"timeout"}"#;
    let ev: neoland_web::api::SseEvent =
        serde_json::from_str(json).expect("valid pipeline_error JSON");
    assert_eq!(ev.event_type, "pipeline_error");
    assert!(ev.is_terminal());
}

#[wasm_bindgen_test]
fn wasm_sse_event_missing_type_defaults_to_empty() {
    let json = r#"{"stage":"senior","content":"review notes"}"#;
    let ev: neoland_web::api::SseEvent =
        serde_json::from_str(json).expect("valid JSON without type");
    assert_eq!(ev.event_type, "");
    assert_eq!(ev.content(), Some("review notes"));
    assert!(!ev.is_terminal());
}

// ---------------------------------------------------------------------------
// Type system: API request/response types
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn wasm_session_deserialization() {
    let json = r#"{
        "session_id": "wasm-sess-001",
        "session_name": "WASM Test",
        "task_count": 3,
        "last_activity": "2026-07-16T12:00:00Z",
        "active": true
    }"#;
    let session: neoland_web::api::Session =
        serde_json::from_str(json).expect("valid session JSON");
    assert_eq!(session.session_id, "wasm-sess-001");
    assert_eq!(session.session_name, "WASM Test");
    assert_eq!(session.task_count, 3);
    assert!(session.active);
}

#[wasm_bindgen_test]
fn wasm_api_message_deserialization() {
    let json = r#"{"role":"agent","content":"WASM response","timestamp":"2026-07-16T12:00:00Z"}"#;
    let msg: neoland_web::api::ApiMessage = serde_json::from_str(json).expect("valid message JSON");
    assert_eq!(msg.role, "agent");
    assert_eq!(msg.content, "WASM response");
    assert_eq!(msg.timestamp.as_deref(), Some("2026-07-16T12:00:00Z"));
}

#[wasm_bindgen_test]
fn wasm_task_submit_response() {
    let json = r#"{"task_id":"wasm-task-1","session_id":"wasm-sess-1"}"#;
    let resp: neoland_web::api::TaskSubmitResponse =
        serde_json::from_str(json).expect("valid task submit response");
    assert_eq!(resp.task_id, "wasm-task-1");
    assert_eq!(resp.session_id, "wasm-sess-1");
}

#[wasm_bindgen_test]
fn wasm_session_detail_with_messages() {
    let json = r#"{
        "session_id": "det-001",
        "session_name": "Detail Test",
        "task_count": 1,
        "last_activity": "2026-07-16T12:00:00Z",
        "active": true,
        "messages": [
            {"role":"user","content":"hello","timestamp":null},
            {"role":"agent","content":"hi there","timestamp":"2026-07-16T12:00:01Z"}
        ]
    }"#;
    let detail: neoland_web::api::SessionDetail =
        serde_json::from_str(json).expect("valid session detail JSON");
    assert_eq!(detail.session_id, "det-001");
    assert_eq!(detail.messages.len(), 2);
    assert_eq!(detail.messages[0].role, "user");
    assert_eq!(detail.messages[1].content, "hi there");
}

// ---------------------------------------------------------------------------
// Protobuf: ChatRequest encode/decode round-trip (prost in WASM)
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn wasm_protobuf_chat_request_roundtrip() {
    use neoland_web::grpc::proto::ChatRequest;

    let req = ChatRequest {
        prompt: "Hello, WASM!".into(),
        model_id: "qwen-1.8b".into(),
        use_local: true,
        temperature: Some(0.8),
        top_p: Some(0.95),
        max_tokens: Some(512),
        repetition_penalty: Some(1.1),
        typical_p: None,
        epsilon_cutoff: None,
        eta_cutoff: None,
        tail_free_sampling: None,
        top_a: None,
        context_top_k: Some(3),
        context_similarity_threshold: Some(0.5),
        disable_context: Some(false),
        system_prompt: None,
        enable_commands: Some(true),
        allowed_commands: vec![],
        session_id: Some("wasm-session-proto".into()),
        streaming: Some(true),
    };

    // Encode to bytes
    let encoded = prost::Message::encode_to_vec(&req);

    // Decode back
    let decoded =
        ChatRequest::decode(encoded.as_slice()).expect("failed to decode protobuf in WASM");

    assert_eq!(decoded.prompt, "Hello, WASM!");
    assert_eq!(decoded.model_id, "qwen-1.8b");
    assert!(decoded.use_local);
    assert_eq!(decoded.temperature, Some(0.8));
    assert_eq!(decoded.max_tokens, Some(512));
    assert_eq!(decoded.session_id.as_deref(), Some("wasm-session-proto"));
    assert_eq!(decoded.streaming, Some(true));
}

#[wasm_bindgen_test]
fn wasm_protobuf_chat_response_decode() {
    use neoland_web::grpc::proto::ChatResponse;

    let resp =
        ChatResponse { content: "WASM response content".into(), is_command: false, metadata: None };

    let encoded = prost::Message::encode_to_vec(&resp);
    let decoded = ChatResponse::decode(encoded.as_slice()).expect("failed to decode ChatResponse");

    assert_eq!(decoded.content, "WASM response content");
    assert!(!decoded.is_command);
}

// ---------------------------------------------------------------------------
// Component mount smoke test — verifies App renders without panicking
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn wasm_app_mount_smoke() {
    // Mount the root App component. This validates that:
    // 1. The Leptos view! macro compiles and renders in WASM
    // 2. No panics occur during initial render (signals, callbacks, etc.)
    // 3. The DOM contains the expected shell structure
    leptos::prelude::mount_to_body(neoland_web::App);

    let window = web_sys::window().expect("no global `window`");
    let document = window.document().expect("no global `document`");

    // Verify the app-shell container is present
    let shell = document
        .query_selector(".app-shell")
        .expect("query_selector should not panic")
        .expect("app-shell div not found in DOM");

    assert!(shell.class_list().contains("app-shell"));

    // Verify the system-line footer is rendered
    let system_line = document
        .query_selector(".system-line")
        .expect("query_selector should not panic")
        .expect("system-line div not found in DOM");

    let text = system_line.text_content().unwrap_or_default();
    assert!(text.contains("NEOLAND://CORE"));
    assert!(text.contains("WASM ONLINE"));
}

#[wasm_bindgen_test]
fn wasm_backend_url_is_empty_string() {
    let url = neoland_web::backend_url();
    // Same-origin in both dev (Trunk proxy) and prod (Neoland static serve)
    assert_eq!(url, "");
}
