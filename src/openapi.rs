//! OpenAPI 3.0 spec generation via utoipa.
//!
//! Routes:
//!   GET /openapi.json  — raw spec
//!   GET /swagger-ui/   — Swagger UI (HTML)

use axum::Router;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

// ─── Schema definitions ───────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AgentTaskRequest {
    pub task: String,
    #[schema(value_type = Option<String>, format = "uuid")]
    pub session_id: Option<uuid::Uuid>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PipelineResult {
    #[schema(value_type = String, format = "uuid")]
    pub task_id: uuid::Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub session_id: uuid::Uuid,
    #[schema(value_type = String, format = "date-time")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub task: String,
    pub junior: serde_json::Value,
    pub senior: serde_json::Value,
    pub architect: Option<serde_json::Value>,
    pub tech_leader: serde_json::Value,
    pub checkpoint_path: String,
}

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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AgentHealthResponse {
    #[schema(example = "ok")]
    pub status: String,
    #[schema(example = "up")]
    pub pipeline: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    #[schema(default = true)]
    pub stream: Option<bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ChatChoice {
    pub message: ChatMessage,
    pub index: u32,
}

// Auth (v0.0.1)
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: AuthUserInfo,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthUserInfo {
    #[schema(value_type = String, format = "uuid")]
    pub id: uuid::Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    #[schema(example = "admin")]
    pub role: String,
    #[schema(example = "voidnx-labs")]
    pub tenant: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthMeResponse {
    #[schema(value_type = String, format = "uuid")]
    pub user_id: uuid::Uuid,
    pub email: String,
    pub display_name: String,
    #[schema(example = "admin")]
    pub role: String,
    pub tenant: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct LogoutResponse {
    #[schema(example = "Logged out")]
    pub message: String,
}

// ─── Security schemes ─────────────────────────────────────────────────────

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

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT access token from /auth/login or /auth/refresh"))
                        .build(),
                ),
            );
        }
    }
}

// ─── OpenAPI spec ─────────────────────────────────────────────────────────

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Neoland API",
        version = "0.0.1",
        description = "Autonomous AI Engineering Platform — Multi-agent ADR pipeline with OAuth2, RBAC, and HA Kubernetes deployment.\n\n## Authentication\n\n- **bearer_auth**: JWT Bearer token (OAuth2 login via Google/GitHub)\n- **api_key**: Legacy API key (X-API-Key header)",
        contact(name = "VoidNxSEC Team", email = "sec@voidnxlabs.com", url = "https://github.com/VoidNxSEC/neoland"),
        license(name = "MIT", url = "https://github.com/VoidNxSEC/neoland/blob/main/LICENSE")
    ),
    paths(
        crate::server::health_handler,
        crate::server::metrics_handler,
        crate::server::agent_health_handler,
        crate::server::submit_agent_task,
        crate::server::list_agent_sessions,
        crate::server::get_agent_session,
        crate::server::list_agent_tools,
        crate::server::call_agent_tool,
        crate::server::resolve_agent_breakpoint,
        crate::server::login_google_handler,
        crate::server::login_github_handler,
        crate::server::callback_google_handler,
        crate::server::callback_github_handler,
        crate::server::refresh_token_handler,
        crate::server::auth_me_handler,
        crate::server::logout_handler,
    ),
    components(schemas(
        AgentTaskRequest, PipelineResult, SessionState,
        ErrorResponse, AgentHealthResponse,
        ChatRequest, ChatMessage, ChatResponse, ChatChoice,
        AuthLoginResponse, AuthUserInfo, RefreshRequest, RefreshResponse,
        AuthMeResponse, LogoutResponse,
    )),
    modifiers(&ApiKeyAuth, &BearerAuth),
    tags(
        (name = "system", description = "Health, metrics, and introspection"),
        (name = "chat", description = "OpenAI-compatible chat completion + SSE streaming"),
        (name = "agents", description = "Multi-agent ADR pipeline (DSPy + 4-stage review)"),
        (name = "auth", description = "OAuth2 login, JWT tokens, session management"),
    )
)]
pub struct NeolandApi;

// ─── Routes ───────────────────────────────────────────────────────────────

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
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
        assert!(json.contains("/health"));
        assert!(json.contains("/auth/login/google"));
    }

    #[test]
    fn openapi_has_security_schemes() {
        let spec = NeolandApi::openapi();
        let components = spec.components.expect("components present");
        assert!(components.security_schemes.contains_key("api_key"));
        assert!(components.security_schemes.contains_key("bearer_auth"));
    }

    #[test]
    fn openapi_schemas_complete() {
        let spec = NeolandApi::openapi();
        let components = spec.components.expect("components");
        let schemas = &components.schemas;
        for name in &[
            "AgentTaskRequest",
            "PipelineResult",
            "SessionState",
            "ChatRequest",
            "ChatResponse",
            "AuthLoginResponse",
            "RefreshResponse",
            "AuthMeResponse",
        ] {
            assert!(schemas.contains_key(*name), "missing schema: {name}");
        }
    }
}
