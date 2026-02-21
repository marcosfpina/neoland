// Unified LLM Client - Local First Strategy with Circuit Breaker
// Abstração unificada para ml-offload-api + securellm-bridge

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    llm::SecureLLMProxy,
    ml_offload::{ChatCompletionRequest, ChatMessage, MLOffloadClient},
    secrets::SecretsManager,
};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests flow through
    Closed,
    /// Too many failures - requests are rejected immediately
    Open,
    /// Testing if backend has recovered - allow one request through
    HalfOpen,
}

/// Circuit breaker for backend resilience
#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<Instant>,
    /// Number of consecutive failures before opening the circuit
    failure_threshold: u32,
    /// How long to wait before trying again (half-open)
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

    /// Check if a request should be allowed through
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

    /// Record a successful request
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    /// Record a failed request
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

/// Retry with exponential backoff
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

/// Update circuit breaker state metric for a backend
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

/// Simple deterministic jitter (0-25% of base delay)
fn rand_jitter(base_ms: u64) -> u64 {
    // Use current time nanos as cheap entropy source
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    (nanos % (base_ms / 4 + 1)).min(500)
}

/// Estratégia de roteamento para LLM requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Prioriza ml-offload local, fallback para SecureLLM externo
    LocalFirst,
    /// Prioriza SecureLLM externo, fallback para ml-offload local
    ExternalFirst,
    /// Selecciona backend com menor latência média (EMA)
    LoadBalanced,
}

/// Exponential moving average latency tracker per backend.
///
/// Records request durations and maintains a smoothed average using:
/// `new_avg = α * sample + (1 − α) * prev_avg`
///
/// Until at least one sample is recorded, `avg_ms()` returns `f64::MAX`
/// so that a backend with real measurements is always preferred.
#[derive(Debug)]
pub struct LatencyTracker {
    /// Current EMA in milliseconds; `None` until first sample.
    avg_ms: Option<f64>,
    /// Smoothing factor α ∈ (0, 1].  Higher = more weight on recent samples.
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

    /// Record a request duration and update the moving average.
    pub fn record(&mut self, elapsed: Duration) {
        let sample_ms = elapsed.as_secs_f64() * 1_000.0;
        self.avg_ms = Some(match self.avg_ms {
            None => sample_ms,
            Some(prev) => self.alpha * sample_ms + (1.0 - self.alpha) * prev,
        });
    }

    /// Current average latency in milliseconds.
    /// Returns `f64::MAX` when no samples have been recorded yet.
    pub fn avg_ms(&self) -> f64 {
        self.avg_ms.unwrap_or(f64::MAX)
    }

    /// `true` if at least one sample has been recorded.
    pub fn has_data(&self) -> bool {
        self.avg_ms.is_some()
    }
}

/// Cliente LLM unificado com roteamento inteligente e circuit breakers
pub struct UnifiedLLMClient {
    ml_offload: Option<MLOffloadClient>,
    securellm: Option<SecureLLMProxy>,
    strategy: RoutingStrategy,
    ml_circuit: Arc<Mutex<CircuitBreaker>>,
    sec_circuit: Arc<Mutex<CircuitBreaker>>,
    max_retries: u32,
    /// Per-backend latency EMA for LoadBalanced routing
    ml_latency: Arc<Mutex<LatencyTracker>>,
    sec_latency: Arc<Mutex<LatencyTracker>>,
}

impl UnifiedLLMClient {
    /// Cria novo cliente com estratégia LocalFirst (padrão)
    ///
    /// Phase 1.2: Now requires SecretsManager for secure API key loading
    pub async fn new_local_first(
        ml_offload_url: String,
        secrets_manager: Arc<SecretsManager>,
        securellm_provider: Option<(&str, Option<String>)>,
    ) -> Result<Self> {
        let ml_offload = match MLOffloadClient::new(ml_offload_url) {
            Ok(client) => {
                info!("ML-Offload client initialized");
                Some(client)
            },
            Err(e) => {
                warn!("Failed to initialize ML-Offload client: {}", e);
                None
            },
        };

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

        if ml_offload.is_none() && securellm.is_none() {
            anyhow::bail!("No LLM backend available (both ml-offload and securellm failed)");
        }

        // Circuit breakers: open after 5 failures, try again after 30s
        let ml_circuit = Arc::new(Mutex::new(CircuitBreaker::new(5, Duration::from_secs(30))));
        let sec_circuit = Arc::new(Mutex::new(CircuitBreaker::new(5, Duration::from_secs(30))));

        Ok(Self {
            ml_offload,
            securellm,
            strategy: RoutingStrategy::LocalFirst,
            ml_circuit,
            sec_circuit,
            max_retries: 2,
            ml_latency: Arc::new(Mutex::new(LatencyTracker::new())),
            sec_latency: Arc::new(Mutex::new(LatencyTracker::new())),
        })
    }

    /// Override the routing strategy (builder pattern).
    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Envia chat request com roteamento baseado na estratégia
    pub async fn chat(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        match self.strategy {
            RoutingStrategy::LocalFirst => {
                self.chat_local_first(prompt, temperature, max_tokens).await
            },
            RoutingStrategy::ExternalFirst => {
                self.chat_external_first(prompt, temperature, max_tokens).await
            },
            RoutingStrategy::LoadBalanced => {
                self.chat_load_balanced(prompt, temperature, max_tokens).await
            },
        }
    }

    /// LocalFirst: Tenta ml-offload (with circuit breaker) → fallback SecureLLM
    async fn chat_local_first(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        // Try ml-offload first (with circuit breaker + retry)
        if let Some(ml) = &self.ml_offload {
            let mut cb = self.ml_circuit.lock().await;
            if cb.should_allow() {
                drop(cb); // Release lock during request

                let ml_ref = ml;
                let max_retries = self.max_retries;
                let result = retry_with_backoff(max_retries, Duration::from_millis(200), || {
                    self.try_ml_offload(ml_ref, prompt, temperature, max_tokens)
                })
                .await;

                let mut cb = self.ml_circuit.lock().await;
                match result {
                    Ok(response) => {
                        cb.record_success();
                        record_circuit_metric("ml-offload", cb.state());
                        info!("Response from ml-offload (local)");
                        return Ok(response);
                    },
                    Err(e) => {
                        cb.record_failure();
                        record_circuit_metric("ml-offload", cb.state());
                        warn!(
                            circuit_state = ?cb.state(),
                            "ml-offload failed after retries: {}, trying SecureLLM fallback", e
                        );
                    },
                }
            } else {
                warn!("ml-offload circuit breaker is open, skipping to SecureLLM");
            }
        }

        // Fallback to SecureLLM (with circuit breaker)
        if let Some(sec) = &self.securellm {
            let mut cb = self.sec_circuit.lock().await;
            if cb.should_allow() {
                drop(cb);

                let result = sec.send_secure(prompt).await;

                let mut cb = self.sec_circuit.lock().await;
                match result {
                    Ok(response) => {
                        cb.record_success();
                        record_circuit_metric("securellm", cb.state());
                        info!(provider = sec.provider(), "Response from SecureLLM (external)");
                        return Ok(response);
                    },
                    Err(e) => {
                        cb.record_failure();
                        record_circuit_metric("securellm", cb.state());
                        warn!("SecureLLM fallback failed: {}", e);
                    },
                }
            } else {
                warn!("SecureLLM circuit breaker is open");
            }
        }

        anyhow::bail!("All LLM backends failed (circuit breakers may be open)")
    }

    /// ExternalFirst: Tenta SecureLLM (with circuit breaker) → fallback ml-offload
    async fn chat_external_first(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        // Try SecureLLM first (with circuit breaker)
        if let Some(sec) = &self.securellm {
            let mut cb = self.sec_circuit.lock().await;
            if cb.should_allow() {
                drop(cb);

                let result = sec.send_secure(prompt).await;

                let mut cb = self.sec_circuit.lock().await;
                match result {
                    Ok(response) => {
                        cb.record_success();
                        record_circuit_metric("securellm", cb.state());
                        info!(provider = sec.provider(), "Response from SecureLLM (external)");
                        return Ok(response);
                    },
                    Err(e) => {
                        cb.record_failure();
                        record_circuit_metric("securellm", cb.state());
                        warn!("SecureLLM failed: {}, trying ml-offload fallback", e);
                    },
                }
            } else {
                warn!("SecureLLM circuit breaker is open, skipping to ml-offload");
            }
        }

        // Fallback to ml-offload (with circuit breaker + retry)
        if let Some(ml) = &self.ml_offload {
            let mut cb = self.ml_circuit.lock().await;
            if cb.should_allow() {
                drop(cb);

                let ml_ref = ml;
                let max_retries = self.max_retries;
                let result = retry_with_backoff(max_retries, Duration::from_millis(200), || {
                    self.try_ml_offload(ml_ref, prompt, temperature, max_tokens)
                })
                .await;

                let mut cb = self.ml_circuit.lock().await;
                match result {
                    Ok(response) => {
                        cb.record_success();
                        record_circuit_metric("ml-offload", cb.state());
                        info!("Response from ml-offload (local fallback)");
                        return Ok(response);
                    },
                    Err(e) => {
                        cb.record_failure();
                        record_circuit_metric("ml-offload", cb.state());
                        warn!("ml-offload fallback failed: {}", e);
                    },
                }
            } else {
                warn!("ml-offload circuit breaker is open");
            }
        }

        anyhow::bail!("All LLM backends failed (circuit breakers may be open)")
    }

    /// LoadBalanced: picks the backend with the lower latency EMA.
    ///
    /// Both backends are checked against their circuit breakers.
    /// The backend with the lower average response time (or the only available one)
    /// is tried first; on failure the other backend is used as fallback.
    /// After each successful call the measured latency is folded into the EMA.
    async fn chat_load_balanced(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let ml_avg = self.ml_latency.lock().await.avg_ms();
        let sec_avg = self.sec_latency.lock().await.avg_ms();

        // prefer ml-offload when its EMA is ≤ securellm (or when no data yet — both MAX)
        let try_ml_first = ml_avg <= sec_avg;

        info!(
            ml_avg_ms = format!("{:.1}", if ml_avg == f64::MAX { -1.0 } else { ml_avg }),
            sec_avg_ms = format!("{:.1}", if sec_avg == f64::MAX { -1.0 } else { sec_avg }),
            try_ml_first,
            "LoadBalanced: selecting backend"
        );

        if try_ml_first {
            // ── Primary: ml-offload ────────────────────────────────────────
            if let Some(ml) = &self.ml_offload {
                let mut cb = self.ml_circuit.lock().await;
                if cb.should_allow() {
                    drop(cb);
                    let start = Instant::now();
                    let ml_ref = ml;
                    let max_retries = self.max_retries;
                    let result =
                        retry_with_backoff(max_retries, Duration::from_millis(200), || {
                            self.try_ml_offload(ml_ref, prompt, temperature, max_tokens)
                        })
                        .await;
                    let elapsed = start.elapsed();

                    let mut cb = self.ml_circuit.lock().await;
                    match result {
                        Ok(response) => {
                            cb.record_success();
                            record_circuit_metric("ml-offload", cb.state());
                            drop(cb);
                            self.ml_latency.lock().await.record(elapsed);
                            info!(latency_ms = elapsed.as_millis(), "LoadBalanced: ml-offload");
                            return Ok(response);
                        },
                        Err(e) => {
                            cb.record_failure();
                            record_circuit_metric("ml-offload", cb.state());
                            warn!("LoadBalanced: ml-offload failed ({e}), trying securellm");
                        },
                    }
                } else {
                    warn!("LoadBalanced: ml-offload circuit open, trying securellm");
                }
            }

            // ── Fallback: securellm ────────────────────────────────────────
            if let Some(sec) = &self.securellm {
                let mut cb = self.sec_circuit.lock().await;
                if cb.should_allow() {
                    drop(cb);
                    let start = Instant::now();
                    let result = sec.send_secure(prompt).await;
                    let elapsed = start.elapsed();

                    let mut cb = self.sec_circuit.lock().await;
                    match result {
                        Ok(response) => {
                            cb.record_success();
                            record_circuit_metric("securellm", cb.state());
                            drop(cb);
                            self.sec_latency.lock().await.record(elapsed);
                            info!(
                                latency_ms = elapsed.as_millis(),
                                "LoadBalanced: securellm fallback"
                            );
                            return Ok(response);
                        },
                        Err(e) => {
                            cb.record_failure();
                            record_circuit_metric("securellm", cb.state());
                            warn!("LoadBalanced: securellm fallback failed: {e}");
                        },
                    }
                } else {
                    warn!("LoadBalanced: securellm circuit open");
                }
            }
        } else {
            // ── Primary: securellm ─────────────────────────────────────────
            if let Some(sec) = &self.securellm {
                let mut cb = self.sec_circuit.lock().await;
                if cb.should_allow() {
                    drop(cb);
                    let start = Instant::now();
                    let result = sec.send_secure(prompt).await;
                    let elapsed = start.elapsed();

                    let mut cb = self.sec_circuit.lock().await;
                    match result {
                        Ok(response) => {
                            cb.record_success();
                            record_circuit_metric("securellm", cb.state());
                            drop(cb);
                            self.sec_latency.lock().await.record(elapsed);
                            info!(latency_ms = elapsed.as_millis(), "LoadBalanced: securellm");
                            return Ok(response);
                        },
                        Err(e) => {
                            cb.record_failure();
                            record_circuit_metric("securellm", cb.state());
                            warn!("LoadBalanced: securellm failed ({e}), trying ml-offload");
                        },
                    }
                } else {
                    warn!("LoadBalanced: securellm circuit open, trying ml-offload");
                }
            }

            // ── Fallback: ml-offload ────────────────────────────────────────
            if let Some(ml) = &self.ml_offload {
                let mut cb = self.ml_circuit.lock().await;
                if cb.should_allow() {
                    drop(cb);
                    let start = Instant::now();
                    let ml_ref = ml;
                    let max_retries = self.max_retries;
                    let result =
                        retry_with_backoff(max_retries, Duration::from_millis(200), || {
                            self.try_ml_offload(ml_ref, prompt, temperature, max_tokens)
                        })
                        .await;
                    let elapsed = start.elapsed();

                    let mut cb = self.ml_circuit.lock().await;
                    match result {
                        Ok(response) => {
                            cb.record_success();
                            record_circuit_metric("ml-offload", cb.state());
                            drop(cb);
                            self.ml_latency.lock().await.record(elapsed);
                            info!(
                                latency_ms = elapsed.as_millis(),
                                "LoadBalanced: ml-offload fallback"
                            );
                            return Ok(response);
                        },
                        Err(e) => {
                            cb.record_failure();
                            record_circuit_metric("ml-offload", cb.state());
                            warn!("LoadBalanced: ml-offload fallback failed: {e}");
                        },
                    }
                } else {
                    warn!("LoadBalanced: ml-offload circuit open");
                }
            }
        }

        anyhow::bail!("All LLM backends failed (LoadBalanced — circuit breakers may be open)")
    }

    /// Helper: Try ml-offload API
    async fn try_ml_offload(
        &self,
        ml: &MLOffloadClient,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let request = ChatCompletionRequest {
            model: "auto".into(), // ml-offload selects best backend
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

    /// Health check for both backends
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

        HealthStatus { ml_offload: ml_healthy, securellm: sec_healthy }
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub ml_offload: bool,
    pub securellm: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_first_creation() {
        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let result = UnifiedLLMClient::new_local_first(
            "http://localhost:9000".to_string(),
            secrets_manager,
            None, // No SecureLLM
        )
        .await;

        // Should succeed if ml-offload client can be created
        if let Ok(client) = result {
            assert_eq!(client.strategy, RoutingStrategy::LocalFirst);
        }
    }

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
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Should not allow requests when open
        assert!(!cb.should_allow());
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(30));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);

        // Should need 3 more failures to open
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_after_timeout() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(15));

        // Should transition to HalfOpen and allow request
        assert!(cb.should_allow());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_half_open_success_closes() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(15));
        cb.should_allow(); // Transitions to HalfOpen
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_failure_reopens() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(15));
        cb.should_allow(); // Transitions to HalfOpen
        cb.record_failure();
        // After 3 total failures (>= threshold 2), should be open
        assert_eq!(cb.state(), CircuitState::Open);
    }

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

    // ── LatencyTracker tests ──────────────────────────────────────────────────

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
        // First sample sets avg directly
        assert!((t.avg_ms() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_latency_tracker_ema_converges() {
        let mut t = LatencyTracker::new();
        // Feed 10 identical 200ms samples; EMA should settle close to 200
        for _ in 0..10 {
            t.record(Duration::from_millis(200));
        }
        assert!(t.has_data());
        assert!((t.avg_ms() - 200.0).abs() < 1.0);
    }

    #[test]
    fn test_latency_tracker_load_balanced_order() {
        // Simulate: ml-offload consistently faster → it should be preferred
        let mut ml = LatencyTracker::new();
        let mut sec = LatencyTracker::new();

        for _ in 0..5 {
            ml.record(Duration::from_millis(50));
            sec.record(Duration::from_millis(300));
        }

        let prefer_ml = ml.avg_ms() <= sec.avg_ms();
        assert!(
            prefer_ml,
            "ml-offload (50ms avg) should be preferred over securellm (300ms avg)"
        );
    }

    #[test]
    fn test_latency_tracker_no_data_prefers_ml_first() {
        // With no data both return MAX → ml_avg <= sec_avg is true → ml first (same as LocalFirst)
        let ml = LatencyTracker::new();
        let sec = LatencyTracker::new();
        let prefer_ml = ml.avg_ms() <= sec.avg_ms();
        assert!(prefer_ml, "Without data, LoadBalanced should fall back to local-first ordering");
    }
}
