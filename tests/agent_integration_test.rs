//! Agent end-to-end integration tests.
//!
//! Requires ALL of:
//!   - DATABASE_URL → live PostgreSQL with migrations 001 + 002 applied
//!   - LLM_API_KEY  → API key for the configured LLM provider
//!   - DSPy pipeline running at NEOLAND_AGENTS_DSPY_URL (default: http://localhost:8001)
//!     Start with: agents-start (inside nix develop)
//!
//! Run with:
//!   DATABASE_URL=postgres://... LLM_API_KEY=sk-... cargo test
//! agent_integration -- --test-threads=1

use neoland::{
    agents::{client::AgentDecision, orchestrator::AgentOrchestrator},
    config::Config,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn db_url() -> Option<String> {
    std::env::var("NEOLAND_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
}

fn dspy_available() -> bool {
    // Quick TCP probe: if 8001 is not listening, skip
    std::net::TcpStream::connect_timeout(
        &"127.0.0.1:8001".parse().unwrap(),
        std::time::Duration::from_secs(1),
    )
    .is_ok()
}

async fn build_orchestrator() -> Option<AgentOrchestrator> {
    let url = db_url()?;
    let pool = PgPoolOptions::new().max_connections(2).connect(&url).await.ok()?;
    let cfg = Config::load();
    AgentOrchestrator::new(pool, &cfg.agents).ok()
}

// ── health_check
// ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_agent_pipeline_health_check() {
    if !dspy_available() {
        eprintln!("Skipping: DSPy pipeline not running on :8001");
        return;
    }
    let Some(orch) = build_orchestrator().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let healthy = orch.health_check().await.expect("health_check failed");
    assert!(healthy, "DSPy pipeline should report healthy");
}

// ── execute_task
// ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_execute_task_returns_valid_decision() {
    if !dspy_available() {
        eprintln!("Skipping: DSPy pipeline not running on :8001");
        return;
    }
    let Some(orch) = build_orchestrator().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let session_id = Uuid::new_v4();
    let result = orch
        .execute_task(
            "Should we add rate limiting to the agent API?",
            session_id,
            "user",
            String::new(),
        )
        .await
        .expect("execute_task failed");

    // Decision must be one of the valid enum variants
    let valid = matches!(
        result.tech_leader.decision,
        AgentDecision::Approve
            | AgentDecision::Reject
            | AgentDecision::Defer
            | AgentDecision::Escalate
    );
    assert!(valid, "tech_leader decision must be a valid enum variant");

    // Junior confidence must be in [0, 1]
    assert!(
        (0.0..=1.0).contains(&result.junior.confidence),
        "confidence must be in [0.0, 1.0], got {}",
        result.junior.confidence
    );

    // ADR title must not be empty
    assert!(!result.tech_leader.adr_title.is_empty(), "adr_title must not be empty");

    // Checkpoint path must not be empty
    assert!(!result.checkpoint_path.is_empty(), "checkpoint_path must not be empty");
}

#[tokio::test]
async fn test_execute_task_persists_session() {
    if !dspy_available() {
        eprintln!("Skipping: DSPy pipeline not running on :8001");
        return;
    }
    let Some(orch) = build_orchestrator().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let session_id = Uuid::new_v4();

    orch.execute_task(
        "Evaluate adding circuit breaker to the LLM client",
        session_id,
        "user",
        String::new(),
    )
    .await
    .expect("execute_task failed");

    // Verify session state was persisted
    let state = orch.get_session(session_id).await.expect("get_session failed");
    assert_eq!(state.session_id, session_id);
    assert_eq!(state.task_count, 1, "task_count should be 1 after one task");
    assert!(state.last_decision.is_some(), "last_decision should be populated");
}

#[tokio::test]
async fn test_execute_two_tasks_same_session_increments_count() {
    if !dspy_available() {
        eprintln!("Skipping: DSPy pipeline not running on :8001");
        return;
    }
    let Some(orch) = build_orchestrator().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let session_id = Uuid::new_v4();

    orch.execute_task("Task 1: evaluate caching strategy", session_id, "user", String::new())
        .await
        .expect("task 1 failed");

    orch.execute_task("Task 2: evaluate retry policy", session_id, "user", String::new())
        .await
        .expect("task 2 failed");

    let state = orch.get_session(session_id).await.expect("get_session");
    assert_eq!(state.task_count, 2, "two tasks in same session should yield task_count=2");
}
