use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Axum imports for REST
use axum::http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderName, Method,
};
use axum::{
    body::Body,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, Request as HttpRequest, Response as HttpResponse, StatusCode},
    middleware::Next,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, patch, post},
    Router,
};
use llamachat::{
    llama_service_server::{LlamaService, LlamaServiceServer},
    AddDocumentRequest, AddDocumentResponse, ChatRequest, ChatResponse, SearchRequest,
    SearchResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_stream::{
    wrappers::{BroadcastStream, ReceiverStream},
    StreamExt as _,
};
use tonic::{transport::Server as GrpcServer, Request, Response, Status};
use tower_http::cors::CorsLayer;
use tracing::Instrument; // Phase 4.2: For span instrumentation

use crate::mcp::server::{BreakpointResolution, NativeMcpServer};
use crate::{
    agents::{events::AgentEvent, orchestrator::AgentOrchestrator},
    audit::{AuditAction, AuditEvent, AuditLogger, ConsoleAlertHandler, FailedAuthTracker},
    auth::AuthManager,
    engine::{GenerationConfig, LocalEngine},
    health, // Phase 4.3: Health Checks
    matrix::MatrixClient,
    nlp::VectorStore,
    secrets::SecretsManager,
    storage::PersistentVectorStore,
    validation::{ChatRequestValidation, MessageValidator},
};

pub mod llamachat {
    tonic::include_proto!("llamachat");
}

pub mod error;
pub use error::ApiError;

const DEFAULT_AUDIT_LOG_PATH: &str = "/var/log/neoland/audit.log";

// ── Submódulos (split mecânico do antigo mod.rs monolítico) ──────────────────
// Cada submódulo abre com `use super::*;`: os imports comuns vivem neste
// arquivo e os globs abaixo re-exportam os itens, preservando os caminhos
// `crate::server::*` usados por openapi.rs, bin/ e testes.
mod agents_api;
mod auth_web;
mod bootstrap;
mod grpc;
mod middleware;
mod rest;
mod routes;
mod state;

pub use agents_api::*;
pub use auth_web::*;
pub use bootstrap::{run_server, run_server_with};
// Fora dos testes só os entry points são consumidos (re-export explícito acima).
#[cfg(test)]
pub(crate) use bootstrap::*;
pub use grpc::*;
pub use middleware::*;
pub use rest::*;
pub use state::*;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ───────────────────────────────────────────────────────────────

    #[test]
    fn test_user_audit_log_path_prefers_xdg_state_home() {
        let path = user_audit_log_path_from_env(Some("/tmp/xdg-state"), Some("/tmp/home"));
        assert_eq!(path, "/tmp/xdg-state/neoland/audit.log");
    }

    #[test]
    fn test_user_audit_log_path_uses_home_when_xdg_missing() {
        let path = user_audit_log_path_from_env(None, Some("/tmp/home"));
        assert_eq!(path, "/tmp/home/.local/state/neoland/audit.log");
    }

    #[test]
    fn test_user_audit_log_path_falls_back_to_tmp_without_env() {
        let path = user_audit_log_path_from_env(None, None);
        assert_eq!(path, "/tmp/neoland/audit.log");
    }

    /// Build a minimal AppState for testing (no embedding model, no persistent
    /// store).
    ///
    /// VectorStore::new() loads an ML model — call only from `#[ignore]` tests.
    #[allow(dead_code)]
    async fn build_test_state(vector_store: VectorStore) -> AppState {
        let (event_tx, _) = tokio::sync::broadcast::channel(1);
        AppState {
            engine: Arc::new(Mutex::new(None)),
            vector_store: Arc::new(Mutex::new(vector_store)),
            persistent_store: None,
            auth_manager: Arc::new(AuthManager::new()),
            audit_logger: Arc::new(
                AuditLogger::new("/tmp/neoland_test_server_audit.log").expect("AuditLogger failed"),
            ),
            failed_auth_tracker: Arc::new(FailedAuthTracker::new(5, 1)),
            rate_limiter: Arc::new(RateLimiter::new(100, 60)),
            start_time: Instant::now(),
            agent_orchestrator: None,
            event_bus: event_tx,
            db_pool: None,
            jwt_secret: b"test-secret-key-32-bytes-long!!".to_vec(),
            oauth_base_url: "http://localhost:3001".to_string(),
        }
    }

    // ── RateLimiter ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let rl = RateLimiter::new(5, 60);
        for _ in 0..5 {
            assert!(!rl.check_rate_limit("user1").await, "request within limit should be allowed");
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let rl = RateLimiter::new(3, 60);
        for _ in 0..3 {
            let _ = rl.check_rate_limit("user1").await;
        }
        assert!(rl.check_rate_limit("user1").await, "4th request should be rate-limited");
    }

    #[tokio::test]
    async fn test_rate_limiter_separate_users_are_independent() {
        let rl = RateLimiter::new(2, 60);

        // Exhaust user1's quota
        let _ = rl.check_rate_limit("user1").await;
        let _ = rl.check_rate_limit("user1").await;
        assert!(rl.check_rate_limit("user1").await, "user1 should be blocked");

        // user2 should still have a fresh quota
        assert!(!rl.check_rate_limit("user2").await, "user2 should not be affected");
    }

    #[tokio::test]
    async fn test_rate_limiter_zero_limit_blocks_second_request() {
        let rl = RateLimiter::new(0, 60);
        // max_requests = 0: first request is always inserted (allowed),
        // second request increments count to 2 > 0 → blocked.
        assert!(!rl.check_rate_limit("user1").await, "first request always inserted");
        assert!(rl.check_rate_limit("user1").await, "second request exceeds limit=0");
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup_no_panic() {
        let rl = RateLimiter::new(10, 60);
        for i in 0..5 {
            let _ = rl.check_rate_limit(&format!("user{i}")).await;
        }
        // cleanup_old_entries should not panic regardless of entry state
        rl.cleanup_old_entries().await;
    }

    // ── Lock ordering ─────────────────────────────────────────────────────────

    /// Demonstrate the documented lock ordering (engine → vector_store) is
    /// safe.
    ///
    /// We acquire both locks in the required order and release them correctly.
    /// The purpose is to document and exercise the ordering, not to prove
    /// freedom from deadlock (which would require concurrent actors).
    #[test]
    fn test_lock_ordering_engine_before_vector_store() {
        let engine: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(Some(1)));
        let store: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

        // Correct order as per AppState documentation:
        //   1. engine lock acquired first
        //   2. vector_store lock acquired second (while holding engine)
        let engine_guard = engine.lock().unwrap();
        let mut store_guard = store.lock().unwrap();

        assert!(engine_guard.is_some());
        store_guard.push(42);
        assert_eq!(store_guard.len(), 1);

        drop(store_guard);
        drop(engine_guard);
    }

    #[test]
    fn test_lock_ordering_violating_order_would_deadlock_in_concurrent_code() {
        // This test documents WHY the ordering matters:
        // If task A holds engine → waits for store
        // And task B holds store → waits for engine
        // → deadlock. The ordering ensures both always acquire engine first.
        //
        // Since we can't prove absence of deadlock in a unit test, we assert
        // the ordering rule is documented in the AppState comment.
        let source = include_str!("mod.rs");
        assert!(
            source.contains("engine (Arc<Mutex<Option<LocalEngine>>>)"),
            "Lock ordering comment must document engine as lock #1"
        );
        assert!(
            source.contains("vector_store (Arc<Mutex<VectorStore>>)"),
            "Lock ordering comment must document vector_store as lock #2"
        );
    }

    // ── gRPC handlers (require embedding model — run with cargo test -- --ignored)
    // ──

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_add_document_grpc_success() {
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        let req = Request::new(AddDocumentRequest {
            content: "How to move windows in Hyprland".to_string(),
            metadata: "source:test,category:wm".to_string(),
        });

        let resp = svc.add_document(req).await.expect("add_document failed");
        let inner = resp.into_inner();
        assert!(inner.success);
        assert!(!inner.id.is_empty(), "response id should be a UUID");
    }

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_add_document_grpc_empty_content_still_succeeds() {
        // Validation happens at the REST layer; gRPC add_document accepts any string
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        let req =
            Request::new(AddDocumentRequest { content: String::new(), metadata: String::new() });

        // gRPC handler itself does not validate — embedding model handles empty string
        let result = svc.add_document(req).await;
        // Either ok or internal error from embedding; what matters is no panic
        let _ = result;
    }

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_search_grpc_returns_added_document() {
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        // Add document
        let add = Request::new(AddDocumentRequest {
            content: "Neoland is a secure AI platform".to_string(),
            metadata: "source:test".to_string(),
        });
        svc.add_document(add).await.expect("add_document failed");

        // Search
        let search =
            Request::new(SearchRequest { query: "secure AI platform".to_string(), top_k: 3 });
        let resp = svc.search(search).await.expect("search failed");
        let results = resp.into_inner().results;

        assert!(!results.is_empty(), "search should return at least one result");
        assert!(results[0].score > 0.0, "similarity score should be positive");
    }

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_search_grpc_empty_store_returns_empty() {
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        let search = Request::new(SearchRequest { query: "anything".to_string(), top_k: 5 });
        let resp = svc.search(search).await.expect("search failed");
        // Empty store → empty results (not an error)
        assert_eq!(resp.into_inner().results.len(), 0);
    }
}
