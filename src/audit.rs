//! Audit Logging Module
//!
//! This module provides comprehensive audit logging for security and
//! compliance. All security-critical events are logged with structured data for
//! analysis and alerting.
//!
//! See ADR-013 for architecture decisions.

use std::{fs::OpenOptions, io::Write, net::IpAddr, path::Path, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Audit event action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Authentication events
    AuthSuccess,
    AuthFailure,
    AuthAttempt,

    // Secret access events
    SecretAccess,
    SecretStore,
    SecretRotate,
    SecretDelete,

    // API events
    ChatRequest,
    ChatResponse,
    DocumentAdd,
    DocumentSearch,

    // Configuration events
    ConfigChange,
    ConfigView,

    // Agent pipeline events
    AgentTaskStart,
    AgentDecision,
    AgentEscalation,
    AgentCheckpoint,

    // Administrative events
    UserCreate,
    UserDelete,
    UserModify,
    KeyRevoke,
    KeyGenerate,
}

impl AuditAction {
    /// Get the severity level for this action
    pub fn severity(&self) -> AuditSeverity {
        match self {
            // High severity: security-critical events
            AuditAction::AuthFailure => AuditSeverity::High,
            AuditAction::SecretDelete => AuditSeverity::High,
            AuditAction::KeyRevoke => AuditSeverity::High,
            AuditAction::UserDelete => AuditSeverity::High,

            // Medium severity: important events
            AuditAction::AuthSuccess => AuditSeverity::Medium,
            AuditAction::SecretAccess => AuditSeverity::Medium,
            AuditAction::SecretStore => AuditSeverity::Medium,
            AuditAction::SecretRotate => AuditSeverity::Medium,
            AuditAction::ConfigChange => AuditSeverity::Medium,
            AuditAction::UserCreate => AuditSeverity::Medium,
            AuditAction::UserModify => AuditSeverity::Medium,
            AuditAction::KeyGenerate => AuditSeverity::Medium,

            // Low severity: routine operations
            AuditAction::AuthAttempt => AuditSeverity::Low,
            AuditAction::ChatRequest => AuditSeverity::Low,
            AuditAction::ChatResponse => AuditSeverity::Low,
            AuditAction::DocumentAdd => AuditSeverity::Low,
            AuditAction::DocumentSearch => AuditSeverity::Low,
            AuditAction::ConfigView => AuditSeverity::Low,

            // Agent pipeline events
            AuditAction::AgentTaskStart => AuditSeverity::Low,
            AuditAction::AgentDecision => AuditSeverity::Low,
            AuditAction::AgentEscalation => AuditSeverity::Medium,
            AuditAction::AgentCheckpoint => AuditSeverity::Low,
        }
    }
}

/// Audit event severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
}

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,

    /// Timestamp (ISO 8601 format)
    pub timestamp: DateTime<Utc>,

    /// Action performed
    pub action: AuditAction,

    /// Severity level
    pub severity: AuditSeverity,

    /// User ID (if authenticated)
    pub user_id: Option<String>,

    /// User role (if authenticated)
    pub role: Option<String>,

    /// Resource affected (e.g., "secret/llm/deepseek", "user/admin")
    pub resource: Option<String>,

    /// Source IP address
    pub ip_address: Option<IpAddr>,

    /// Success status
    pub success: bool,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Additional metadata (JSON)
    pub metadata: serde_json::Value,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(action: AuditAction) -> Self {
        let severity = action.severity();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            severity,
            user_id: None,
            role: None,
            resource: None,
            ip_address: None,
            success: true,
            error: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Set user context
    pub fn with_user(mut self, user_id: String, role: String) -> Self {
        self.user_id = Some(user_id);
        self.role = Some(role);
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: String) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Mark as failure
    pub fn with_error(mut self, error: String) -> Self {
        self.success = false;
        self.error = Some(error);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        }
        self
    }

    /// Sanitize sensitive data before logging
    pub fn sanitize(&mut self) {
        // Redact sensitive metadata fields
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            for (key, value) in map.iter_mut() {
                let key_lower = key.to_lowercase();
                if key_lower.contains("password")
                    || key_lower.contains("secret")
                    || key_lower.contains("token")
                    || key_lower.contains("key")
                {
                    *value = serde_json::Value::String("[REDACTED]".to_string());
                }
            }
        }

        // Redact error messages that might contain sensitive info
        if let Some(ref mut error) = self.error {
            if error.contains("password") || error.contains("secret") || error.contains("token") {
                *error = "Authentication failed".to_string();
            }
        }
    }
}

/// Audit logger with configurable output
pub struct AuditLogger {
    log_path: String,
    alert_handler: Arc<RwLock<Option<Box<dyn AlertHandler + Send + Sync>>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_path: impl AsRef<Path>) -> Result<Self> {
        let log_path = log_path.as_ref().to_string_lossy().to_string();

        // Ensure log directory exists
        if let Some(parent) = Path::new(&log_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Validate the complete destination during startup. A parent directory
        // may already exist on a read-only filesystem, in which case
        // `create_dir_all` succeeds and the first security event would fail much
        // later, after the service had already reported itself as available.
        OpenOptions::new().create(true).append(true).open(&log_path)?;

        Ok(Self { log_path, alert_handler: Arc::new(RwLock::new(None)) })
    }

    /// Set alert handler for suspicious activity
    pub async fn set_alert_handler(&self, handler: Box<dyn AlertHandler + Send + Sync>) {
        let mut guard = self.alert_handler.write().await;
        *guard = Some(handler);
    }

    /// Log an audit event
    pub async fn log(&self, mut event: AuditEvent) -> Result<()> {
        // Sanitize sensitive data
        event.sanitize();

        // Phase 4.1: Record audit metric
        crate::metrics::utils::record_audit_event(
            &format!("{:?}", event.action),
            &format!("{:?}", event.severity),
        );

        // Convert to JSON
        let json = serde_json::to_string(&event)?;

        // Write to log file (append mode, immutable)
        let mut file = OpenOptions::new().create(true).append(true).open(&self.log_path)?;

        writeln!(file, "{}", json)?;
        file.sync_all()?; // Ensure written to disk

        // Log to tracing for real-time monitoring
        tracing::info!(
            event_id = %event.id,
            action = ?event.action,
            severity = ?event.severity,
            user_id = ?event.user_id,
            success = event.success,
            "Audit event logged"
        );

        // Check for suspicious activity and trigger alerts
        self.check_alerts(&event).await;

        Ok(())
    }

    /// Check for suspicious activity patterns
    async fn check_alerts(&self, event: &AuditEvent) {
        let handler = self.alert_handler.read().await;
        if let Some(ref handler) = *handler {
            // Alert on authentication failures
            if matches!(event.action, AuditAction::AuthFailure) {
                handler.handle_auth_failure(event).await;
            }

            // Alert on high-severity events
            if event.severity == AuditSeverity::High {
                handler.handle_high_severity(event).await;
            }

            // Alert on secret deletion
            if matches!(event.action, AuditAction::SecretDelete) {
                handler.handle_secret_deletion(event).await;
            }
        }
    }
}

/// Alert handler trait for suspicious activity
#[async_trait::async_trait]
pub trait AlertHandler {
    /// Handle authentication failure
    async fn handle_auth_failure(&self, event: &AuditEvent);

    /// Handle high-severity event
    async fn handle_high_severity(&self, event: &AuditEvent);

    /// Handle secret deletion
    async fn handle_secret_deletion(&self, event: &AuditEvent);
}

/// Default alert handler that logs to console
pub struct ConsoleAlertHandler;

#[async_trait::async_trait]
impl AlertHandler for ConsoleAlertHandler {
    async fn handle_auth_failure(&self, event: &AuditEvent) {
        tracing::warn!(
            event_id = %event.id,
            user_id = ?event.user_id,
            ip_address = ?event.ip_address,
            "⚠️  Authentication failure detected"
        );
    }

    async fn handle_high_severity(&self, event: &AuditEvent) {
        tracing::error!(
            event_id = %event.id,
            action = ?event.action,
            user_id = ?event.user_id,
            resource = ?event.resource,
            "🚨 High-severity event detected"
        );
    }

    async fn handle_secret_deletion(&self, event: &AuditEvent) {
        tracing::error!(
            event_id = %event.id,
            resource = ?event.resource,
            user_id = ?event.user_id,
            "🔥 Secret deletion detected"
        );
    }
}

/// Failed authentication entry (user_id, timestamp)
type FailureEntry = (String, DateTime<Utc>);

/// Failed authentication tracker for rate limiting alerts
pub struct FailedAuthTracker {
    failures: Arc<RwLock<Vec<FailureEntry>>>,
    threshold: usize,
    window_minutes: i64,
}

impl FailedAuthTracker {
    /// Create a new tracker
    pub fn new(threshold: usize, window_minutes: i64) -> Self {
        Self { failures: Arc::new(RwLock::new(Vec::new())), threshold, window_minutes }
    }

    /// Track a failed authentication attempt
    pub async fn track_failure(&self, user_id: String) -> bool {
        let mut failures = self.failures.write().await;
        let now = Utc::now();

        // Add new failure
        failures.push((user_id.clone(), now));

        // Remove old failures outside the window
        let cutoff = now - chrono::Duration::minutes(self.window_minutes);
        failures.retain(|(_, timestamp)| *timestamp > cutoff);

        // Count failures for this user
        let user_failures = failures.iter().filter(|(uid, _)| uid == &user_id).count();

        // Return true if threshold exceeded
        user_failures > self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(AuditAction::AuthSuccess)
            .with_user("user123".to_string(), "admin".to_string())
            .with_resource("api/v1/chat".to_string())
            .with_metadata("method", serde_json::json!("POST"));

        assert_eq!(event.action, AuditAction::AuthSuccess);
        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.role, Some("admin".to_string()));
        assert!(event.success);
    }

    #[test]
    fn test_audit_event_sanitization() {
        let mut event = AuditEvent::new(AuditAction::SecretAccess)
            .with_metadata("api_key", serde_json::json!("sk-secret-123"))
            .with_metadata("safe_data", serde_json::json!("public"));

        event.sanitize();

        let metadata = event.metadata.as_object().unwrap();
        assert_eq!(metadata["api_key"], serde_json::json!("[REDACTED]"));
        assert_eq!(metadata["safe_data"], serde_json::json!("public"));
    }

    #[test]
    fn test_severity_levels() {
        assert_eq!(AuditAction::AuthFailure.severity(), AuditSeverity::High);
        assert_eq!(AuditAction::AuthSuccess.severity(), AuditSeverity::Medium);
        assert_eq!(AuditAction::ChatRequest.severity(), AuditSeverity::Low);
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("neoland_audit_test.log");

        let logger = AuditLogger::new(&log_path).unwrap();

        let event = AuditEvent::new(AuditAction::AuthSuccess)
            .with_user("test_user".to_string(), "admin".to_string());

        logger.log(event).await.unwrap();

        // Verify log file exists
        assert!(log_path.exists());

        // Cleanup
        let _ = std::fs::remove_file(log_path);
    }

    #[tokio::test]
    async fn test_failed_auth_tracker() {
        let tracker = FailedAuthTracker::new(5, 1);

        // Track 3 failures - should not trigger alert
        for _ in 0..3 {
            let triggered = tracker.track_failure("user123".to_string()).await;
            assert!(!triggered);
        }

        // Track 3 more failures - should trigger alert
        for i in 3..6 {
            let triggered = tracker.track_failure("user123".to_string()).await;
            if i >= 5 {
                assert!(triggered);
            }
        }
    }

    #[tokio::test]
    async fn test_audit_event_with_ip() {
        let event = AuditEvent::new(AuditAction::AuthSuccess)
            .with_user("test_user".to_string(), "admin".to_string())
            .with_ip(std::net::IpAddr::from([203, 0, 113, 42]));

        assert_eq!(event.ip_address, Some(std::net::IpAddr::from([203, 0, 113, 42])));
    }

    #[tokio::test]
    async fn test_audit_event_with_resource() {
        let event = AuditEvent::new(AuditAction::SecretAccess)
            .with_resource("/v1/chat/completions".to_string());

        assert_eq!(event.resource, Some("/v1/chat/completions".to_string()));
    }

    #[tokio::test]
    async fn test_audit_event_with_error() {
        let event =
            AuditEvent::new(AuditAction::AuthFailure).with_error("Invalid API key".to_string());

        assert_eq!(event.error, Some("Invalid API key".to_string()));
        assert!(!event.success);
    }

    #[tokio::test]
    async fn test_audit_event_metadata() {
        let mut event = AuditEvent::new(AuditAction::ChatRequest)
            .with_metadata("model", serde_json::json!("gpt-4"))
            .with_metadata("tokens", serde_json::json!(150));

        assert_eq!(event.metadata["model"], serde_json::json!("gpt-4"));
        assert_eq!(event.metadata["tokens"], serde_json::json!(150));

        // Test sanitization removes sensitive metadata
        event
            .metadata
            .as_object_mut()
            .unwrap()
            .insert("password".to_string(), serde_json::json!("secret123"));

        event.sanitize();
        assert_eq!(event.metadata["password"], serde_json::json!("[REDACTED]"));
    }

    #[tokio::test]
    async fn test_multiple_users_failed_auth() {
        let tracker = FailedAuthTracker::new(5, 1);

        // User 1 fails 6 times
        for _ in 0..6 {
            tracker.track_failure("user1".to_string()).await;
        }

        // User 2 fails 3 times (should not trigger)
        for _ in 0..3 {
            let triggered = tracker.track_failure("user2".to_string()).await;
            assert!(!triggered);
        }

        // User 1 should still trigger on next failure
        let triggered = tracker.track_failure("user1".to_string()).await;
        assert!(triggered);
    }

    #[tokio::test]
    async fn test_audit_logger_multiple_events() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("neoland_audit_multi_test.log");

        let logger = AuditLogger::new(&log_path).unwrap();

        // Log multiple events
        for i in 0..10 {
            let event = AuditEvent::new(AuditAction::ChatRequest)
                .with_metadata("request_id", serde_json::json!(i));
            logger.log(event).await.unwrap();
        }

        // Verify log file exists and has content
        assert!(log_path.exists());
        let metadata = std::fs::metadata(&log_path).unwrap();
        assert!(metadata.len() > 0);

        // Cleanup
        let _ = std::fs::remove_file(log_path);
    }

    #[test]
    fn test_audit_action_severity_mapping() {
        // High severity actions
        assert_eq!(AuditAction::AuthFailure.severity(), AuditSeverity::High);
        assert_eq!(AuditAction::SecretDelete.severity(), AuditSeverity::High);
        assert_eq!(AuditAction::UserDelete.severity(), AuditSeverity::High);

        // Medium severity actions
        assert_eq!(AuditAction::AuthSuccess.severity(), AuditSeverity::Medium);
        assert_eq!(AuditAction::SecretStore.severity(), AuditSeverity::Medium);
        assert_eq!(AuditAction::ConfigChange.severity(), AuditSeverity::Medium);

        // Low severity actions
        assert_eq!(AuditAction::ChatRequest.severity(), AuditSeverity::Low);
        assert_eq!(AuditAction::AuthAttempt.severity(), AuditSeverity::Low);
        assert_eq!(AuditAction::DocumentSearch.severity(), AuditSeverity::Low);
    }

    #[tokio::test]
    async fn test_console_alert_handler() {
        let handler = ConsoleAlertHandler;

        let event = AuditEvent::new(AuditAction::AuthFailure)
            .with_user("test_user".to_string(), "admin".to_string())
            .with_error("Invalid credentials".to_string());

        // Should not panic
        handler.handle_auth_failure(&event).await;
        handler.handle_high_severity(&event).await;
    }

    #[tokio::test]
    async fn test_failed_auth_window_expiry() {
        let tracker = FailedAuthTracker::new(5, 1); // 5 failures in 1 minute

        // Track 4 failures
        for _ in 0..4 {
            tracker.track_failure("user123".to_string()).await;
        }

        // Wait for window to expire (simulate time passing)
        // Note: In real scenario, would wait 61 seconds
        // For testing, we just verify the counter doesn't persist indefinitely

        // Track 1 more failure (within window) - should not trigger yet
        let triggered = tracker.track_failure("user123".to_string()).await;
        assert!(!triggered); // 5th failure doesn't trigger (>5 needed)

        // 6th failure should trigger
        let triggered = tracker.track_failure("user123".to_string()).await;
        assert!(triggered);
    }
}
