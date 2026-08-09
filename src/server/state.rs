//! Estado compartilhado do servidor: `AppState` e `RateLimiter`.

use super::*;

// Rate limiter for tracking requests per user/IP
pub struct RateLimiter {
    // Map of (user_id or IP) -> (request_count, window_start_time)
    requests: RwLock<HashMap<String, (u32, Instant)>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
        }
    }

    /// Check if request is allowed, returns true if rate limit exceeded
    pub async fn check_rate_limit(&self, identifier: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        if let Some((count, window_start)) = requests.get_mut(identifier) {
            // Check if we're still in the same time window
            if now.duration_since(*window_start) < self.window_duration {
                *count += 1;
                if *count > self.max_requests {
                    return true; // Rate limit exceeded
                }
            } else {
                // New time window, reset counter
                *window_start = now;
                *count = 1;
            }
        } else {
            // First request from this identifier
            requests.insert(identifier.to_string(), (1, now));
        }

        false // Not rate limited
    }

    /// Clean up old entries (optional, for memory management)
    pub async fn cleanup_old_entries(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        requests.retain(|_, (_, window_start)| {
            now.duration_since(*window_start) < self.window_duration * 2
        });
    }
}

// Shared State for gRPC and REST
//
// ## Lock Ordering (MUST be followed to prevent deadlocks)
//
// When acquiring multiple locks, always acquire them in this order:
//   1. engine (Arc<Mutex<Option<LocalEngine>>>)
//   2. vector_store (Arc<Mutex<VectorStore>>)
//
// The chat_stream handler acquires engine first, then vector_store while
// holding the engine lock. All other code paths must follow this same
// ordering. Never acquire engine while holding vector_store.
//
// Other fields (auth_manager, audit_logger, etc.) use interior mutability
// or Arc-only patterns and don't participate in lock ordering.
pub struct AppState {
    pub(crate) engine: Arc<Mutex<Option<LocalEngine>>>,
    pub(crate) vector_store: Arc<Mutex<VectorStore>>,
    pub(crate) persistent_store: Option<Arc<PersistentVectorStore>>,
    pub(crate) auth_manager: Arc<AuthManager>,
    pub(crate) audit_logger: Arc<AuditLogger>,
    pub(crate) failed_auth_tracker: Arc<FailedAuthTracker>,
    pub(crate) rate_limiter: Arc<RateLimiter>,
    pub(crate) start_time: Instant,
    pub(crate) agent_orchestrator: Option<Arc<AgentOrchestrator>>,
    pub(crate) event_bus: tokio::sync::broadcast::Sender<AgentEvent>,
    /// Database pool (v0.0.1 enterprise auth)
    pub db_pool: Option<sqlx::PgPool>,
    /// JWT signing secret (v0.0.1)
    pub jwt_secret: Vec<u8>,
    /// OAuth2 base URL (v0.0.1)
    pub oauth_base_url: String,
}
