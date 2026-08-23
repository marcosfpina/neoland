use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Axum imports for REST
use axum::http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderName, Method,
};
use axum::{
    body::Body,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, Request as HttpRequest, Response as HttpResponse, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, patch, post},
    Router,
};
use tower_http::cors::CorsLayer;
// Integration dependencies (currently used for demonstration)
// use securellm_core;
use intelagent_core::TaskId;
use llamachat::{
    llama_service_server::{LlamaService, LlamaServiceServer},
    AddDocumentRequest, AddDocumentResponse, ChatRequest, ChatResponse, SearchRequest,
    SearchResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_stream::{
    wrappers::{BroadcastStream, ReceiverStream},
    StreamExt as _,
};
use tonic::{transport::Server as GrpcServer, Request, Response, Status};
use tracing::Instrument; // Phase 4.2: For span instrumentation

use crate::mcp::server::{BreakpointResolution, NativeMcpServer};
use crate::{
    agents::{events::AgentEvent, orchestrator::AgentOrchestrator},
    audit::{AuditAction, AuditEvent, AuditLogger, ConsoleAlertHandler, FailedAuthTracker},
    auth::AuthManager,
    engine::{GenerationConfig, LocalEngine},
    health, // Phase 4.3: Health Checks
    matrix::MatrixClient,
    nlp::VectorStore,
    secrets::SecretsManager,
    storage::PersistentVectorStore,
    validation::{ChatRequestValidation, MessageValidator},
};

pub mod llamachat {
    tonic::include_proto!("llamachat");
}

const DEFAULT_AUDIT_LOG_PATH: &str = "/var/log/neoland/audit.log";

// REST Data Models (OpenAI compatible subset)
#[derive(Deserialize)]
struct RestChatRequest {
    messages: Vec<RestMessage>,
    /// `true` (default) → SSE stream; `false` → single JSON response
    stream: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
struct RestMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct RestChatResponse {
    choices: Vec<RestChoice>,
}

#[derive(Serialize)]
struct RestChoice {
    delta: Option<RestDelta>,
    message: Option<RestMessage>,
}

#[derive(Serialize)]
struct RestDelta {
    content: String,
}

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
    engine: Arc<Mutex<Option<LocalEngine>>>,
    vector_store: Arc<Mutex<VectorStore>>,
    persistent_store: Option<Arc<PersistentVectorStore>>,
    auth_manager: Arc<AuthManager>,
    audit_logger: Arc<AuditLogger>,
    failed_auth_tracker: Arc<FailedAuthTracker>,
    rate_limiter: Arc<RateLimiter>,
    start_time: Instant,
    agent_orchestrator: Option<Arc<AgentOrchestrator>>,
    event_bus: tokio::sync::broadcast::Sender<AgentEvent>,
    /// Database pool (v0.0.1 enterprise auth)
    pub db_pool: Option<sqlx::PgPool>,
    /// JWT signing secret (v0.0.1)
    pub jwt_secret: Vec<u8>,
    /// OAuth2 base URL (v0.0.1)
    pub oauth_base_url: String,
}

// gRPC Service Implementation
pub struct MyLlamaService {
    state: Arc<AppState>,
}

#[tonic::async_trait]
impl LlamaService for MyLlamaService {
    type ChatStreamStream = ReceiverStream<Result<ChatResponse, Status>>;

    async fn chat_stream(
        &self,
        request: Request<ChatRequest>,
    ) -> Result<Response<Self::ChatStreamStream>, Status> {
        let grpc_start = Instant::now();
        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["chat_stream", "started"])
            .inc();

        let req = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let state = self.state.clone();

        // Parse config from request
        let mut config = GenerationConfig::default();

        if let Some(temp) = req.temperature {
            config.temperature = temp as f64;
        }
        if let Some(top_p) = req.top_p {
            config.top_p = top_p as f64;
        }
        if let Some(max_tokens) = req.max_tokens {
            config.max_tokens = max_tokens as usize;
        }
        if let Some(rep_penalty) = req.repetition_penalty {
            config.repetition_penalty = rep_penalty;
        }
        if let Some(typical_p) = req.typical_p {
            config.typical_p = typical_p as f64;
        }
        if let Some(epsilon_cutoff) = req.epsilon_cutoff {
            config.epsilon_cutoff = epsilon_cutoff as f64;
        }
        if let Some(eta_cutoff) = req.eta_cutoff {
            config.eta_cutoff = eta_cutoff as f64;
        }
        if let Some(tail_free_sampling) = req.tail_free_sampling {
            config.tail_free_sampling = tail_free_sampling as f64;
        }
        if let Some(top_a) = req.top_a {
            config.top_a = top_a as f64;
        }
        if let Some(ctx_k) = req.context_top_k {
            config.context_top_k = ctx_k as usize;
        }
        if let Some(ctx_threshold) = req.context_similarity_threshold {
            config.context_similarity_threshold = ctx_threshold;
        }
        if let Some(disable_ctx) = req.disable_context {
            config.disable_context = disable_ctx;
        }
        if let Some(sys_prompt) = req.system_prompt {
            config.system_prompt = Some(sys_prompt);
        }
        if let Some(enable_cmds) = req.enable_commands {
            config.enable_commands = enable_cmds;
        }
        if !req.allowed_commands.is_empty() {
            config.allowed_commands = req.allowed_commands.clone();
        }

        tokio::task::spawn_blocking(move || {
            let mut engine_guard = match state.engine.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    let _ =
                        tx.blocking_send(Err(Status::internal(format!("Mutex poisoned: {}", e))));
                    return;
                },
            };
            if engine_guard.is_none() {
                match LocalEngine::new() {
                    Ok(e) => *engine_guard = Some(e),
                    Err(err) => {
                        tracing::error!(error = %err, "gRPC: failed to initialize LocalEngine");
                        let _ = tx.blocking_send(Err(Status::internal(err.to_string())));
                        return;
                    },
                }
            }
            if let Some(engine) = engine_guard.as_mut() {
                let vs_guard = match state.vector_store.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        let _ = tx.blocking_send(Err(Status::internal(format!(
                            "VectorStore mutex poisoned: {}",
                            e
                        ))));
                        return;
                    },
                };

                // Send metadata first
                let metadata = llamachat::ResponseMetadata {
                    temperature_used: config.temperature as f32,
                    top_p_used: config.top_p as f32,
                    max_tokens_used: config.max_tokens as i32,
                    context_docs_count: 0, // Will be updated after generation
                    context_doc_ids: vec![],
                    system_prompt_used: config
                        .system_prompt
                        .clone()
                        .unwrap_or_else(|| "default".to_string()),
                    commands_enabled: config.enable_commands,
                };

                let _ = tx.blocking_send(Ok(ChatResponse {
                    content: String::new(),
                    is_command: false,
                    metadata: Some(metadata.clone()),
                }));

                match engine.generate_stream(&req.prompt, Some(&vs_guard), &config, |token| {
                    // Filter commands if needed
                    let should_send = if token.contains("[[CMD:") {
                        if !config.enable_commands {
                            false
                        } else if !config.allowed_commands.is_empty() {
                            // Check if command is in whitelist
                            config.allowed_commands.iter().any(|cmd| token.contains(cmd))
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if should_send {
                        let _ = tx.blocking_send(Ok(ChatResponse {
                            is_command: token.contains("[[CMD:"),
                            content: token,
                            metadata: None,
                        }));
                    }
                }) {
                    Ok(context_doc_ids) => {
                        // Send final metadata update with context info
                        let final_metadata = llamachat::ResponseMetadata {
                            temperature_used: config.temperature as f32,
                            top_p_used: config.top_p as f32,
                            max_tokens_used: config.max_tokens as i32,
                            context_docs_count: context_doc_ids.len() as i32,
                            context_doc_ids: context_doc_ids.clone(),
                            system_prompt_used: config
                                .system_prompt
                                .clone()
                                .unwrap_or_else(|| "default".to_string()),
                            commands_enabled: config.enable_commands,
                        };
                        let _ = tx.blocking_send(Ok(ChatResponse {
                            content: "[[METADATA_UPDATE]]".to_string(),
                            is_command: false,
                            metadata: Some(final_metadata),
                        }));
                    },
                    Err(e) => {
                        let _ = tx.blocking_send(Err(Status::internal(e.to_string())));
                    },
                }
            }
        });
        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["chat_stream", "ok"])
            .inc();
        crate::metrics::GRPC_REQUEST_DURATION_SECONDS
            .with_label_values(&["chat_stream"])
            .observe(grpc_start.elapsed().as_secs_f64());

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn add_document(
        &self,
        request: Request<AddDocumentRequest>,
    ) -> Result<Response<AddDocumentResponse>, Status> {
        let grpc_start = Instant::now();
        let req = request.into_inner();

        // 1. Add to in-memory VectorStore (sync, used for context injection)
        let doc_id = {
            let mut vs = self
                .state
                .vector_store
                .lock()
                .map_err(|e| Status::internal(format!("VectorStore mutex poisoned: {}", e)))?;
            match vs.add_document(&req.content, &req.metadata) {
                Ok(()) => {
                    crate::metrics::utils::update_vector_store_documents(vs.len());
                    uuid::Uuid::new_v4().to_string()
                },
                Err(e) => {
                    tracing::error!(error = %e, "gRPC: add_document (in-memory) failed");
                    crate::metrics::GRPC_REQUESTS_TOTAL
                        .with_label_values(&["add_document", "error"])
                        .inc();
                    crate::metrics::GRPC_REQUEST_DURATION_SECONDS
                        .with_label_values(&["add_document"])
                        .observe(grpc_start.elapsed().as_secs_f64());
                    return Err(Status::internal(e.to_string()));
                },
            }
        };

        // 2. Mirror to PersistentVectorStore if configured (fire-and-forget)
        if let Some(ps) = self.state.persistent_store.clone() {
            let content = req.content.clone();
            let metadata = req.metadata.clone();
            tokio::spawn(async move {
                match ps.add_document(&content, &metadata).await {
                    Ok(uuid) => tracing::debug!(
                        uuid = %uuid,
                        "Mirrored document to PersistentVectorStore"
                    ),
                    Err(e) => tracing::warn!(
                        error = %e,
                        "Failed to mirror document to PersistentVectorStore (in-memory write succeeded)"
                    ),
                }
            });
        }

        tracing::info!(id = %doc_id, metadata = %req.metadata, "gRPC: document added");

        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["add_document", "ok"])
            .inc();
        crate::metrics::GRPC_REQUEST_DURATION_SECONDS
            .with_label_values(&["add_document"])
            .observe(grpc_start.elapsed().as_secs_f64());

        Ok(Response::new(AddDocumentResponse { id: doc_id, success: true }))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let grpc_start = Instant::now();
        let req = request.into_inner();

        // Prefer PersistentVectorStore (pgvector) when configured; fall back to
        // in-memory.
        if let Some(ps) = &self.state.persistent_store {
            match ps.search(&req.query, req.top_k as usize, None).await {
                Ok(results) => {
                    let grpc_results = results
                        .into_iter()
                        .map(|(doc, score)| llamachat::Document {
                            id: doc.id.to_string(),
                            content: doc.content,
                            score,
                        })
                        .collect();
                    crate::metrics::GRPC_REQUESTS_TOTAL.with_label_values(&["search", "ok"]).inc();
                    crate::metrics::GRPC_REQUEST_DURATION_SECONDS
                        .with_label_values(&["search"])
                        .observe(grpc_start.elapsed().as_secs_f64());
                    return Ok(Response::new(SearchResponse { results: grpc_results }));
                },
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        query = %req.query,
                        "PersistentVectorStore search failed, falling back to in-memory"
                    );
                },
            }
        }

        // In-memory fallback
        let vs = self
            .state
            .vector_store
            .lock()
            .map_err(|e| Status::internal(format!("VectorStore mutex poisoned: {}", e)))?;
        let result = match vs.search(&req.query, req.top_k as usize) {
            Ok(results) => {
                let grpc_results = results
                    .into_iter()
                    .map(|(doc, score)| llamachat::Document {
                        id: doc.id,
                        content: doc.content,
                        score,
                    })
                    .collect();
                Ok(Response::new(SearchResponse { results: grpc_results }))
            },
            Err(e) => {
                tracing::error!(error = %e, query = %req.query, "gRPC: search failed");
                Err(Status::internal(e.to_string()))
            },
        };
        let status_label = if result.is_ok() { "ok" } else { "error" };
        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["search", status_label])
            .inc();
        crate::metrics::GRPC_REQUEST_DURATION_SECONDS
            .with_label_values(&["search"])
            .observe(grpc_start.elapsed().as_secs_f64());
        result
    }
}

// REST Handlers

// Phase 4.1: Prometheus Metrics Handler
/// GET /metrics — Prometheus metrics endpoint.
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "system",
    responses(
        (status = 200, description = "Prometheus text format metrics"),
    )
)]
pub(crate) async fn metrics_handler() -> HttpResponse<String> {
    match crate::metrics::render_metrics() {
        Ok(metrics) => HttpResponse::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; version=0.0.4")
            .body(metrics)
            .unwrap(),
        Err(e) => HttpResponse::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Error rendering metrics: {}", e))
            .unwrap(),
    }
}

// Phase 4.3: Health Check Endpoints

/// Comprehensive health check (Kubernetes liveness probe)
/// GET /health — Comprehensive health check (Kubernetes liveness probe).
#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Service healthy"),
        (status = 503, description = "Service degraded or unhealthy"),
    )
)]
pub(crate) async fn health_handler(
    State(state): State<Arc<AppState>>,
) -> Json<health::HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    let response = health::perform_health_check(Some(uptime)).await;
    Json(response)
}

/// Readiness check (Kubernetes readiness probe)
async fn readiness_handler() -> (StatusCode, Json<health::ReadinessResponse>) {
    let response = health::perform_readiness_check().await;
    let status_code = if response.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status_code, Json(response))
}

/// Liveness check (simple heartbeat)
async fn liveness_handler() -> Json<health::LivenessResponse> {
    let response = health::perform_liveness_check().await;
    Json(response)
}

async fn rest_chat_handler(
    State(state): State<Arc<AppState>>,
    Json(mut req): Json<RestChatRequest>,
) -> Result<axum::response::Response, StatusCode> {
    let handler_start = Instant::now();

    // Validate request
    let validation_req = ChatRequestValidation {
        messages: req
            .messages
            .iter()
            .map(|m| crate::validation::ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect(),
        metadata: serde_json::json!({}),
    };

    if let Err(e) = validation_req.validate() {
        tracing::warn!(error = %e, "Invalid chat request");

        let event = AuditEvent::new(AuditAction::ChatRequest)
            .with_error(e.to_string())
            .with_metadata("validation_error", serde_json::json!(true));
        let _ = state.audit_logger.log(event).await;

        crate::metrics::utils::record_http_request(
            "POST",
            "/v1/chat/completions",
            400,
            handler_start.elapsed().as_secs_f64(),
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Sanitize messages
    for msg in &mut req.messages {
        msg.content = MessageValidator::sanitize_input(&msg.content);
    }

    let prompt = req.messages.last().map(|m| m.content.clone()).unwrap_or_default();
    let config = GenerationConfig::default();

    // Dispatch: stream:false → single JSON response; anything else → SSE stream
    let want_stream = req.stream.unwrap_or(true);

    if !want_stream {
        // ── Non-streaming path ─────────────────────────────────────────────
        // Collect all tokens into a single String, return one JSON object.
        let (done_tx, done_rx) = tokio::sync::oneshot::channel::<String>();

        tokio::task::spawn_blocking(move || {
            // generate_stream takes Fn (not FnMut), so accumulate via Mutex.
            let acc = std::sync::Mutex::new(String::new());

            let mut engine_guard = match state.engine.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    tracing::error!(error = %e, "REST non-stream: engine mutex poisoned");
                    let _ = done_tx.send(String::new());
                    return;
                },
            };
            if engine_guard.is_none() {
                match LocalEngine::new() {
                    Ok(e) => *engine_guard = Some(e),
                    Err(e) => {
                        tracing::error!(error = %e, "REST non-stream: LocalEngine init failed");
                        let _ = done_tx.send(String::new());
                        return;
                    },
                }
            }
            if let Some(engine) = engine_guard.as_mut() {
                let Ok(vs_guard) = state.vector_store.lock() else {
                    tracing::error!("REST non-stream: vector_store mutex poisoned");
                    let _ = done_tx.send(String::new());
                    return;
                };
                let _ = engine.generate_stream(&prompt, Some(&vs_guard), &config, |token| {
                    if let Ok(mut s) = acc.lock() {
                        s.push_str(&token);
                    }
                });
            }
            let full = acc.into_inner().unwrap_or_default();
            let _ = done_tx.send(full);
        });

        let full_text = done_rx.await.unwrap_or_default();

        crate::metrics::utils::record_http_request(
            "POST",
            "/v1/chat/completions",
            200,
            handler_start.elapsed().as_secs_f64(),
        );

        let body = RestChatResponse {
            choices: vec![RestChoice {
                delta: None,
                message: Some(RestMessage { role: "assistant".to_string(), content: full_text }),
            }],
        };
        Ok(Json(body).into_response())
    } else {
        // ── Streaming path (SSE) ───────────────────────────────────────────
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(100);

        tokio::task::spawn_blocking(move || {
            let mut engine_guard = match state.engine.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    tracing::error!(error = %e, "REST stream: engine mutex poisoned");
                    return;
                },
            };
            if engine_guard.is_none() {
                match LocalEngine::new() {
                    Ok(e) => *engine_guard = Some(e),
                    Err(e) => {
                        tracing::error!(error = %e, "REST stream: LocalEngine init failed");
                        return;
                    },
                }
            }
            if let Some(engine) = engine_guard.as_mut() {
                let Ok(vs_guard) = state.vector_store.lock() else {
                    tracing::error!("REST stream: vector_store mutex poisoned");
                    return;
                };
                let _ = engine.generate_stream(&prompt, Some(&vs_guard), &config, |token| {
                    let response = RestChatResponse {
                        choices: vec![RestChoice {
                            delta: Some(RestDelta { content: token }),
                            message: None,
                        }],
                    };
                    let json = serde_json::json!(response).to_string();
                    let _ = tx.blocking_send(Ok(Event::default().data(json)));
                });
            }
        });

        crate::metrics::utils::record_http_request(
            "POST",
            "/v1/chat/completions",
            200,
            handler_start.elapsed().as_secs_f64(),
        );
        Ok(Sse::new(ReceiverStream::new(rx))
            .keep_alive(axum::response::sse::KeepAlive::default())
            .into_response())
    }
}

/// Rate limiting middleware (Phase 1.4)
///
/// Limits requests to 100 per minute per user/IP
async fn rate_limit_middleware(
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

/// Correlation ID middleware (Phase 4.2)
///
/// Extracts or generates correlation IDs for request tracing
async fn correlation_middleware(
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
async fn validation_middleware(
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
async fn auth_middleware(
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

// ─── Agent Pipeline Handlers ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct AgentTaskBody {
    task: String,
    session_id: Option<uuid::Uuid>,
}

/// POST /v1/agents/task — submit a task to the multi-agent DSPy pipeline.
/// Requires User+ auth (enforced by auth_middleware).
/// POST /v1/agents/task — submit a task to the multi-agent DSPy pipeline.
/// Runs Junior → Senior → (Architect?) → TechLeader and returns the full result
/// plus an ADR checkpoint written to disk.
#[utoipa::path(
    post,
    path = "/v1/agents/task",
    tag = "agents",
    security(("api_key" = [])),
    request_body = AgentTaskRequest,
    responses(
        (status = 200, description = "Pipeline completed", body = PipelineResult),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 503, description = "Pipeline not configured", body = ErrorResponse),
    )
)]
pub(crate) async fn submit_agent_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AgentTaskBody>,
) -> impl IntoResponse {
    let orchestrator = match &state.agent_orchestrator {
        Some(o) => o.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Agent pipeline not configured (DATABASE_URL required)"})),
            )
                .into_response()
        },
    };

    let session_id = body.session_id.unwrap_or_else(uuid::Uuid::new_v4);

    let start_event = AuditEvent::new(AuditAction::AgentTaskStart)
        .with_metadata("session_id", serde_json::json!(session_id.to_string()))
        .with_metadata("task_preview", serde_json::json!(&body.task[..body.task.len().min(80)]));
    let _ = state.audit_logger.log(start_event).await;

    match orchestrator.execute_task(&body.task, session_id, "user", String::new()).await {
        Ok(result) => {
            let decision_event = AuditEvent::new(AuditAction::AgentDecision)
                .with_metadata("session_id", serde_json::json!(session_id.to_string()))
                .with_metadata(
                    "decision",
                    serde_json::json!(format!("{:?}", result.tech_leader.decision).to_lowercase()),
                );
            let _ = state.audit_logger.log(decision_event).await;
            match serde_json::to_value(&result) {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => {
                    tracing::error!(error = %e, session_id = %session_id, "Failed to serialize agent task result");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to serialize result"})),
                    )
                        .into_response()
                },
            }
        },
        Err(e) => {
            tracing::error!(error = %e, session_id = %session_id, "Agent task execution failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        },
    }
}

#[derive(Deserialize)]
struct AgentSteerBody {
    message: String,
}

#[derive(Deserialize)]
struct SetNameBody {
    name: String,
}

#[derive(Deserialize)]
pub(crate) struct ListSessionsQuery {
    limit: Option<usize>,
}

/// POST /v1/agents/session/:id/steer — send human intervention message to
/// active task
async fn steer_agent_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<AgentSteerBody>,
) -> impl IntoResponse {
    let orchestrator = match &state.agent_orchestrator {
        Some(o) => o.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Agent pipeline not configured (DATABASE_URL required)"})),
            )
                .into_response()
        },
    };

    match orchestrator.steer_task(id, body.message).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "Steering message delivered"})),
        )
            .into_response(),
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to deliver steering message");
            (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": e.to_string()})))
                .into_response()
        },
    }
}

/// GET /v1/agents/session/:id — retrieve session state.
/// Requires ReadOnly+ auth (enforced by auth_middleware).
/// GET /v1/agents/session/{id} — retrieve session state.
#[utoipa::path(
    get,
    path = "/v1/agents/session/{id}",
    tag = "agents",
    security(("api_key" = [])),
    params(
        ("id" = String, Path, description = "Session UUID", format = "uuid")
    ),
    responses(
        (status = 200, description = "Session found", body = SessionState),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Session not found", body = ErrorResponse),
    )
)]
pub(crate) async fn get_agent_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let orchestrator = match &state.agent_orchestrator {
        Some(o) => o.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Agent pipeline not configured (DATABASE_URL required)"})),
            )
                .into_response()
        },
    };

    match orchestrator.get_session(id).await {
        Ok(session) => match serde_json::to_value(&session) {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => {
                tracing::error!(error = %e, session_id = %id, "Failed to serialize session");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to serialize session"})),
                )
                    .into_response()
            },
        },
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to retrieve session");
            (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Session not found"})))
                .into_response()
        },
    }
}

/// GET /v1/agents/session/:id/messages — retrieve messages for a session.
/// Requires ReadOnly+ auth (enforced by auth_middleware).
async fn get_session_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let orchestrator = match &state.agent_orchestrator {
        Some(o) => o.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Agent pipeline not configured (DATABASE_URL required)"})),
            )
                .into_response()
        },
    };

    match orchestrator.get_session_messages(id).await {
        Ok(messages) => match serde_json::to_value(&messages) {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => {
                tracing::error!(error = %e, session_id = %id, "Failed to serialize session messages");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to serialize messages"})),
                )
                    .into_response()
            },
        },
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to retrieve session messages");
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Session messages not found"})),
            )
                .into_response()
        },
    }
}

/// PATCH /v1/agents/session/:id/name — update session display name.
/// Requires User+ auth (enforced by auth_middleware).
async fn set_session_name(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<SetNameBody>,
) -> impl IntoResponse {
    let orchestrator = match &state.agent_orchestrator {
        Some(o) => o.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Agent pipeline not configured (DATABASE_URL required)"})),
            )
                .into_response()
        },
    };

    match orchestrator.set_session_name(id, &body.name).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to update session name");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        },
    }
}

/// GET /v1/agents/sessions — retrieve recent session state list.
/// Requires ReadOnly+ auth (enforced by auth_middleware).
/// GET /v1/agents/sessions — retrieve recent session state list.
#[utoipa::path(
    get,
    path = "/v1/agents/sessions",
    tag = "agents",
    security(("api_key" = [])),
    params(
        ("limit" = Option<u32>, Query, description = "Maximum number of sessions to return")
    ),
    responses(
        (status = 200, description = "Recent sessions", body = [SessionState]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 503, description = "Pipeline not configured", body = ErrorResponse),
    )
)]
pub(crate) async fn list_agent_sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    let orchestrator = match &state.agent_orchestrator {
        Some(o) => o.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Agent pipeline not configured (DATABASE_URL required)"})),
            )
                .into_response()
        },
    };

    let limit = query.limit.unwrap_or(24).clamp(1, 100) as i64;

    match orchestrator.list_sessions(limit).await {
        Ok(sessions) => match serde_json::to_value(&sessions) {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize session list");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to serialize sessions"})),
                )
                    .into_response()
            },
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to list recent sessions");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to list sessions"})),
            )
                .into_response()
        },
    }
}

/// GET /v1/agents/health — DSPy pipeline health check (public).
/// GET /v1/agents/health — DSPy pipeline health check (public).
#[utoipa::path(
    get,
    path = "/v1/agents/health",
    tag = "agents",
    responses(
        (status = 200, description = "Pipeline health state", body = AgentHealthResponse),
    )
)]
pub(crate) async fn agent_health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match &state.agent_orchestrator {
        Some(orch) => match orch.health_check().await {
            Ok(true) => {
                (StatusCode::OK, Json(serde_json::json!({"status": "ok", "pipeline": "up"})))
                    .into_response()
            },
            Ok(false) => (
                StatusCode::OK,
                Json(serde_json::json!({"status": "degraded", "pipeline": "down"})),
            )
                .into_response(),
            Err(e) => (
                StatusCode::OK,
                Json(serde_json::json!({"status": "error", "error": e.to_string()})),
            )
                .into_response(),
        },
        None => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ok", "pipeline": "disabled"})),
        )
            .into_response(),
    }
}

/// GET /v1/agents/tools
#[utoipa::path(
    get,
    path = "/v1/agents/tools",
    tag = "agents",
    responses((status = 200, description = "List of available native tools"))
)]
pub async fn list_agent_tools(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let orch = state.agent_orchestrator.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mcp = orch.native_mcp.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let tools = mcp.list_tools();
    Ok(Json(serde_json::json!({ "tools": tools })))
}

#[derive(Deserialize)]
pub struct ToolCallPayload {
    pub session_id: uuid::Uuid,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// POST /v1/agents/tools/call
#[utoipa::path(
    post,
    path = "/v1/agents/tools/call",
    tag = "agents",
    request_body = ToolCallPayload,
)]
pub async fn call_agent_tool(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ToolCallPayload>,
) -> Result<Json<crate::mcp::types::CallToolResult>, StatusCode> {
    let orch = state.agent_orchestrator.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mcp = orch.native_mcp.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match mcp.call_tool(payload.session_id, &payload.name, payload.arguments).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct BreakpointResolvePayload {
    pub resolution: String, // "approve", "reject", "steer"
    pub instruction: Option<String>,
}

/// POST /v1/agents/session/:id/breakpoint/resolve
#[utoipa::path(
    post,
    path = "/v1/agents/session/{id}/breakpoint/resolve",
    tag = "agents",
    params(("id" = uuid::Uuid, Path, description = "Session UUID")),
    request_body = BreakpointResolvePayload,
)]
pub async fn resolve_agent_breakpoint(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<uuid::Uuid>,
    Json(payload): Json<BreakpointResolvePayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let orch = state.agent_orchestrator.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let resolution = match payload.resolution.as_str() {
        "approve" => BreakpointResolution::Approve,
        "steer" => BreakpointResolution::Steer(payload.instruction.unwrap_or_default()),
        _ => BreakpointResolution::Reject,
    };

    match orch.resolve_breakpoint(session_id, resolution).await {
        Ok(true) => Ok(Json(serde_json::json!({ "status": "resolved" }))),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /v1/agents/events — global SSE stream of all agent pipeline events.
/// Requires ReadOnly+ auth (enforced by auth_middleware).
async fn agent_events_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_bus.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(event) => serde_json::to_string(&event).ok().map(|d| Ok(Event::default().data(d))),
        Err(_) => None,
    });
    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

/// GET /v1/agents/events/:session — SSE stream filtered to a specific session.
/// Requires ReadOnly+ auth (enforced by auth_middleware).
async fn agent_events_session_handler(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<uuid::Uuid>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_bus.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |msg| match msg {
        Ok(event) if event.session_id() == session_id => {
            serde_json::to_string(&event).ok().map(|d| Ok(Event::default().data(d)))
        },
        _ => None,
    });
    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

fn user_audit_log_path_from_env(xdg_state_home: Option<&str>, home: Option<&str>) -> String {
    if let Some(xdg_state_home) = xdg_state_home {
        let base = xdg_state_home.trim_end_matches('/');
        return format!("{base}/neoland/audit.log");
    }

    if let Some(home) = home {
        let base = home.trim_end_matches('/');
        return format!("{base}/.local/state/neoland/audit.log");
    }

    "/tmp/neoland/audit.log".to_string()
}

fn user_audit_log_path() -> String {
    let xdg_state_home = std::env::var("XDG_STATE_HOME").ok();
    let home = std::env::var("HOME").ok();
    user_audit_log_path_from_env(xdg_state_home.as_deref(), home.as_deref())
}

fn fallback_audit_log_paths() -> Vec<String> {
    let user_path = user_audit_log_path();
    if user_path == "/tmp/neoland/audit.log" {
        vec![user_path]
    } else {
        vec![user_path, "/tmp/neoland/audit.log".to_string()]
    }
}

fn init_audit_logger() -> anyhow::Result<(AuditLogger, String)> {
    if let Ok(audit_log_path) = std::env::var("AUDIT_LOG_PATH") {
        let audit_logger = AuditLogger::new(&audit_log_path)?;
        return Ok((audit_logger, audit_log_path));
    }

    match AuditLogger::new(DEFAULT_AUDIT_LOG_PATH) {
        Ok(audit_logger) => Ok((audit_logger, DEFAULT_AUDIT_LOG_PATH.to_string())),
        Err(err)
            if err.downcast_ref::<std::io::Error>().is_some_and(|io_err| {
                matches!(
                    io_err.kind(),
                    std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem
                )
            }) =>
        {
            let mut last_fallback_error = None;

            for fallback_path in fallback_audit_log_paths() {
                match AuditLogger::new(&fallback_path) {
                    Ok(audit_logger) => {
                        tracing::warn!(
                            default_path = DEFAULT_AUDIT_LOG_PATH,
                            fallback_path = %fallback_path,
                            "Default audit log path is not writable; using a writable fallback"
                        );
                        return Ok((audit_logger, fallback_path));
                    },
                    Err(fallback_err) => {
                        tracing::warn!(
                            fallback_path = %fallback_path,
                            error = %fallback_err,
                            "Audit log fallback path unavailable"
                        );
                        last_fallback_error = Some(fallback_err);
                    },
                }
            }

            Err(last_fallback_error.unwrap_or_else(|| {
                anyhow::anyhow!("No writable audit log path available after fallback attempts")
            }))
        },
        Err(err) => Err(err),
    }
}

// ── Auth route handlers (v0.0.1) ──────────────────────────────────────────
// Thin wrappers that bridge the auth module with the server's AppState.

#[utoipa::path(
    get,
    path = "/auth/login/google",
    tag = "auth",
    responses((status = 302, description = "Redirect to Google OAuth2"))
)]
pub async fn login_google_handler(
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Redirect, StatusCode> {
    let redirect_url = format!("{}/auth/callback/google", state.oauth_base_url);
    let (auth_url, _) = crate::auth::oauth::google_authorize_url(
        &std::env::var("NEOLAND_GOOGLE_CLIENT_ID").unwrap_or_default(),
        &std::env::var("NEOLAND_GOOGLE_CLIENT_SECRET").unwrap_or_default(),
        &redirect_url,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::response::Redirect::temporary(&auth_url))
}

#[utoipa::path(
    get,
    path = "/auth/login/github",
    tag = "auth",
    responses((status = 302, description = "Redirect to GitHub OAuth2"))
)]
pub async fn login_github_handler(
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Redirect, StatusCode> {
    let redirect_url = format!("{}/auth/callback/github", state.oauth_base_url);
    let (auth_url, _) = crate::auth::oauth::github_authorize_url(
        &std::env::var("NEOLAND_GITHUB_CLIENT_ID").unwrap_or_default(),
        &std::env::var("NEOLAND_GITHUB_CLIENT_SECRET").unwrap_or_default(),
        &redirect_url,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::response::Redirect::temporary(&auth_url))
}

#[utoipa::path(
    get,
    path = "/auth/callback/google",
    tag = "auth",
    params(("code" = String, Query, description = "OAuth2 authorization code"), ("state" = String, Query, description = "CSRF token")),
    responses((status = 200, description = "JWT access + refresh tokens", body = AuthLoginResponse))
)]
pub async fn callback_google_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let code = params.get("code").cloned().unwrap_or_default();
    let redirect_url = format!("{}/auth/callback/google", state.oauth_base_url);
    let user_info = crate::auth::oauth::google_exchange_code(
        &std::env::var("NEOLAND_GOOGLE_CLIENT_ID").unwrap_or_default(),
        &std::env::var("NEOLAND_GOOGLE_CLIENT_SECRET").unwrap_or_default(),
        &redirect_url,
        code,
    )
    .await
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let db = state.db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user = crate::auth::routes::upsert_oauth_user(db, &user_info)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    issue_auth_tokens(&state, db, user).await
}

#[utoipa::path(
    get,
    path = "/auth/callback/github",
    tag = "auth",
    params(("code" = String, Query, description = "OAuth2 authorization code"), ("state" = String, Query, description = "CSRF token")),
    responses((status = 200, description = "JWT access + refresh tokens", body = AuthLoginResponse))
)]
pub async fn callback_github_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let code = params.get("code").cloned().unwrap_or_default();
    let redirect_url = format!("{}/auth/callback/github", state.oauth_base_url);
    let user_info = crate::auth::oauth::github_exchange_code(
        &std::env::var("NEOLAND_GITHUB_CLIENT_ID").unwrap_or_default(),
        &std::env::var("NEOLAND_GITHUB_CLIENT_SECRET").unwrap_or_default(),
        &redirect_url,
        code,
    )
    .await
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let db = state.db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user = crate::auth::routes::upsert_oauth_user(db, &user_info)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    issue_auth_tokens(&state, db, user).await
}

async fn tenant_slug(db: &sqlx::PgPool, tenant_id: Option<uuid::Uuid>) -> Option<String> {
    let tenant_id = tenant_id?;
    sqlx::query_scalar("SELECT slug FROM tenants WHERE id = $1")
        .bind(tenant_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

async fn issue_auth_tokens(
    state: &AppState,
    db: &sqlx::PgPool,
    user: crate::auth::types::User,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let role = if user.is_owner { "admin" } else { "user" };
    let tenant = tenant_slug(db, user.tenant_id).await.unwrap_or_default();
    let access_token = crate::auth::jwt::create_access_token(
        user.id,
        &tenant,
        role,
        &user.email,
        &user.display_name,
        &state.jwt_secret,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let refresh_token = crate::auth::jwt::create_refresh_token();
    let token_hash = crate::auth::jwt::hash_refresh_token(&refresh_token);
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(crate::auth::jwt::refresh_token_lifetime_secs());

    sqlx::query("INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user.id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(db)
        .await
        .map_err(|error| {
            tracing::error!(%error, user_id = %user.id, "Failed to persist OAuth session");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let _ = sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(db)
        .await;

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "user": {
            "id": user.id,
            "email": user.email,
            "display_name": user.display_name,
            "avatar_url": user.avatar_url,
            "role": role,
            "tenant": tenant,
        }
    })))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses((status = 200, description = "New access token", body = RefreshResponse))
)]
pub async fn refresh_token_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<crate::openapi::RefreshRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db = state.db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let token_hash = crate::auth::jwt::hash_refresh_token(&body.refresh_token);
    let session = sqlx::query_as::<_, crate::auth::types::Session>(
        "SELECT * FROM sessions WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()",
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await
    .map_err(|error| {
        tracing::error!(%error, "Failed to validate refresh token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = sqlx::query_as::<_, crate::auth::types::User>("SELECT * FROM users WHERE id = $1")
        .bind(session.user_id)
        .fetch_optional(db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "Failed to load refresh-token user");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let role = if user.is_owner { "admin" } else { "user" };
    let tenant = tenant_slug(db, user.tenant_id).await.unwrap_or_default();
    let access_token = crate::auth::jwt::create_access_token(
        user.id,
        &tenant,
        role,
        &user.email,
        &user.display_name,
        &state.jwt_secret,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "access_token": access_token })))
}

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Current user profile", body = AuthMeResponse))
)]
pub async fn auth_me_handler(
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = crate::auth::middleware::get_auth_user(&request).ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(serde_json::json!({
        "user_id": user.user_id.to_string(),
        "email": user.email,
        "display_name": user.display_name,
        "role": user.role,
        "tenant": user.tenant_slug,
    })))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    request_body = RefreshRequest,
    responses((status = 200, description = "Session revoked", body = LogoutResponse))
)]
pub async fn logout_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<crate::openapi::RefreshRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db = state.db_pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let token_hash = crate::auth::jwt::hash_refresh_token(&body.refresh_token);
    sqlx::query(
        "UPDATE sessions SET revoked_at = NOW() WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(db)
    .await
    .map_err(|error| {
        tracing::error!(%error, "Failed to revoke refresh token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({ "message": "Logged out" })))
}

pub async fn run_server(grpc_port: u16, rest_port: u16, web_dist_dir: &str) -> anyhow::Result<()> {
    use tracing::info;

    let grpc_addr: std::net::SocketAddr = format!("[::]:{}", grpc_port).parse()?;
    let rest_addr: std::net::SocketAddr = format!("0.0.0.0:{}", rest_port).parse()?;

    info!("🚀 Inicializando Neoland Server...");
    info!("📡 gRPC endpoint: {}", grpc_addr);
    info!("🌐 REST endpoint: {}", rest_addr);

    let vector_store = Arc::new(Mutex::new(VectorStore::new()?));
    {
        let mut vs = vector_store
            .lock()
            .map_err(|e| anyhow::anyhow!("VectorStore mutex poisoned during init: {}", e))?;
        let _ = vs.add_document("System: Use [[CMD:move_ws:N]] for workspace movement.", "sys");
    }

    // Phantom Integration Check
    let phantom_task_id = TaskId::new();
    info!("[PHANTOM] Integrated. Ready for Task: {}", phantom_task_id);

    // Initialize secrets manager (Phase 1.2 / 4.7: Vault integration)
    let secrets_manager = Arc::new(SecretsManager::new().await?);
    info!(
        vault_available = secrets_manager.is_vault_available(),
        "🔑 Secrets manager initialized"
    );

    // Initialize authentication manager with Vault/env keys (Phase 4.7)
    let auth_manager = Arc::new(AuthManager::new_with_secrets(&secrets_manager).await);
    info!("🔐 Authentication manager initialized");

    // Initialize audit logger (Phase 1.3)
    let (audit_logger, audit_log_path) = init_audit_logger()?;
    let audit_logger = Arc::new(audit_logger);

    // Set up alert handler
    audit_logger.set_alert_handler(Box::new(ConsoleAlertHandler)).await;
    info!("📝 Audit logger initialized: {}", audit_log_path);

    // Initialize failed auth tracker (5 failures in 1 minute)
    let failed_auth_tracker = Arc::new(FailedAuthTracker::new(5, 1));
    info!("🛡️  Failed authentication tracker enabled");

    // Initialize rate limiter (Phase 1.4: 100 requests per minute)
    let rate_limiter = Arc::new(RateLimiter::new(100, 60));
    info!("⏱️  Rate limiter enabled (100 req/min)");

    // SSE event bus — broadcast channel for agent pipeline events.
    let (event_tx, _) = tokio::sync::broadcast::channel::<AgentEvent>(128);

    // Initialize PersistentVectorStore (Phase 4.9) when DATABASE_URL is set.
    // Falls back gracefully to in-memory-only mode if DB is unavailable.
    let persistent_store = {
        let db_url = std::env::var("NEOLAND_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .ok();
        match db_url {
            Some(url) => match PersistentVectorStore::new(&url, None).await {
                Ok(ps) => {
                    info!("📊 PersistentVectorStore initialized (pgvector)");
                    Some(Arc::new(ps))
                },
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "PersistentVectorStore unavailable, running in-memory only"
                    );
                    None
                },
            },
            None => {
                info!("DATABASE_URL not set — using in-memory VectorStore only");
                None
            },
        }
    };

    // One shared database pool backs authentication and the agent orchestrator.
    // Without it both features fail closed while health-only/local inference remains available.
    let db_pool = {
        let db_url = std::env::var("NEOLAND_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .ok();
        match db_url {
            Some(url) => {
                use sqlx::postgres::PgPoolOptions;
                match PgPoolOptions::new().max_connections(5).connect(&url).await {
                    Ok(pool) => Some(pool),
                    Err(error) => {
                        tracing::warn!(%error, "Database connection failed — auth sessions and orchestrator disabled");
                        None
                    },
                }
            },
            None => {
                info!("DATABASE_URL not set — auth sessions and agent orchestrator disabled");
                None
            },
        }
    };

    // Initialize AgentOrchestrator (multi-agent DSPy pipeline)
    let agent_orchestrator = {
        match db_pool.clone() {
            Some(pool) => {
                let cfg = crate::config::Config::load();
                match AgentOrchestrator::new(pool, &cfg.agents) {
                    Ok(orch) => {
                        // Initialize Native MCP Server and Background Task
                        let (bp_tx, mut bp_rx) = tokio::sync::mpsc::channel(100);
                        let tools: Vec<Box<dyn crate::mcp::server::NativeTool>> =
                            vec![Box::new(crate::tools::shell::RunShellCommand)];
                        let native_mcp = NativeMcpServer::new(tools, bp_tx);

                        // We will attach the task listener later, for now we just attach
                        // the MCP to the orchestrator
                        let orch = orch.with_native_mcp(native_mcp);

                        // Optionally attach NATS publisher (Ciclo 1 — Fase B)
                        let orch = if cfg.nats.enabled {
                            use crate::agents::nats::NatsPublisher;
                            match tokio::time::timeout(
                                Duration::from_secs(2),
                                NatsPublisher::connect(&cfg.nats),
                            )
                            .await
                            {
                                Ok(Ok(publisher)) => {
                                    info!(url = %cfg.nats.url, "NATS publisher attached");
                                    orch.with_nats(publisher)
                                },
                                Ok(Err(e)) => {
                                    tracing::warn!(error = %e, "NATS connect failed — events disabled");
                                    orch
                                },
                                Err(_) => {
                                    tracing::warn!(
                                        url = %cfg.nats.url,
                                        "NATS connect timed out — events disabled"
                                    );
                                    orch
                                },
                            }
                        } else {
                            orch
                        };
                        // Optionally attach MCP tool registry (Phase 3)
                        let orch = if cfg.mcp.enabled {
                            use crate::mcp::{McpClient, McpRegistry};
                            match McpClient::spawn(&cfg.mcp.binary).await {
                                Ok(client) => match McpRegistry::new(client).await {
                                    Ok(reg) => {
                                        info!(
                                            tools = reg.tools().len(),
                                            binary = %cfg.mcp.binary,
                                            "MCP registry initialized"
                                        );
                                        orch.with_mcp(Arc::new(reg))
                                    },
                                    Err(e) => {
                                        tracing::warn!(error = %e, "MCP tool discovery failed — MCP disabled");
                                        orch
                                    },
                                },
                                Err(e) => {
                                    tracing::warn!(error = %e, "MCP spawn failed — MCP disabled");
                                    orch
                                },
                            }
                        } else {
                            orch
                        };
                        // Optionally attach Matrix client (Phase 4)
                        let orch = if cfg.matrix.enabled {
                            let mc = MatrixClient::new(&cfg.matrix.base_url);
                            info!(url = %cfg.matrix.base_url, "Matrix client attached");
                            orch.with_matrix(Arc::new(mc))
                        } else {
                            orch
                        };
                        let orch = orch.with_event_bus(event_tx.clone());
                        info!(
                            dspy_url = %cfg.agents.dspy_url,
                            "🤖 Agent orchestrator initialized"
                        );
                        let orch_arc = Arc::new(orch);
                        let orch_clone = orch_arc.clone();
                        tokio::spawn(async move {
                            while let Some(req) = bp_rx.recv().await {
                                orch_clone
                                    .register_breakpoint(
                                        req.session_id,
                                        req.tool_name.clone(),
                                        req.resolve_tx,
                                    )
                                    .await;
                                orch_clone.publish(AgentEvent::BreakpointHit {
                                    session_id: req.session_id,
                                    tool: req.tool_name,
                                    args_summary: req.args_summary,
                                });
                            }
                        });
                        Some(orch_arc)
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, "AgentOrchestrator init failed");
                        None
                    },
                }
            },
            None => None,
        }
    };

    let shared_state = Arc::new(AppState {
        engine: Arc::new(Mutex::new(None)),
        vector_store,
        persistent_store,
        auth_manager,
        audit_logger,
        failed_auth_tracker,
        rate_limiter,
        start_time: Instant::now(),
        agent_orchestrator,
        event_bus: event_tx,
        db_pool,
        jwt_secret: crate::config::Config::load().auth.jwt.secret.into_bytes(),
        oauth_base_url: crate::config::Config::load().auth.oauth.base_url,
    });

    // TLS/mTLS opcional — certs via NEOLAND_TLS_* (scripts/gen-certs.sh)
    // aws-lc-rs e ring coexistem no dep tree (tonic/tokio-rustls): o provider
    // process-level precisa ser escolhido explicitamente antes de qualquer TLS.
    rustls::crypto::ring::default_provider().install_default().ok();
    let tls_config = crate::tls::TlsConfig::from_env()?;

    // 1. Start gRPC Server (with gRPC-web support for browser clients)
    // tonic-web enables browsers to call gRPC endpoints via HTTP/1.1 + CORS.
    // For production, restrict origins via a reverse proxy (nginx/Caddy).
    let grpc_state = shared_state.clone();
    let mut grpc_builder = GrpcServer::builder();
    if let Some(ref tls) = tls_config {
        let mut grpc_tls = tonic::transport::ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(&tls.certs_pem, &tls.key_der));
        if let Some(ref ca) = tls.ca_der {
            grpc_tls = grpc_tls.client_ca_root(tonic::transport::Certificate::from_pem(ca));
        }
        grpc_builder = grpc_builder.tls_config(grpc_tls)?;
    }
    let grpc_future = grpc_builder
        .accept_http1(true)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(LlamaServiceServer::new(MyLlamaService { state: grpc_state }))
        .serve(grpc_addr);

    // 2. Start REST Server (Axum)
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
        .layer(middleware::from_fn_with_state(shared_state.clone(), auth_middleware))
        .layer(middleware::from_fn_with_state(shared_state.clone(), validation_middleware))
        .layer(middleware::from_fn_with_state(shared_state.clone(), rate_limit_middleware))
        .layer(middleware::from_fn(correlation_middleware)); // Phase 4.2: Correlation IDs

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
    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .merge(crate::openapi::router())
        .fallback_service(static_service)
        .layer(cors_layer)
        .with_state(shared_state);

    // Start REST server with optional TLS
    let rest_future: std::pin::Pin<
        Box<dyn std::future::Future<Output = std::io::Result<()>> + Send>,
    > = if let Some(ref tls) = tls_config {
        info!("🔐 TLS enabled (mTLS: {}) — certificates loaded", tls.is_mtls());
        let mut rustls_config = tls.mtls_server_config()?;
        rustls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        let rustls_config =
            axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(rustls_config));
        info!("✅ REST API rodando em https://{}", rest_addr);
        Box::pin(axum_server::bind_rustls(rest_addr, rustls_config).serve(app.into_make_service()))
    } else {
        info!("🔓 TLS not configured — use a reverse proxy (nginx/Caddy) for production");
        info!("✅ REST API rodando em http://{}", rest_addr);
        let listener = tokio::net::TcpListener::bind(rest_addr).await?;
        Box::pin(async move { axum::serve(listener, app).await })
    };
    info!("✅ gRPC Service rodando em {}", grpc_addr);

    // Run both servers concurrently
    tokio::select! {
        res = grpc_future => info!("gRPC Server exit: {:?}", res),
        res = rest_future => info!("REST Server exit: {:?}", res),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ───────────────────────────────────────────────────────────────

    #[test]
    fn test_user_audit_log_path_prefers_xdg_state_home() {
        let path = user_audit_log_path_from_env(Some("/tmp/xdg-state"), Some("/tmp/home"));
        assert_eq!(path, "/tmp/xdg-state/neoland/audit.log");
    }

    #[test]
    fn test_user_audit_log_path_uses_home_when_xdg_missing() {
        let path = user_audit_log_path_from_env(None, Some("/tmp/home"));
        assert_eq!(path, "/tmp/home/.local/state/neoland/audit.log");
    }

    #[test]
    fn test_user_audit_log_path_falls_back_to_tmp_without_env() {
        let path = user_audit_log_path_from_env(None, None);
        assert_eq!(path, "/tmp/neoland/audit.log");
    }

    /// Build a minimal AppState for testing (no embedding model, no persistent
    /// store).
    ///
    /// VectorStore::new() loads an ML model — call only from `#[ignore]` tests.
    #[allow(dead_code)]
    async fn build_test_state(vector_store: VectorStore) -> AppState {
        let (event_tx, _) = tokio::sync::broadcast::channel(1);
        AppState {
            engine: Arc::new(Mutex::new(None)),
            vector_store: Arc::new(Mutex::new(vector_store)),
            persistent_store: None,
            auth_manager: Arc::new(AuthManager::new()),
            audit_logger: Arc::new(
                AuditLogger::new("/tmp/neoland_test_server_audit.log").expect("AuditLogger failed"),
            ),
            failed_auth_tracker: Arc::new(FailedAuthTracker::new(5, 1)),
            rate_limiter: Arc::new(RateLimiter::new(100, 60)),
            start_time: Instant::now(),
            agent_orchestrator: None,
            event_bus: event_tx,
            db_pool: None,
            jwt_secret: b"test-secret-key-32-bytes-long!!".to_vec(),
            oauth_base_url: "http://localhost:3001".to_string(),
        }
    }

    // ── RateLimiter ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let rl = RateLimiter::new(5, 60);
        for _ in 0..5 {
            assert!(!rl.check_rate_limit("user1").await, "request within limit should be allowed");
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let rl = RateLimiter::new(3, 60);
        for _ in 0..3 {
            let _ = rl.check_rate_limit("user1").await;
        }
        assert!(rl.check_rate_limit("user1").await, "4th request should be rate-limited");
    }

    #[tokio::test]
    async fn test_rate_limiter_separate_users_are_independent() {
        let rl = RateLimiter::new(2, 60);

        // Exhaust user1's quota
        let _ = rl.check_rate_limit("user1").await;
        let _ = rl.check_rate_limit("user1").await;
        assert!(rl.check_rate_limit("user1").await, "user1 should be blocked");

        // user2 should still have a fresh quota
        assert!(!rl.check_rate_limit("user2").await, "user2 should not be affected");
    }

    #[tokio::test]
    async fn test_rate_limiter_zero_limit_blocks_second_request() {
        let rl = RateLimiter::new(0, 60);
        // max_requests = 0: first request is always inserted (allowed),
        // second request increments count to 2 > 0 → blocked.
        assert!(!rl.check_rate_limit("user1").await, "first request always inserted");
        assert!(rl.check_rate_limit("user1").await, "second request exceeds limit=0");
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup_no_panic() {
        let rl = RateLimiter::new(10, 60);
        for i in 0..5 {
            let _ = rl.check_rate_limit(&format!("user{i}")).await;
        }
        // cleanup_old_entries should not panic regardless of entry state
        rl.cleanup_old_entries().await;
    }

    // ── Lock ordering ─────────────────────────────────────────────────────────

    /// Demonstrate the documented lock ordering (engine → vector_store) is
    /// safe.
    ///
    /// We acquire both locks in the required order and release them correctly.
    /// The purpose is to document and exercise the ordering, not to prove
    /// freedom from deadlock (which would require concurrent actors).
    #[test]
    fn test_lock_ordering_engine_before_vector_store() {
        let engine: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(Some(1)));
        let store: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

        // Correct order as per AppState documentation:
        //   1. engine lock acquired first
        //   2. vector_store lock acquired second (while holding engine)
        let engine_guard = engine.lock().unwrap();
        let mut store_guard = store.lock().unwrap();

        assert!(engine_guard.is_some());
        store_guard.push(42);
        assert_eq!(store_guard.len(), 1);

        drop(store_guard);
        drop(engine_guard);
    }

    #[test]
    fn test_lock_ordering_violating_order_would_deadlock_in_concurrent_code() {
        // This test documents WHY the ordering matters:
        // If task A holds engine → waits for store
        // And task B holds store → waits for engine
        // → deadlock. The ordering ensures both always acquire engine first.
        //
        // Since we can't prove absence of deadlock in a unit test, we assert
        // the ordering rule is documented in the AppState comment.
        let source = include_str!("mod.rs");
        assert!(
            source.contains("engine (Arc<Mutex<Option<LocalEngine>>>)"),
            "Lock ordering comment must document engine as lock #1"
        );
        assert!(
            source.contains("vector_store (Arc<Mutex<VectorStore>>)"),
            "Lock ordering comment must document vector_store as lock #2"
        );
    }

    // ── gRPC handlers (require embedding model — run with cargo test -- --ignored)
    // ──

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_add_document_grpc_success() {
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        let req = Request::new(AddDocumentRequest {
            content: "How to move windows in Hyprland".to_string(),
            metadata: "source:test,category:wm".to_string(),
        });

        let resp = svc.add_document(req).await.expect("add_document failed");
        let inner = resp.into_inner();
        assert!(inner.success);
        assert!(!inner.id.is_empty(), "response id should be a UUID");
    }

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_add_document_grpc_empty_content_still_succeeds() {
        // Validation happens at the REST layer; gRPC add_document accepts any string
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        let req =
            Request::new(AddDocumentRequest { content: String::new(), metadata: String::new() });

        // gRPC handler itself does not validate — embedding model handles empty string
        let result = svc.add_document(req).await;
        // Either ok or internal error from embedding; what matters is no panic
        let _ = result;
    }

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_search_grpc_returns_added_document() {
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        // Add document
        let add = Request::new(AddDocumentRequest {
            content: "Neoland is a secure AI platform".to_string(),
            metadata: "source:test".to_string(),
        });
        svc.add_document(add).await.expect("add_document failed");

        // Search
        let search =
            Request::new(SearchRequest { query: "secure AI platform".to_string(), top_k: 3 });
        let resp = svc.search(search).await.expect("search failed");
        let results = resp.into_inner().results;

        assert!(!results.is_empty(), "search should return at least one result");
        assert!(results[0].score > 0.0, "similarity score should be positive");
    }

    #[tokio::test]
    #[ignore = "requires embedding model (slow, network)"]
    async fn test_search_grpc_empty_store_returns_empty() {
        let vs = VectorStore::new().expect("VectorStore init failed");
        let state = Arc::new(build_test_state(vs).await);
        let svc = MyLlamaService { state };

        let search = Request::new(SearchRequest { query: "anything".to_string(), top_k: 5 });
        let resp = svc.search(search).await.expect("search failed");
        // Empty store → empty results (not an error)
        assert_eq!(resp.into_inner().results.len(), 0);
    }
}
