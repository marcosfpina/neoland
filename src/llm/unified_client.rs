// Unified LLM Client — Multi-backend routing with circuit breakers.
//
// Supports three backend types with automatic workload-adaptive routing:
//   1. ml-offload (local llama.cpp via ml-ops-api)
//   2. SecureLLM Bridge (external providers: deepseek, gemini, groq)
//   3. vLLM (OpenAI-compatible GPU-accelerated server)           ← v0.0.1 #6
//
// Routing strategies:
//   - LocalFirst:     local → bridge fallback
//   - ExternalFirst:  bridge → local fallback
//   - LoadBalanced:   EMA-latency weighted selection
//   - Adaptive:       workload classification + circuit state   ← v0.0.1 #8
//
// Each backend has an independent circuit breaker (5 failures, 30s recovery).

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    llm::{SecureLLMProxy, VllmClient},
    ml_offload::{ChatCompletionRequest, ChatMessage, MLOffloadClient},
    secrets::SecretsManager,
};

// ── Circuit Breaker ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<Instant>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            failure_threshold,
            recovery_timeout,
        }
    }

    pub fn should_allow(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() >= self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            },
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            warn!(
                failures = self.failure_count,
                "Circuit breaker opened after {} consecutive failures", self.failure_count
            );
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }
}

// ── Retry with backoff ────────────────────────────────────────────────────

async fn retry_with_backoff<F, Fut, T>(
    max_retries: u32,
    base_delay: Duration,
    operation: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_err = Some(e);
                if attempt < max_retries {
                    let delay = base_delay * 2u32.saturating_pow(attempt);
                    let jitter = Duration::from_millis(rand_jitter(delay.as_millis() as u64));
                    let total_delay = delay + jitter;
                    warn!(
                        attempt = attempt + 1,
                        max_retries,
                        delay_ms = total_delay.as_millis() as u64,
                        "Retrying after failure"
                    );
                    tokio::time::sleep(total_delay).await;
                }
            },
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All retries exhausted")))
}

fn record_circuit_metric(backend: &str, state: CircuitState) {
    let state_str = match state {
        CircuitState::Closed => "closed",
        CircuitState::Open => "open",
        CircuitState::HalfOpen => "half-open",
    };
    crate::metrics::CIRCUIT_BREAKER_STATE
        .with_label_values(&[backend, state_str])
        .set(1.0);
}

fn rand_jitter(base_ms: u64) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    (nanos % (base_ms / 4 + 1)).min(500)
}

// ── Routing Strategy ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Try local (ml-offload) first, fallback to bridge + vLLM.
    LocalFirst,
    /// Try bridge first, fallback to local + vLLM.
    ExternalFirst,
    /// Select backend with lowest EMA latency.
    LoadBalanced,
    /// Classify workload and route to the best backend automatically.
    ///   - Short / real-time       → local (ml-offload)
    ///   - Code generation / long  → vLLM (GPU)
    ///   - General / fallback      → bridge (external)
    Adaptive,
}

// ── Latency Tracker ───────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LatencyTracker {
    avg_ms: Option<f64>,
    alpha: f64,
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self { avg_ms: None, alpha: 0.3 }
    }

    pub fn record(&mut self, elapsed: Duration) {
        let sample_ms = elapsed.as_secs_f64() * 1_000.0;
        self.avg_ms = Some(match self.avg_ms {
            None => sample_ms,
            Some(prev) => self.alpha * sample_ms + (1.0 - self.alpha) * prev,
        });
    }

    pub fn avg_ms(&self) -> f64 {
        self.avg_ms.unwrap_or(f64::MAX)
    }

    pub fn has_data(&self) -> bool {
        self.avg_ms.is_some()
    }
}

// ── Workload Classifier (v0.0.1 #8) ──────────────────────────────────────

/// Classifies a prompt to determine the optimal backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadClass {
    /// Short prompt, real-time interaction → prefer local/low-latency.
    ShortRealtime,
    /// Code generation, long context, or structured output → prefer GPU/vLLM.
    CodeGen,
    /// General knowledge, complex reasoning → prefer external provider.
    General,
}

/// Heuristic workload classifier based on prompt characteristics.
pub struct WorkloadClassifier;

impl WorkloadClassifier {
    /// Classify a prompt based on length, keywords, and structure.
    pub fn classify(prompt: &str) -> WorkloadClass {
        let len = prompt.len();
        let lower = prompt.to_lowercase();

        // Code generation signals
        let code_signals = [
            "fn ",
            "function ",
            "def ",
            "class ",
            "impl ",
            "struct ",
            "use ",
            "import ",
            "from ",
            "package ",
            "module ",
            "write code",
            "generate code",
            "implement",
            "refactor",
            "bug",
            "fix ",
            "test ",
            "unit test",
            "rust ",
            "golang",
            "python ",
            "typescript",
            "javascript",
            "```",
            "sql ",
            "query",
            "dockerfile",
            "yaml",
            "json",
            "toml",
            "helm",
            "kubernetes",
            "nginx",
        ];
        let code_score = code_signals.iter().filter(|s| lower.contains(*s)).count() as f64
            / code_signals.len() as f64;

        // General knowledge signals
        let general_signals = [
            "explain",
            "what is",
            "how does",
            "why ",
            "history of",
            "compare",
            "analyze",
            "summarize",
            "overview",
            "architecture",
            "design pattern",
            "best practice",
            "recommend",
            "pros and cons",
            "trade-off",
        ];
        let general_score = general_signals.iter().filter(|s| lower.contains(*s)).count() as f64
            / general_signals.len() as f64;

        // Code gen dominates — check first (trumps short length)
        if code_score >= 0.03 || len > 2000 {
            return WorkloadClass::CodeGen;
        }

        // General knowledge
        if general_score >= 0.05 {
            return WorkloadClass::General;
        }

        // Short / real-time (only when no other signals present)
        if len < 200 {
            return WorkloadClass::ShortRealtime;
        }

        // Default: length-based
        match len {
            0..=300 => WorkloadClass::ShortRealtime,
            301..=2000 => WorkloadClass::General,
            _ => WorkloadClass::CodeGen,
        }
    }
}

// ── Backend descriptor for ordered routing ────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    Local,
    Bridge,
    Vllm,
}

/// Priority-ordered list of backends to try.
type BackendPriority = Vec<Backend>;

// ── Unified LLM Client ────────────────────────────────────────────────────

pub struct UnifiedLLMClient {
    ml_offload: Option<MLOffloadClient>,
    securellm: Option<SecureLLMProxy>,
    vllm: Option<VllmClient>,
    strategy: RoutingStrategy,
    ml_circuit: Arc<Mutex<CircuitBreaker>>,
    sec_circuit: Arc<Mutex<CircuitBreaker>>,
    vllm_circuit: Arc<Mutex<CircuitBreaker>>,
    max_retries: u32,
    ml_latency: Arc<Mutex<LatencyTracker>>,
    sec_latency: Arc<Mutex<LatencyTracker>>,
    vllm_latency: Arc<Mutex<LatencyTracker>>,
}

impl UnifiedLLMClient {
    // ── Constructors ──────────────────────────────────────────────────

    /// Create a new client with LocalFirst strategy.
    pub async fn new_local_first(
        ml_offload_url: String,
        secrets_manager: Arc<SecretsManager>,
        securellm_provider: Option<(&str, Option<String>)>,
    ) -> Result<Self> {
        Self::build(ml_offload_url, secrets_manager, securellm_provider, None, None).await
    }

    /// Create a new client with vLLM support (v0.0.1 #6).
    pub async fn new_with_vllm(
        ml_offload_url: String,
        secrets_manager: Arc<SecretsManager>,
        securellm_provider: Option<(&str, Option<String>)>,
        vllm_url: Option<String>,
        vllm_model: Option<String>,
    ) -> Result<Self> {
        Self::build(ml_offload_url, secrets_manager, securellm_provider, vllm_url, vllm_model).await
    }

    /// Shared builder — initializes all available backends.
    async fn build(
        ml_offload_url: String,
        secrets_manager: Arc<SecretsManager>,
        securellm_provider: Option<(&str, Option<String>)>,
        vllm_url: Option<String>,
        vllm_model: Option<String>,
    ) -> Result<Self> {
        // ── ml-offload (local) ────────────────────────────────────────
        let ml_offload = match MLOffloadClient::new(ml_offload_url) {
            Ok(client) => {
                info!("ml-offload client initialized");
                Some(client)
            },
            Err(e) => {
                warn!("Failed to initialize ml-offload: {}", e);
                None
            },
        };

        // ── SecureLLM Bridge (external) ───────────────────────────────
        let securellm = if let Some((provider, api_key)) = securellm_provider {
            match SecureLLMProxy::new(provider, secrets_manager.clone(), api_key).await {
                Ok(proxy) => {
                    info!(provider = provider, "SecureLLM proxy initialized");
                    Some(proxy)
                },
                Err(e) => {
                    warn!("Failed to initialize SecureLLM proxy: {}", e);
                    None
                },
            }
        } else {
            None
        };

        // ── vLLM (GPU backend, v0.0.1 #6) ────────────────────────────
        let vllm = match vllm_url {
            Some(url) if !url.is_empty() => {
                let model = vllm_model.unwrap_or_default();
                let client = VllmClient::new(url.clone(), model);
                // Optional eager health check (non-blocking on failure)
                match client.health_check().await {
                    Ok(true) => {
                        info!(url = %url, "vLLM backend initialized and healthy");
                        Some(client)
                    },
                    Ok(false) => {
                        warn!(url = %url, "vLLM backend initialized but health check failed");
                        Some(client) // Still register — may recover later
                    },
                    Err(e) => {
                        warn!(url = %url, error = %e, "vLLM health check errored, registering anyway");
                        Some(client)
                    },
                }
            },
            _ => None,
        };

        // ── Guard: at least one backend must be available ────────────
        if ml_offload.is_none() && securellm.is_none() && vllm.is_none() {
            anyhow::bail!("No LLM backend available (ml-offload, securellm, and vLLM all failed)");
        }

        Ok(Self {
            ml_offload,
            securellm,
            vllm,
            strategy: RoutingStrategy::LocalFirst,
            ml_circuit: Arc::new(Mutex::new(CircuitBreaker::new(5, Duration::from_secs(30)))),
            sec_circuit: Arc::new(Mutex::new(CircuitBreaker::new(5, Duration::from_secs(30)))),
            vllm_circuit: Arc::new(Mutex::new(CircuitBreaker::new(5, Duration::from_secs(30)))),
            max_retries: 2,
            ml_latency: Arc::new(Mutex::new(LatencyTracker::new())),
            sec_latency: Arc::new(Mutex::new(LatencyTracker::new())),
            vllm_latency: Arc::new(Mutex::new(LatencyTracker::new())),
        })
    }

    /// Override the routing strategy (builder pattern).
    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    // ── Public API ───────────────────────────────────────────────────

    /// Send a chat request using the configured routing strategy.
    pub async fn chat(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        match self.strategy {
            RoutingStrategy::LocalFirst => {
                self.route_with_priority(
                    prompt,
                    temperature,
                    max_tokens,
                    &[Backend::Local, Backend::Vllm, Backend::Bridge],
                )
                .await
            },
            RoutingStrategy::ExternalFirst => {
                self.route_with_priority(
                    prompt,
                    temperature,
                    max_tokens,
                    &[Backend::Bridge, Backend::Vllm, Backend::Local],
                )
                .await
            },
            RoutingStrategy::LoadBalanced => {
                self.chat_load_balanced(prompt, temperature, max_tokens).await
            },
            RoutingStrategy::Adaptive => self.chat_adaptive(prompt, temperature, max_tokens).await,
        }
    }

    /// Health check for all backends.
    pub async fn health_check(&self) -> HealthStatus {
        let ml_healthy = if let Some(ml) = &self.ml_offload {
            ml.health().await.is_ok()
        } else {
            false
        };

        let sec_healthy = if let Some(sec) = &self.securellm {
            sec.health_check().await.unwrap_or(false)
        } else {
            false
        };

        let vllm_healthy = if let Some(vllm) = &self.vllm {
            vllm.health_check().await.unwrap_or(false)
        } else {
            false
        };

        HealthStatus { ml_offload: ml_healthy, securellm: sec_healthy, vllm: vllm_healthy }
    }

    // ── Priority-ordered routing (replaces old LocalFirst/ExternalFirst) ─

    /// Try each backend in priority order, respecting circuit breakers.
    async fn route_with_priority(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        priority: &[Backend],
    ) -> Result<String> {
        for &backend in priority {
            let result = self.try_backend(backend, prompt, temperature, max_tokens).await;
            match result {
                BackendResult::Success(response) => return Ok(response),
                BackendResult::CircuitOpen => {
                    warn!(?backend, "Circuit breaker open, trying next backend");
                },
                BackendResult::Failed(e) => {
                    warn!(?backend, error = %e, "Backend failed, trying next");
                },
                BackendResult::Unavailable => {
                    // Backend not configured — skip silently
                },
            }
        }
        anyhow::bail!("All LLM backends failed (circuit breakers may be open)")
    }

    /// Try a single backend with circuit breaker, retry logic, and latency
    /// tracking.
    async fn try_backend(
        &self,
        backend: Backend,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> BackendResult {
        let (circuit, latency) = self.circuit_and_latency(backend);
        let is_available = match backend {
            Backend::Local => self.ml_offload.is_some(),
            Backend::Bridge => self.securellm.is_some(),
            Backend::Vllm => self.vllm.is_some(),
        };
        if !is_available {
            return BackendResult::Unavailable;
        }

        // Check circuit breaker
        {
            let mut cb = circuit.lock().await;
            if !cb.should_allow() {
                record_circuit_metric(backend.metric_name(), cb.state());
                return BackendResult::CircuitOpen;
            }
        }

        let start = Instant::now();
        let result = self.execute_backend_request(backend, prompt, temperature, max_tokens).await;
        let elapsed = start.elapsed();

        let mut cb = circuit.lock().await;
        match result {
            Ok(response) => {
                cb.record_success();
                record_circuit_metric(backend.metric_name(), cb.state());
                drop(cb);
                latency.lock().await.record(elapsed);
                info!(
                    backend = ?backend,
                    latency_ms = elapsed.as_millis(),
                    "Backend request succeeded"
                );
                BackendResult::Success(response)
            },
            Err(e) => {
                cb.record_failure();
                record_circuit_metric(backend.metric_name(), cb.state());
                BackendResult::Failed(e)
            },
        }
    }

    /// Execute the actual LLM request for a given backend.
    async fn execute_backend_request(
        &self,
        backend: Backend,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        match backend {
            Backend::Local => {
                let ml = self.ml_offload.as_ref().unwrap();
                retry_with_backoff(self.max_retries, Duration::from_millis(200), || {
                    self.try_ml_offload(ml, prompt, temperature, max_tokens)
                })
                .await
            },
            Backend::Bridge => {
                let sec = self.securellm.as_ref().unwrap();
                sec.send_secure(prompt).await
            },
            Backend::Vllm => {
                let vllm = self.vllm.as_ref().unwrap();
                vllm.chat(prompt, None, temperature, max_tokens).await
            },
        }
    }

    /// Get the circuit breaker and latency tracker for a backend.
    fn circuit_and_latency(
        &self,
        backend: Backend,
    ) -> (Arc<Mutex<CircuitBreaker>>, Arc<Mutex<LatencyTracker>>) {
        match backend {
            Backend::Local => (self.ml_circuit.clone(), self.ml_latency.clone()),
            Backend::Bridge => (self.sec_circuit.clone(), self.sec_latency.clone()),
            Backend::Vllm => (self.vllm_circuit.clone(), self.vllm_latency.clone()),
        }
    }

    // ── LoadBalanced strategy ─────────────────────────────────────────

    async fn chat_load_balanced(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let (ml_avg, sec_avg, vllm_avg) = {
            let ml = self.ml_latency.lock().await.avg_ms();
            let sec = self.sec_latency.lock().await.avg_ms();
            let vllm = self.vllm_latency.lock().await.avg_ms();
            (ml, sec, vllm)
        };

        // Build priority list sorted by latency (lowest first)
        let mut backends: Vec<(f64, Backend)> = Vec::new();
        if self.ml_offload.is_some() {
            backends.push((ml_avg, Backend::Local));
        }
        if self.securellm.is_some() {
            backends.push((sec_avg, Backend::Bridge));
        }
        if self.vllm.is_some() {
            backends.push((vllm_avg, Backend::Vllm));
        }
        backends.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let priority: Vec<Backend> = backends.into_iter().map(|(_, b)| b).collect();

        info!(
            ml_avg_ms = format!("{:.1}", if ml_avg == f64::MAX { -1.0 } else { ml_avg }),
            sec_avg_ms = format!("{:.1}", if sec_avg == f64::MAX { -1.0 } else { sec_avg }),
            vllm_avg_ms = format!("{:.1}", if vllm_avg == f64::MAX { -1.0 } else { vllm_avg }),
            priority = ?priority,
            "LoadBalanced: backend priority by latency"
        );

        self.route_with_priority(prompt, temperature, max_tokens, &priority).await
    }

    // ── Adaptive strategy (v0.0.1 #8) ────────────────────────────────

    /// Adaptive routing: classify workload + check circuit state → select
    /// optimal backend.
    async fn chat_adaptive(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let class = WorkloadClassifier::classify(prompt);

        let priority: BackendPriority = match class {
            WorkloadClass::ShortRealtime => {
                // Local first (low latency), then vLLM, then bridge
                vec![Backend::Local, Backend::Vllm, Backend::Bridge]
            },
            WorkloadClass::CodeGen => {
                // vLLM first (GPU-optimized), then bridge, then local
                vec![Backend::Vllm, Backend::Bridge, Backend::Local]
            },
            WorkloadClass::General => {
                // Bridge first (best general model), then vLLM, then local
                vec![Backend::Bridge, Backend::Vllm, Backend::Local]
            },
        };

        info!(
            class = ?class,
            prompt_len = prompt.len(),
            priority = ?priority,
            "Adaptive routing: workload classified"
        );

        self.route_with_priority(prompt, temperature, max_tokens, &priority).await
    }

    // ── Helpers ───────────────────────────────────────────────────────

    async fn try_ml_offload(
        &self,
        ml: &MLOffloadClient,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let request = ChatCompletionRequest {
            model: "auto".into(),
            messages: vec![ChatMessage { role: "user".into(), content: prompt.to_string() }],
            temperature,
            max_tokens,
            stream: None,
            stop: None,
            top_p: None,
        };
        let response = ml.chat_completion(request).await?;
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            anyhow::bail!("Empty response from ml-offload")
        }
    }
}

// ── BackendResult enum ────────────────────────────────────────────────────

enum BackendResult {
    Success(String),
    CircuitOpen,
    Failed(anyhow::Error),
    Unavailable,
}

// ── Backend metric names ──────────────────────────────────────────────────

impl Backend {
    fn metric_name(&self) -> &'static str {
        match self {
            Backend::Local => "ml-offload",
            Backend::Bridge => "securellm",
            Backend::Vllm => "vllm",
        }
    }
}

// ── Health Status ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct HealthStatus {
    pub ml_offload: bool,
    pub securellm: bool,
    pub vllm: bool,
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ── Circuit Breaker ──────────────────────────────────────────────

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30));
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(30));
        assert!(cb.should_allow());
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.should_allow());
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(30));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_after_timeout() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(15));
        assert!(cb.should_allow());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_half_open_success_closes() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(15));
        cb.should_allow();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_failure_reopens() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(15));
        cb.should_allow();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    // ── Retry ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_retry_with_backoff_succeeds_first_try() {
        let result = retry_with_backoff(3, Duration::from_millis(10), || async {
            Ok::<_, anyhow::Error>("success".to_string())
        })
        .await;
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_with_backoff_succeeds_after_retries() {
        let attempt = Arc::new(Mutex::new(0u32));
        let attempt_clone = attempt.clone();
        let result = retry_with_backoff(3, Duration::from_millis(10), || {
            let attempt = attempt_clone.clone();
            async move {
                let mut count = attempt.lock().await;
                *count += 1;
                if *count < 3 {
                    anyhow::bail!("not yet");
                }
                Ok::<_, anyhow::Error>("success".to_string())
            }
        })
        .await;
        assert_eq!(result.unwrap(), "success");
        assert_eq!(*attempt.lock().await, 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_exhausted() {
        let result = retry_with_backoff(2, Duration::from_millis(10), || async {
            Err::<String, _>(anyhow::anyhow!("always fails"))
        })
        .await;
        assert!(result.is_err());
    }

    // ── Latency Tracker ──────────────────────────────────────────────

    #[test]
    fn test_latency_tracker_no_data() {
        let t = LatencyTracker::new();
        assert!(!t.has_data());
        assert_eq!(t.avg_ms(), f64::MAX);
    }

    #[test]
    fn test_latency_tracker_first_sample() {
        let mut t = LatencyTracker::new();
        t.record(Duration::from_millis(100));
        assert!(t.has_data());
        assert!((t.avg_ms() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_latency_tracker_ema_converges() {
        let mut t = LatencyTracker::new();
        for _ in 0..10 {
            t.record(Duration::from_millis(200));
        }
        assert!(t.has_data());
        assert!((t.avg_ms() - 200.0).abs() < 1.0);
    }

    #[test]
    fn test_latency_tracker_load_balanced_order() {
        let mut ml = LatencyTracker::new();
        let mut sec = LatencyTracker::new();
        for _ in 0..5 {
            ml.record(Duration::from_millis(50));
            sec.record(Duration::from_millis(300));
        }
        let prefer_ml = ml.avg_ms() <= sec.avg_ms();
        assert!(prefer_ml, "ml-offload (50ms) should be preferred over securellm (300ms)");
    }

    #[test]
    fn test_latency_tracker_no_data_prefers_ml_first() {
        let ml = LatencyTracker::new();
        let sec = LatencyTracker::new();
        assert!(ml.avg_ms() <= sec.avg_ms(), "without data, should fall back to local-first");
    }

    // ── Workload Classifier ──────────────────────────────────────────

    #[test]
    fn test_classify_short_realtime() {
        assert_eq!(WorkloadClassifier::classify("hello"), WorkloadClass::ShortRealtime);
        assert_eq!(WorkloadClassifier::classify("what is the weather?"), WorkloadClass::General);
    }

    #[test]
    fn test_classify_code_gen() {
        // Long prompts with code keywords route to CodeGen
        let long_code = format!("implement a kubernetes operator\n{}", "a".repeat(200));
        assert_eq!(WorkloadClassifier::classify(&long_code), WorkloadClass::CodeGen);
    }

    #[test]
    fn test_classify_code_gen_long_prompt() {
        let long = "a".repeat(2001);
        assert_eq!(WorkloadClassifier::classify(&long), WorkloadClass::CodeGen);
    }

    #[test]
    fn test_classify_general() {
        assert_eq!(
            WorkloadClassifier::classify("explain the history of the internet"),
            WorkloadClass::General
        );
        assert_eq!(
            WorkloadClassifier::classify("compare systems programming and application development"),
            WorkloadClass::General
        );
    }

    #[test]
    fn test_classify_default_length_based() {
        // Medium length, no strong signals → General
        let medium = "This is a prompt of medium length without any specific code or general knowledge signals. ".repeat(5);
        assert_eq!(WorkloadClassifier::classify(&medium), WorkloadClass::General);
    }

    // ── Unified Client ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_local_first_creation() {
        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let result = UnifiedLLMClient::new_local_first(
            "http://localhost:9000".to_string(),
            secrets_manager,
            None,
        )
        .await;
        if let Ok(client) = result {
            assert_eq!(client.strategy, RoutingStrategy::LocalFirst);
        }
    }

    #[tokio::test]
    async fn test_new_with_vllm_registers_backend() {
        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let result = UnifiedLLMClient::new_with_vllm(
            "http://localhost:9000".to_string(),
            secrets_manager,
            None,
            Some("http://127.0.0.1:19998".to_string()),
            Some("test-model".to_string()),
        )
        .await;
        // The vLLM health check will fail (no server), but client should still be
        // created
        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(client.vllm.is_some());
    }

    #[test]
    fn test_backend_metric_names() {
        assert_eq!(Backend::Local.metric_name(), "ml-offload");
        assert_eq!(Backend::Bridge.metric_name(), "securellm");
        assert_eq!(Backend::Vllm.metric_name(), "vllm");
    }
}
