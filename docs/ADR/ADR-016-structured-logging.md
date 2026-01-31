# ADR-016: Structured Logging Implementation

**Status**: Accepted
**Date**: 2026-01-31
**Decision Makers**: Architecture Team
**Phase**: 4.2 - Operational Readiness

---

## Context

NEOLAND requires production-grade logging for:
- **Debugging**: Root cause analysis of production issues
- **Monitoring**: Real-time operational visibility
- **Compliance**: Audit trail for regulatory requirements
- **Performance**: Identifying slow operations and bottlenecks
- **Distributed Tracing**: Request tracking across service boundaries

The current logging setup uses basic `tracing-subscriber` with console output, which is insufficient for production operations.

---

## Decision

Implement **structured logging** with the following components:

### 1. Multi-Format Support
- **Pretty Format**: Human-readable console output for development
- **JSON Format**: Machine-parseable logs for production (Loki, ELK, CloudWatch)
- **Compact Format**: Minimal output for CI/CD pipelines

### 2. Correlation IDs
- Unique identifier for each HTTP/gRPC request
- Extracted from `X-Correlation-ID` header or auto-generated (UUID v4)
- Propagated through all log entries for request tracing
- Stored in request extensions for handlers

### 3. Performance Logging
- Automatic operation duration tracking
- Slow operation detection (>100ms threshold)
- Integration with Prometheus metrics

### 4. Log Levels
- **TRACE**: Very detailed debugging (disabled in production)
- **DEBUG**: Detailed debugging information
- **INFO**: General informational messages (production default)
- **WARN**: Warning conditions
- **ERROR**: Error conditions requiring attention

### 5. Environment-Based Configuration
- `LOG_FORMAT` env var: `pretty`, `json`, `compact`
- `RUST_LOG` env var: Standard tracing filter
- CLI flags: `--log-level` for runtime override

---

## Implementation

### Module: `src/logging.rs`

```rust
// Log configuration
pub struct LogConfig {
    format: LogFormat,      // Pretty, JSON, Compact
    level: Level,           // TRACE, DEBUG, INFO, WARN, ERROR
    enable_performance: bool,
    enable_correlation_ids: bool,
}

// Correlation ID for request tracing
pub struct CorrelationId(String);

// Performance tracking
pub struct PerformanceLogger {
    operation: String,
    start: Instant,
    correlation_id: Option<CorrelationId>,
}
```

### Middleware Stack

```
Request
  ↓
correlation_middleware (Phase 4.2) - Extract/generate correlation ID
  ↓
rate_limit_middleware (Phase 1.4) - Check rate limits
  ↓
validation_middleware (Phase 1.4) - Validate input
  ↓
auth_middleware (Phase 1.1) - Authenticate request
  ↓
Handler
```

### Log Output Examples

**Development (Pretty Format)**:
```
2026-01-31T12:34:56.789Z  INFO http_request{correlation_id=a1b2c3d4 method=POST uri=/v1/chat/completions}: neoland::server: Request received
2026-01-31T12:34:56.890Z  INFO operation="llm_request" duration_ms=101 correlation_id=a1b2c3d4: Operation completed
```

**Production (JSON Format)**:
```json
{
  "timestamp": "2026-01-31T12:34:56.789Z",
  "level": "INFO",
  "target": "neoland::server",
  "span": {
    "correlation_id": "a1b2c3d4-ef56-7890-abcd-1234567890ab",
    "method": "POST",
    "uri": "/v1/chat/completions"
  },
  "message": "Request received"
}
```

---

## Alternatives Considered

### 1. Continue with Basic Logging
**Rejected**: Insufficient for production debugging and log aggregation.

### 2. Use OpenTelemetry Exclusively
**Rejected**: Adds complexity; tracing-subscriber is sufficient for current needs. We can add OTel later if needed.

### 3. Use `slog` or `log4rs`
**Rejected**: `tracing` is more modern, has better async support, and integrates seamlessly with the ecosystem.

### 4. Custom Logging Framework
**Rejected**: Reinventing the wheel; `tracing` + `tracing-subscriber` covers all requirements.

---

## Consequences

### Positive
✅ **Production-Ready**: JSON logs compatible with log aggregation systems (Loki, ELK, Datadog)
✅ **Debuggability**: Correlation IDs enable request tracing across services
✅ **Performance Visibility**: Automatic duration tracking for operations
✅ **Flexibility**: Environment-based configuration (dev vs prod)
✅ **Zero Runtime Cost**: Pretty format has no JSON serialization overhead in dev
✅ **Compliance**: Structured logs for audit trail requirements

### Negative
⚠️ **Log Volume**: JSON logs are larger than plain text (acceptable trade-off)
⚠️ **Learning Curve**: Developers need to understand structured logging practices
⚠️ **Performance**: JSON serialization adds ~5-10μs per log entry (negligible)

### Neutral
🔧 **Configuration Required**: Must set `LOG_FORMAT=json` in production
🔧 **Breaking Change**: None (backward compatible with RUST_LOG)

---

## Compliance Mapping

| Requirement | Implementation | Status |
|-------------|----------------|--------|
| **SOC 2 (CC7.2)** | Audit trail via structured logs | ✅ |
| **GDPR (Art. 30)** | Request tracing with correlation IDs | ✅ |
| **OWASP ASVS 7.1** | Security event logging | ✅ |
| **ISO 27001 (A.12.4)** | Log management and monitoring | ✅ |

---

## Testing

### Unit Tests (8 tests)
```bash
$ cargo test logging::tests
test logging::tests::test_log_config_defaults ... ok
test logging::tests::test_production_config ... ok
test logging::tests::test_development_config ... ok
test logging::tests::test_ci_config ... ok
test logging::tests::test_correlation_id_generation ... ok
test logging::tests::test_correlation_id_from_string ... ok
test logging::tests::test_performance_logger_basic ... ok
test logging::tests::test_performance_logger_with_correlation ... ok
```

### Integration Testing
```bash
# Development (pretty logs)
cargo run --bin neoland -- server

# Production (JSON logs)
LOG_FORMAT=json cargo run --bin neoland -- server

# CI/CD (compact logs)
LOG_FORMAT=compact RUST_LOG=warn cargo run --bin neoland -- server
```

---

## Usage Examples

### Development (Default)
```bash
# Pretty logs with DEBUG level
cargo run --bin neoland -- server --log-level debug
```

### Production
```bash
# JSON logs with INFO level
LOG_FORMAT=json cargo run --bin neoland -- server
```

### Kubernetes Deployment
```yaml
env:
  - name: LOG_FORMAT
    value: "json"
  - name: RUST_LOG
    value: "info,tower_http=warn,hyper=warn"
```

### Log Aggregation (Loki Query)
```logql
{job="neoland"} | json | correlation_id="a1b2c3d4-ef56-7890-abcd-1234567890ab"
```

### Performance Tracking
```rust
use crate::logging::{PerformanceLogger, CorrelationId};

let correlation_id = CorrelationId::new();
let perf = PerformanceLogger::start_with_correlation(
    "database_query",
    correlation_id
);

// ... perform operation ...

perf.finish(); // Logs: operation="database_query" duration_ms=42
```

---

## Migration Guide

### For Developers
1. **No Changes Required**: Existing code continues to work
2. **To Add Correlation Tracking**:
   ```rust
   use crate::logging::PerformanceLogger;

   let perf = PerformanceLogger::start("operation_name");
   // ... do work ...
   perf.finish();
   ```

### For Operations
1. **Development**: No changes (pretty logs by default)
2. **Production**: Set `LOG_FORMAT=json` environment variable
3. **Log Aggregation**: Configure Loki/ELK to parse JSON logs

---

## Future Enhancements

1. **OpenTelemetry Integration** (Phase 5)
   - Distributed tracing across microservices
   - Integration with Jaeger/Zipkin

2. **Log Sampling** (if volume becomes issue)
   - Sample DEBUG logs in production
   - Always log WARN/ERROR

3. **Dynamic Log Level** (Phase 4.3)
   - Runtime log level adjustment via `/admin/log-level` endpoint
   - Temporary DEBUG for troubleshooting

4. **Log Encryption** (Phase 6)
   - Encrypt sensitive log data at rest
   - Required for PCI-DSS compliance

---

## References

- [tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [tracing-subscriber JSON Format](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Json.html)
- [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
- [Correlation IDs in Microservices](https://www.rapid7.com/blog/post/2016/12/23/the-value-of-correlation-ids/)

---

**Approved By**: Architecture Team
**Implementation**: Phase 4.2
**Status**: ✅ Implemented
