//! Enterprise SSO — LDAP & OpenID Connect identity providers.
//!
//! v0.0.1 #3

use anyhow::{Context, Result};
use async_trait::async_trait;
use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::types::OAuthUserInfo;

// ── Identity Provider trait ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IdentityInfo {
    pub provider_user_id: String,
    pub provider: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub groups: Vec<String>,
}

impl From<IdentityInfo> for OAuthUserInfo {
    fn from(info: IdentityInfo) -> Self {
        OAuthUserInfo {
            provider: info.provider,
            provider_user_id: info.provider_user_id,
            email: info.email,
            display_name: info.display_name,
            avatar_url: info.avatar_url,
        }
    }
}

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn authenticate(&self, credentials: &str) -> Result<IdentityInfo>;
    async fn health_check(&self) -> Result<bool>;
    fn provider_name(&self) -> &str;
}

// ── LDAP ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: Option<String>,
    pub bind_password: Option<String>,
    pub email_attr: String,
    pub display_name_attr: String,
    pub user_filter: String,
    pub starttls: bool,
    pub ca_cert: Option<String>,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            url: "ldap://localhost:389".into(),
            base_dn: "dc=example,dc=com".into(),
            bind_dn: None,
            bind_password: None,
            email_attr: "mail".into(),
            display_name_attr: "displayName".into(),
            user_filter: "(uid={username})".into(),
            starttls: false,
            ca_cert: None,
        }
    }
}

pub struct LdapProvider {
    config: LdapConfig,
}

impl LdapProvider {
    pub fn new(config: LdapConfig) -> Self {
        Self { config }
    }

    fn connect(&self) -> Result<LdapConn> {
        let settings = LdapConnSettings::new().set_no_tls_verify(self.config.ca_cert.is_none());
        let conn = LdapConn::with_settings(settings, &self.config.url)
            .context("Failed to connect to LDAP server")?;
        Ok(conn)
    }

    fn search_user(&self, conn: &mut LdapConn, username: &str) -> Result<SearchEntry> {
        let filter = self.config.user_filter.replace("{username}", username);

        debug!(filter = %filter, base = %self.config.base_dn, "LDAP search");

        let mut results = conn
            .search(
                &self.config.base_dn,
                Scope::Subtree,
                &filter,
                vec!["dn", &self.config.email_attr, &self.config.display_name_attr, "memberOf"],
            )
            .context("LDAP search failed")?;

        if results.0.is_empty() {
            anyhow::bail!("User '{}' not found in LDAP directory", username);
        }

        if results.0.len() > 1 {
            warn!(count = results.0.len(), username, "Multiple LDAP entries found, using first");
        }

        let entry = SearchEntry::construct(results.0.remove(0));
        Ok(entry)
    }
}

#[async_trait]
impl IdentityProvider for LdapProvider {
    async fn authenticate(&self, credentials: &str) -> Result<IdentityInfo> {
        let parts: Vec<&str> = credentials.splitn(2, '\n').collect();
        let (username, password) = match parts.as_slice() {
            [u, p] => (*u, *p),
            _ => anyhow::bail!("Invalid LDAP credentials format (expected username\\npassword)"),
        };

        let mut conn = self.connect()?;

        if let (Some(bind_dn), Some(bind_pw)) = (&self.config.bind_dn, &self.config.bind_password) {
            let _result =
                conn.simple_bind(bind_dn, bind_pw).context("LDAP service account bind failed")?;
            _result.success().context("LDAP service account bind rejected")?;
        }

        let entry = self.search_user(&mut conn, username)?;

        let _result = conn.simple_bind(&entry.dn, password).context("LDAP user bind failed")?;
        _result
            .success()
            .context("LDAP authentication rejected — invalid credentials")?;

        let email = entry
            .attrs
            .get(&self.config.email_attr)
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_default();

        let display_name = entry
            .attrs
            .get(&self.config.display_name_attr)
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| username.to_string());

        let groups = entry.attrs.get("memberOf").cloned().unwrap_or_default();

        info!(username, dn = %entry.dn, email = %email, "LDAP authentication successful");

        Ok(IdentityInfo {
            provider_user_id: entry.dn.clone(),
            provider: "ldap".into(),
            email,
            display_name,
            avatar_url: None,
            groups,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        match self.connect() {
            Ok(mut conn) => {
                let healthy = conn.simple_bind("", "").is_ok();
                Ok(healthy)
            },
            Err(e) => {
                warn!(error = %e, "LDAP health check failed");
                Ok(false)
            },
        }
    }

    fn provider_name(&self) -> &str {
        "ldap"
    }
}

// ── OpenID Connect ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub display_name: String,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            issuer_url: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            redirect_url: "http://localhost:3001/auth/callback/sso".into(),
            scopes: vec!["openid".into(), "profile".into(), "email".into()],
            display_name: "Enterprise SSO".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct OidcDiscovery {
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: Option<String>,
    #[allow(dead_code)]
    issuer: String,
    #[allow(dead_code)]
    jwks_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OidcTokenResponse {
    access_token: String,
    id_token: Option<String>,
    #[allow(dead_code)]
    token_type: Option<String>,
    #[allow(dead_code)]
    expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    preferred_username: Option<String>,
    picture: Option<String>,
    #[allow(dead_code)]
    iss: Option<String>,
    #[allow(dead_code)]
    aud: Option<String>,
}

pub struct OidcProvider {
    config: OidcConfig,
    http: HttpClient,
}

impl OidcProvider {
    pub fn new(config: OidcConfig) -> Self {
        Self { config, http: HttpClient::new() }
    }

    pub(crate) async fn discover(&self) -> Result<OidcDiscovery> {
        let url = format!(
            "{}/.well-known/openid-configuration",
            self.config.issuer_url.trim_end_matches('/')
        );

        debug!(url = %url, "Discovering OIDC provider");

        let discovery: OidcDiscovery = self
            .http
            .get(&url)
            .send()
            .await
            .context("OIDC discovery request failed")?
            .error_for_status()
            .context("OIDC discovery returned error")?
            .json()
            .await
            .context("Failed to parse OIDC discovery document")?;

        Ok(discovery)
    }

    pub async fn authorize_url(&self) -> Result<String> {
        let discovery = self.discover().await?;

        let state = uuid::Uuid::new_v4().to_string();
        let nonce = uuid::Uuid::new_v4().to_string();
        let scopes = self.config.scopes.join(" ");

        let auth_url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&nonce={}",
            discovery.authorization_endpoint,
            urlencoding(&self.config.client_id),
            urlencoding(&self.config.redirect_url),
            urlencoding(&scopes),
            &state,
            &nonce,
        );

        info!(issuer = %self.config.issuer_url, "Generated OIDC authorization URL");
        Ok(auth_url)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<IdentityInfo> {
        let discovery = self.discover().await?;

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_url),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let token: OidcTokenResponse = self
            .http
            .post(&discovery.token_endpoint)
            .form(&params)
            .send()
            .await
            .context("OIDC token request failed")?
            .error_for_status()
            .context("OIDC token endpoint returned error")?
            .json()
            .await
            .context("Failed to parse OIDC token response")?;

        let claims = if let Some(id_token) = &token.id_token {
            Self::decode_id_token(id_token)?
        } else if let Some(userinfo_url) = &discovery.userinfo_endpoint {
            let userinfo: IdTokenClaims = self
                .http
                .get(userinfo_url)
                .bearer_auth(&token.access_token)
                .send()
                .await
                .context("OIDC userinfo request failed")?
                .json()
                .await
                .context("Failed to parse OIDC userinfo")?;
            userinfo
        } else {
            anyhow::bail!("OIDC provider returned neither id_token nor userinfo endpoint");
        };

        let email = claims.email.unwrap_or_default();
        let display_name =
            claims.name.or(claims.preferred_username).unwrap_or_else(|| claims.sub.clone());

        info!(sub = %claims.sub, email = %email, issuer = %self.config.issuer_url, "OIDC authentication successful");

        Ok(IdentityInfo {
            provider_user_id: claims.sub,
            provider: format!("oidc:{}", self.config.issuer_url),
            email,
            display_name,
            avatar_url: claims.picture,
            groups: Vec::new(),
        })
    }

    fn decode_id_token(token: &str) -> Result<IdTokenClaims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid JWT: expected 3 parts, got {}", parts.len());
        }

        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .context("Failed to base64-decode JWT payload")?;

        let claims: IdTokenClaims =
            serde_json::from_slice(&payload_bytes).context("Failed to parse id_token claims")?;

        Ok(claims)
    }
}

#[async_trait]
impl IdentityProvider for OidcProvider {
    async fn authenticate(&self, code: &str) -> Result<IdentityInfo> {
        self.exchange_code(code).await
    }

    async fn health_check(&self) -> Result<bool> {
        match self.discover().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!(error = %e, issuer = %self.config.issuer_url, "OIDC health check failed");
                Ok(false)
            },
        }
    }

    fn provider_name(&self) -> &str {
        "oidc"
    }
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('#', "%23")
        .replace('+', "%2B")
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_id_token_valid() {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD
            .encode(r#"{"sub":"123","email":"test@example.com","name":"Test User"}"#);
        let signature = "fake-signature";
        let token = format!("{}.{}.{}", header, payload, signature);

        let claims = OidcProvider::decode_id_token(&token).expect("decode should succeed");
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.email.unwrap(), "test@example.com");
        assert_eq!(claims.name.unwrap(), "Test User");
    }

    #[test]
    fn test_decode_id_token_invalid_format() {
        let result = OidcProvider::decode_id_token("not-a-jwt");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_id_token_no_email() {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256"}"#);
        let payload = URL_SAFE_NO_PAD.encode(r#"{"sub":"456","name":"No Email"}"#);
        let token = format!("{}.{}.sig", header, payload);

        let claims = OidcProvider::decode_id_token(&token).expect("decode should succeed");
        assert_eq!(claims.sub, "456");
        assert!(claims.email.is_none());
    }

    #[test]
    fn test_identity_info_to_oauth_user_info() {
        let info = IdentityInfo {
            provider_user_id: "cn=test,dc=example,dc=com".into(),
            provider: "ldap".into(),
            email: "test@example.com".into(),
            display_name: "Test User".into(),
            avatar_url: None,
            groups: vec!["admins".into()],
        };

        let oauth: OAuthUserInfo = info.into();
        assert_eq!(oauth.provider, "ldap");
        assert_eq!(oauth.email, "test@example.com");
    }

    #[test]
    fn test_urlencoding_spaces() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
    }

    #[test]
    fn test_ldap_config_default_values() {
        let cfg = LdapConfig::default();
        assert_eq!(cfg.url, "ldap://localhost:389");
        assert_eq!(cfg.email_attr, "mail");
        assert_eq!(cfg.user_filter, "(uid={username})");
    }

    #[test]
    fn test_oidc_config_default_values() {
        let cfg = OidcConfig::default();
        assert!(cfg.scopes.contains(&"openid".to_string()));
        assert!(cfg.scopes.contains(&"email".to_string()));
    }
}
