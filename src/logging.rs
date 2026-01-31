// Phase 4.2: Structured Logging
// Production-ready logging with JSON output, correlation IDs, and performance tracking

use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Logging configuration for different environments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable output for development (default)
    Pretty,
    /// JSON output for production log aggregation
    Json,
    /// Compact output for CI/CD
    Compact,
}

impl Default for LogFormat {
    fn default() -> Self {
        // Use JSON in production, Pretty in dev
        if cfg!(debug_assertions) {
            LogFormat::Pretty
        } else {
            LogFormat::Json
        }
    }
}

/// Logging configuration
pub struct LogConfig {
    /// Log format (Pretty, JSON, Compact)
    pub format: LogFormat,
    /// Log level (trace, debug, info, warn, error)
    pub level: Level,
    /// Enable performance logging (request duration, etc.)
    pub enable_performance: bool,
    /// Enable correlation IDs for request tracing
    pub enable_correlation_ids: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::default(),
            level: if cfg!(debug_assertions) {
                Level::DEBUG
            } else {
                Level::INFO
            },
            enable_performance: true,
            enable_correlation_ids: true,
        }
    }
}

impl LogConfig {
    /// Create production configuration (JSON logs, INFO level)
    pub fn production() -> Self {
        Self {
            format: LogFormat::Json,
            level: Level::INFO,
            enable_performance: true,
            enable_correlation_ids: true,
        }
    }

    /// Create development configuration (Pretty logs, DEBUG level)
    pub fn development() -> Self {
        Self {
            format: LogFormat::Pretty,
            level: Level::DEBUG,
            enable_performance: true,
            enable_correlation_ids: true,
        }
    }

    /// Create minimal configuration for CI/CD (Compact logs, WARN level)
    pub fn ci() -> Self {
        Self {
            format: LogFormat::Compact,
            level: Level::WARN,
            enable_performance: false,
            enable_correlation_ids: false,
        }
    }
}

/// Initialize structured logging with the given configuration
pub fn init_logging(config: LogConfig) -> anyhow::Result<()> {
    // Build environment filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default filter based on level
        EnvFilter::new(format!(
            "{}={}",
            env!("CARGO_PKG_NAME").replace('-', "_"),
            config.level
        ))
        // Additional module filters
        .add_directive("tower_http=info".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap())
    });

    // Build subscriber based on format
    match config.format {
        LogFormat::Pretty => {
            // Human-readable format for development
            let fmt_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_line_number(true)
                .with_file(true)
                .with_level(true)
                .pretty();

            tracing_subscriber::registry().with(env_filter).with(fmt_layer).try_init()?;
        },
        LogFormat::Json => {
            // JSON format for production log aggregation (Loki, ELK, etc.)
            let fmt_layer = fmt::layer()
                .json()
                .with_target(true)
                .with_current_span(true)
                .with_span_list(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .flatten_event(true);

            tracing_subscriber::registry().with(env_filter).with(fmt_layer).try_init()?;
        },
        LogFormat::Compact => {
            // Compact format for CI/CD
            let fmt_layer = fmt::layer().compact().with_target(false).with_thread_ids(false);

            tracing_subscriber::registry().with(env_filter).with(fmt_layer).try_init()?;
        },
    }

    Ok(())
}

/// Initialize logging from environment variables
///
/// Environment variables:
/// - `RUST_LOG`: Log level filter (e.g., "info", "debug", "llamachat_poc=trace")
/// - `LOG_FORMAT`: Output format ("pretty", "json", "compact")
pub fn init_from_env() -> anyhow::Result<()> {
    let format = std::env::var("LOG_FORMAT")
        .ok()
        .and_then(|f| match f.to_lowercase().as_str() {
            "pretty" => Some(LogFormat::Pretty),
            "json" => Some(LogFormat::Json),
            "compact" => Some(LogFormat::Compact),
            _ => None,
        })
        .unwrap_or_default();

    let config = LogConfig { format, ..Default::default() };

    init_logging(config)
}

/// Correlation ID for request tracing
///
/// Use this to track requests across service boundaries and log aggregation
#[derive(Debug, Clone)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Generate a new correlation ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from existing ID (e.g., from HTTP header)
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the ID as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Performance logger for tracking request/operation duration
pub struct PerformanceLogger {
    operation: String,
    start: std::time::Instant,
    correlation_id: Option<CorrelationId>,
}

impl PerformanceLogger {
    /// Start tracking an operation
    pub fn start(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            start: std::time::Instant::now(),
            correlation_id: None,
        }
    }

    /// Start tracking with correlation ID
    pub fn start_with_correlation(
        operation: impl Into<String>,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            operation: operation.into(),
            start: std::time::Instant::now(),
            correlation_id: Some(correlation_id),
        }
    }

    /// Finish tracking and log performance
    pub fn finish(self) {
        let duration = self.start.elapsed();

        if let Some(ref correlation_id) = self.correlation_id {
            tracing::info!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                correlation_id = %correlation_id,
                "Operation completed"
            );
        } else {
            tracing::info!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                "Operation completed"
            );
        }

        // Also record in metrics if duration is significant
        if duration.as_millis() > 100 {
            tracing::debug!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                "Slow operation detected"
            );
        }
    }

    /// Finish with error
    pub fn finish_with_error(self, error: &dyn std::error::Error) {
        let duration = self.start.elapsed();

        if let Some(ref correlation_id) = self.correlation_id {
            tracing::error!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                correlation_id = %correlation_id,
                error = %error,
                "Operation failed"
            );
        } else {
            tracing::error!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                error = %error,
                "Operation failed"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_defaults() {
        let config = LogConfig::default();
        if cfg!(debug_assertions) {
            assert_eq!(config.format, LogFormat::Pretty);
            assert_eq!(config.level, Level::DEBUG);
        } else {
            assert_eq!(config.format, LogFormat::Json);
            assert_eq!(config.level, Level::INFO);
        }
        assert!(config.enable_performance);
        assert!(config.enable_correlation_ids);
    }

    #[test]
    fn test_production_config() {
        let config = LogConfig::production();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, Level::INFO);
    }

    #[test]
    fn test_development_config() {
        let config = LogConfig::development();
        assert_eq!(config.format, LogFormat::Pretty);
        assert_eq!(config.level, Level::DEBUG);
    }

    #[test]
    fn test_ci_config() {
        let config = LogConfig::ci();
        assert_eq!(config.format, LogFormat::Compact);
        assert_eq!(config.level, Level::WARN);
        assert!(!config.enable_performance);
        assert!(!config.enable_correlation_ids);
    }

    #[test]
    fn test_correlation_id_generation() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();
        assert_ne!(id1.as_str(), id2.as_str());
        assert_eq!(id1.as_str().len(), 36); // UUID v4 format
    }

    #[test]
    fn test_correlation_id_from_string() {
        let id_str = "test-correlation-id-123".to_string();
        let id = CorrelationId::from_string(id_str.clone());
        assert_eq!(id.as_str(), &id_str);
    }

    #[test]
    fn test_performance_logger_basic() {
        let logger = PerformanceLogger::start("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(1));
        logger.finish();
    }

    #[test]
    fn test_performance_logger_with_correlation() {
        let correlation_id = CorrelationId::new();
        let logger = PerformanceLogger::start_with_correlation("test_op", correlation_id);
        std::thread::sleep(std::time::Duration::from_millis(1));
        logger.finish();
    }
}
