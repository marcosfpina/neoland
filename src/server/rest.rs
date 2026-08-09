//! Handlers REST: chat (SSE e não-stream), health, readiness, métricas.

use super::*;

// REST Data Models (OpenAI compatible subset)
#[derive(Deserialize)]
pub(crate) struct RestChatRequest {
    messages: Vec<RestMessage>,
    /// `true` (default) → SSE stream; `false` → single JSON response
    stream: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct RestMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
pub(crate) struct RestChatResponse {
    choices: Vec<RestChoice>,
}

#[derive(Serialize)]
pub(crate) struct RestChoice {
    delta: Option<RestDelta>,
    message: Option<RestMessage>,
}

#[derive(Serialize)]
pub(crate) struct RestDelta {
    content: String,
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
    let response =
        health::perform_health_check(Some(uptime), &state.auth_manager, &state.audit_logger).await;
    Json(response)
}

/// Readiness check (Kubernetes readiness probe)
pub(crate) async fn readiness_handler() -> (StatusCode, Json<health::ReadinessResponse>) {
    let response = health::perform_readiness_check().await;
    let status_code = if response.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status_code, Json(response))
}

/// Liveness check (simple heartbeat)
pub(crate) async fn liveness_handler() -> Json<health::LivenessResponse> {
    let response = health::perform_liveness_check().await;
    Json(response)
}

pub(crate) async fn rest_chat_handler(
    State(state): State<Arc<AppState>>,
    _auth: RequireUser,
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
