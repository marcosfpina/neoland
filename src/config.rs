// Neoland configuration loading
// Priority: config file -> env vars -> CLI flags (CLI flags override all)
//
// Config file locations (searched in order):
//   1. ./neoland.toml (project-local)
//   2. ~/.config/neoland/config.toml (user-global)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub inference: InferenceConfig,
    pub vault: VaultConfig,
    pub database: DatabaseConfig,
    pub agents: AgentsConfig,
    pub mmap: MmapConfig,
    pub nats: NatsConfig,
    pub mcp: McpConfig,
    pub matrix: MatrixConfig,
    pub auth: AuthConfig,
    pub shell_tool: ShellToolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub grpc_port: u16,
    pub rest_port: u16,
    pub web_dist_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    pub server_url: String,
    pub neoland_gateway_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceConfig {
    pub provider: String,
    pub temperature: f32,
    pub max_tokens: u32,
    #[serde(default = "default_vllm_url")]
    pub vllm_url: String,
    #[serde(default)]
    pub vllm_model: String,
}

fn default_vllm_url() -> String {
    "http://localhost:8000".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VaultConfig {
    pub addr: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { grpc_port: 50051, rest_port: 3001, web_dist_dir: "web/dist".to_string() }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://[::1]:50051".to_string(),
            neoland_gateway_url: "http://localhost:8080".to_string(),
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            vllm_url: default_vllm_url(),
            vllm_model: String::new(),
        }
    }
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self { addr: "http://localhost:8200".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentsConfig {
    pub dspy_url: String,
    pub pipeline_timeout_secs: u64,
    pub junior_confidence_warn_threshold: f64,
    pub tech_leader_defer_ttl_hours: u32,
    pub checkpoint_dir: String,
    pub rag_top_k: usize,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            dspy_url: "http://localhost:8001".to_string(),
            pipeline_timeout_secs: 120,
            junior_confidence_warn_threshold: 0.4,
            tech_leader_defer_ttl_hours: 24,
            checkpoint_dir: "/var/lib/neoland/checkpoints/adr".to_string(),
            rag_top_k: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MmapConfig {
    pub shm_path: String,
}

impl Default for MmapConfig {
    fn default() -> Self {
        Self { shm_path: "/run/neoland/agent-flags.shm".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NatsConfig {
    pub url: String,
    pub enabled: bool,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self { url: "nats://localhost:4222".to_string(), enabled: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    pub binary: String,
    pub enabled: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self { binary: "securellm-mcp".to_string(), enabled: false }
    }
}

/// Limites do `RunShellCommand` (native tool do MCP interno).
///
/// A tool executa bash arbitrário atrás de auth User+ e de um breakpoint
/// humano. Estes dois limites cobrem o que a aprovação humana não cobre:
/// um comando aprovado que nunca termina, e (opcionalmente) restringir
/// quais binários podem ser invocados.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellToolConfig {
    /// Tempo máximo de execução. Estourado, o processo é morto
    /// (`kill_on_drop`) e a tool devolve erro em vez de segurar o handler.
    pub timeout_secs: u64,
    /// Binários permitidos. **Vazia = sem restrição** (comportamento
    /// histórico preservado). Não-vazia ativa o modo restrito, que também
    /// rejeita encadeamento de shell — sem isso a lista seria contornável
    /// com `permitido; proibido`.
    pub allowlist: Vec<String>,
}

impl Default for ShellToolConfig {
    fn default() -> Self {
        Self { timeout_secs: 30, allowlist: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MatrixConfig {
    pub base_url: String,
    pub enabled: bool,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self { base_url: "http://localhost:8002".to_string(), enabled: false }
    }
}

// ── Auth (v0.0.1 enterprise multi-tenant) ────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    pub oauth: OAuthConfig,
    pub jwt: JwtConfig,
    pub ldap: LdapAuthConfig,
    pub oidc: OidcAuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OAuthConfig {
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct JwtConfig {
    pub secret: String,
}

/// LDAP directory authentication (v0.0.1 #3)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LdapAuthConfig {
    pub enabled: bool,
    pub url: String,
    pub base_dn: String,
    pub bind_dn: Option<String>,
    pub bind_password: Option<String>,
    pub email_attr: String,
    pub display_name_attr: String,
    pub user_filter: String,
    pub starttls: bool,
}

/// OpenID Connect enterprise SSO (v0.0.1 #3)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OidcAuthConfig {
    pub enabled: bool,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub display_name: String,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            google_client_id: None,
            google_client_secret: None,
            github_client_id: None,
            github_client_secret: None,
            base_url: "http://localhost:3001".to_string(),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self { secret: "neoland-dev-jwt-secret-change-in-production".to_string() }
    }
}

impl Default for LdapAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "ldap://localhost:389".to_string(),
            base_dn: "dc=example,dc=com".to_string(),
            bind_dn: None,
            bind_password: None,
            email_attr: "mail".to_string(),
            display_name_attr: "displayName".to_string(),
            user_filter: "(uid={username})".to_string(),
            starttls: false,
        }
    }
}

impl Default for OidcAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            issuer_url: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            redirect_url: "http://localhost:3001/auth/callback/sso".to_string(),
            display_name: "Enterprise SSO".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();
        config
    }

    /// Candidate config file locations, in load order (first existing wins).
    pub fn candidate_paths() -> Vec<std::path::PathBuf> {
        vec![std::path::PathBuf::from("neoland.toml"), dirs_candidate()]
    }

    /// Copy of the effective config with secret material masked — safe to print.
    pub fn redacted(&self) -> Self {
        fn mask(s: &str) -> String {
            if s.is_empty() {
                String::new()
            } else {
                "***".to_string()
            }
        }
        let mut c = self.clone();
        c.auth.jwt.secret = mask(&c.auth.jwt.secret);
        c.auth.oauth.google_client_secret = c.auth.oauth.google_client_secret.as_deref().map(mask);
        c.auth.oauth.github_client_secret = c.auth.oauth.github_client_secret.as_deref().map(mask);
        c.auth.ldap.bind_password = c.auth.ldap.bind_password.as_deref().map(mask);
        c.auth.oidc.client_secret = mask(&c.auth.oidc.client_secret);
        if let Ok(mut url) = reqwest::Url::parse(&c.database.url) {
            if url.password().is_some() && url.set_password(Some("***")).is_ok() {
                c.database.url = url.to_string();
            }
        }
        c
    }

    /// Semantic validation of the effective config. Empty vec = valid.
    pub fn validate(&self) -> Vec<String> {
        fn bad_url(name: &str, value: &str) -> Option<String> {
            if value.is_empty() || reqwest::Url::parse(value).is_ok() {
                None
            } else {
                Some(format!("{name}: URL inválida '{value}'"))
            }
        }

        let mut problems = Vec::new();
        if self.server.grpc_port == self.server.rest_port {
            problems.push(format!(
                "server.grpc_port e server.rest_port são a mesma porta ({})",
                self.server.grpc_port
            ));
        }
        let urls = [
            ("client.server_url", self.client.server_url.as_str()),
            ("client.neoland_gateway_url", self.client.neoland_gateway_url.as_str()),
            ("agents.dspy_url", self.agents.dspy_url.as_str()),
            ("vault.addr", self.vault.addr.as_str()),
            ("database.url", self.database.url.as_str()),
        ];
        problems.extend(urls.iter().filter_map(|(name, value)| bad_url(name, value)));
        if self.nats.enabled {
            problems.extend(bad_url("nats.url", &self.nats.url));
        }
        if self.matrix.enabled {
            problems.extend(bad_url("matrix.base_url", &self.matrix.base_url));
        }
        if self.auth.oidc.enabled {
            problems.extend(bad_url("auth.oidc.issuer_url", &self.auth.oidc.issuer_url));
        }
        if self.mcp.enabled && self.mcp.binary.is_empty() {
            problems.push("mcp.enabled=true mas mcp.binary está vazio".to_string());
        }
        if self.agents.pipeline_timeout_secs == 0 {
            problems.push(
                "agents.pipeline_timeout_secs = 0 — toda chamada de pipeline expira".to_string(),
            );
        }
        if self.auth.jwt.secret.is_empty() {
            problems.push("auth.jwt.secret vazio — tokens JWT não podem ser assinados".to_string());
        }
        problems
    }

    fn load_from_file() -> Option<Self> {
        let candidates = Self::candidate_paths();

        for path in &candidates {
            if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(content) => match toml::from_str(&content) {
                        Ok(cfg) => {
                            tracing::info!(path = %path.display(), "Loaded config file");
                            return Some(cfg);
                        },
                        Err(e) => {
                            tracing::warn!(
                                path = %path.display(),
                                error = %e,
                                "Failed to parse config file, using defaults"
                            );
                        },
                    },
                    Err(e) => {
                        tracing::warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to read config file, using defaults"
                        );
                    },
                }
            }
        }
        None
    }

    fn apply_env_overrides(&mut self) {
        self.apply_env_overrides_from(|key| std::env::var(key).ok());
    }

    fn apply_env_overrides_from<F>(&mut self, get_env: F)
    where
        F: Fn(&str) -> Option<String>,
    {
        if let Some(v) = get_env("NEOLAND_GRPC_PORT") {
            if let Ok(p) = v.parse() {
                self.server.grpc_port = p;
            }
        }
        if let Some(v) = get_env("NEOLAND_REST_PORT") {
            if let Ok(p) = v.parse() {
                self.server.rest_port = p;
            }
        }
        if let Some(v) = get_env("NEOLAND_SERVER_URL") {
            self.client.server_url = v;
        }
        if let Some(v) = get_env("NEOLAND_GATEWAY_URL") {
            self.client.neoland_gateway_url = v;
        }
        if let Some(v) = get_env("NEOLAND_ML_API_URL") {
            self.client.neoland_gateway_url = v;
        }
        if let Some(v) = get_env("NEOLAND_INFERENCE_PROVIDER") {
            self.inference.provider = v;
        }
        if let Some(v) = get_env("NEOLAND_VLLM_URL") {
            self.inference.vllm_url = v;
        }
        if let Some(v) = get_env("NEOLAND_VLLM_MODEL") {
            self.inference.vllm_model = v;
        }
        if let Some(v) = get_env("VAULT_ADDR") {
            self.vault.addr = v;
        }
        if let Some(v) = get_env("NEOLAND_DATABASE_URL") {
            self.database.url = v;
        } else if let Some(v) = get_env("DATABASE_URL") {
            self.database.url = v;
        }
        if let Some(v) = get_env("NEOLAND_DSPY_URL") {
            self.agents.dspy_url = v;
        }
        if let Some(v) = get_env("NEOLAND_PIPELINE_TIMEOUT_SECS") {
            if let Ok(timeout) = v.parse() {
                self.agents.pipeline_timeout_secs = timeout;
            }
        }
        if let Some(v) = get_env("NEOLAND_JUNIOR_CONFIDENCE_WARN_THRESHOLD") {
            if let Ok(threshold) = v.parse() {
                self.agents.junior_confidence_warn_threshold = threshold;
            }
        }
        if let Some(v) = get_env("NEOLAND_TECH_LEADER_DEFER_TTL_HOURS") {
            if let Ok(ttl) = v.parse() {
                self.agents.tech_leader_defer_ttl_hours = ttl;
            }
        }
        if let Some(v) = get_env("NEOLAND_CHECKPOINT_DIR") {
            self.agents.checkpoint_dir = v;
        }
        if let Some(v) = get_env("NEOLAND_RAG_TOP_K") {
            if let Ok(top_k) = v.parse() {
                self.agents.rag_top_k = top_k;
            }
        }
        if let Some(v) = get_env("NEOLAND_SHM_PATH") {
            self.mmap.shm_path = v;
        }
        if let Some(v) = get_env("NEOLAND_NATS_URL") {
            self.nats.url = v;
            self.nats.enabled = true;
        }
        if let Some(v) = get_env("NEOLAND_NATS_ENABLED") {
            match v.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => self.nats.enabled = true,
                "0" | "false" | "no" | "off" => self.nats.enabled = false,
                _ => {},
            }
        }
        if let Some(v) = get_env("NEOLAND_MCP_BINARY") {
            self.mcp.binary = v;
            self.mcp.enabled = true;
        }
        if let Some(v) = get_env("NEOLAND_MCP_ENABLED") {
            match v.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => self.mcp.enabled = true,
                "0" | "false" | "no" | "off" => self.mcp.enabled = false,
                _ => {},
            }
        }
        if let Some(v) = get_env("NEOLAND_SHELL_TIMEOUT_SECS") {
            if let Ok(secs) = v.parse::<u64>() {
                self.shell_tool.timeout_secs = secs;
            }
        }
        if let Some(v) = get_env("NEOLAND_SHELL_ALLOWLIST") {
            self.shell_tool.allowlist =
                v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        }
        if let Some(v) = get_env("NEOLAND_MATRIX_URL") {
            self.matrix.base_url = v;
            self.matrix.enabled = true;
        }
        if let Some(v) = get_env("NEOLAND_MATRIX_ENABLED") {
            match v.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => self.matrix.enabled = true,
                "0" | "false" | "no" | "off" => self.matrix.enabled = false,
                _ => {},
            }
        }
        // Auth (v0.0.1)
        if let Some(v) = get_env("NEOLAND_GOOGLE_CLIENT_ID") {
            self.auth.oauth.google_client_id = Some(v);
        }
        if let Some(v) = get_env("NEOLAND_GOOGLE_CLIENT_SECRET") {
            self.auth.oauth.google_client_secret = Some(v);
        }
        if let Some(v) = get_env("NEOLAND_GITHUB_CLIENT_ID") {
            self.auth.oauth.github_client_id = Some(v);
        }
        if let Some(v) = get_env("NEOLAND_GITHUB_CLIENT_SECRET") {
            self.auth.oauth.github_client_secret = Some(v);
        }
        if let Some(v) = get_env("NEOLAND_OAUTH_BASE_URL") {
            self.auth.oauth.base_url = v;
        }
        if let Some(v) = get_env("NEOLAND_JWT_SECRET") {
            self.auth.jwt.secret = v;
        }
        // SSO / LDAP (v0.0.1 #3)
        if let Some(v) = get_env("NEOLAND_LDAP_ENABLED") {
            self.auth.ldap.enabled =
                matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on");
        }
        if let Some(v) = get_env("NEOLAND_LDAP_URL") {
            self.auth.ldap.url = v;
        }
        if let Some(v) = get_env("NEOLAND_LDAP_BASE_DN") {
            self.auth.ldap.base_dn = v;
        }
        if let Some(v) = get_env("NEOLAND_LDAP_BIND_DN") {
            self.auth.ldap.bind_dn = Some(v);
        }
        if let Some(v) = get_env("NEOLAND_LDAP_BIND_PASSWORD") {
            self.auth.ldap.bind_password = Some(v);
        }
        if let Some(v) = get_env("NEOLAND_LDAP_EMAIL_ATTR") {
            self.auth.ldap.email_attr = v;
        }
        if let Some(v) = get_env("NEOLAND_LDAP_DISPLAY_NAME_ATTR") {
            self.auth.ldap.display_name_attr = v;
        }
        if let Some(v) = get_env("NEOLAND_LDAP_USER_FILTER") {
            self.auth.ldap.user_filter = v;
        }
        if let Some(v) = get_env("NEOLAND_OIDC_ENABLED") {
            self.auth.oidc.enabled =
                matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on");
        }
        if let Some(v) = get_env("NEOLAND_OIDC_ISSUER_URL") {
            self.auth.oidc.issuer_url = v;
        }
        if let Some(v) = get_env("NEOLAND_OIDC_CLIENT_ID") {
            self.auth.oidc.client_id = v;
        }
        if let Some(v) = get_env("NEOLAND_OIDC_CLIENT_SECRET") {
            self.auth.oidc.client_secret = v;
        }
        if let Some(v) = get_env("NEOLAND_OIDC_REDIRECT_URL") {
            self.auth.oidc.redirect_url = v;
        }
        if let Some(v) = get_env("NEOLAND_OIDC_DISPLAY_NAME") {
            self.auth.oidc.display_name = v;
        }
    }
}

fn dirs_candidate() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("neoland")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.server.grpc_port, 50051);
        assert_eq!(cfg.server.rest_port, 3001);
        assert_eq!(cfg.client.server_url, "http://[::1]:50051");
        assert_eq!(cfg.client.neoland_gateway_url, "http://localhost:8080");
        assert_eq!(cfg.inference.provider, "local");
    }

    #[test]
    fn test_toml_roundtrip() {
        let cfg = Config::default();
        let toml_str = toml::to_string(&cfg).expect("serialization failed");
        let parsed: Config = toml::from_str(&toml_str).expect("deserialization failed");
        assert_eq!(parsed.server.grpc_port, cfg.server.grpc_port);
    }

    #[test]
    fn test_partial_toml() {
        let toml_str = r#"
[server]
grpc_port = 9999
"#;
        let parsed: Config = toml::from_str(toml_str).expect("deserialization failed");
        assert_eq!(parsed.server.grpc_port, 9999);
        assert_eq!(parsed.server.rest_port, 3001);
    }

    #[test]
    fn test_apply_env_overrides_from_updates_agent_and_nats_settings() {
        use std::collections::HashMap;
        let env = HashMap::from([
            ("NEOLAND_DSPY_URL", "http://127.0.0.1:8100".to_string()),
            ("NEOLAND_PIPELINE_TIMEOUT_SECS", "240".to_string()),
            ("NEOLAND_CHECKPOINT_DIR", "/srv/neoland/checkpoints".to_string()),
            ("NEOLAND_RAG_TOP_K", "11".to_string()),
            ("NEOLAND_NATS_ENABLED", "true".to_string()),
        ]);
        let mut cfg = Config::default();
        cfg.apply_env_overrides_from(|key| env.get(key).cloned());
        assert_eq!(cfg.agents.dspy_url, "http://127.0.0.1:8100");
        assert_eq!(cfg.agents.pipeline_timeout_secs, 240);
        assert!(cfg.nats.enabled);
    }
}
