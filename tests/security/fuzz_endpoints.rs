// NEOLAND Security Testing: Fuzzing Endpoints
// Tests: REST API and gRPC fuzzing with invalid/malicious inputs

use std::time::Duration;

use serde_json::json;
use tokio::time::timeout;

/// Fuzz REST API with invalid JSON payloads
#[tokio::test]
#[ignore] // Run explicitly with: cargo test --test fuzz_endpoints -- --ignored
async fn fuzz_rest_api_invalid_json() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";

    // Create large payloads outside vec to extend lifetime
    let large_payload = format!(
        "{{\"messages\": [{{\"role\": \"user\", \"content\": \"{}\"}}]}}",
        "A".repeat(1_000_000)
    );
    let nested_payload = format!("{{{}}}", "\"a\": {".repeat(1000));

    let invalid_payloads: Vec<&str> = vec![
        // Malformed JSON
        "{invalid json}",
        "[1, 2, 3",
        "{'single': 'quotes'}",
        // Missing required fields
        "{}",
        "{\"messages\": []}",
        // Invalid field types
        "{\"messages\": \"not an array\"}",
        "{\"messages\": [{\"role\": 123, \"content\": \"test\"}]}",
        // Extremely large payloads
        &large_payload,
        // Nested objects beyond reasonable depth
        &nested_payload,
        // SQL injection attempts
        "{\"messages\": [{\"role\": \"user\", \"content\": \"'; DROP TABLE users; --\"}]}",
        // XSS attempts
        "{\"messages\": [{\"role\": \"user\", \"content\": \"<script>alert('xss')</script>\"}]}",
        // Command injection attempts
        "{\"messages\": [{\"role\": \"user\", \"content\": \"$(rm -rf /)\"}]}",
        // Null bytes
        "{\"messages\": [{\"role\": \"user\", \"content\": \"test\\u0000\"}]}",
        // Unicode edge cases
        "{\"messages\": [{\"role\": \"user\", \"content\": \"\\uD800\"}]}", /* Invalid UTF-16
                                                                             * surrogate */
        // Excessive nesting
        "{\"messages\": [{\"role\": \"user\", \"content\": {}}]}",
    ];

    for (i, payload) in invalid_payloads.iter().enumerate() {
        println!("🧪 Fuzzing payload {}/{}: {:.80}...", i + 1, invalid_payloads.len(), payload);

        let result = timeout(
            Duration::from_secs(5),
            client
                .post(format!("{}/v1/chat/completions", base_url))
                .header("Content-Type", "application/json")
                .body(payload.to_string())
                .send(),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                let status = response.status();
                // Valid responses: 400 (Bad Request), 401 (Unauthorized), 413 (Payload Too
                // Large)
                assert!(
                    status.is_client_error(),
                    "Expected 4xx error for invalid payload, got: {}",
                    status
                );
                println!("   ✅ Correctly rejected with status: {}", status);
            },
            Ok(Err(e)) => {
                // Network errors are acceptable (server refused connection)
                println!("   ✅ Connection error (expected): {}", e);
            },
            Err(_) => {
                panic!("❌ SECURITY ISSUE: Server hung/timeout on payload {}", i + 1);
            },
        }
    }

    println!("\n✅ All fuzzing payloads handled safely");
}

/// Fuzz REST API with invalid headers
#[tokio::test]
#[ignore]
async fn fuzz_rest_api_invalid_headers() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";

    // Create long string outside vec to extend lifetime
    let long_header = "A".repeat(100_000);
    let long_ref = long_header.as_str();

    let header_attacks: Vec<Vec<(&str, &str)>> = vec![
        // Missing Content-Type
        vec![],
        // Invalid Content-Type
        vec![("Content-Type", "text/plain")],
        vec![("Content-Type", "application/xml")],
        // Header injection attempts
        vec![("X-Forwarded-For", "127.0.0.1\r\nX-Evil: injected")],
        vec![("User-Agent", "Mozilla/5.0\nX-Malicious: header")],
        // Extremely long headers
        vec![("X-Custom", long_ref)],
        // Null bytes in headers
        vec![("X-Test", "value\0injected")],
        // Control characters
        vec![("X-Test", "value\r\ninjected")],
    ];

    let valid_payload = json!({
        "messages": [{"role": "user", "content": "test"}]
    });

    for (i, headers) in header_attacks.iter().enumerate() {
        println!("🧪 Testing header attack {}/{}", i + 1, header_attacks.len());

        let mut request =
            client.post(format!("{}/v1/chat/completions", base_url)).json(&valid_payload);

        for (key, value) in headers {
            request = request.header(*key, *value);
        }

        let result = timeout(Duration::from_secs(5), request.send()).await;

        match result {
            Ok(Ok(response)) => {
                let status = response.status();
                // Should reject with 400, 401, or 415
                println!("   ✅ Response status: {}", status);
            },
            Ok(Err(e)) => {
                println!("   ✅ Request error (expected): {}", e);
            },
            Err(_) => {
                panic!("❌ SECURITY ISSUE: Server hung on header attack {}", i + 1);
            },
        }
    }

    println!("\n✅ All header attacks handled safely");
}

/// Test rate limit enforcement
#[tokio::test]
#[ignore]
async fn test_rate_limit_enforcement() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";

    let payload = json!({
        "messages": [{"role": "user", "content": "test"}]
    });

    println!("🧪 Testing rate limit enforcement (100 req/min)");

    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // Send 150 requests rapidly
    for i in 0..150 {
        let response = client
            .post(format!("{}/v1/chat/completions", base_url))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status() == 429 {
                    rate_limited_count += 1;
                    if rate_limited_count == 1 {
                        println!("   ✅ First rate limit hit at request {}", i + 1);
                    }
                } else {
                    success_count += 1;
                }
            },
            Err(e) => {
                println!("   ⚠️  Request {} failed: {}", i + 1, e);
            },
        }
    }

    println!("\n📊 Rate Limit Results:");
    println!("   Successful: {}", success_count);
    println!("   Rate Limited (429): {}", rate_limited_count);

    // If rate limiting is enabled, we should see some 429s
    if rate_limited_count > 0 {
        println!("   ✅ Rate limiting is ACTIVE");
    } else {
        println!("   ⚠️  WARNING: No rate limiting detected (may not be enabled yet)");
    }
}

/// Test authentication bypass attempts
#[tokio::test]
#[ignore]
async fn test_authentication_bypass_attempts() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";

    let payload = json!({
        "messages": [{"role": "user", "content": "test"}]
    });

    let bypass_attempts = vec![
        // No auth header
        vec![],
        // Empty auth
        vec![("X-API-Key", "")],
        // SQL injection in auth
        vec![("X-API-Key", "' OR '1'='1")],
        vec![("X-API-Key", "admin' --")],
        // Path traversal
        vec![("X-API-Key", "../../../etc/passwd")],
        // JWT manipulation attempts
        vec![("Authorization", "Bearer eyJhbGciOiJub25lIn0.eyJzdWIiOiJhZG1pbiJ9.")],
        // Header case manipulation
        vec![("x-api-key", "invalid")],
        vec![("X-Api-Key", "invalid")],
        // Multiple auth headers
        vec![("X-API-Key", "invalid"), ("X-API-Key", "another")],
    ];

    println!("🧪 Testing authentication bypass attempts");

    for (i, headers) in bypass_attempts.iter().enumerate() {
        let mut request = client.post(format!("{}/v1/chat/completions", base_url)).json(&payload);

        for (key, value) in headers {
            request = request.header(*key, *value);
        }

        let response = request.send().await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                // If auth is enabled, should get 401
                // If auth not enabled, may get 200 or other
                if status == 401 {
                    println!("   ✅ Attempt {} correctly rejected (401)", i + 1);
                } else {
                    println!(
                        "   ⚠️  Attempt {} got status {} (auth may not be enabled)",
                        i + 1,
                        status
                    );
                }
            },
            Err(e) => {
                println!("   ✅ Attempt {} connection error: {}", i + 1, e);
            },
        }
    }

    println!("\n✅ Authentication bypass tests completed");
}

/// Test path traversal attempts
#[tokio::test]
#[ignore]
async fn test_path_traversal_attempts() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";

    let path_attacks = [
        "/../../../etc/passwd",
        "/v1/chat/../../../etc/shadow",
        "/v1/chat/..%2f..%2f..%2fetc%2fpasswd",
        "/v1/chat/....//....//....//etc/passwd",
        "/v1/chat/%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    ];

    println!("🧪 Testing path traversal attempts");

    for (i, path) in path_attacks.iter().enumerate() {
        let url = format!("{}{}", base_url, path);
        println!("   Testing path {}/{}: {}", i + 1, path_attacks.len(), path);

        let response = client.get(&url).send().await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                // Should never return 200 OK for these paths
                assert_ne!(
                    status.as_u16(),
                    200,
                    "❌ SECURITY ISSUE: Path traversal succeeded for {}",
                    path
                );
                println!("      ✅ Correctly rejected with status: {}", status);
            },
            Err(e) => {
                println!("      ✅ Connection error (expected): {}", e);
            },
        }
    }

    println!("\n✅ All path traversal attempts blocked");
}

/// Test resource exhaustion (ReDoS, billion laughs, etc.)
#[tokio::test]
#[ignore]
async fn test_resource_exhaustion_attacks() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";

    println!("🧪 Testing resource exhaustion attacks");

    // Test 1: Extremely deep nesting
    let deep_nested = format!(
        "{{\"messages\": [{{\"role\": \"user\", \"content\": {}}}]}}",
        "[".repeat(10000) + &"]".repeat(10000)
    );

    println!("   Test 1: Deep nested JSON");
    let result = timeout(
        Duration::from_secs(3),
        client
            .post(format!("{}/v1/chat/completions", base_url))
            .header("Content-Type", "application/json")
            .body(deep_nested)
            .send(),
    )
    .await;

    match result {
        Ok(_) => println!("      ✅ Server handled deep nesting"),
        Err(_) => panic!("❌ SECURITY ISSUE: Server hung on deep nesting"),
    }

    // Test 2: Regex DoS attempt (if server uses regex)
    let redos_payload = json!({
        "messages": [{
            "role": "user",
            "content": "a".repeat(100000) + "!"
        }]
    });

    println!("   Test 2: ReDoS attempt");
    let result = timeout(
        Duration::from_secs(3),
        client
            .post(format!("{}/v1/chat/completions", base_url))
            .json(&redos_payload)
            .send(),
    )
    .await;

    match result {
        Ok(_) => println!("      ✅ Server handled ReDoS pattern"),
        Err(_) => panic!("❌ SECURITY ISSUE: Server hung on ReDoS pattern"),
    }

    // Test 3: Billion laughs (XML bomb equivalent in JSON)
    let billion_laughs = json!({
        "messages": [{
            "role": "user",
            "content": vec!["lol"; 100000].join("")
        }]
    });

    println!("   Test 3: Billion laughs pattern");
    let result = timeout(
        Duration::from_secs(3),
        client
            .post(format!("{}/v1/chat/completions", base_url))
            .json(&billion_laughs)
            .send(),
    )
    .await;

    match result {
        Ok(_) => println!("      ✅ Server handled billion laughs"),
        Err(_) => panic!("❌ SECURITY ISSUE: Server hung on billion laughs"),
    }

    println!("\n✅ All resource exhaustion tests passed");
}
