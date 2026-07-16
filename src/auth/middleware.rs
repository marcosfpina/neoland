//! Axum middleware for JWT authentication.
//!
//! Extracts the Bearer token from the Authorization header, validates it,
//! and injects `AuthUser` into request extensions for downstream handlers.

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;

use super::jwt::validate_access_token;
use super::types::AuthUser;

/// JWT secret for token validation — passed via axum State.
#[derive(Clone)]
pub struct JwtSecret(pub Vec<u8>);

/// Error returned when authentication fails.
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    ExpiredToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing Authorization header"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token expired"),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Middleware that validates the JWT Bearer token and injects `AuthUser` into
/// request extensions. Routes that need auth should be wrapped with this.
pub async fn require_auth(
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::MissingToken)?;

    // Get JWT secret from request extensions (set up in server setup)
    let secret = request
        .extensions()
        .get::<JwtSecret>()
        .ok_or(AuthError::InvalidToken)?;

    let claims = validate_access_token(token, &secret.0)
        .map_err(|_| AuthError::ExpiredToken)?;

    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::InvalidToken)?;

    let auth_user = AuthUser {
        user_id,
        tenant_id: None,
        tenant_slug: claims.tenant,
        email: claims.email,
        display_name: claims.name,
        role: claims.role,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

/// Extract `AuthUser` from request extensions (set by `require_auth` middleware).
pub fn get_auth_user(request: &Request) -> Option<&AuthUser> {
    request.extensions().get::<AuthUser>()
}
