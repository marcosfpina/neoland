//! OAuth2 provider implementations — Google & GitHub.
//!
//! Handles the OAuth2 authorization code flow:
//! 1. Generate authorization URL → redirect user
//! 2. Exchange authorization code for token
//! 3. Fetch user info from provider API

use anyhow::{Context, Result};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;

use super::types::OAuthUserInfo;

// ── Google ────────────────────────────────────────────────────────────────

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    name: String,
    picture: Option<String>,
}

/// Generate the Google authorization URL.
pub fn google_authorize_url(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<(String, CsrfToken)> {
    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string())?)
        .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()))
        .url();

    Ok((auth_url.to_string(), csrf_token))
}

/// Exchange authorization code for Google user info.
pub async fn google_exchange_code(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    code: String,
) -> Result<OAuthUserInfo> {
    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string())?)
        .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);

    let http = HttpClient::new();

    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(&http)
        .await
        .context("Failed to exchange Google auth code")?;

    let user: GoogleUserInfo = http
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .context("Failed to fetch Google user info")?
        .json()
        .await
        .context("Failed to parse Google user info")?;

    Ok(OAuthUserInfo {
        provider: "google".into(),
        provider_user_id: user.sub,
        email: user.email,
        display_name: user.name,
        avatar_url: user.picture,
    })
}

// ── GitHub ────────────────────────────────────────────────────────────────

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

#[derive(Deserialize)]
struct GitHubUserInfo {
    id: i64,
    login: String,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

/// Generate the GitHub authorization URL.
pub fn github_authorize_url(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<(String, CsrfToken)> {
    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(AuthUrl::new(GITHUB_AUTH_URL.to_string())?)
        .set_token_uri(TokenUrl::new(GITHUB_TOKEN_URL.to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    Ok((auth_url.to_string(), csrf_token))
}

/// Exchange authorization code for GitHub user info.
pub async fn github_exchange_code(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    code: String,
) -> Result<OAuthUserInfo> {
    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(AuthUrl::new(GITHUB_AUTH_URL.to_string())?)
        .set_token_uri(TokenUrl::new(GITHUB_TOKEN_URL.to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);

    let http = HttpClient::new();

    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(&http)
        .await
        .context("Failed to exchange GitHub auth code")?;

    let user: GitHubUserInfo = http
        .get(GITHUB_USER_URL)
        .header("User-Agent", "Neoland/0.4.0")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .context("Failed to fetch GitHub user info")?
        .json()
        .await
        .context("Failed to parse GitHub user info")?;

    let email = if user.email.is_some() {
        user.email
    } else {
        fetch_github_primary_email(&http, token.access_token().secret()).await
    };

    let display_name = user.name.unwrap_or_else(|| user.login.clone());

    Ok(OAuthUserInfo {
        provider: "github".into(),
        provider_user_id: user.id.to_string(),
        email: email.unwrap_or_default(),
        display_name,
        avatar_url: user.avatar_url,
    })
}

async fn fetch_github_primary_email(http: &HttpClient, access_token: &str) -> Option<String> {
    let emails: Vec<GitHubEmail> = http
        .get("https://api.github.com/user/emails")
        .header("User-Agent", "Neoland/0.4.0")
        .bearer_auth(access_token)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    emails.into_iter().find(|e| e.primary && e.verified).map(|e| e.email)
}
