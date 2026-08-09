//! Bootstrap do servidor: audit logger, `run_server`/`run_server_with`.

use super::*;

pub(crate) fn user_audit_log_path_from_env(
    xdg_state_home: Option<&str>,
    home: Option<&str>,
) -> String {
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

pub(crate) fn user_audit_log_path() -> String {
    let xdg_state_home = std::env::var("XDG_STATE_HOME").ok();
    let home = std::env::var("HOME").ok();
    user_audit_log_path_from_env(xdg_state_home.as_deref(), home.as_deref())
}

pub(crate) fn fallback_audit_log_paths() -> Vec<String> {
    let user_path = user_audit_log_path();
    if user_path == "/tmp/neoland/audit.log" {
        vec![user_path]
    } else {
        vec![user_path, "/tmp/neoland/audit.log".to_string()]
    }
}

pub(crate) fn init_audit_logger() -> anyhow::Result<(AuditLogger, String)> {
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

pub async fn run_server(grpc_port: u16, rest_port: u16, web_dist_dir: &str) -> anyhow::Result<()> {
    let grpc_addr: std::net::SocketAddr = format!("[::]:{}", grpc_port).parse()?;
    let rest_addr: std::net::SocketAddr = format!("0.0.0.0:{}", rest_port).parse()?;
    let grpc_listener = tokio::net::TcpListener::bind(grpc_addr).await?;
    let rest_listener = tokio::net::TcpListener::bind(rest_addr).await?;
    run_server_with(grpc_listener, rest_listener, web_dist_dir, None).await
}

// ─── Composition root ────────────────────────────────────────────────────────
// A ordem de chamada em run_server_with é o contrato de inicialização: cada
// init_* abaixo preserva literalmente o comportamento do antigo bloco inline.

fn init_vector_store() -> anyhow::Result<Arc<Mutex<VectorStore>>> {
    let vector_store = Arc::new(Mutex::new(VectorStore::new()?));
    {
        let mut vs = vector_store
            .lock()
            .map_err(|e| anyhow::anyhow!("VectorStore mutex poisoned during init: {}", e))?;
        let _ = vs.add_document("System: Use [[CMD:move_ws:N]] for workspace movement.", "sys");
    }
    Ok(vector_store)
}

/// Secrets manager (Phase 1.2 / 4.7: Vault integration) + auth manager.
async fn init_auth() -> anyhow::Result<Arc<AuthManager>> {
    use tracing::info;

    let secrets_manager = Arc::new(SecretsManager::new().await?);
    info!(
        vault_available = secrets_manager.is_vault_available(),
        "🔑 Secrets manager initialized"
    );

    let auth_manager = Arc::new(AuthManager::new_with_secrets(&secrets_manager).await);
    info!("🔐 Authentication manager initialized");
    Ok(auth_manager)
}

/// Audit logger (Phase 1.3) com alert handler de console.
async fn init_audit() -> anyhow::Result<Arc<AuditLogger>> {
    use tracing::info;

    let (audit_logger, audit_log_path) = init_audit_logger()?;
    let audit_logger = Arc::new(audit_logger);
    audit_logger.set_alert_handler(Box::new(ConsoleAlertHandler)).await;
    info!("📝 Audit logger initialized: {}", audit_log_path);
    Ok(audit_logger)
}

/// PersistentVectorStore (Phase 4.9) quando DATABASE_URL está setada.
/// Falls back gracefully to in-memory-only mode if DB is unavailable.
async fn init_persistent_store() -> Option<Arc<PersistentVectorStore>> {
    use tracing::info;

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
}

/// One shared database pool backs authentication and the agent orchestrator.
/// Without it both features fail closed while health-only/local inference
/// remains available. Errors only on a failed opt-in migration.
async fn init_db_pool() -> anyhow::Result<Option<sqlx::PgPool>> {
    use tracing::info;

    let db_url = std::env::var("NEOLAND_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok();
    match db_url {
        Some(url) => {
            use sqlx::postgres::PgPoolOptions;
            match PgPoolOptions::new().max_connections(5).connect(&url).await {
                Ok(pool) => {
                    // Opt-in: databases migrated by hand have no _sqlx_migrations
                    // table, so running unconditionally could re-apply 001.
                    let auto_migrate = std::env::var("NEOLAND_AUTO_MIGRATE")
                        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                        .unwrap_or(false);
                    if auto_migrate {
                        sqlx::migrate!("./migrations")
                            .run(&pool)
                            .await
                            .map_err(|e| anyhow::anyhow!("database migration failed: {e}"))?;
                        info!("Database migrations applied (NEOLAND_AUTO_MIGRATE)");
                    }
                    Ok(Some(pool))
                },
                Err(error) => {
                    tracing::warn!(%error, "Database connection failed — auth sessions and orchestrator disabled");
                    Ok(None)
                },
            }
        },
        None => {
            info!("DATABASE_URL not set — auth sessions and agent orchestrator disabled");
            Ok(None)
        },
    }
}

/// AgentOrchestrator (multi-agent DSPy pipeline) + Native MCP + integrações
/// opcionais (NATS, MCP externo, Matrix). None sem pool ou em falha de init.
async fn try_init_agent_pipeline(
    db_pool: Option<sqlx::PgPool>,
    event_tx: tokio::sync::broadcast::Sender<AgentEvent>,
) -> Option<Arc<AgentOrchestrator>> {
    use tracing::info;

    match db_pool {
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
}

#[allow(clippy::too_many_arguments)]
fn build_state(
    vector_store: Arc<Mutex<VectorStore>>,
    persistent_store: Option<Arc<PersistentVectorStore>>,
    auth_manager: Arc<AuthManager>,
    audit_logger: Arc<AuditLogger>,
    failed_auth_tracker: Arc<FailedAuthTracker>,
    rate_limiter: Arc<RateLimiter>,
    agent_orchestrator: Option<Arc<AgentOrchestrator>>,
    event_tx: tokio::sync::broadcast::Sender<AgentEvent>,
    db_pool: Option<sqlx::PgPool>,
) -> Arc<AppState> {
    Arc::new(AppState {
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
    })
}

/// gRPC server future (with gRPC-web support for browser clients).
/// tonic-web enables browsers to call gRPC endpoints via HTTP/1.1 + CORS.
/// For production, restrict origins via a reverse proxy (nginx/Caddy).
fn build_grpc_future(
    grpc_listener: tokio::net::TcpListener,
    state: Arc<AppState>,
    tls_config: &Option<crate::tls::TlsConfig>,
    shutdown_tx: &tokio::sync::broadcast::Sender<()>,
) -> anyhow::Result<impl std::future::Future<Output = Result<(), tonic::transport::Error>>> {
    let mut grpc_builder = GrpcServer::builder();
    if let Some(tls) = tls_config {
        let mut grpc_tls = tonic::transport::ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(&tls.certs_pem, &tls.key_der));
        if let Some(ref ca) = tls.ca_der {
            grpc_tls = grpc_tls.client_ca_root(tonic::transport::Certificate::from_pem(ca));
        }
        grpc_builder = grpc_builder.tls_config(grpc_tls)?;
    }
    let mut grpc_shutdown = shutdown_tx.subscribe();
    Ok(grpc_builder
        .accept_http1(true)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(LlamaServiceServer::new(MyLlamaService { state }))
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(grpc_listener),
            async move {
                let _ = grpc_shutdown.recv().await;
            },
        ))
}

/// REST server future com TLS opcional (axum-server) ou plain (axum::serve).
fn build_rest_future(
    rest_listener: tokio::net::TcpListener,
    app: Router,
    tls_config: &Option<crate::tls::TlsConfig>,
    shutdown_tx: &tokio::sync::broadcast::Sender<()>,
    rest_addr: std::net::SocketAddr,
) -> anyhow::Result<std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<()>> + Send>>>
{
    use tracing::info;

    if let Some(tls) = tls_config {
        info!("🔐 TLS enabled (mTLS: {}) — certificates loaded", tls.is_mtls());
        let mut rustls_config = tls.mtls_server_config()?;
        rustls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        let rustls_config =
            axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(rustls_config));
        info!("✅ REST API rodando em https://{}", rest_addr);
        let tls_handle = axum_server::Handle::new();
        {
            let handle = tls_handle.clone();
            let mut rx = shutdown_tx.subscribe();
            tokio::spawn(async move {
                let _ = rx.recv().await;
                handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
            });
        }
        Ok(Box::pin(
            axum_server::from_tcp_rustls(rest_listener.into_std()?, rustls_config)
                .handle(tls_handle)
                .serve(app.into_make_service()),
        ))
    } else {
        info!("🔓 TLS not configured — use a reverse proxy (nginx/Caddy) for production");
        info!("✅ REST API rodando em http://{}", rest_addr);
        let mut rest_shutdown = shutdown_tx.subscribe();
        Ok(Box::pin(async move {
            axum::serve(rest_listener, app)
                .with_graceful_shutdown(async move {
                    let _ = rest_shutdown.recv().await;
                })
                .await
        }))
    }
}

/// Inner server entry point over pre-bound listeners.
///
/// Tests bind port 0, read `local_addr()` and pass an external shutdown
/// sender to stop the server cleanly; production (`run_server`) passes
/// `None` and relies on the SIGTERM/Ctrl+C signal task.
pub async fn run_server_with(
    grpc_listener: tokio::net::TcpListener,
    rest_listener: tokio::net::TcpListener,
    web_dist_dir: &str,
    external_shutdown: Option<tokio::sync::broadcast::Sender<()>>,
) -> anyhow::Result<()> {
    use tracing::info;

    let grpc_addr = grpc_listener.local_addr()?;
    let rest_addr = rest_listener.local_addr()?;

    info!("🚀 Inicializando Neoland Server...");
    info!("📡 gRPC endpoint: {}", grpc_addr);
    info!("🌐 REST endpoint: {}", rest_addr);

    let vector_store = init_vector_store()?;
    let auth_manager = init_auth().await?;
    let audit_logger = init_audit().await?;

    // Failed auth tracker (5 failures in 1 minute)
    let failed_auth_tracker = Arc::new(FailedAuthTracker::new(5, 1));
    info!("🛡️  Failed authentication tracker enabled");

    // Rate limiter (Phase 1.4: 100 requests per minute)
    let rate_limiter = Arc::new(RateLimiter::new(100, 60));
    info!("⏱️  Rate limiter enabled (100 req/min)");

    // SSE event bus — broadcast channel for agent pipeline events.
    let (event_tx, _) = tokio::sync::broadcast::channel::<AgentEvent>(128);

    let persistent_store = init_persistent_store().await;
    let db_pool = init_db_pool().await?;
    let agent_orchestrator = try_init_agent_pipeline(db_pool.clone(), event_tx.clone()).await;

    let shared_state = build_state(
        vector_store,
        persistent_store,
        auth_manager,
        audit_logger,
        failed_auth_tracker,
        rate_limiter,
        agent_orchestrator,
        event_tx,
        db_pool,
    );

    // TLS/mTLS opcional — certs via NEOLAND_TLS_* (scripts/gen-certs.sh)
    // aws-lc-rs e ring coexistem no dep tree (tonic/tokio-rustls): o provider
    // process-level precisa ser escolhido explicitamente antes de qualquer TLS.
    rustls::crypto::ring::default_provider().install_default().ok();
    let tls_config = crate::tls::TlsConfig::from_env()?;

    // Graceful shutdown: SIGTERM/Ctrl+C drains both servers instead of cutting
    // in-flight requests. If either server exits on its own, the other is shut
    // down too (preserves the previous fail-fast semantics of tokio::select!).
    let shutdown_tx =
        external_shutdown.unwrap_or_else(|| tokio::sync::broadcast::channel::<()>(1).0);
    {
        let tx = shutdown_tx.clone();
        tokio::spawn(async move {
            crate::health::ShutdownHandler::new().wait_for_shutdown_signal().await;
            let _ = tx.send(());
        });
    }

    let grpc_future =
        build_grpc_future(grpc_listener, shared_state.clone(), &tls_config, &shutdown_tx)?;

    // Keep MEMORY_USAGE_BYTES fresh (registered gauges stay 0 otherwise)
    crate::metrics::spawn_resource_collector();

    let app = routes::build_router(shared_state, web_dist_dir);
    let rest_future = build_rest_future(rest_listener, app, &tls_config, &shutdown_tx, rest_addr)?;
    info!("✅ gRPC Service rodando em {}", grpc_addr);

    // Run both servers concurrently. Each one triggers the shutdown signal on
    // exit, so a crashed server drains the healthy one instead of leaving it
    // running (fail-fast) — and a signal drains both before join! returns.
    let grpc_exit_tx = shutdown_tx.clone();
    let rest_exit_tx = shutdown_tx.clone();
    let (grpc_res, rest_res) = tokio::join!(
        async move {
            let res = grpc_future.await;
            let _ = grpc_exit_tx.send(());
            res
        },
        async move {
            let res = rest_future.await;
            let _ = rest_exit_tx.send(());
            res
        }
    );
    info!("gRPC Server exit: {:?}", grpc_res);
    info!("REST Server exit: {:?}", rest_res);
    info!("Shutdown complete");

    Ok(())
}
