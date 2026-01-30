
use tonic::{transport::Server as GrpcServer, Request, Response, Status};
use llamachat::llama_service_server::{LlamaService, LlamaServiceServer};
use llamachat::{ChatRequest, ChatResponse, AddDocumentRequest, AddDocumentResponse, SearchRequest, SearchResponse};
use tokio_stream::wrappers::ReceiverStream;
use std::sync::{Arc, Mutex};
use crate::engine::{LocalEngine, GenerationConfig};
use crate::nlp::VectorStore;
// Integration dependencies (currently used for demonstration)
// use securellm_core;
use intelagent_core::TaskId;

// Axum imports for REST
use axum::{
    routing::{post, get},
    extract::{State, Json},
    response::sse::{Event, Sse},
    Router,
    middleware::{self, Next},
    http::{Request as HttpRequest, StatusCode, HeaderMap},
    body::Body,
};
use serde::{Deserialize, Serialize};
use futures::stream::Stream;
use std::convert::Infallible;
use crate::auth::AuthManager;
use crate::audit::{AuditLogger, AuditEvent, AuditAction, FailedAuthTracker, ConsoleAlertHandler};

pub mod llamachat {
    tonic::include_proto!("llamachat");
}

// REST Data Models (OpenAI compatible subset)
#[derive(Deserialize)]
struct RestChatRequest {
    messages: Vec<RestMessage>,
    #[allow(dead_code)] // Will be used for non-streaming responses
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

// Shared State for gRPC and REST
pub struct AppState {
    engine: Arc<Mutex<Option<LocalEngine>>>,
    vector_store: Arc<Mutex<VectorStore>>,
    auth_manager: Arc<AuthManager>,
    audit_logger: Arc<AuditLogger>,
    failed_auth_tracker: Arc<FailedAuthTracker>,
}

// gRPC Service Implementation
pub struct MyLlamaService {
    state: Arc<AppState>,
}

#[tonic::async_trait]
impl LlamaService for MyLlamaService {
    type ChatStreamStream = ReceiverStream<Result<ChatResponse, Status>>;

    async fn chat_stream(&self, request: Request<ChatRequest>) -> Result<Response<Self::ChatStreamStream>, Status> {
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
                    let _ = tx.blocking_send(Err(Status::internal(format!("Mutex poisoned: {}", e))));
                    return;
                }
            };
            if engine_guard.is_none() {
                match LocalEngine::new() {
                    Ok(e) => *engine_guard = Some(e),
                    Err(err) => {
                        let _ = tx.blocking_send(Err(Status::internal(err.to_string())));
                        return;
                    }
                }
            }
            if let Some(engine) = engine_guard.as_mut() {
                let vs_guard = match state.vector_store.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        let _ = tx.blocking_send(Err(Status::internal(format!("VectorStore mutex poisoned: {}", e))));
                        return;
                    }
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
                    }
                    Err(e) => {
                        let _ = tx.blocking_send(Err(Status::internal(e.to_string())));
                    }
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn add_document(&self, request: Request<AddDocumentRequest>) -> Result<Response<AddDocumentResponse>, Status> {
        let req = request.into_inner();
        let mut vs = self.state.vector_store.lock()
            .map_err(|e| Status::internal(format!("VectorStore mutex poisoned: {}", e)))?;
        match vs.add_document(&req.content, &req.metadata) {
            Ok(_) => {
                 // SecureLLM Audit
                 println!("[SECURELLM] AUDIT: Document added. Metadata: {}", req.metadata);
                 Ok(Response::new(AddDocumentResponse { id: "ok".to_string(), success: true }))
            },
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn search(&self, request: Request<SearchRequest>) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let vs = self.state.vector_store.lock()
            .map_err(|e| Status::internal(format!("VectorStore mutex poisoned: {}", e)))?;
        match vs.search(&req.query, req.top_k as usize) {
            Ok(results) => {
                let grpc_results = results.into_iter().map(|(doc, score)| {
                    llamachat::Document { id: doc.id, content: doc.content, score }
                }).collect();
                Ok(Response::new(SearchResponse { results: grpc_results }))
            },
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

// REST Handlers
async fn rest_chat_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RestChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Get last user message
    let prompt = req.messages.last().map(|m| m.content.clone()).unwrap_or_default();

    // Use default config for REST (could be extended later)
    let config = GenerationConfig::default();

    tokio::task::spawn_blocking(move || {
        let mut engine_guard = match state.engine.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if engine_guard.is_none() {
            if let Ok(e) = LocalEngine::new() {
                *engine_guard = Some(e);
            }
        }
        if let Some(engine) = engine_guard.as_mut() {
            let Ok(vs_guard) = state.vector_store.lock() else {
                return;
            };
            let _ = engine.generate_stream(&prompt, Some(&vs_guard), &config, |token| {
                let response = RestChatResponse {
                    choices: vec![RestChoice {
                        delta: Some(RestDelta { content: token }),
                        message: None,
                    }]
                };
                let json = serde_json::json!(response).to_string();
                let _ = tx.blocking_send(Ok(Event::default().data(json)));
            });
        }
    });

    Sse::new(tokio_stream::wrappers::ReceiverStream::new(rx))
        .keep_alive(axum::response::sse::KeepAlive::default())
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
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

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
        }
        Err(e) => {
            // Log failed authentication attempt (Phase 1.3)
            let user_id = "unknown".to_string();
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
        }
    }
}

pub async fn run_server(grpc_port: u16, rest_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use tracing::info;
    
    let grpc_addr: std::net::SocketAddr = format!("[::]:{}", grpc_port).parse()?;
    let rest_addr: std::net::SocketAddr = format!("0.0.0.0:{}", rest_port).parse()?;
    
    info!("🚀 Inicializando Neoland Server...");
    info!("📡 gRPC endpoint: {}", grpc_addr);
    info!("🌐 REST endpoint: {}", rest_addr);
    
    let vector_store = Arc::new(Mutex::new(VectorStore::new()?));
    {
        let mut vs = vector_store.lock()
            .map_err(|e| format!("VectorStore mutex poisoned during init: {}", e))?;
        let _ = vs.add_document("System: Use [[CMD:move_ws:N]] for workspace movement.", "sys");
    }

    // Phantom Integration Check
    let phantom_task_id = TaskId::new();
    info!("[PHANTOM] Integrated. Ready for Task: {}", phantom_task_id);

    // Initialize authentication manager
    let auth_manager = Arc::new(AuthManager::new());
    info!("🔐 Authentication manager initialized");
    info!("⚠️  Using development API keys (change in production)");

    // Initialize audit logger (Phase 1.3)
    let audit_log_path = std::env::var("AUDIT_LOG_PATH")
        .unwrap_or_else(|_| "/var/log/neoland/audit.log".to_string());
    let audit_logger = Arc::new(AuditLogger::new(&audit_log_path)?);

    // Set up alert handler
    audit_logger.set_alert_handler(Box::new(ConsoleAlertHandler)).await;
    info!("📝 Audit logger initialized: {}", audit_log_path);

    // Initialize failed auth tracker (5 failures in 1 minute)
    let failed_auth_tracker = Arc::new(FailedAuthTracker::new(5, 1));
    info!("🛡️  Failed authentication tracker enabled");

    let shared_state = Arc::new(AppState {
        engine: Arc::new(Mutex::new(None)),
        vector_store,
        auth_manager,
        audit_logger,
        failed_auth_tracker,
    });

    // 1. Start gRPC Server
    let grpc_state = shared_state.clone();
    let grpc_future = GrpcServer::builder()
        .add_service(LlamaServiceServer::new(MyLlamaService { state: grpc_state }))
        .serve(grpc_addr);

    // 2. Start REST Server (Axum)
    // Protected routes require authentication
    let protected_routes = Router::new()
        .route("/v1/chat/completions", post(rest_chat_handler))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            auth_middleware,
        ));

    // Public routes (no authentication)
    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }));

    // Combine all routes
    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .with_state(shared_state);
        
    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    info!("✅ REST API rodando em http://{}", rest_addr);
    info!("✅ gRPC Service rodando em {}", grpc_addr);

    // Run both servers concurrently
    tokio::select! {
        res = grpc_future => info!("gRPC Server exit: {:?}", res),
        res = axum::serve(listener, app) => info!("REST Server exit: {:?}", res),
    }

    Ok(())
}