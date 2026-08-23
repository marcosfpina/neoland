//! Rotas de auth web (v0.0.1): OAuth2 Google/GitHub, JWT refresh, me, logout.

use super::*;

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

pub(crate) async fn tenant_slug(
    db: &sqlx::PgPool,
    tenant_id: Option<uuid::Uuid>,
) -> Option<String> {
    let tenant_id = tenant_id?;
    sqlx::query_scalar("SELECT slug FROM tenants WHERE id = $1")
        .bind(tenant_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

pub(crate) async fn issue_auth_tokens(
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
