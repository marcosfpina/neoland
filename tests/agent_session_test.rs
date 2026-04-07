//! Agent session persistence tests — PostgreSQL real, no LLM.
//!
//! Requires:
//!   - DATABASE_URL env var set to a live PostgreSQL with migrations applied
//!
//! Run with:
//!   DATABASE_URL=postgres://... cargo test agent_session -- --test-threads=1

use neoland::agents::session::SessionManager;
use uuid::Uuid;

fn db_url() -> Option<String> {
    std::env::var("NEOLAND_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
}

async fn pool() -> Option<sqlx::PgPool> {
    let url = db_url()?;
    sqlx::postgres::PgPoolOptions::new().max_connections(2).connect(&url).await.ok()
}

// ── get_or_create ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_session_get_or_create_initializes_new_session() {
    let Some(pool) = pool().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let mgr = SessionManager::new(pool);
    let session_id = Uuid::new_v4();

    let state = mgr.get_or_create(session_id).await.expect("get_or_create failed");

    assert_eq!(state.session_id, session_id);
    assert_eq!(state.task_count, 0);
    assert!(state.last_decision.is_none());
    assert!(state.active);
}

#[tokio::test]
async fn test_session_get_or_create_is_idempotent() {
    let Some(pool) = pool().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let mgr = SessionManager::new(pool);
    let session_id = Uuid::new_v4();

    let s1 = mgr.get_or_create(session_id).await.expect("first get_or_create");
    let s2 = mgr.get_or_create(session_id).await.expect("second get_or_create");

    assert_eq!(s1.session_id, s2.session_id);
    assert_eq!(s1.task_count, s2.task_count);
}

// ── update_after_pipeline ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_session_update_increments_task_count() {
    let Some(pool) = pool().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let mgr = SessionManager::new(pool);
    let session_id = Uuid::new_v4();

    mgr.get_or_create(session_id).await.expect("create session");

    let decision = serde_json::json!({
        "decision": "approve",
        "adr_title": "ADR-0001: test",
        "session_summary": "Test approved"
    });

    mgr.update_after_pipeline(session_id, decision.clone())
        .await
        .expect("update_after_pipeline");

    let state = mgr.get_or_create(session_id).await.expect("fetch after update");
    assert_eq!(state.task_count, 1);
    assert!(state.last_decision.is_some());

    let last = state.last_decision.unwrap();
    assert_eq!(last["decision"], "approve");
}

#[tokio::test]
async fn test_session_update_accumulates_task_count() {
    let Some(pool) = pool().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let mgr = SessionManager::new(pool);
    let session_id = Uuid::new_v4();
    mgr.get_or_create(session_id).await.expect("create");

    for i in 0..3u32 {
        let decision = serde_json::json!({"decision": "approve", "adr_title": format!("ADR-{i}"), "session_summary": "ok"});
        mgr.update_after_pipeline(session_id, decision).await.expect("update");
    }

    let state = mgr.get_or_create(session_id).await.expect("fetch");
    assert_eq!(state.task_count, 3);
}

// ── isolation ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_sessions_are_isolated_by_session_id() {
    let Some(pool) = pool().await else {
        eprintln!("Skipping: DATABASE_URL not set or unreachable");
        return;
    };

    let mgr = SessionManager::new(pool);
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();

    mgr.get_or_create(id_a).await.expect("create A");
    mgr.get_or_create(id_b).await.expect("create B");

    let decision =
        serde_json::json!({"decision": "reject", "adr_title": "ADR", "session_summary": "ok"});
    mgr.update_after_pipeline(id_a, decision).await.expect("update A");

    let state_b = mgr.get_or_create(id_b).await.expect("fetch B");
    assert_eq!(
        state_b.task_count, 0,
        "Session B should not be affected by updates to Session A"
    );
}
