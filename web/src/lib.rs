//! neoland-web — Leptos WASM Web Console for the Neoland control plane.
//!
//! This crate contains the browser-side SPA that communicates with the
//! Neoland REST API and SSE event stream.

pub mod api;
pub mod components;
pub mod grpc;
pub mod model;

use components::{
    composer::Composer, conversation::Conversation, reasoning::ReasoningPanel,
    sessions::SessionsPanel, topbar::Topbar,
};
use leptos::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use wasm_bindgen_futures::spawn_local;

/// Returns the backend base URL.
///
/// In dev, Trunk proxies `/v1/*` → `localhost:3001`, so the empty string
/// (same origin) hits the backend through the proxy.  In production the
/// Neoland server serves the WASM bundle directly — same origin again.
pub fn backend_url() -> String {
    // Same-origin: Trunk proxy in dev, Neoland static serve in prod
    String::new()
}

/// Root application component — mounted by `main.rs` and exposed for testing.
#[component]
pub fn App() -> impl IntoView {
    let base = backend_url();

    let (sessions, set_sessions) = signal(Vec::new());
    let (active_idx, set_active_idx) = signal(0usize);
    let (messages, set_messages) = signal(Vec::new());
    let (input, set_input) = signal(String::new());
    let (streaming, set_streaming) = signal(false);
    let (stream_text, set_stream_text) = signal(String::new());
    let (sessions_open, set_sessions_open) = signal(false);
    let (reasoning_open, set_reasoning_open) = signal(false);
    let (active_session_id, set_active_session_id) = signal(None::<String>);

    // Monotonically-increasing stream generation counter.
    // Each SSE connection is tagged with a generation; callbacks from old
    // (superseded) connections silently discard their events.
    let stream_gen: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));

    // Load sessions on mount
    {
        let base = base.clone();
        spawn_local(async move {
            match api::list_sessions(&base).await {
                Ok(list) => set_sessions.set(list),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load sessions: {e}").into())
                },
            }
        });
    }

    let new_chat = {
        let base = base.clone();
        Callback::new(move |_| {
            set_messages.set(Vec::new());
            set_input.set(String::new());
            set_active_session_id.set(None);
            set_streaming.set(false);
            // Reload session list
            let base = base.clone();
            spawn_local(async move {
                if let Ok(list) = api::list_sessions(&base).await {
                    set_sessions.set(list);
                }
            });
        })
    };

    let select_session = {
        let base = base.clone();
        Callback::new(move |idx: usize| {
            set_active_idx.set(idx);
            if let Some(session) = sessions.get().get(idx) {
                let sid = session.session_id.clone();
                set_active_session_id.set(Some(sid.clone()));
                let base = base.clone();
                spawn_local(async move {
                    if let Ok(msgs) = api::fetch_messages(&base, &sid).await {
                        set_messages.set(
                            msgs.into_iter()
                                .map(|m| model::Message {
                                    role: if m.role == "user" { "user" } else { "agent" },
                                    body: m.content,
                                    code: None,
                                })
                                .collect(),
                        );
                    }
                });
            }
        })
    };

    let submit = {
        let base = base.clone();
        let stream_gen = stream_gen.clone();
        Callback::new(move |_| {
            let prompt = input.get_untracked().trim().to_string();
            if prompt.is_empty() || streaming.get_untracked() {
                return;
            }
            // Push user message immediately
            set_messages.update(|items| {
                items.push(model::Message { role: "user", body: prompt.clone(), code: None });
            });
            set_input.set(String::new());
            set_streaming.set(true);
            set_stream_text.set(String::new());

            // Bump generation — invalidates any still-firing callbacks from a previous SSE.
            let current_gen = stream_gen.fetch_add(1, Ordering::Relaxed);

            let base = base.clone();
            let sid = active_session_id.get_untracked();
            let stream_gen = stream_gen.clone();
            spawn_local(async move {
                match api::submit_task(&base, &prompt, sid.as_deref()).await {
                    Ok(resp) => {
                        set_active_session_id.set(Some(resp.session_id.clone()));
                        // The EventSource handle is dropped at the end of this block,
                        // but the JS object lives on thanks to the `.forget()`-ed
                        // closure.  The generation guard below ensures only the most
                        // recent stream writes to the UI.
                        let _source = api::connect_sse(&base, &resp.session_id, {
                            let stream_gen = stream_gen.clone();
                            move |ev: api::SseEvent| {
                                // Discard events from a superseded connection.
                                if stream_gen.load(Ordering::Relaxed) != current_gen {
                                    return;
                                }
                                if let Some(content) = ev.content() {
                                    let prefix = if let Some(stage) = ev.stage() {
                                        format!("[{}] ", stage)
                                    } else {
                                        String::new()
                                    };
                                    set_stream_text.update(|t| {
                                        t.push_str(&format!("{}{}\n", prefix, content))
                                    });
                                }
                                if ev.is_terminal() {
                                    set_streaming.set(false);
                                }
                            }
                        });
                    },
                    Err(e) => {
                        set_streaming.set(false);
                        web_sys::console::error_1(&format!("Task failed: {e}").into());
                    },
                }
            });
        })
    };

    let cancel = Callback::new(move |_| {
        set_streaming.set(false);
    });

    view! {
        <div class="app-shell">
            <div class="scanlines" aria-hidden="true"></div>
            <Topbar
                on_sessions=Callback::new(move |_| set_sessions_open.update(|v| *v = !*v))
                on_reasoning=Callback::new(move |_| set_reasoning_open.update(|v| *v = !*v))
            />
            <div class="workspace">
                <SessionsPanel
                    sessions=sessions
                    active=active_idx
                    set_active=set_active_idx
                    on_select=select_session
                    open=sessions_open
                    on_new=new_chat
                />
                <Conversation messages=messages streaming=streaming stream_text=stream_text />
                <ReasoningPanel open=reasoning_open />
            </div>
            <Composer
                input=input
                set_input=set_input
                streaming=streaming
                submit=submit
                cancel=cancel
            />
            <div class="system-line">
                <span>"NEOLAND://CORE"</span>
                <span>"WASM ONLINE"</span>
                <span>"BUILD 2026.07"</span>
            </div>
        </div>
    }
}

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
