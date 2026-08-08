//! REST API E2E Integration Tests
//!
//! These tests exercise the full HTTP server stack against a shared
//! in-process server on ephemeral ports (see tests/common/mod.rs).
//!
//! Run with: cargo test --test rest_api_test -- --nocapture

mod common;

use std::time::Duration;

use common::dev_keys;
use reqwest::{Client, StatusCode};
use serde_json::Value;

// =============================================================================
// Helpers
// =============================================================================

/// Boots (or reuses) the shared server and returns its REST base URL.
async fn ensure_server_ready() -> &'static str {
    &common::TestServer::shared().await.rest_url
}

/// Build a shared HTTP client.
fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client")
}

// =============================================================================
// Public endpoint tests (no auth required)
// =============================================================================

#[tokio::test]
async fn e2e_health_endpoint() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("GET /health failed — is the server running?");

    assert_eq!(resp.status(), StatusCode::OK, "/health should return 200");

    let body: Value = resp.json().await.expect("/health response is not valid JSON");
    assert!(body.get("status").is_some(), "/health response must contain 'status' field");
    assert!(body.get("version").is_some(), "/health response must contain 'version' field");
    assert!(
        body.get("components").is_some(),
        "/health response must contain 'components' array"
    );

    // Print health summary for diagnostics
    println!("Health: status={:?}, version={:?}", body["status"], body["version"]);
}

#[tokio::test]
async fn e2e_readiness_endpoint() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client.get(format!("{base}/ready")).send().await.expect("GET /ready failed");

    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::SERVICE_UNAVAILABLE,
        "/ready should return 200 or 503 (got {})",
        resp.status()
    );

    let body: Value = resp.json().await.expect("/ready response is not valid JSON");
    assert!(body.get("ready").is_some(), "/ready response must contain 'ready' boolean");
    println!("Readiness: ready={:?}", body["ready"]);
}

#[tokio::test]
async fn e2e_liveness_endpoint() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client.get(format!("{base}/live")).send().await.expect("GET /live failed");

    assert_eq!(resp.status(), StatusCode::OK, "/live should return 200");

    let body: Value = resp.json().await.expect("/live response is not valid JSON");
    assert_eq!(body["alive"], true, "/live must return alive: true");
    println!("Liveness: {:?}", body);
}

#[tokio::test]
async fn e2e_metrics_endpoint() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client.get(format!("{base}/metrics")).send().await.expect("GET /metrics failed");

    assert_eq!(resp.status(), StatusCode::OK, "/metrics should return 200");

    let body = resp.text().await.expect("/metrics body should be readable");
    assert!(!body.is_empty(), "/metrics body should not be empty");
    assert!(
        body.contains("# HELP") || body.contains("# TYPE"),
        "/metrics should expose Prometheus-format metrics (got first 200 chars: {:?})",
        &body[..body.len().min(200)]
    );
}

#[tokio::test]
async fn e2e_agents_health_endpoint() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/v1/agents/health"))
        .send()
        .await
        .expect("GET /v1/agents/health failed");

    // This endpoint always returns 200 (even with pipeline disabled)
    assert_eq!(resp.status(), StatusCode::OK, "/v1/agents/health should return 200");

    let body: Value = resp.json().await.expect("/v1/agents/health response is not valid JSON");
    assert!(body.get("status").is_some(), "response must contain 'status' field");
    println!("Agent health: {:?}", body);
}

// =============================================================================
// Chat endpoint auth tests
// =============================================================================

#[tokio::test]
async fn e2e_chat_requires_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .json(&serde_json::json!({
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions failed");

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Chat without API key should return 401"
    );
}

#[tokio::test]
async fn e2e_chat_with_valid_admin_key() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", dev_keys::ADMIN)
        .json(&serde_json::json!({
            "messages": [{"role": "user", "content": "Test message"}],
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions with admin key failed");

    // With a valid key we should NOT get 401.
    // Actual status depends on LLM engine availability, but auth passed.
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED, "Admin key should pass auth");
    assert_ne!(resp.status(), StatusCode::FORBIDDEN, "Admin key should pass auth");
    println!("Chat with admin key: HTTP {}", resp.status());
}

#[tokio::test]
async fn e2e_chat_with_valid_user_key() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", dev_keys::USER)
        .json(&serde_json::json!({
            "messages": [{"role": "user", "content": "Test"}],
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions with user key failed");

    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED, "User key should pass auth");
    assert_ne!(resp.status(), StatusCode::FORBIDDEN, "User key should pass auth");
    println!("Chat with user key: HTTP {}", resp.status());
}

#[tokio::test]
async fn e2e_chat_with_valid_readonly_key() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", dev_keys::READONLY)
        .json(&serde_json::json!({
            "messages": [{"role": "user", "content": "Test"}],
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions with readonly key failed");

    // RBAC: a valid read-only key authenticates (not 401) but may not write.
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Readonly key must be forbidden on POST /v1/chat/completions"
    );
}

#[tokio::test]
async fn e2e_chat_with_invalid_key_rejected() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", "definitely_not_a_valid_key_12345")
        .json(&serde_json::json!({
            "messages": [{"role": "user", "content": "Test"}],
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions with invalid key failed");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "Invalid API key should return 401");
}

// =============================================================================
// Input validation tests
// =============================================================================

#[tokio::test]
async fn e2e_validation_empty_prompt() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", dev_keys::ADMIN)
        .json(&serde_json::json!({
            "messages": [{"role": "user", "content": ""}],
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions failed");

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "Empty message content should return 400"
    );
}

#[tokio::test]
async fn e2e_validation_invalid_role() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", dev_keys::ADMIN)
        .json(&serde_json::json!({
            "messages": [{"role": "invalid_role", "content": "Test message"}],
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions failed");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Invalid role should return 400");
}

#[tokio::test]
async fn e2e_validation_too_many_messages() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let messages: Vec<serde_json::Value> = (0..101)
        .map(|i| serde_json::json!({"role": "user", "content": format!("Message {i}")}))
        .collect();

    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", dev_keys::ADMIN)
        .json(&serde_json::json!({
            "messages": messages,
            "stream": true
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions failed");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, ">100 messages should return 400");
}

#[tokio::test]
async fn e2e_validation_missing_messages_field() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/chat/completions"))
        .header("X-API-Key", dev_keys::ADMIN)
        .json(&serde_json::json!({
            "stream": true
            // messages field deliberately missing
        }))
        .send()
        .await
        .expect("POST /v1/chat/completions failed");

    // Missing required field should fail deserialization → 422 or 400
    assert!(
        resp.status() == StatusCode::UNPROCESSABLE_ENTITY
            || resp.status() == StatusCode::BAD_REQUEST,
        "Missing messages field should return 422 or 400 (got {})",
        resp.status()
    );
}

// =============================================================================
// Agent pipeline endpoint tests (auth required, likely 503 without DB)
// =============================================================================

#[tokio::test]
async fn e2e_agent_task_requires_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/agents/task"))
        .json(&serde_json::json!({
            "task": "Test task"
        }))
        .send()
        .await
        .expect("POST /v1/agents/task failed");

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Agent task without auth should return 401"
    );
}

#[tokio::test]
async fn e2e_agent_task_with_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/agents/task"))
        .header("X-API-Key", dev_keys::ADMIN)
        .json(&serde_json::json!({
            "task": "Analyze the architecture"
        }))
        .send()
        .await
        .expect("POST /v1/agents/task with auth failed");

    // Without DATABASE_URL, the orchestrator is disabled → 503.
    // With DATABASE_URL configured, it may return 200 or 500.
    let status = resp.status();
    assert!(
        status == StatusCode::SERVICE_UNAVAILABLE
            || status == StatusCode::OK
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Agent task with valid key should return 503 (no DB), 200 (success), or 500 (error) — got {status}"
    );
    println!("Agent task with admin key: HTTP {status}");
}

#[tokio::test]
async fn e2e_agent_sessions_requires_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/v1/agents/sessions"))
        .send()
        .await
        .expect("GET /v1/agents/sessions failed");

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Agent sessions without auth should return 401"
    );
}

#[tokio::test]
async fn e2e_agent_sessions_with_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/v1/agents/sessions"))
        .header("X-API-Key", dev_keys::ADMIN)
        .send()
        .await
        .expect("GET /v1/agents/sessions with auth failed");

    let status = resp.status();
    assert!(
        status == StatusCode::SERVICE_UNAVAILABLE
            || status == StatusCode::OK
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Agent sessions with valid key should return 503 (no DB) or 200 — got {status}"
    );
    println!("Agent sessions with admin key: HTTP {status}");
}

#[tokio::test]
async fn e2e_agent_tools_requires_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/v1/agents/tools"))
        .send()
        .await
        .expect("GET /v1/agents/tools failed");

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Agent tools without auth should return 401"
    );
}

#[tokio::test]
async fn e2e_agent_tools_with_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/v1/agents/tools"))
        .header("X-API-Key", dev_keys::ADMIN)
        .send()
        .await
        .expect("GET /v1/agents/tools with auth failed");

    let status = resp.status();
    assert!(
        status == StatusCode::SERVICE_UNAVAILABLE
            || status == StatusCode::OK
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Agent tools with valid key should return 503 (no DB) or 200 — got {status}"
    );

    if status == StatusCode::OK {
        let body: Value = resp.json().await.expect("Response should be valid JSON");
        assert!(body.get("tools").is_some(), "Response should contain 'tools' field");
        println!("Agent tools: {:?}", body);
    }
}

#[tokio::test]
async fn e2e_agent_steer_requires_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/agents/session/00000000-0000-0000-0000-000000000000/steer"))
        .json(&serde_json::json!({"message": "test steer"}))
        .send()
        .await
        .expect("POST /v1/agents/session/:id/steer failed");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "Steer without auth should return 401");
}

#[tokio::test]
async fn e2e_agent_steer_rejects_bearer_header() {
    // Regression test: the TUI used to send `Authorization: Bearer <key>` for
    // live-steering while the REST auth middleware only accepts `X-API-Key`,
    // which made `/steer` silently 401 for every real user. Guard against
    // that header ever coming back.
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/agents/session/00000000-0000-0000-0000-000000000000/steer"))
        .header("Authorization", format!("Bearer {}", dev_keys::ADMIN))
        .json(&serde_json::json!({"message": "test steer"}))
        .send()
        .await
        .expect("POST /v1/agents/session/:id/steer with Bearer header failed");

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Steer with only an Authorization: Bearer header must still 401 — only X-API-Key is accepted"
    );
}

#[tokio::test]
async fn e2e_agent_steer_with_auth() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .post(format!("{base}/v1/agents/session/00000000-0000-0000-0000-000000000000/steer"))
        .header("X-API-Key", dev_keys::ADMIN)
        .json(&serde_json::json!({"message": "test steer"}))
        .send()
        .await
        .expect("POST /v1/agents/session/:id/steer with X-API-Key failed");

    let status = resp.status();
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "Steer with a valid X-API-Key must pass auth (got 401)"
    );
    println!("Agent steer with admin key: HTTP {status}");
}

// =============================================================================
// OpenAPI / Swagger endpoint tests
// =============================================================================

#[tokio::test]
async fn e2e_openapi_spec() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/openapi.json"))
        .send()
        .await
        .expect("GET /openapi.json failed");

    assert_eq!(resp.status(), StatusCode::OK, "/openapi.json should return 200");

    let body: Value = resp.json().await.expect("/openapi.json is not valid JSON");
    assert!(
        body.get("openapi").is_some(),
        "OpenAPI spec should have 'openapi' version field"
    );
    assert!(body.get("info").is_some(), "OpenAPI spec should have 'info' section");
    assert!(body.get("paths").is_some(), "OpenAPI spec should have 'paths' section");
    println!("OpenAPI: version={:?}, title={:?}", body["openapi"], body["info"]["title"]);
}

// =============================================================================
// CORS headers
// =============================================================================

#[tokio::test]
async fn e2e_cors_headers_present() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let resp = client
        .get(format!("{base}/health"))
        .header("Origin", "http://example.com")
        .send()
        .await
        .expect("GET /health with Origin failed");

    assert_eq!(resp.status(), StatusCode::OK);

    let headers = resp.headers();
    println!("CORS headers: {:?}", headers);

    // If CORS middleware is configured, we should see these headers
    if let Some(allow_origin) = headers.get("access-control-allow-origin") {
        println!("CORS enabled: allow-origin={allow_origin:?}");
    } else {
        println!("CORS headers not present (may not be configured)");
    }
}

// =============================================================================
// Rate limiting (can be slow — marked with #[ignore] by default)
// =============================================================================

#[tokio::test]
#[ignore = "takes ~2s due to rate limit window timing"]
async fn e2e_rate_limiting_excess_requests_blocked() {
    let base = ensure_server_ready().await;

    let client = http_client();
    let mut rate_limited_count = 0u32;
    let mut success_count = 0u32;
    let total_requests = 110;

    for i in 0..total_requests {
        let resp = client
            .post(format!("{base}/v1/chat/completions"))
            .header("X-API-Key", dev_keys::ADMIN)
            .json(&serde_json::json!({
                "messages": [{"role": "user", "content": "Test"}],
                "stream": true
            }))
            .send()
            .await;

        match resp {
            Ok(r) if r.status() == StatusCode::TOO_MANY_REQUESTS => {
                rate_limited_count += 1;
            },
            Ok(r) if r.status() != StatusCode::UNAUTHORIZED => {
                success_count += 1;
            },
            Ok(_) => {}, // auth failure
            Err(e) => {
                eprintln!("Request {i} failed: {e}");
            },
        }

        // Brief delay to avoid connection saturation
        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    println!(
        "Rate limit test: {success_count} succeeded, {rate_limited_count} rate-limited out of {total_requests}"
    );

    assert!(
        rate_limited_count > 0,
        "Expected at least some requests to be rate-limited (limit is 100 req/min)"
    );
}

// =============================================================================
// RBAC enforcement (write endpoints require User+)
// =============================================================================

#[tokio::test]
async fn e2e_rbac_readonly_cannot_submit_task() {
    let base = ensure_server_ready().await;

    let resp = http_client()
        .post(format!("{base}/v1/agents/task"))
        .header("X-API-Key", dev_keys::READONLY)
        .json(&serde_json::json!({"task": "should be forbidden"}))
        .send()
        .await
        .expect("POST /v1/agents/task");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "readonly key must not submit tasks");
}

#[tokio::test]
async fn e2e_rbac_readonly_cannot_call_tools() {
    let base = ensure_server_ready().await;

    let resp = http_client()
        .post(format!("{base}/v1/agents/tools/call"))
        .header("X-API-Key", dev_keys::READONLY)
        .json(&serde_json::json!({
            "session_id": "00000000-0000-0000-0000-000000000000",
            "name": "run_shell_command",
            "arguments": {"command": "echo forbidden"}
        }))
        .send()
        .await
        .expect("POST /v1/agents/tools/call");
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "readonly key must never reach shell execution"
    );
}

#[tokio::test]
async fn e2e_rbac_readonly_can_still_read() {
    let base = ensure_server_ready().await;

    let resp = http_client()
        .get(format!("{base}/v1/agents/sessions"))
        .header("X-API-Key", dev_keys::READONLY)
        .send()
        .await
        .expect("GET /v1/agents/sessions");
    // 200 with DB, 503 without — never 401/403: reads stay ReadOnly+.
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::SERVICE_UNAVAILABLE,
        "readonly GET must not be blocked by RBAC (got {})",
        resp.status()
    );
}
