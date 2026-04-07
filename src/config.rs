// Neoland configuration loading
// Priority: config file → env vars → CLI flags (CLI flags override all)
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
    pub agents: AgentsConfig,
    pub mmap: MmapConfig,
    pub nats: NatsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub grpc_port: u16,
    pub rest_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    pub server_url: String,
    pub ml_api_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceConfig {
    /// Which inference provider to use: local | deepseek | llamacpp | securellm
    pub provider: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VaultConfig {
    pub addr: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { grpc_port: 50051, rest_port: 3001 }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://[::1]:50051".to_string(),
            ml_api_url: "http://localhost:8080".to_string(),
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self { provider: "local".to_string(), temperature: 0.7, max_tokens: 2048 }
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
    /// Path to the shared-memory file used for zero-copy IPC with the Python pipeline.
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
    /// NATS server URL. Set to empty string to disable.
    pub url: String,
    /// Whether to attempt NATS connection on startup.
    pub enabled: bool,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self { url: "nats://localhost:4222".to_string(), enabled: false }
    }
}

impl Config {
    /// Load config from file, with env var overrides applied on top.
    /// Returns default config if no file is found (no error).
    pub fn load() -> Self {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();
        config
    }

    /// Try loading from ./neoland.toml, then ~/.config/neoland/config.toml
    fn load_from_file() -> Option<Self> {
        let candidates = [std::path::PathBuf::from("neoland.toml"), dirs_candidate()];

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

    /// Apply env var overrides (NEOLAND_* prefix)
    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("NEOLAND_GRPC_PORT") {
            if let Ok(p) = v.parse() {
                self.server.grpc_port = p;
            }
        }
        if let Ok(v) = std::env::var("NEOLAND_REST_PORT") {
            if let Ok(p) = v.parse() {
                self.server.rest_port = p;
            }
        }
        if let Ok(v) = std::env::var("NEOLAND_SERVER_URL") {
            self.client.server_url = v;
        }
        if let Ok(v) = std::env::var("NEOLAND_ML_API_URL") {
            self.client.ml_api_url = v;
        }
        if let Ok(v) = std::env::var("NEOLAND_INFERENCE_PROVIDER") {
            self.inference.provider = v;
        }
        if let Ok(v) = std::env::var("VAULT_ADDR") {
            self.vault.addr = v;
        }
        if let Ok(v) = std::env::var("NEOLAND_SHM_PATH") {
            self.mmap.shm_path = v;
        }
        if let Ok(v) = std::env::var("NEOLAND_NATS_URL") {
            self.nats.url = v;
            self.nats.enabled = true;
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
        assert_eq!(cfg.client.ml_api_url, "http://localhost:8080");
        assert_eq!(cfg.inference.provider, "local");
        assert!((cfg.inference.temperature - 0.7).abs() < f32::EPSILON);
        assert_eq!(cfg.inference.max_tokens, 2048);
        assert_eq!(cfg.vault.addr, "http://localhost:8200");
    }

    #[test]
    fn test_toml_roundtrip() {
        let cfg = Config::default();
        let toml_str = toml::to_string(&cfg).expect("serialization failed");
        let parsed: Config = toml::from_str(&toml_str).expect("deserialization failed");
        assert_eq!(parsed.server.grpc_port, cfg.server.grpc_port);
        assert_eq!(parsed.client.server_url, cfg.client.server_url);
        assert_eq!(parsed.inference.provider, cfg.inference.provider);
    }

    #[test]
    fn test_partial_toml() {
        // Only override server section; other sections keep defaults
        let toml_str = r#"
[server]
grpc_port = 9999
"#;
        let parsed: Config = toml::from_str(toml_str).expect("deserialization failed");
        assert_eq!(parsed.server.grpc_port, 9999);
        assert_eq!(parsed.server.rest_port, 3001); // default
        assert_eq!(parsed.client.server_url, "http://[::1]:50051"); // default
    }
}
