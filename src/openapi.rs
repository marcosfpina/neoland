//! OpenAPI 3.0 spec generation via utoipa.
//!
//! Routes:
//!   GET /openapi.json  — raw spec
//!   GET /swagger-ui/   — Swagger UI (HTML)
//!
//! Add new schemas to `NeolandApi::components` and handlers to the
//! `#[openapi(paths(...))]` list when you add endpoints.

use axum::Router;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

// ─── Schema definitions ───────────────────────────────────────────────────────

/// Incoming task for the multi-agent ADR pipeline.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AgentTaskRequest {
    /// Natural-language task description for the pipeline.
    pub task: String,
    /// Existing session to resume. Omit to start a new session.
    #[schema(value_type = Option<String>, format = "uuid")]
    pub session_id: Option<uuid::Uuid>,
}

/// Junior agent output — creative hypothesis + self-assessed confidence.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct JuniorOutput {
    pub hypothesis: String,
    /// Confidence score [0.0, 1.0].
    #[schema(minimum = 0.0, maximum = 1.0)]
    pub confidence: f64,
    #[schema(value_type = String, example = "low")]
    pub risk_level: String,
    pub unknowns: Vec<String>,
    pub innovation_vectors: Vec<String>,
}

/// Senior agent output — sceptical refinement + escalation decision.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SeniorOutput {
    pub valid_parts: Vec<String>,
    pub rejected_parts: Vec<String>,
    pub risk_assessment: String,
    pub escalate_to_architect: bool,
    pub refined_hypothesis: String,
}

/// Architect agent output — structural soundness review (optional tier).
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ArchitectOutput {
    pub structural_soundness: bool,
    #[schema(minimum = 0.0, maximum = 1.0)]
    pub composability_score: f64,
    pub long_term_concerns: Vec<String>,
    pub recommended_structure: String,
    pub blockers: Vec<String>,
}

/// Tech-Leader final decision + ADR checkpoint.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TechLeaderOutput {
    #[schema(example = "approve")]
    pub decision: String,
    pub rationale: String,
    pub action_items: Vec<String>,
    pub adr_title: String,
    pub session_summary: String,
}

/// Full pipeline result returned by POST /v1/agents/task.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PipelineResult {
    #[schema(value_type = String, format = "uuid")]
    pub task_id: uuid::Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub session_id: uuid::Uuid,
    #[schema(value_type = String, format = "date-time")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub junior: JuniorOutput,
    pub senior: SeniorOutput,
    pub architect: Option<ArchitectOutput>,
    pub tech_leader: TechLeaderOutput,
    pub checkpoint_path: String,
}

/// Session state from PostgreSQL.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SessionState {
    #[schema(value_type = String, format = "uuid")]
    pub session_id: uuid::Uuid,
    pub task_count: i32,
    #[schema(value_type = String, format = "date-time")]
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub last_decision: Option<serde_json::Value>,
    pub active: bool,
}

/// Generic error response.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Agent pipeline health response.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AgentHealthResponse {
    #[schema(example = "ok")]
    pub status: String,
    #[schema(example = "up")]
    pub pipeline: String,
}

// ─── Handler doc stubs (no-op — real handlers live in server/mod.rs) ─────────

/// POST /v1/agents/task
///
/// Submit a task to the multi-agent DSPy ADR pipeline.
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
pub fn _doc_submit_agent_task() {}

/// GET /v1/agents/session/{id}
///
/// Retrieve session state (task count, last decision, active flag).
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
pub fn _doc_get_agent_session() {}

/// GET /v1/agents/health
///
/// Health check for the DSPy Python pipeline (no auth required).
#[utoipa::path(
    get,
    path = "/v1/agents/health",
    tag = "agents",
    responses(
        (status = 200, description = "Pipeline up", body = AgentHealthResponse),
        (status = 503, description = "Pipeline down or disabled", body = AgentHealthResponse),
    )
)]
pub fn _doc_agent_health() {}

/// GET /health
///
/// Service health (no auth required). Returns component status map.
#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Service healthy"),
        (status = 503, description = "Service degraded or unhealthy"),
    )
)]
pub fn _doc_health() {}

/// GET /metrics
///
/// Prometheus metrics endpoint (no auth required).
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "system",
    responses(
        (status = 200, description = "Prometheus text format metrics"),
    )
)]
pub fn _doc_metrics() {}

// ─── Security modifier ────────────────────────────────────────────────────────

struct ApiKeyAuth;

impl Modify for ApiKeyAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
        }
    }
}

// ─── OpenAPI document ─────────────────────────────────────────────────────────

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Neoland API",
        version = "0.1.0",
        description = "Multi-agent ADR pipeline control plane (Rust + DSPy)",
        contact(name = "VoidNxLabs", email = "dev@voidnxlabs.io"),
        license(name = "Proprietary")
    ),
    paths(
        _doc_submit_agent_task,
        _doc_get_agent_session,
        _doc_agent_health,
        _doc_health,
        _doc_metrics,
    ),
    components(schemas(
        AgentTaskRequest,
        PipelineResult,
        JuniorOutput,
        SeniorOutput,
        ArchitectOutput,
        TechLeaderOutput,
        SessionState,
        ErrorResponse,
        AgentHealthResponse,
    )),
    modifiers(&ApiKeyAuth),
    tags(
        (name = "agents", description = "Multi-agent ADR pipeline endpoints"),
        (name = "system", description = "Health, metrics, and introspection"),
    )
)]
pub struct NeolandApi;

// ─── Routes ──────────────────────────────────────────────────────────────────

/// Returns axum Router with GET /openapi.json and GET /swagger-ui/* mounted.
///
/// Generic over state so it can be merged into any typed Router without
/// requiring the caller to strip/re-apply state.
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // `SwaggerUi::url("/openapi.json", ...)` already mounts the spec route.
    // Adding an explicit `.route("/openapi.json", ...)` duplicates the GET
    // handler and panics at router construction time in axum 0.7.
    Router::new().merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", NeolandApi::openapi()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_spec_is_valid() {
        let spec = NeolandApi::openapi();
        let json = serde_json::to_string(&spec).expect("spec serialises");
        assert!(json.contains("\"openapi\""));
        assert!(json.contains("Neoland API"));
        assert!(json.contains("/v1/agents/task"));
        assert!(json.contains("/v1/agents/session/{id}"));
    }

    #[test]
    fn openapi_has_security_scheme() {
        let spec = NeolandApi::openapi();
        let components = spec.components.expect("components present");
        assert!(components.security_schemes.contains_key("api_key"));
    }

    #[test]
    fn openapi_schemas_complete() {
        let spec = NeolandApi::openapi();
        let components = spec.components.expect("components");
        let schemas = &components.schemas;
        for name in &[
            "AgentTaskRequest",
            "PipelineResult",
            "JuniorOutput",
            "SeniorOutput",
            "TechLeaderOutput",
            "SessionState",
        ] {
            assert!(schemas.contains_key(*name), "missing schema: {name}");
        }
    }
}
