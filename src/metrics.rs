// Phase 4.1: Prometheus Metrics Integration
// Provides comprehensive observability for production monitoring

use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge, register_gauge_vec, register_histogram_vec, CounterVec,
    Encoder, Gauge, GaugeVec, HistogramVec, TextEncoder,
};
use std::time::Instant;

lazy_static! {
    // HTTP Request Metrics
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "neoland_http_requests_total",
        "Total number of HTTP requests",
        &["method", "endpoint", "status"]
    )
    .unwrap();

    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "neoland_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap();

    // gRPC Request Metrics
    pub static ref GRPC_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "neoland_grpc_requests_total",
        "Total number of gRPC requests",
        &["method", "status"]
    )
    .unwrap();

    pub static ref GRPC_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "neoland_grpc_request_duration_seconds",
        "gRPC request duration in seconds",
        &["method"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap();

    // LLM Metrics
    pub static ref LLM_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "neoland_llm_requests_total",
        "Total number of LLM requests",
        &["provider", "model", "status"]
    )
    .unwrap();

    pub static ref LLM_TOKENS_TOTAL: CounterVec = register_counter_vec!(
        "neoland_llm_tokens_total",
        "Total number of tokens processed",
        &["provider", "model", "type"] // type: prompt, completion
    )
    .unwrap();

    pub static ref LLM_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "neoland_llm_request_duration_seconds",
        "LLM request duration in seconds",
        &["provider", "model"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]
    )
    .unwrap();

    pub static ref LLM_ESTIMATED_COST_USD: CounterVec = register_counter_vec!(
        "neoland_llm_estimated_cost_usd",
        "Estimated cost in USD for LLM requests",
        &["provider", "model"]
    )
    .unwrap();

    // Authentication Metrics
    pub static ref AUTH_ATTEMPTS_TOTAL: CounterVec = register_counter_vec!(
        "neoland_auth_attempts_total",
        "Total number of authentication attempts",
        &["method", "status"] // method: api_key, jwt; status: success, failure
    )
    .unwrap();

    pub static ref AUTH_FAILED_ATTEMPTS: CounterVec = register_counter_vec!(
        "neoland_auth_failed_attempts",
        "Number of failed authentication attempts",
        &["user_id", "ip_address"]
    )
    .unwrap();

    // Rate Limiting Metrics
    pub static ref RATE_LIMIT_EXCEEDED_TOTAL: CounterVec = register_counter_vec!(
        "neoland_rate_limit_exceeded_total",
        "Total number of rate limit violations",
        &["identifier"] // user_id or IP
    )
    .unwrap();

    // Vector Store Metrics
    pub static ref VECTOR_STORE_DOCUMENTS_TOTAL: Gauge = register_gauge!(
        "neoland_vector_store_documents_total",
        "Total number of documents in vector store"
    )
    .unwrap();

    pub static ref VECTOR_STORE_SEARCH_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "neoland_vector_store_search_duration_seconds",
        "Vector store search duration in seconds",
        &["operation"], // operation: add, search, delete
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]
    )
    .unwrap();

    // Secrets Management Metrics
    pub static ref SECRETS_ACCESS_TOTAL: CounterVec = register_counter_vec!(
        "neoland_secrets_access_total",
        "Total number of secrets accesses",
        &["secret_type", "source"] // source: vault, env, cache
    )
    .unwrap();

    pub static ref SECRETS_CACHE_HIT_RATE: GaugeVec = register_gauge_vec!(
        "neoland_secrets_cache_hit_rate",
        "Secrets cache hit rate (0.0 to 1.0)",
        &["secret_type"]
    )
    .unwrap();

    // System Metrics
    pub static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "neoland_active_connections",
        "Number of active connections"
    )
    .unwrap();

    pub static ref MEMORY_USAGE_BYTES: Gauge = register_gauge!(
        "neoland_memory_usage_bytes",
        "Memory usage in bytes"
    )
    .unwrap();

    // Audit Log Metrics
    pub static ref AUDIT_EVENTS_TOTAL: CounterVec = register_counter_vec!(
        "neoland_audit_events_total",
        "Total number of audit events",
        &["action", "severity"]
    )
    .unwrap();

    // Circuit Breaker Metrics
    pub static ref CIRCUIT_BREAKER_STATE: GaugeVec = register_gauge_vec!(
        "neoland_circuit_breaker_state",
        "Circuit breaker state (1.0 = active state for this backend+state combo)",
        &["backend", "state"] // backend: ml-offload, securellm; state: closed, open, half-open
    )
    .unwrap();
}

/// Timer utility for measuring request duration
pub struct Timer {
    start: Instant,
    labels: Vec<String>,
    histogram: &'static HistogramVec,
}

impl Timer {
    pub fn new(histogram: &'static HistogramVec, labels: Vec<String>) -> Self {
        Self { start: Instant::now(), labels, histogram }
    }

    pub fn observe_duration(self) {
        let duration = self.start.elapsed().as_secs_f64();
        let label_refs: Vec<&str> = self.labels.iter().map(|s| s.as_str()).collect();
        self.histogram.with_label_values(&label_refs).observe(duration);
    }
}

/// Helper function to render metrics in Prometheus format
pub fn render_metrics() -> Result<String, Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

/// Utility functions for common metric operations
pub mod utils {
    use super::*;

    /// Record HTTP request
    pub fn record_http_request(method: &str, endpoint: &str, status: u16, duration: f64) {
        HTTP_REQUESTS_TOTAL
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();

        HTTP_REQUEST_DURATION_SECONDS
            .with_label_values(&[method, endpoint])
            .observe(duration);
    }

    /// Record gRPC request
    pub fn record_grpc_request(method: &str, status: &str, duration: f64) {
        GRPC_REQUESTS_TOTAL.with_label_values(&[method, status]).inc();

        GRPC_REQUEST_DURATION_SECONDS.with_label_values(&[method]).observe(duration);
    }

    /// Record LLM request
    pub fn record_llm_request(
        provider: &str,
        model: &str,
        status: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        duration: f64,
    ) {
        LLM_REQUESTS_TOTAL.with_label_values(&[provider, model, status]).inc();

        LLM_TOKENS_TOTAL
            .with_label_values(&[provider, model, "prompt"])
            .inc_by(prompt_tokens as f64);

        LLM_TOKENS_TOTAL
            .with_label_values(&[provider, model, "completion"])
            .inc_by(completion_tokens as f64);

        LLM_REQUEST_DURATION_SECONDS
            .with_label_values(&[provider, model])
            .observe(duration);

        // Estimate cost (rough approximation - adjust per provider)
        let estimated_cost = estimate_llm_cost(provider, model, prompt_tokens, completion_tokens);
        LLM_ESTIMATED_COST_USD
            .with_label_values(&[provider, model])
            .inc_by(estimated_cost);
    }

    /// Estimate LLM cost in USD (rough approximation)
    pub fn estimate_llm_cost(
        provider: &str,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> f64 {
        // Prices per 1M tokens (approximate as of 2026-01)
        let (prompt_price, completion_price) = match (provider, model) {
            // Local providers (FREE)
            ("llamacpp", _) => (0.0, 0.0), // Local model, no cost
            ("ollama", _) => (0.0, 0.0),   // Local model, no cost

            // DeepSeek (very cheap)
            ("deepseek", _) => (0.14, 0.28),

            // OpenAI
            ("openai", m) if m.contains("gpt-4") => (30.0, 60.0),
            ("openai", m) if m.contains("gpt-3.5") => (0.50, 1.50),

            // Anthropic Claude
            ("anthropic", m) if m.contains("claude-3-opus") => (15.0, 75.0),
            ("anthropic", m) if m.contains("claude-3-sonnet") => (3.0, 15.0),
            ("anthropic", m) if m.contains("claude-3-haiku") => (0.25, 1.25),

            // Google Gemini
            ("gemini", m) if m.contains("pro") => (0.50, 1.50),
            ("gemini", m) if m.contains("flash") => (0.075, 0.30),

            // Groq (very fast, cheap)
            ("groq", _) => (0.05, 0.10),

            // Default fallback
            _ => (1.0, 2.0),
        };

        let prompt_cost = (prompt_tokens as f64 / 1_000_000.0) * prompt_price;
        let completion_cost = (completion_tokens as f64 / 1_000_000.0) * completion_price;

        prompt_cost + completion_cost
    }

    /// Record authentication attempt
    pub fn record_auth_attempt(
        method: &str,
        success: bool,
        user_id: Option<&str>,
        ip: Option<&str>,
    ) {
        let status = if success { "success" } else { "failure" };
        AUTH_ATTEMPTS_TOTAL.with_label_values(&[method, status]).inc();

        if !success {
            if let (Some(user), Some(ip_addr)) = (user_id, ip) {
                AUTH_FAILED_ATTEMPTS.with_label_values(&[user, ip_addr]).inc();
            }
        }
    }

    /// Record rate limit violation
    pub fn record_rate_limit_exceeded(identifier: &str) {
        RATE_LIMIT_EXCEEDED_TOTAL.with_label_values(&[identifier]).inc();
    }

    /// Record audit event
    pub fn record_audit_event(action: &str, severity: &str) {
        AUDIT_EVENTS_TOTAL.with_label_values(&[action, severity]).inc();
    }

    /// Update vector store document count
    pub fn update_vector_store_documents(count: usize) {
        VECTOR_STORE_DOCUMENTS_TOTAL.set(count as f64);
    }

    /// Record secrets access
    pub fn record_secrets_access(secret_type: &str, source: &str) {
        SECRETS_ACCESS_TOTAL.with_label_values(&[secret_type, source]).inc();
    }

    /// Update active connections
    pub fn update_active_connections(count: usize) {
        ACTIVE_CONNECTIONS.set(count as f64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registration() {
        // Verify metrics are registered (just accessing them is enough)
        let _ = &*HTTP_REQUESTS_TOTAL;
        let _ = &*LLM_REQUESTS_TOTAL;
        // Metrics are lazily initialized via lazy_static
    }

    #[test]
    fn test_http_metrics() {
        utils::record_http_request("GET", "/health", 200, 0.001);
        // Metrics should be incremented (can't assert exact value in parallel tests)
    }

    #[test]
    fn test_llm_cost_estimation() {
        // Test local providers (FREE)
        let cost = utils::estimate_llm_cost("llamacpp", "local", 1_000_000, 1_000_000);
        assert_eq!(cost, 0.0);

        // Test DeepSeek pricing (very cheap)
        let cost = utils::estimate_llm_cost("deepseek", "chat", 1_000_000, 1_000_000);
        assert!((cost - 0.42).abs() < 0.01); // 0.14 + 0.28 = 0.42

        // Test GPT-4 pricing (expensive)
        let cost = utils::estimate_llm_cost("openai", "gpt-4", 1_000_000, 1_000_000);
        assert!((cost - 90.0).abs() < 0.01); // 30 + 60 = 90

        // Test Gemini Flash (cheap)
        let cost = utils::estimate_llm_cost("gemini", "flash", 1_000_000, 1_000_000);
        assert!((cost - 0.375).abs() < 0.01); // 0.075 + 0.30 = 0.375

        // Test Groq (very cheap and fast)
        let cost = utils::estimate_llm_cost("groq", "llama", 1_000_000, 1_000_000);
        assert!((cost - 0.15).abs() < 0.01); // 0.05 + 0.10 = 0.15
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new(
            &HTTP_REQUEST_DURATION_SECONDS,
            vec!["GET".to_string(), "/test".to_string()],
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
        timer.observe_duration();
    }

    #[test]
    fn test_render_metrics() {
        // Trigger metric registration by accessing them
        utils::record_http_request("GET", "/test", 200, 0.001);

        let result = render_metrics();
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.contains("neoland_http_requests_total"));
    }
}
