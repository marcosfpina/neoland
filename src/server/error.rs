//! Error da fronteira HTTP.
//!
//! O TUI e o Web Console parseiam os bodies de erro literalmente, então este
//! tipo reproduz byte a byte os dois shapes que o servidor sempre devolveu
//! (travados por tests/rest_api_test.rs, seção "Error body contract"):
//!
//! - [`ApiError::Message`] → `(status, {"error": "<msg>"})`
//! - [`ApiError::Bare`]    → status puro, body vazio
//!
//! thiserror fica restrito a esta fronteira; o interior do servidor continua
//! em `anyhow`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{message}")]
    Message { status: StatusCode, message: String },
    #[error("{0}")]
    Bare(StatusCode),
}

impl ApiError {
    pub fn message(status: StatusCode, message: impl Into<String>) -> Self {
        Self::Message { status, message: message.into() }
    }

    /// 401 do extractor RBAC (extensão de auth ausente — rota fora do
    /// middleware de auth). O 401 do próprio `auth_middleware` é `Bare`.
    pub fn unauthorized() -> Self {
        Self::message(StatusCode::UNAUTHORIZED, "Authentication required")
    }

    /// 403 do extractor RBAC — chave válida sem o role exigido.
    pub fn forbidden(required: &crate::auth::Role) -> Self {
        Self::message(StatusCode::FORBIDDEN, format!("Requires {required:?} role or higher"))
    }

    /// 503 uniforme quando o orchestrator não foi configurado.
    pub fn pipeline_unavailable() -> Self {
        Self::message(
            StatusCode::SERVICE_UNAVAILABLE,
            "Agent pipeline not configured (DATABASE_URL required)",
        )
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::message(StatusCode::NOT_FOUND, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::message(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Message { status, message } => {
                (status, Json(serde_json::json!({ "error": message }))).into_response()
            },
            Self::Bare(status) => status.into_response(),
        }
    }
}

impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
        Self::Bare(status)
    }
}
