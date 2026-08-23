//! Montagem do Router axum: rotas protegidas, públicas, CORS, estáticos.
//!
//! O bloco de `.layer()` foi movido literalmente do antigo `run_server_with` —
//! a ordem dos middlewares é contrato (aplicados em ordem reversa:
//! correlation → rate_limit → validation → auth → handler).

use tracing::info;

use super::*;

pub(crate) fn build_router(shared_state: Arc<AppState>, web_dist_dir: &str) -> Router {
    // Protected routes with full security stack (Phase 1.4 + 4.2)
    // Middleware order (applied in reverse): correlation → rate_limit → validation
    // → auth → handler
    let protected_routes = Router::new()
        .route("/v1/chat/completions", post(rest_chat_handler))
        .route("/v1/agents/task", post(submit_agent_task))
        .route("/v1/agents/sessions", get(list_agent_sessions))
        .route("/v1/agents/session/:id", get(get_agent_session))
        .route("/v1/agents/session/:id/steer", post(steer_agent_task))
        .route("/v1/agents/session/:id/messages", get(get_session_messages))
        .route("/v1/agents/session/:id/name", patch(set_session_name))
        .route("/v1/agents/session/:id/breakpoint/resolve", post(resolve_agent_breakpoint))
        .route("/v1/agents/tools", get(list_agent_tools))
        .route("/v1/agents/tools/call", post(call_agent_tool))
        .route("/v1/agents/events", get(agent_events_handler))
        .route("/v1/agents/events/:session", get(agent_events_session_handler))
        .layer(axum::middleware::from_fn_with_state(shared_state.clone(), auth_middleware))
        .layer(axum::middleware::from_fn_with_state(
            shared_state.clone(),
            validation_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            shared_state.clone(),
            rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn(correlation_middleware)); // Phase 4.2: Correlation IDs

    // Public routes (no authentication)
    let public_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/live", get(liveness_handler))
        .route("/metrics", get(metrics_handler))
        .route("/v1/agents/health", get(agent_health_handler))
        .route("/auth/login/google", get(login_google_handler))
        .route("/auth/login/github", get(login_github_handler))
        .route("/auth/callback/google", get(callback_google_handler))
        .route("/auth/callback/github", get(callback_github_handler))
        .route("/auth/refresh", post(refresh_token_handler))
        .route("/auth/me", get(auth_me_handler))
        .route("/auth/logout", post(logout_handler));

    // CORS layer — allows the Leptos WASM Web Console (Trunk dev server) and
    // the Neoland server itself to access the REST API from different origins.
    let cors_layer = CorsLayer::new()
        .allow_origin([
            "http://localhost:8080".parse().unwrap(),
            "http://localhost:3001".parse().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-correlation-id"),
        ])
        .allow_credentials(true);

    // Static file serving for the Web Console (Leptos WASM).
    // In production, `trunk build --release` outputs to web/dist/.
    // The path is configurable via --web-dist CLI flag or NEOLAND_WEB_DIST_DIR.
    // The Neoland server serves it directly — single binary, single port.
    let web_dist = std::path::Path::new(web_dist_dir);
    let static_service = if web_dist.exists() {
        info!("🌐 Serving Web Console from {}", web_dist.display());
        tower_http::services::ServeDir::new(web_dist)
            .fallback(tower_http::services::ServeFile::new(web_dist.join("index.html")))
    } else {
        tracing::warn!(
            path = %web_dist.display(),
            "Web Console static directory not found — SPA routes will 404"
        );
        tower_http::services::ServeDir::new(web_dist)
            .fallback(tower_http::services::ServeFile::new(web_dist.join("index.html")))
    };

    // Combine all routes
    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .merge(crate::openapi::router())
        .fallback_service(static_service)
        .layer(cors_layer)
        .layer(axum::middleware::from_fn(track_connections_middleware))
        .with_state(shared_state)
}
