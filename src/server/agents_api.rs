//! Rotas do pipeline de agentes: task, steer, sessões, tools, breakpoints, SSE.

use super::*;

// ─── Agent Pipeline Handlers ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct AgentTaskBody {
    task: String,
    session_id: Option<uuid::Uuid>,
}

/// POST /v1/agents/task — submit a task to the multi-agent DSPy pipeline.
/// Requires User+ role (enforced by the RequireUser extractor).
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
    _auth: RequireUser,
    AgentPipelineDep(orchestrator): AgentPipelineDep,
    Json(body): Json<AgentTaskBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
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
            serde_json::to_value(&result).map(Json).map_err(|e| {
                tracing::error!(error = %e, session_id = %session_id, "Failed to serialize agent task result");
                ApiError::internal("Failed to serialize result")
            })
        },
        Err(e) => {
            tracing::error!(error = %e, session_id = %session_id, "Agent task execution failed");
            Err(ApiError::internal(e.to_string()))
        },
    }
}

#[derive(Deserialize)]
pub(crate) struct AgentSteerBody {
    message: String,
}

#[derive(Deserialize)]
pub(crate) struct SetNameBody {
    name: String,
}

#[derive(Deserialize)]
pub(crate) struct ListSessionsQuery {
    limit: Option<usize>,
}

/// POST /v1/agents/session/:id/steer — send human intervention message to
/// active task
pub(crate) async fn steer_agent_task(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    _auth: RequireUser,
    AgentPipelineDep(orchestrator): AgentPipelineDep,
    Json(body): Json<AgentSteerBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    match orchestrator.steer_task(id, body.message).await {
        Ok(_) => Ok(Json(serde_json::json!({"status": "Steering message delivered"}))),
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to deliver steering message");
            Err(ApiError::not_found(e.to_string()))
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
    State(_state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    AgentPipelineDep(orchestrator): AgentPipelineDep,
) -> Result<Json<serde_json::Value>, ApiError> {
    match orchestrator.get_session(id).await {
        Ok(session) => serde_json::to_value(&session).map(Json).map_err(|e| {
            tracing::error!(error = %e, session_id = %id, "Failed to serialize session");
            ApiError::internal("Failed to serialize session")
        }),
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to retrieve session");
            Err(ApiError::not_found("Session not found"))
        },
    }
}

/// GET /v1/agents/session/:id/messages — retrieve messages for a session.
/// Requires ReadOnly+ auth (enforced by auth_middleware).
pub(crate) async fn get_session_messages(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    AgentPipelineDep(orchestrator): AgentPipelineDep,
) -> Result<Json<serde_json::Value>, ApiError> {
    match orchestrator.get_session_messages(id).await {
        Ok(messages) => serde_json::to_value(&messages).map(Json).map_err(|e| {
            tracing::error!(error = %e, session_id = %id, "Failed to serialize session messages");
            ApiError::internal("Failed to serialize messages")
        }),
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to retrieve session messages");
            Err(ApiError::not_found("Session messages not found"))
        },
    }
}

/// PATCH /v1/agents/session/:id/name — update session display name.
/// Requires User+ role (enforced by the RequireUser extractor).
pub(crate) async fn set_session_name(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    _auth: RequireUser,
    AgentPipelineDep(orchestrator): AgentPipelineDep,
    Json(body): Json<SetNameBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    match orchestrator.set_session_name(id, &body.name).await {
        Ok(()) => Ok(Json(serde_json::json!({"status": "ok"}))),
        Err(e) => {
            tracing::warn!(error = %e, session_id = %id, "Failed to update session name");
            Err(ApiError::internal(e.to_string()))
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
    State(_state): State<Arc<AppState>>,
    AgentPipelineDep(orchestrator): AgentPipelineDep,
    Query(query): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(24).clamp(1, 100) as i64;

    match orchestrator.list_sessions(limit).await {
        Ok(sessions) => serde_json::to_value(&sessions).map(Json).map_err(|e| {
            tracing::error!(error = %e, "Failed to serialize session list");
            ApiError::internal("Failed to serialize sessions")
        }),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to list recent sessions");
            Err(ApiError::internal("Failed to list sessions"))
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
    State(_state): State<Arc<AppState>>,
    AgentPipelineDep(orch): AgentPipelineDep,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mcp = orch
        .native_mcp
        .as_ref()
        .ok_or(ApiError::Bare(StatusCode::SERVICE_UNAVAILABLE))?;

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
    State(_state): State<Arc<AppState>>,
    _auth: RequireUser,
    AgentPipelineDep(orch): AgentPipelineDep,
    Json(payload): Json<ToolCallPayload>,
) -> Result<Json<crate::mcp::types::CallToolResult>, ApiError> {
    let mcp = orch
        .native_mcp
        .as_ref()
        .ok_or(ApiError::Bare(StatusCode::SERVICE_UNAVAILABLE))?;

    match mcp.call_tool(payload.session_id, &payload.name, payload.arguments).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ApiError::Bare(StatusCode::INTERNAL_SERVER_ERROR)),
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
    State(_state): State<Arc<AppState>>,
    Path(session_id): Path<uuid::Uuid>,
    _auth: RequireUser,
    AgentPipelineDep(orch): AgentPipelineDep,
    Json(payload): Json<BreakpointResolvePayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let resolution = match payload.resolution.as_str() {
        "approve" => BreakpointResolution::Approve,
        "steer" => BreakpointResolution::Steer(payload.instruction.unwrap_or_default()),
        _ => BreakpointResolution::Reject,
    };

    match orch.resolve_breakpoint(session_id, resolution).await {
        Ok(true) => Ok(Json(serde_json::json!({ "status": "resolved" }))),
        Ok(false) => Err(ApiError::Bare(StatusCode::NOT_FOUND)),
        Err(_) => Err(ApiError::Bare(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

/// GET /v1/agents/events — global SSE stream of all agent pipeline events.
/// Requires ReadOnly+ auth (enforced by auth_middleware).
pub(crate) async fn agent_events_handler(
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
pub(crate) async fn agent_events_session_handler(
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
