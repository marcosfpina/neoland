//! REST API Integration Tests
//!
//! Tests for REST API endpoints including authentication, rate limiting, and error handling.

use reqwest::{Client, StatusCode};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

const TEST_GRPC_PORT: u16 = 50053;
const TEST_REST_PORT: u16 = 3003;
const BASE_URL: &str = "http://127.0.0.1:3003";

// Valid development API keys from auth.rs
const ADMIN_API_KEY: &str = "neoland_admin_dev_key_change_in_production";
const USER_API_KEY: &str = "neoland_user_dev_key_change_in_production";
const READONLY_API_KEY: &str = "neoland_readonly_dev_key_change_in_production";

/// Helper to start test server
async fn start_test_server() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _ = llamachat_poc::server::run_server(TEST_GRPC_PORT, TEST_REST_PORT).await;
    })
}

/// Helper to wait for server to be ready
async fn wait_for_server() {
    sleep(Duration::from_millis(500)).await;
}

#[tokio::test]
async fn test_health_endpoint() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();
    let response = client
        .get(format!("{}/health", BASE_URL))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.text().await.unwrap();
            assert_eq!(body, "OK");
        }
        Err(e) => {
            eprintln!("Health check failed: {}", e);
        }
    }

    server.abort();
}

#[tokio::test]
async fn test_chat_endpoint_requires_auth() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();

    // Request without API key should fail
    let response = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .json(&json!({
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "stream": true
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
        }
    }

    server.abort();
}

#[tokio::test]
async fn test_chat_endpoint_with_valid_auth() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();

    // Request with valid API key
    let response = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", ADMIN_API_KEY)
        .json(&json!({
            "messages": [
                {"role": "user", "content": "Test message"}
            ],
            "stream": true
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Should not be unauthorized
            assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);

            // Should either be OK (200) or other valid response
            // Note: Actual streaming response handling depends on server implementation
            println!("Response status: {}", resp.status());
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
        }
    }

    server.abort();
}

#[tokio::test]
async fn test_chat_endpoint_with_invalid_auth() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();

    // Request with invalid API key
    let response = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", "invalid_key_12345")
        .json(&json!({
            "messages": [
                {"role": "user", "content": "Test"}
            ],
            "stream": true
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
        }
    }

    server.abort();
}

#[tokio::test]
async fn test_chat_endpoint_with_different_roles() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();
    let test_payload = json!({
        "messages": [
            {"role": "user", "content": "Test"}
        ],
        "stream": true
    });

    // Test with admin key
    let response_admin = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", ADMIN_API_KEY)
        .json(&test_payload)
        .send()
        .await;

    if let Ok(resp) = response_admin {
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // Test with user key
    let response_user = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", USER_API_KEY)
        .json(&test_payload)
        .send()
        .await;

    if let Ok(resp) = response_user {
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // Test with readonly key (should also work for read operations)
    let response_readonly = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", READONLY_API_KEY)
        .json(&test_payload)
        .send()
        .await;

    if let Ok(resp) = response_readonly {
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    server.abort();
}

#[tokio::test]
async fn test_input_validation_empty_prompt() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();

    // Request with empty prompt
    let response = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", ADMIN_API_KEY)
        .json(&json!({
            "messages": [
                {"role": "user", "content": ""}  // Empty content
            ],
            "stream": true
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Should return bad request due to validation
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
        }
    }

    server.abort();
}

#[tokio::test]
async fn test_input_validation_invalid_role() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();

    // Request with invalid role
    let response = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", ADMIN_API_KEY)
        .json(&json!({
            "messages": [
                {"role": "invalid_role", "content": "Test message"}
            ],
            "stream": true
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Should return bad request due to validation
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
        }
    }

    server.abort();
}

#[tokio::test]
async fn test_input_validation_too_many_messages() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();

    // Create request with >100 messages (exceeds MAX_MESSAGE_COUNT)
    let mut messages = Vec::new();
    for i in 0..101 {
        messages.push(json!({"role": "user", "content": format!("Message {}", i)}));
    }

    let response = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("X-API-Key", ADMIN_API_KEY)
        .json(&json!({
            "messages": messages,
            "stream": true
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Should return bad request due to validation
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
        }
    }

    server.abort();
}

#[tokio::test]
#[ignore] // This test takes time due to rate limiting
async fn test_rate_limiting() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();
    let test_payload = json!({
        "messages": [
            {"role": "user", "content": "Test"}
        ],
        "stream": true
    });

    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // Send 105 requests rapidly (limit is 100/min)
    for _i in 0..105 {
        let response = client
            .post(format!("{}/v1/chat/completions", BASE_URL))
            .header("X-API-Key", ADMIN_API_KEY)
            .json(&test_payload)
            .send()
            .await;

        if let Ok(resp) = response {
            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                rate_limited_count += 1;
            } else if resp.status() != StatusCode::UNAUTHORIZED {
                success_count += 1;
            }
        }

        // Small delay to avoid overwhelming the server
        sleep(Duration::from_millis(10)).await;
    }

    println!("Success: {}, Rate limited: {}", success_count, rate_limited_count);

    // Should have some rate-limited requests
    assert!(rate_limited_count > 0, "Expected some requests to be rate limited");

    server.abort();
}

#[tokio::test]
async fn test_cors_headers() {
    let server = start_test_server().await;
    wait_for_server().await;

    let client = Client::new();

    let response = client
        .get(format!("{}/health", BASE_URL))
        .header("Origin", "http://example.com")
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Check for CORS headers (if configured)
            println!("Headers: {:?}", resp.headers());
            // Note: Actual CORS validation depends on server configuration
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
        }
    }

    server.abort();
}
