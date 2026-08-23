//! End-to-end tests for the agent pipeline control plane.
//!
//! Exercises the previously untested critical path against real
//! dependencies (per project rule: no mocks for infra):
//!   task → orchestrator → DSPy (in-test stub, real HTTP) → Postgres
//!   persistence → SSE events → steering → breakpoints → tool calls.
//!
//! Requires PostgreSQL (`just db-up && just db-migrate`); skips visibly
//! without it, fails when `NEOLAND_TEST_REQUIRE_DEPS=1` (CI e2e job).

mod common;

use std::sync::OnceLock;
use std::time::Duration;

use common::{dev_keys, dspy_stub};
use futures::StreamExt;
use serde_json::Value;
use uuid::Uuid;

// ── Bootstrap ────────────────────────────────────────────────────────────────

struct E2eContext {
    server: &'static common::TestServer,
    stub: &'static dspy_stub::DspyStub,
}

/// Stub → env → server, in that order (the server reads NEOLAND_DSPY_URL at
/// boot). Returns None (after skip_or_fail) when Postgres is unavailable.
async fn e2e_context() -> Option<E2eContext> {
    if std::env::var("NEOLAND_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .is_err()
    {
        common::skip_or_fail("PostgreSQL", "set DATABASE_URL (just db-up && just db-migrate)");
        return None;
    }

    static STUB: OnceLock<dspy_stub::DspyStub> = OnceLock::new();
    let stub = STUB.get_or_init(|| {
        let stub = dspy_stub::DspyStub::spawn();
        // SAFETY: first thing this binary does, before the server boots.
        unsafe {
            std::env::set_var("NEOLAND_TEST_DSPY_URL", &stub.url);
            std::env::set_var("NEOLAND_TEST_KEEP_DATABASE", "1");
            // The steering test scripts a 2.5s stub delay — keep it inside
            // the pipeline timeout (the harness default is a fail-fast 2s).
            std::env::set_var("NEOLAND_PIPELINE_TIMEOUT_SECS", "15");
        }
        stub
    });

    let server = common::TestServer::shared().await;
    Some(E2eContext { server, stub })
}

/// The stub's script knobs are process-global — serialize the tests that
/// reconfigure them so parallel test threads can't race each other.
async fn serial_lock() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(())).lock().await
}

// ── SSE helpers ──────────────────────────────────────────────────────────────

/// Subscribes to the global agent event stream and forwards each decoded
/// event JSON into an unbounded channel.
async fn subscribe_events(base: &str) -> tokio::sync::mpsc::UnboundedReceiver<Value> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let url = format!("{base}/v1/agents/events");
    let resp = reqwest::Client::new()
        .get(url)
        .header("X-API-Key", dev_keys::READONLY)
        .send()
        .await
        .expect("SSE connect failed");
    assert!(resp.status().is_success(), "SSE subscribe returned {}", resp.status());

    tokio::spawn(async move {
        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            let Ok(chunk) = chunk else { break };
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(pos) = buf.find("\n\n") {
                let frame: String = buf.drain(..pos + 2).collect();
                for line in frame.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(v) = serde_json::from_str::<Value>(data) {
                            if tx.send(v).is_err() {
                                return;
                            }
                        }
                    }
                }
            }
        }
    });
    rx
}

/// Waits until an event of `event_type` for `session` arrives, returning it.
/// Panics after `secs` seconds, printing what did arrive.
async fn wait_for_event(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Value>,
    session: Uuid,
    event_type: &str,
    secs: u64,
) -> Value {
    let mut seen = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(event)) => {
                if event["session_id"] == session.to_string().as_str() {
                    if event["type"] == event_type {
                        return event;
                    }
                    seen.push(event["type"].to_string());
                }
            },
            Ok(None) => panic!("SSE stream closed while waiting for {event_type}"),
            Err(_) => {
                panic!("no {event_type} event for session {session} within {secs}s (saw: {seen:?})")
            },
        }
    }
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client")
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// task → stub pipeline → decision approve → events → session persisted.
#[tokio::test]
async fn e2e_task_full_flow_persists_and_streams() {
    let Some(ctx) = e2e_context().await else {
        return;
    };
    let _guard = serial_lock().await;
    ctx.stub.set_script(dspy_stub::StubScript::default()).await;

    let base = &ctx.server.rest_url;
    let session = Uuid::new_v4();
    let mut events = subscribe_events(base).await;

    let resp = http()
        .post(format!("{base}/v1/agents/task"))
        .header("X-API-Key", dev_keys::USER)
        .json(&serde_json::json!({"task": "e2e full flow", "session_id": session}))
        .send()
        .await
        .expect("POST /v1/agents/task");
    assert_eq!(
        resp.status(),
        200,
        "task submit failed: {}",
        resp.text().await.unwrap_or_default()
    );
    let result: Value = resp.json().await.expect("pipeline result JSON");
    assert_eq!(result["tech_leader"]["decision"], "approve");
    assert_eq!(result["session_id"], session.to_string().as_str());

    wait_for_event(&mut events, session, "pipeline_started", 10).await;
    wait_for_event(&mut events, session, "pipeline_done", 10).await;

    // Session state persisted in Postgres
    let resp = http()
        .get(format!("{base}/v1/agents/session/{session}"))
        .header("X-API-Key", dev_keys::READONLY)
        .send()
        .await
        .expect("GET session");
    assert_eq!(resp.status(), 200);
    let state: Value = resp.json().await.expect("session JSON");
    assert_eq!(state["session_id"], session.to_string().as_str());

    // And listed
    let resp = http()
        .get(format!("{base}/v1/agents/sessions"))
        .header("X-API-Key", dev_keys::READONLY)
        .send()
        .await
        .expect("GET sessions");
    assert_eq!(resp.status(), 200);
}

/// Steering delivered while the pipeline is mid-flight (stub delay opens the
/// window) is acknowledged and echoed as a steering_received event.
#[tokio::test]
async fn e2e_steering_during_active_pipeline() {
    let Some(ctx) = e2e_context().await else {
        return;
    };
    let _guard = serial_lock().await;
    ctx.stub
        .set_script(dspy_stub::StubScript { delay_ms: 2500, ..Default::default() })
        .await;

    let base = ctx.server.rest_url.clone();
    let session = Uuid::new_v4();
    let mut events = subscribe_events(&base).await;

    let submit = {
        let base = base.clone();
        tokio::spawn(async move {
            http()
                .post(format!("{base}/v1/agents/task"))
                .header("X-API-Key", dev_keys::USER)
                .json(&serde_json::json!({"task": "e2e steering", "session_id": session}))
                .send()
                .await
                .expect("POST task")
                .status()
        })
    };

    // Enter the steering window after the pipeline starts.
    wait_for_event(&mut events, session, "pipeline_started", 10).await;
    let resp = http()
        .post(format!("{base}/v1/agents/session/{session}/steer"))
        .header("X-API-Key", dev_keys::USER)
        .json(&serde_json::json!({"message": "focus on the e2e test"}))
        .send()
        .await
        .expect("POST steer");
    assert_eq!(resp.status(), 200, "steer failed: {}", resp.text().await.unwrap_or_default());

    let event = wait_for_event(&mut events, session, "steering_received", 10).await;
    assert_eq!(event["message"], "focus on the e2e test");

    assert_eq!(submit.await.expect("join submit"), 200);
    ctx.stub.set_script(dspy_stub::StubScript::default()).await;
}

/// Pipeline failure (stub 500) surfaces as HTTP 500 + pipeline_error event.
#[tokio::test]
async fn e2e_pipeline_failure_is_reported() {
    let Some(ctx) = e2e_context().await else {
        return;
    };
    let _guard = serial_lock().await;
    ctx.stub
        .set_script(dspy_stub::StubScript { fail: true, ..Default::default() })
        .await;

    let base = &ctx.server.rest_url;
    let session = Uuid::new_v4();
    let mut events = subscribe_events(base).await;

    let resp = http()
        .post(format!("{base}/v1/agents/task"))
        .header("X-API-Key", dev_keys::USER)
        .json(&serde_json::json!({"task": "e2e failure", "session_id": session}))
        .send()
        .await
        .expect("POST task");
    assert_eq!(resp.status(), 500);
    let body: Value = resp.json().await.expect("error JSON");
    assert!(body["error"].is_string(), "error body must carry a message: {body}");

    wait_for_event(&mut events, session, "pipeline_error", 10).await;
    ctx.stub.set_script(dspy_stub::StubScript::default()).await;
}

/// Tool listing exposes the native shell tool.
#[tokio::test]
async fn e2e_tools_list_contains_shell() {
    let Some(ctx) = e2e_context().await else {
        return;
    };
    let base = &ctx.server.rest_url;

    let resp = http()
        .get(format!("{base}/v1/agents/tools"))
        .header("X-API-Key", dev_keys::READONLY)
        .send()
        .await
        .expect("GET tools");
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.expect("tools JSON");
    let tools = body["tools"].as_array().expect("tools array");
    assert!(
        tools.iter().any(|t| t["name"].as_str().unwrap_or_default().contains("shell")),
        "expected a shell tool in {tools:?}"
    );
}

/// Tool call flow with human-in-the-loop breakpoint: the call blocks on a
/// breakpoint, the breakpoint is approved via the API, the shell command
/// runs and its output comes back.
#[tokio::test]
async fn e2e_tool_call_breakpoint_approve() {
    let Some(ctx) = e2e_context().await else {
        return;
    };
    let _guard = serial_lock().await;
    let base = ctx.server.rest_url.clone();
    let session = Uuid::new_v4();
    let mut events = subscribe_events(&base).await;

    let tool_name = shell_tool_name(&base).await;
    let call = {
        let base = base.clone();
        let tool_name = tool_name.clone();
        tokio::spawn(async move {
            let resp = http()
                .post(format!("{base}/v1/agents/tools/call"))
                .header("X-API-Key", dev_keys::USER)
                .json(&serde_json::json!({
                    "session_id": session,
                    "name": tool_name,
                    "arguments": {"command": "echo e2e-tool-ok"}
                }))
                .send()
                .await
                .expect("POST tools/call");
            let status = resp.status();
            let body: Value = resp.json().await.expect("tool result JSON");
            (status, body)
        })
    };

    // The call parks on a human breakpoint — approve it via the API.
    wait_for_event(&mut events, session, "breakpoint_hit", 10).await;
    let resp = http()
        .post(format!("{base}/v1/agents/session/{session}/breakpoint/resolve"))
        .header("X-API-Key", dev_keys::USER)
        .json(&serde_json::json!({"resolution": "approve"}))
        .send()
        .await
        .expect("POST breakpoint/resolve");
    assert_eq!(resp.status(), 200);

    let (status, body) = call.await.expect("join tool call");
    assert_eq!(status, 200, "tool call failed: {body}");
    assert_eq!(body["is_error"], false, "tool result flagged error: {body}");
    assert!(
        body["text"].as_str().unwrap_or_default().contains("e2e-tool-ok"),
        "stdout missing from tool result: {body}"
    );
}

/// Rejecting the breakpoint must NOT execute the command.
#[tokio::test]
async fn e2e_tool_call_breakpoint_reject() {
    let Some(ctx) = e2e_context().await else {
        return;
    };
    let _guard = serial_lock().await;
    let base = ctx.server.rest_url.clone();
    let session = Uuid::new_v4();
    let mut events = subscribe_events(&base).await;

    let marker = format!("/tmp/neoland-e2e-reject-{}", Uuid::new_v4());
    let tool_name = shell_tool_name(&base).await;
    let call = {
        let base = base.clone();
        let marker = marker.clone();
        let tool_name = tool_name.clone();
        tokio::spawn(async move {
            let resp = http()
                .post(format!("{base}/v1/agents/tools/call"))
                .header("X-API-Key", dev_keys::USER)
                .json(&serde_json::json!({
                    "session_id": session,
                    "name": tool_name,
                    "arguments": {"command": format!("touch {marker}")}
                }))
                .send()
                .await
                .expect("POST tools/call");
            let status = resp.status();
            let body = resp.json::<Value>().await.unwrap_or_default();
            (status, body)
        })
    };

    wait_for_event(&mut events, session, "breakpoint_hit", 10).await;
    let resp = http()
        .post(format!("{base}/v1/agents/session/{session}/breakpoint/resolve"))
        .header("X-API-Key", dev_keys::USER)
        .json(&serde_json::json!({"resolution": "reject"}))
        .send()
        .await
        .expect("POST breakpoint/resolve");
    assert_eq!(resp.status(), 200);

    let (status, body) = call.await.expect("join tool call");
    // A rejected call must not report success-with-output…
    assert!(
        status != 200 || body["is_error"] == true,
        "rejected tool call looked successful: {status} {body}"
    );
    // …and above all the command must never have run.
    assert!(
        !std::path::Path::new(&marker).exists(),
        "rejected command executed anyway ({marker} exists)"
    );
}

/// Resolves the exact registered name of the native shell tool.
async fn shell_tool_name(base: &str) -> String {
    let body: Value = http()
        .get(format!("{base}/v1/agents/tools"))
        .header("X-API-Key", dev_keys::READONLY)
        .send()
        .await
        .expect("GET tools")
        .json()
        .await
        .expect("tools JSON");
    body["tools"]
        .as_array()
        .and_then(|tools| {
            tools.iter().find_map(|t| {
                let name = t["name"].as_str()?;
                name.contains("shell").then(|| name.to_string())
            })
        })
        .expect("shell tool not registered")
}
