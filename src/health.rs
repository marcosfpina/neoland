// Phase 4.3: Health Checks & Readiness Probes
// Production-ready health monitoring for Kubernetes liveness/readiness probes

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical systems degraded, but service is still operational
    Degraded,
    /// Critical systems down, service unavailable
    Unhealthy,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name (e.g., "vector_store", "llm_provider")
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Human-readable status message
    pub message: String,
    /// Response time in milliseconds (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}

/// Overall health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall service status
    pub status: HealthStatus,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    /// Individual component health
    pub components: Vec<ComponentHealth>,
}

/// Readiness check response (simpler than health)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessResponse {
    /// Whether service is ready to accept traffic
    pub ready: bool,
    /// Components that must be ready
    pub components: Vec<ComponentHealth>,
}

/// Liveness check response (minimal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResponse {
    /// Whether service is alive (not deadlocked)
    pub alive: bool,
}

/// Health checker for vector store.
///
/// Checks `NEOLAND_DATABASE_URL` (or `DATABASE_URL`):
/// - Not set  → Healthy  (in-memory mode)
/// - Set, reachable + pgvector present → Healthy
/// - Set, reachable but no pgvector   → Degraded
/// - Set, unreachable or timeout       → Degraded
pub async fn check_vector_store_health() -> ComponentHealth {
    let db_url = std::env::var("NEOLAND_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok();
    check_vector_store_health_with_url(db_url).await
}

/// Inner implementation — accepts URL directly for testability without env-var
/// side effects.
async fn check_vector_store_health_with_url(db_url: Option<String>) -> ComponentHealth {
    let start = std::time::Instant::now();

    let Some(url) = db_url else {
        return ComponentHealth {
            name: "vector_store".to_string(),
            status: HealthStatus::Healthy,
            message: "In-memory mode (set NEOLAND_DATABASE_URL for PostgreSQL persistence)"
                .to_string(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        };
    };

    // Attempt connection with 2-second timeout
    use sqlx::Connection;
    let connect_result =
        tokio::time::timeout(Duration::from_secs(2), sqlx::PgConnection::connect(&url)).await;

    match connect_result {
        Ok(Ok(mut conn)) => {
            let pgvector: Result<bool, sqlx::Error> = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')",
            )
            .fetch_one(&mut conn)
            .await;
            let _ = conn.close().await;

            match pgvector {
                Ok(true) => ComponentHealth {
                    name: "vector_store".to_string(),
                    status: HealthStatus::Healthy,
                    message: "PostgreSQL + pgvector operational".to_string(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                },
                Ok(false) => ComponentHealth {
                    name: "vector_store".to_string(),
                    status: HealthStatus::Degraded,
                    message: "PostgreSQL reachable but pgvector not installed (run: CREATE \
                              EXTENSION vector)"
                        .to_string(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                },
                Err(e) => ComponentHealth {
                    name: "vector_store".to_string(),
                    status: HealthStatus::Degraded,
                    message: format!("PostgreSQL connected but query failed: {e}"),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                },
            }
        },
        Ok(Err(e)) => {
            warn!(error = %e, "PostgreSQL health check: connection failed");
            ComponentHealth {
                name: "vector_store".to_string(),
                status: HealthStatus::Degraded,
                message: format!("PostgreSQL unreachable: {e}"),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            }
        },
        Err(_timeout) => {
            warn!("PostgreSQL health check: connection timed out (2s)");
            ComponentHealth {
                name: "vector_store".to_string(),
                status: HealthStatus::Degraded,
                message: "PostgreSQL connection timed out (>2s)".to_string(),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            }
        },
    }
}

/// Health checker for LLM providers — probes the SecureLLM Bridge gateway.
pub async fn check_llm_health() -> ComponentHealth {
    let url = std::env::var("NEOLAND_GATEWAY_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    check_llm_health_with_url(&url).await
}

async fn check_llm_health_with_url(base_url: &str) -> ComponentHealth {
    let start = std::time::Instant::now();
    let base = base_url.trim_end_matches('/');
    let probe_urls = [format!("{base}/health"), format!("{base}/api/health")];
    let client = reqwest::Client::new();
    let mut last_error = None;

    for probe_url in &probe_urls {
        match client.get(probe_url).timeout(Duration::from_secs(2)).send().await {
            Ok(resp) if resp.status().is_success() => {
                let elapsed = start.elapsed().as_millis() as u64;
                debug!("LLM gateway health check OK in {}ms", elapsed);
                return ComponentHealth {
                    name: "llm_provider".to_string(),
                    status: HealthStatus::Healthy,
                    message: format!("SecureLLM Bridge reachable at {}", probe_url),
                    response_time_ms: Some(elapsed),
                };
            },
            Ok(resp) => {
                last_error = Some(format!("{} returned HTTP {}", probe_url, resp.status()));
            },
            Err(e) => {
                last_error = Some(format!("{} failed ({})", probe_url, e));
            },
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;
    let message =
        last_error.unwrap_or_else(|| format!("SecureLLM Bridge unreachable ({base_url})"));
    warn!("LLM gateway health check degraded: {}", message);
    ComponentHealth {
        name: "llm_provider".to_string(),
        status: HealthStatus::Degraded,
        message,
        response_time_ms: Some(elapsed),
    }
}

/// Health checker for authentication system — verifies the API key store is
/// reachable and actually holds keys (an empty store fails every request).
pub async fn check_auth_health(auth: &crate::auth::AuthManager) -> ComponentHealth {
    let start = std::time::Instant::now();

    let (status, message) = match auth.list_api_keys() {
        Ok(keys) if !keys.is_empty() => {
            (HealthStatus::Healthy, format!("{} API key(s) loaded", keys.len()))
        },
        Ok(_) => (
            HealthStatus::Degraded,
            "No API keys loaded — all authenticated requests will fail".to_string(),
        ),
        Err(e) => (HealthStatus::Degraded, format!("API key store unavailable: {e}")),
    };

    ComponentHealth {
        name: "auth_system".to_string(),
        status,
        message,
        response_time_ms: Some(start.elapsed().as_millis() as u64),
    }
}

/// Health checker for audit logging — probes that the audit log file is still
/// writable (disk full, permissions or a rotated-away directory show up here).
pub async fn check_audit_health(audit: &crate::audit::AuditLogger) -> ComponentHealth {
    let start = std::time::Instant::now();

    let (status, message) = match audit.probe_writable() {
        Ok(()) => (HealthStatus::Healthy, "Audit log writable".to_string()),
        Err(e) => (HealthStatus::Degraded, format!("Audit log not writable: {e}")),
    };

    ComponentHealth {
        name: "audit_logging".to_string(),
        status,
        message,
        response_time_ms: Some(start.elapsed().as_millis() as u64),
    }
}

/// Health checker for metrics — verifies the Prometheus registry has content.
pub async fn check_metrics_health() -> ComponentHealth {
    let start = std::time::Instant::now();

    let families = prometheus::gather();
    let (status, message) = if families.is_empty() {
        (HealthStatus::Degraded, "Prometheus registry is empty".to_string())
    } else {
        (HealthStatus::Healthy, format!("{} metric families registered", families.len()))
    };

    ComponentHealth {
        name: "metrics".to_string(),
        status,
        message,
        response_time_ms: Some(start.elapsed().as_millis() as u64),
    }
}

/// Perform complete health check
pub async fn perform_health_check(
    uptime_seconds: Option<u64>,
    auth: &crate::auth::AuthManager,
    audit: &crate::audit::AuditLogger,
) -> HealthResponse {
    debug!("Performing comprehensive health check");

    // Check all components in parallel
    let (vector_store, llm, auth, audit, metrics) = tokio::join!(
        check_vector_store_health(),
        check_llm_health(),
        check_auth_health(auth),
        check_audit_health(audit),
        check_metrics_health(),
    );

    let components = vec![vector_store, llm, auth, audit, metrics];

    // Determine overall status based on components
    let overall_status = if components.iter().any(|c| c.status == HealthStatus::Unhealthy) {
        HealthStatus::Unhealthy
    } else if components.iter().any(|c| c.status == HealthStatus::Degraded) {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    if overall_status != HealthStatus::Healthy {
        warn!("System health check: {:?}", overall_status);
    }

    HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        components,
    }
}

/// Perform readiness check (simpler, only checks critical components)
pub async fn perform_readiness_check() -> ReadinessResponse {
    debug!("Performing readiness check");

    // Only check critical components for readiness
    let (vector_store, llm) = tokio::join!(check_vector_store_health(), check_llm_health(),);

    let components = vec![vector_store, llm];

    // Service is ready unless a critical component is fully Unhealthy.
    // Degraded is acceptable (e.g. DB unreachable but in-memory fallback active).
    let ready = components.iter().all(|c| c.status != HealthStatus::Unhealthy);

    if !ready {
        warn!("Readiness check failed: one or more critical components unhealthy");
    }

    ReadinessResponse { ready, components }
}

/// Perform liveness check (very simple, just confirms process is responsive)
pub async fn perform_liveness_check() -> LivenessResponse {
    // If we can respond, we're alive
    LivenessResponse { alive: true }
}

/// Graceful shutdown handler
pub struct ShutdownHandler {
    start_time: std::time::Instant,
}

impl ShutdownHandler {
    pub fn new() -> Self {
        Self { start_time: std::time::Instant::now() }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown_signal(&self) {
        use tokio::signal;

        let ctrl_c = async {
            signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received Ctrl+C, initiating graceful shutdown");
            },
            _ = terminate => {
                tracing::info!("Received SIGTERM, initiating graceful shutdown");
            },
        }
    }

    /// Perform graceful shutdown
    pub async fn shutdown(&self, grace_period: Duration) {
        tracing::info!(
            "Shutting down gracefully (uptime: {}s, grace period: {:?})",
            self.uptime_seconds(),
            grace_period
        );

        // Give in-flight requests time to complete
        tokio::time::sleep(grace_period).await;

        tracing::info!("Shutdown complete");
    }
}

impl Default for ShutdownHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_store_health_in_memory() {
        let health = check_vector_store_health_with_url(None).await;
        assert_eq!(health.name, "vector_store");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.contains("in-memory") || health.message.contains("In-memory"));
        assert!(health.response_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_vector_store_health_unreachable_db() {
        // localhost:1 is always connection-refused immediately (no timeout needed)
        let health =
            check_vector_store_health_with_url(Some("postgresql://localhost:1/neoland".into()))
                .await;
        assert_eq!(health.name, "vector_store");
        assert_eq!(health.status, HealthStatus::Degraded);
        assert!(health.response_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_llm_health() {
        let health = check_llm_health().await;
        assert_eq!(health.name, "llm_provider");
        // Healthy when gateway is up, Degraded when not running — never Unhealthy
        assert_ne!(health.status, HealthStatus::Unhealthy);
        assert!(health.response_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_llm_health_unreachable() {
        let health = check_llm_health_with_url("http://127.0.0.1:19999").await;
        assert_eq!(health.name, "llm_provider");
        assert_eq!(health.status, HealthStatus::Degraded);
        assert!(health.response_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_auth_health() {
        // AuthManager::new() seeds the dev keys, so the store is non-empty
        let auth = crate::auth::AuthManager::new();
        let health = check_auth_health(&auth).await;
        assert_eq!(health.name, "auth_system");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.contains("API key"));
    }

    #[tokio::test]
    async fn test_audit_health_writable_and_not() {
        let dir = tempfile::tempdir().expect("tempdir");
        let audit =
            crate::audit::AuditLogger::new(dir.path().join("audit.log")).expect("audit logger");
        let health = check_audit_health(&audit).await;
        assert_eq!(health.name, "audit_logging");
        assert_eq!(health.status, HealthStatus::Healthy);

        // Remove the directory underneath the logger — probe must degrade
        drop(dir);
        let health = check_audit_health(&audit).await;
        assert_eq!(health.status, HealthStatus::Degraded);
        assert!(health.message.contains("not writable"));
    }

    #[tokio::test]
    async fn test_metrics_health_reports_registry() {
        // The lazy_static metrics register on first touch; force one
        crate::metrics::HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/health", "200"])
            .inc();
        let health = check_metrics_health().await;
        assert_eq!(health.name, "metrics");
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_complete_health_check() {
        // perform_health_check reads NEOLAND_DATABASE_URL; ensure it's absent
        // by calling the inner function directly for the vector_store component
        let vs = check_vector_store_health_with_url(None).await;
        assert_eq!(vs.status, HealthStatus::Healthy);

        let auth = crate::auth::AuthManager::new();
        let dir = tempfile::tempdir().expect("tempdir");
        let audit =
            crate::audit::AuditLogger::new(dir.path().join("audit.log")).expect("audit logger");
        let response = perform_health_check(Some(100), &auth, &audit).await;
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(response.uptime_seconds, Some(100));
        assert_eq!(response.components.len(), 5);
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let response = perform_readiness_check().await;
        // ready == true as long as no component is Unhealthy
        assert!(response.components.iter().all(|c| c.status != HealthStatus::Unhealthy));
        assert_eq!(response.components.len(), 2);
    }

    #[tokio::test]
    async fn test_readiness_degraded_still_ready() {
        // A Degraded vector_store (unreachable DB) must not block readiness
        let vs =
            check_vector_store_health_with_url(Some("postgresql://localhost:1/neoland".into()))
                .await;
        assert_eq!(vs.status, HealthStatus::Degraded);
        // Degraded ≠ Unhealthy, so service remains ready
        assert!(vs.status != HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let response = perform_liveness_check().await;
        assert!(response.alive);
    }

    #[test]
    fn test_shutdown_handler_uptime() {
        let handler = ShutdownHandler::new();
        std::thread::sleep(Duration::from_millis(100));
        // uptime_seconds() should be at least 0 (always true for u64, but good for
        // documentation)
        let uptime = handler.uptime_seconds();
        assert!(uptime < 10); // Should be less than 10 seconds for this test
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"healthy\"");

        let status = HealthStatus::Degraded;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"degraded\"");

        let status = HealthStatus::Unhealthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"unhealthy\"");
    }

    #[test]
    fn test_component_health_serialization() {
        let component = ComponentHealth {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            message: "All good".to_string(),
            response_time_ms: Some(42),
        };

        let json = serde_json::to_string(&component).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"response_time_ms\":42"));
    }
}
