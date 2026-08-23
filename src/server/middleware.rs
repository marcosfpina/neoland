//! Middlewares axum (auth, rate limit, validação, correlação, conexões)
//! e extractors de RBAC/dependência (`RequireUser`, `AgentPipelineDep`).

use super::*;

/// Rate limiting middleware (Phase 1.4)
///
/// Limits requests to 100 per minute per user/IP
pub(crate) async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: HttpRequest<Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    // Identify user by API key or IP address
    let identifier = if let Some(api_key) = headers.get("X-API-Key").and_then(|v| v.to_str().ok()) {
        format!("key:{}", api_key)
    } else if let Some(ip) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
    {
        format!("ip:{}", ip.trim())
    } else {
        // Fallback to connection IP (not ideal but better than nothing)
        "unknown".to_string()
    };

    // Check rate limit
    if state.rate_limiter.check_rate_limit(&identifier).await {
        tracing::warn!(identifier = identifier, "Rate limit exceeded (>100 req/min)");

        // Phase 4.1: Record rate limit metric
        crate::metrics::utils::record_rate_limit_exceeded(&identifier);

        // Log rate limit event
        let event = AuditEvent::new(AuditAction::AuthFailure)
            .with_error("Rate limit exceeded".to_string())
            .with_metadata("identifier", serde_json::json!(identifier));
        let _ = state.audit_logger.log(event).await;

        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

/// RBAC extractor: the request's API key must carry the User role or higher.
///
/// `auth_middleware` validates the key and injects [`crate::auth::ApiKey`]
/// into request extensions; until now no handler ever read it back, so a
/// read-only key could hit every write endpoint (including shell execution
/// via /v1/agents/tools/call). Appearing in the handler signature makes the
/// requirement impossible to forget on new routes.
pub struct RequireUser(#[allow(dead_code)] pub(crate) crate::auth::ApiKey);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for RequireUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        require_role(parts, crate::auth::Role::User).map(Self)
    }
}

pub(crate) fn require_role(
    parts: &axum::http::request::Parts,
    required: crate::auth::Role,
) -> Result<crate::auth::ApiKey, ApiError> {
    let info = parts
        .extensions
        .get::<crate::auth::ApiKey>()
        .cloned()
        .ok_or_else(ApiError::unauthorized)?;
    if info.role.has_permission(&required) {
        Ok(info)
    } else {
        Err(ApiError::forbidden(&required))
    }
}

/// Dependency extractor: the agent orchestrator, or a uniform 503.
///
/// The exact "Agent pipeline not configured (DATABASE_URL required)" body
/// was previously copy-pasted in six handlers; three others answered a
/// bare 503 with no body. One definition, one behavior.
pub struct AgentPipelineDep(pub(crate) Arc<AgentOrchestrator>);

#[axum::async_trait]
impl axum::extract::FromRequestParts<Arc<AppState>> for AgentPipelineDep {
    type Rejection = ApiError;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        state
            .agent_orchestrator
            .clone()
            .map(Self)
            .ok_or_else(ApiError::pipeline_unavailable)
    }
}

/// In-flight request tracking for the ACTIVE_CONNECTIONS gauge.
/// Drop guard so the decrement survives handler panics/unwinds.
pub(crate) async fn track_connections_middleware(
    req: HttpRequest<Body>,
    next: Next,
) -> axum::response::Response {
    struct ConnGuard;
    impl Drop for ConnGuard {
        fn drop(&mut self) {
            crate::metrics::ACTIVE_CONNECTIONS.dec();
        }
    }
    crate::metrics::ACTIVE_CONNECTIONS.inc();
    let _guard = ConnGuard;
    next.run(req).await
}

/// Correlation ID middleware (Phase 4.2)
///
/// Extracts or generates correlation IDs for request tracing
pub(crate) async fn correlation_middleware(
    headers: HeaderMap,
    mut req: HttpRequest<Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    use crate::logging::CorrelationId;

    // Extract correlation ID from header or generate new one
    let correlation_id = headers
        .get("X-Correlation-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| CorrelationId::from_string(s.to_string()))
        .unwrap_or_else(CorrelationId::new);

    // Store correlation ID in request extensions for handlers
    req.extensions_mut().insert(correlation_id.clone());

    // Create tracing span with correlation ID
    let span = tracing::info_span!(
        "http_request",
        correlation_id = %correlation_id,
        method = %req.method(),
        uri = %req.uri(),
    );

    // Run the rest of the middleware stack within this span
    let response = next.run(req).instrument(span).await;

    Ok(response)
}

/// Input validation middleware (Phase 1.4)
///
/// Validates and sanitizes request body before processing
pub(crate) async fn validation_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: HttpRequest<Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    // Get content length
    if let Some(content_length) = headers.get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                // Validate request body size (1MB max)
                if length > 1024 * 1024 {
                    tracing::warn!(size = length, "Request body too large (>1MB)");

                    let event = AuditEvent::new(AuditAction::AuthFailure)
                        .with_error(format!("Request body too large: {} bytes", length));
                    let _ = state.audit_logger.log(event).await;

                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    // Note: Detailed validation (prompt size, message count) is done in the handler
    // because we need to parse the JSON body first, which Axum does automatically

    Ok(next.run(req).await)
}

/// Authentication middleware for REST API (Phase 1.3: with audit logging)
///
/// Validates the X-API-Key header and attaches user info to request extensions
pub(crate) async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut req: HttpRequest<Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extract API key from X-API-Key header
    let api_key = headers.get("X-API-Key").and_then(|v| v.to_str().ok());

    // Extract IP address for audit logging
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok());

    // Check if API key was provided
    if api_key.is_none() {
        // Log missing API key attempt
        let event = AuditEvent::new(AuditAction::AuthFailure)
            .with_error("Missing X-API-Key header".to_string());
        let _ = state.audit_logger.log(event).await;

        return Err(StatusCode::UNAUTHORIZED);
    }

    // Validate API key
    match state.auth_manager.validate_api_key(api_key.unwrap()) {
        Ok(api_key_info) => {
            // Phase 4.1: Record successful auth metric
            let ip_str = ip_address.as_ref().map(|ip: &std::net::IpAddr| ip.to_string());
            crate::metrics::utils::record_auth_attempt(
                "api_key",
                true,
                Some(&api_key_info.user_id),
                ip_str.as_deref(),
            );

            // Log successful authentication (Phase 1.3)
            let event = AuditEvent::new(AuditAction::AuthSuccess)
                .with_user(api_key_info.user_id.clone(), format!("{:?}", api_key_info.role))
                .with_resource(req.uri().path().to_string());

            let event = if let Some(ip) = ip_address {
                event.with_ip(ip)
            } else {
                event
            };

            let _ = state.audit_logger.log(event).await;

            // Attach user info to request extensions for use in handlers
            req.extensions_mut().insert(api_key_info);

            // Continue to next middleware/handler
            Ok(next.run(req).await)
        },
        Err(e) => {
            // Log failed authentication attempt (Phase 1.3)
            let user_id = "unknown".to_string();

            // Phase 4.1: Record failed auth metric
            let ip_str = ip_address.as_ref().map(|ip: &std::net::IpAddr| ip.to_string());
            crate::metrics::utils::record_auth_attempt(
                "api_key",
                false,
                Some(&user_id),
                ip_str.as_deref(),
            );

            let event = AuditEvent::new(AuditAction::AuthFailure)
                .with_user(user_id.clone(), "none".to_string())
                .with_resource(req.uri().path().to_string())
                .with_error(e.to_string());

            let event = if let Some(ip) = ip_address {
                event.with_ip(ip)
            } else {
                event
            };

            let _ = state.audit_logger.log(event).await;

            // Track failed authentication for brute force detection
            if state.failed_auth_tracker.track_failure(user_id.clone()).await {
                tracing::error!(
                    user = user_id,
                    "🚨 Brute force attack detected: >5 failed auth attempts in 1 minute"
                );
            }

            Err(StatusCode::UNAUTHORIZED)
        },
    }
}
