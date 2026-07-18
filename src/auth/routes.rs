//! Authentication routes — OAuth2, LDAP, and OIDC SSO.
//!
//! Endpoints:
//!   GET  /auth/login/:provider   -> redirect to OAuth2 provider
//!   GET  /auth/callback/:provider -> exchange code, create session, return JWT
//!   POST /auth/login/ldap        -> LDAP bind authentication (v0.0.1 #3)
//!   GET  /auth/login/sso         -> OIDC redirect to enterprise IdP (v0.0.1 #3)
//!   GET  /auth/callback/sso      -> OIDC code exchange + JWT (v0.0.1 #3)
//!   POST /auth/refresh           -> exchange refresh token for new access token
//!   GET  /auth/me                -> return current user info
//!   POST /auth/logout            -> revoke session

use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use super::jwt::{
    create_access_token, create_refresh_token, hash_refresh_token, refresh_token_lifetime_secs,
};
use super::oauth;
use super::sso::{IdentityProvider, LdapConfig, LdapProvider, OidcConfig, OidcProvider};
use super::types::{OAuthUserInfo, User};
use super::JwtSecret;
use crate::auth::middleware::get_auth_user;

// ── Shared state ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AuthState {
    pub db: PgPool,
    pub jwt_secret: JwtSecret,
    pub base_url: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    /// LDAP config (v0.0.1 #3)
    pub ldap_config: Option<LdapConfig>,
    /// OIDC SSO config (v0.0.1 #3)
    pub oidc_config: Option<OidcConfig>,
}

// ── Query params ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CallbackParams {
    code: String,
    #[allow(dead_code)]
    state: String,
}

#[derive(Deserialize)]
struct RefreshParams {
    refresh_token: String,
}

#[derive(Deserialize)]
struct LdapLoginRequest {
    username: String,
    password: String,
}

// ── Router ────────────────────────────────────────────────────────────────

pub fn auth_routes() -> Router<AuthState> {
    Router::new()
        .route("/auth/login/google", get(login_google))
        .route("/auth/login/github", get(login_github))
        .route("/auth/callback/google", get(callback_google))
        .route("/auth/callback/github", get(callback_github))
        .route("/auth/login/ldap", post(login_ldap))
        .route("/auth/login/sso", get(login_sso))
        .route("/auth/callback/sso", get(callback_sso))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
}

// ── OAuth2 Handlers ───────────────────────────────────────────────────────

async fn login_google(State(state): State<AuthState>) -> Result<Redirect, StatusCode> {
    let (client_id, client_secret) = state
        .google_client_id
        .as_deref()
        .zip(state.google_client_secret.as_deref())
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let redirect_url = format!("{}/auth/callback/google", state.base_url);
    let (auth_url, _csrf) = oauth::google_authorize_url(client_id, client_secret, &redirect_url)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::temporary(&auth_url))
}

async fn login_github(State(state): State<AuthState>) -> Result<Redirect, StatusCode> {
    let (client_id, client_secret) = state
        .github_client_id
        .as_deref()
        .zip(state.github_client_secret.as_deref())
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let redirect_url = format!("{}/auth/callback/github", state.base_url);
    let (auth_url, _csrf) = oauth::github_authorize_url(client_id, client_secret, &redirect_url)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::temporary(&auth_url))
}

async fn callback_google(
    State(state): State<AuthState>,
    Query(params): Query<CallbackParams>,
    jar: CookieJar,
) -> Result<Response, StatusCode> {
    let (client_id, client_secret) = state
        .google_client_id
        .as_deref()
        .zip(state.google_client_secret.as_deref())
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let redirect_url = format!("{}/auth/callback/google", state.base_url);
    let user_info =
        oauth::google_exchange_code(client_id, client_secret, &redirect_url, params.code)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

    complete_oauth_login(&state, user_info, jar).await
}

async fn callback_github(
    State(state): State<AuthState>,
    Query(params): Query<CallbackParams>,
    jar: CookieJar,
) -> Result<Response, StatusCode> {
    let (client_id, client_secret) = state
        .github_client_id
        .as_deref()
        .zip(state.github_client_secret.as_deref())
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let redirect_url = format!("{}/auth/callback/github", state.base_url);
    let user_info =
        oauth::github_exchange_code(client_id, client_secret, &redirect_url, params.code)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

    complete_oauth_login(&state, user_info, jar).await
}

async fn complete_oauth_login(
    state: &AuthState,
    user_info: OAuthUserInfo,
    _jar: CookieJar,
) -> Result<Response, StatusCode> {
    let user = upsert_oauth_user(&state.db, &user_info)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    issue_tokens_and_respond(state, user).await
}

// ── LDAP Handler (v0.0.1 #3) ──────────────────────────────────────────────

async fn login_ldap(
    State(state): State<AuthState>,
    Json(creds): Json<LdapLoginRequest>,
) -> Result<Response, StatusCode> {
    let ldap_cfg = state.ldap_config.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let provider = LdapProvider::new(ldap_cfg);

    // Format credentials as expected by the LDAP provider
    let credentials = format!("{}\n{}", creds.username, creds.password);

    let identity = provider
        .authenticate(&credentials)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Convert IdentityInfo -> OAuthUserInfo and reuse the existing user upsert flow
    let user_info: OAuthUserInfo = identity.into();
    let user = upsert_oauth_user(&state.db, &user_info)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    issue_tokens_and_respond(&state, user).await
}

// ── OIDC SSO Handlers (v0.0.1 #3) ─────────────────────────────────────────

async fn login_sso(State(state): State<AuthState>) -> Result<Redirect, StatusCode> {
    let oidc_cfg = state.oidc_config.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let provider = OidcProvider::new(oidc_cfg);

    let auth_url = provider.authorize_url().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to generate OIDC authorization URL");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::temporary(&auth_url))
}

async fn callback_sso(
    State(state): State<AuthState>,
    Query(params): Query<CallbackParams>,
) -> Result<Response, StatusCode> {
    let oidc_cfg = state.oidc_config.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let provider = OidcProvider::new(oidc_cfg);

    let identity = provider
        .authenticate(&params.code)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_info: OAuthUserInfo = identity.into();
    let user = upsert_oauth_user(&state.db, &user_info)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    issue_tokens_and_respond(&state, user).await
}

// ── Token management ──────────────────────────────────────────────────────

async fn refresh(
    State(state): State<AuthState>,
    Json(params): Json<RefreshParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let refresh_hash = hash_refresh_token(&params.refresh_token);

    let session = sqlx::query_as::<_, super::types::Session>(
        "SELECT * FROM sessions WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()",
    )
    .bind(&refresh_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(session.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let role = if user.is_owner { "admin" } else { "user" };
    let tenant_slug = get_tenant_slug(&state.db, user.tenant_id).await.unwrap_or_default();

    let access_token = create_access_token(
        user.id,
        &tenant_slug,
        role,
        &user.email,
        &user.display_name,
        &state.jwt_secret.0,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "access_token": access_token })))
}

async fn me(request: Request) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = get_auth_user(&request).ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(json!({
        "user_id": user.user_id.to_string(),
        "email": user.email,
        "display_name": user.display_name,
        "role": user.role,
        "tenant": user.tenant_slug,
    })))
}

async fn logout(
    State(state): State<AuthState>,
    Json(params): Json<RefreshParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let refresh_hash = hash_refresh_token(&params.refresh_token);

    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE token_hash = $1")
        .bind(&refresh_hash)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "message": "Logged out" })))
}

// ── Shared helpers ────────────────────────────────────────────────────────

/// Issue JWT access + refresh tokens and return the standard login response.
async fn issue_tokens_and_respond(state: &AuthState, user: User) -> Result<Response, StatusCode> {
    let role = if user.is_owner { "admin" } else { "user" };
    let tenant_slug = get_tenant_slug(&state.db, user.tenant_id).await.unwrap_or_default();

    let access_token = create_access_token(
        user.id,
        &tenant_slug,
        role,
        &user.email,
        &user.display_name,
        &state.jwt_secret.0,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_token = create_refresh_token();
    let refresh_hash = hash_refresh_token(&refresh_token);

    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(refresh_token_lifetime_secs());
    sqlx::query("INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user.id)
        .bind(&refresh_hash)
        .bind(expires_at)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .ok();

    Ok(Json(json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "user": {
            "id": user.id,
            "email": user.email,
            "display_name": user.display_name,
            "avatar_url": user.avatar_url,
            "role": role,
            "tenant": tenant_slug,
        }
    }))
    .into_response())
}

pub async fn upsert_oauth_user(db: &PgPool, info: &OAuthUserInfo) -> Result<User, sqlx::Error> {
    let existing = sqlx::query_as::<_, super::types::OAuthAccount>(
        "SELECT * FROM oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(&info.provider)
    .bind(&info.provider_user_id)
    .fetch_optional(db)
    .await?;

    let user_id = if let Some(account) = existing {
        sqlx::query(
            "UPDATE users SET display_name = $1, avatar_url = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(&info.display_name)
        .bind(&info.avatar_url)
        .bind(account.user_id)
        .execute(db)
        .await?;
        account.user_id
    } else {
        let user_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, display_name, avatar_url, provider) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(&info.email)
        .bind(&info.display_name)
        .bind(&info.avatar_url)
        .bind(&info.provider)
        .fetch_one(db)
        .await?;

        sqlx::query(
            "INSERT INTO oauth_accounts (user_id, provider, provider_user_id) VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(&info.provider)
        .bind(&info.provider_user_id)
        .execute(db)
        .await?;

        let slug = info.email.split('@').next().unwrap_or("user");
        sqlx::query(
            "INSERT INTO tenants (name, slug) VALUES ($1, $2) ON CONFLICT (slug) DO NOTHING",
        )
        .bind(&info.display_name)
        .bind(slug)
        .execute(db)
        .await?;

        user_id
    };

    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
}

async fn get_tenant_slug(db: &PgPool, tenant_id: Option<uuid::Uuid>) -> Option<String> {
    let tid = tenant_id?;
    sqlx::query_scalar::<_, String>("SELECT slug FROM tenants WHERE id = $1")
        .bind(tid)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}
