//! In-test stub of the DSPy FastAPI pipeline.
//!
//! Implements the contract `src/agents/client.rs` consumes
//! (`POST /v1/pipeline/run`, `GET /health`) with a scriptable response:
//! configurable delay (opens the live-steering window), failure mode and
//! architect escalation. No LLM involved — this exercises the control
//! plane end-to-end, not the agents' reasoning.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

/// Mutable behavior knobs, shared with the running stub.
#[derive(Clone, Default)]
pub struct StubScript {
    /// Milliseconds to wait before answering `/v1/pipeline/run`.
    pub delay_ms: u64,
    /// Respond HTTP 500 instead of a pipeline result.
    pub fail: bool,
    /// Set `senior.escalate_to_architect` and include an architect output.
    pub escalate: bool,
}

#[derive(Clone)]
struct StubState {
    script: Arc<tokio::sync::Mutex<StubScript>>,
    calls: Arc<AtomicU64>,
}

pub struct DspyStub {
    pub url: String,
    script: Arc<tokio::sync::Mutex<StubScript>>,
    calls: Arc<AtomicU64>,
}

impl DspyStub {
    /// Reconfigures the stub's behavior for the next requests.
    pub async fn set_script(&self, script: StubScript) {
        *self.script.lock().await = script;
    }

    /// Number of `/v1/pipeline/run` calls served so far.
    pub fn calls(&self) -> u64 {
        self.calls.load(Ordering::SeqCst)
    }

    /// Boots the stub on an ephemeral port, in its own OS thread + runtime so
    /// it outlives the per-test tokio runtimes (same pattern as TestServer).
    pub fn spawn() -> DspyStub {
        let script = Arc::new(tokio::sync::Mutex::new(StubScript::default()));
        let calls = Arc::new(AtomicU64::new(0));
        let state = StubState { script: script.clone(), calls: calls.clone() };

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind stub");
        listener.set_nonblocking(true).expect("nonblocking stub");
        let addr = listener.local_addr().expect("stub addr");

        std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("stub runtime")
                .block_on(async move {
                    let app = Router::new()
                        .route("/v1/pipeline/run", post(run_handler))
                        .route("/health", get(|| async { "ok" }))
                        .with_state(state);
                    let listener =
                        tokio::net::TcpListener::from_std(listener).expect("stub listener");
                    if let Err(e) = axum::serve(listener, app).await {
                        eprintln!("[DSPY STUB] serve failed: {e}");
                    }
                });
        });

        DspyStub { url: format!("http://{addr}"), script, calls }
    }
}

async fn run_handler(
    State(state): State<StubState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    state.calls.fetch_add(1, Ordering::SeqCst);
    let script = state.script.lock().await.clone();

    if script.delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(script.delay_ms)).await;
    }
    if script.fail {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "stub scripted failure"})),
        )
            .into_response();
    }

    let architect = script.escalate.then(|| {
        serde_json::json!({
            "structural_soundness": true,
            "composability_score": 0.8,
            "long_term_concerns": [],
            "recommended_structure": "stub structure",
            "blockers": []
        })
    });

    // Mirrors PipelineResult in src/agents/client.rs.
    Json(serde_json::json!({
        "task_id": req["task_id"],
        "session_id": req["session_id"],
        "task": req["task"],
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "junior": {
            "hypothesis": "stub hypothesis",
            "confidence": 0.9,
            "risk_level": "low",
            "unknowns": [],
            "innovation_vectors": []
        },
        "senior": {
            "valid_parts": ["stub"],
            "rejected_parts": [],
            "risk_assessment": "low risk",
            "escalate_to_architect": script.escalate,
            "refined_hypothesis": "stub refined hypothesis"
        },
        "architect": architect,
        "tech_leader": {
            "decision": "approve",
            "rationale": "stub rationale",
            "action_items": [],
            "adr_title": "stub adr",
            "session_summary": "stub summary"
        },
        "checkpoint_path": "/tmp/neoland-stub-checkpoint.json",
        "stage_latencies_ms": {}
    }))
    .into_response()
}
