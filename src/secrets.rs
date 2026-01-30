//! Secrets Management Module
//!
//! This module provides secure secret storage and retrieval using HashiCorp Vault.
//! It supports both Vault integration (production) and environment variable fallback (development).
//!
//! See ADR-012 for architecture decisions.

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;
use crate::audit::{AuditLogger, AuditEvent, AuditAction};

/// Secret types supported by the secrets manager
#[derive(Debug, Clone)]
pub enum SecretType {
    /// LLM provider API keys (DeepSeek, OpenAI, etc.)
    LLMApiKey(String), // provider name
    /// Neoland API keys for client authentication
    NeolandApiKey(String), // role name (admin, user, readonly)
    /// Database credentials
    DatabaseCredential(String), // credential type (password, connection_string)
    /// TLS certificates and keys
    TLSCertificate(String), // cert type (server, client, ca)
}

impl SecretType {
    /// Get the Vault path for this secret type
    fn vault_path(&self) -> String {
        match self {
            SecretType::LLMApiKey(provider) => format!("neoland/llm/{}", provider),
            SecretType::NeolandApiKey(role) => format!("neoland/api-keys/{}", role),
            SecretType::DatabaseCredential(cred_type) => format!("neoland/database/{}", cred_type),
            SecretType::TLSCertificate(cert_type) => format!("neoland/tls/{}", cert_type),
        }
    }

    /// Get the environment variable name for this secret type (fallback)
    fn env_var_name(&self) -> String {
        match self {
            SecretType::LLMApiKey(provider) => format!("{}_API_KEY", provider.to_uppercase()),
            SecretType::NeolandApiKey(role) => format!("NEOLAND_{}_API_KEY", role.to_uppercase()),
            SecretType::DatabaseCredential(cred_type) => {
                format!("DATABASE_{}", cred_type.to_uppercase())
            }
            SecretType::TLSCertificate(cert_type) => {
                format!("TLS_{}_CERT", cert_type.to_uppercase())
            }
        }
    }
}

/// Secrets manager with Vault integration and environment variable fallback
pub struct SecretsManager {
    vault_client: Option<Arc<VaultClient>>,
    mount: String,
    // In-memory cache with TTL (30 seconds)
    cache: Arc<tokio::sync::RwLock<HashMap<String, CachedSecret>>>,
    // Audit logger (Phase 1.3)
    audit_logger: Option<Arc<AuditLogger>>,
}

/// Cached secret with expiration
#[derive(Clone)]
struct CachedSecret {
    value: String,
    expires_at: std::time::Instant,
}

impl SecretsManager {
    /// Create a new SecretsManager
    ///
    /// Attempts to connect to Vault. If Vault is unavailable, falls back to environment variables.
    pub async fn new() -> Result<Self> {
        let vault_addr = std::env::var("VAULT_ADDR").ok();
        let vault_token = std::env::var("VAULT_TOKEN").ok();

        let vault_client = if let (Some(addr), Some(token)) = (vault_addr, vault_token) {
            match Self::create_vault_client(&addr, &token) {
                Ok(client) => {
                    tracing::info!(
                        vault_addr = %addr,
                        "Connected to Vault successfully"
                    );
                    Some(Arc::new(client))
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "Failed to connect to Vault, falling back to environment variables"
                    );
                    None
                }
            }
        } else {
            tracing::info!("Vault not configured (VAULT_ADDR/VAULT_TOKEN missing), using environment variables");
            None
        };

        Ok(Self {
            vault_client,
            mount: "secret".to_string(),
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            audit_logger: None, // Will be set via set_audit_logger
        })
    }

    /// Set audit logger for secret access logging (Phase 1.3)
    pub fn set_audit_logger(&mut self, logger: Arc<AuditLogger>) {
        self.audit_logger = Some(logger);
    }

    /// Create a Vault client with the given address and token
    fn create_vault_client(addr: &str, token: &str) -> Result<VaultClient> {
        let settings = VaultClientSettingsBuilder::default()
            .address(addr)
            .token(token)
            .build()
            .context("Failed to build Vault client settings")?;

        VaultClient::new(settings).context("Failed to create Vault client")
    }

    /// Get a secret by type (Phase 1.3: with audit logging)
    ///
    /// Tries the following sources in order:
    /// 1. In-memory cache (if not expired)
    /// 2. Vault (if configured)
    /// 3. Environment variables (fallback)
    pub async fn get_secret(&self, secret_type: SecretType) -> Result<String> {
        let cache_key = secret_type.vault_path();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if cached.expires_at > std::time::Instant::now() {
                    tracing::debug!(
                        secret_type = ?secret_type,
                        "Retrieved secret from cache"
                    );
                    // Don't log cache hits to reduce noise
                    return Ok(cached.value.clone());
                }
            }
        }

        // Try Vault
        if let Some(client) = &self.vault_client {
            match self.get_from_vault(client, &secret_type).await {
                Ok(secret) => {
                    // Log secret access from Vault (Phase 1.3)
                    if let Some(ref logger) = self.audit_logger {
                        let event = AuditEvent::new(AuditAction::SecretAccess)
                            .with_resource(cache_key.clone())
                            .with_metadata("source", serde_json::json!("vault"));
                        let _ = logger.log(event).await;
                    }

                    self.cache_secret(&cache_key, &secret).await;
                    return Ok(secret);
                }
                Err(e) => {
                    tracing::warn!(
                        secret_type = ?secret_type,
                        error = %e,
                        "Failed to get secret from Vault, trying environment variables"
                    );
                }
            }
        }

        // Fallback to environment variables
        let result = self.get_from_env(&secret_type);

        // Log secret access from environment (Phase 1.3)
        if result.is_ok() {
            if let Some(ref logger) = self.audit_logger {
                let event = AuditEvent::new(AuditAction::SecretAccess)
                    .with_resource(cache_key)
                    .with_metadata("source", serde_json::json!("environment"));
                let _ = logger.log(event).await;
            }
        }

        result
    }

    /// Get secret from Vault
    async fn get_from_vault(
        &self,
        client: &Arc<VaultClient>,
        secret_type: &SecretType,
    ) -> Result<String> {
        let path = secret_type.vault_path();

        let secret: HashMap<String, String> = kv2::read(client.as_ref(), &self.mount, &path)
            .await
            .context(format!("Failed to read secret from Vault: {}", path))?;

        // Extract the appropriate key based on secret type
        let key = match secret_type {
            SecretType::LLMApiKey(_) => "api_key",
            SecretType::NeolandApiKey(_) => "key",
            SecretType::DatabaseCredential(_) => "value",
            SecretType::TLSCertificate(_) => "certificate",
        };

        secret
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Secret key '{}' not found in Vault response", key))
    }

    /// Get secret from environment variable
    fn get_from_env(&self, secret_type: &SecretType) -> Result<String> {
        let env_var = secret_type.env_var_name();

        std::env::var(&env_var).with_context(|| {
            format!(
                "Secret not found in Vault or environment. Set {} or configure Vault",
                env_var
            )
        })
    }

    /// Cache a secret with 30-second TTL
    async fn cache_secret(&self, key: &str, value: &str) {
        let mut cache = self.cache.write().await;
        cache.insert(
            key.to_string(),
            CachedSecret {
                value: value.to_string(),
                expires_at: std::time::Instant::now() + std::time::Duration::from_secs(30),
            },
        );
    }

    /// Store a secret in Vault (admin operation, Phase 1.3: with audit logging)
    ///
    /// Note: This requires appropriate Vault permissions
    pub async fn store_secret(
        &self,
        secret_type: SecretType,
        value: String,
    ) -> Result<()> {
        let client = self
            .vault_client
            .as_ref()
            .ok_or_else(|| anyhow!("Vault not configured, cannot store secrets"))?;

        let path = secret_type.vault_path();

        // Create secret data based on type
        let mut secret_data = HashMap::new();
        let key = match secret_type {
            SecretType::LLMApiKey(_) => "api_key",
            SecretType::NeolandApiKey(_) => "key",
            SecretType::DatabaseCredential(_) => "value",
            SecretType::TLSCertificate(_) => "certificate",
        };
        secret_data.insert(key.to_string(), value);

        kv2::set(client.as_ref(), &self.mount, &path, &secret_data)
            .await
            .context(format!("Failed to store secret in Vault: {}", path))?;

        tracing::info!(
            secret_type = ?secret_type,
            "Secret stored in Vault successfully"
        );

        // Log secret storage (Phase 1.3)
        if let Some(ref logger) = self.audit_logger {
            let event = AuditEvent::new(AuditAction::SecretStore)
                .with_resource(path);
            let _ = logger.log(event).await;
        }

        Ok(())
    }

    /// Clear the cache (useful for testing or forcing refresh)
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::debug!("Secrets cache cleared");
    }

    /// Check if Vault is available
    pub fn is_vault_available(&self) -> bool {
        self.vault_client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_type_paths() {
        assert_eq!(
            SecretType::LLMApiKey("deepseek".to_string()).vault_path(),
            "neoland/llm/deepseek"
        );
        assert_eq!(
            SecretType::NeolandApiKey("admin".to_string()).vault_path(),
            "neoland/api-keys/admin"
        );
        assert_eq!(
            SecretType::DatabaseCredential("password".to_string()).vault_path(),
            "neoland/database/password"
        );
    }

    #[test]
    fn test_secret_type_env_vars() {
        assert_eq!(
            SecretType::LLMApiKey("deepseek".to_string()).env_var_name(),
            "DEEPSEEK_API_KEY"
        );
        assert_eq!(
            SecretType::NeolandApiKey("admin".to_string()).env_var_name(),
            "NEOLAND_ADMIN_API_KEY"
        );
    }

    #[tokio::test]
    async fn test_secrets_manager_env_fallback() {
        // Set test environment variable
        std::env::set_var("DEEPSEEK_API_KEY", "test_key_123");

        let manager = SecretsManager::new().await.unwrap();
        let secret = manager
            .get_secret(SecretType::LLMApiKey("deepseek".to_string()))
            .await;

        assert!(secret.is_ok());
        assert_eq!(secret.unwrap(), "test_key_123");

        // Cleanup
        std::env::remove_var("DEEPSEEK_API_KEY");
    }

    #[tokio::test]
    async fn test_secrets_manager_missing_secret() {
        let manager = SecretsManager::new().await.unwrap();
        let secret = manager
            .get_secret(SecretType::LLMApiKey("nonexistent".to_string()))
            .await;

        assert!(secret.is_err());
        assert!(secret
            .unwrap_err()
            .to_string()
            .contains("NONEXISTENT_API_KEY"));
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        std::env::set_var("TEST_API_KEY", "cached_value");

        let manager = SecretsManager::new().await.unwrap();

        // First call - should retrieve from env and cache
        let secret1 = manager
            .get_secret(SecretType::LLMApiKey("test".to_string()))
            .await
            .unwrap();

        // Change environment variable
        std::env::set_var("TEST_API_KEY", "new_value");

        // Second call - should retrieve from cache (old value)
        let secret2 = manager
            .get_secret(SecretType::LLMApiKey("test".to_string()))
            .await
            .unwrap();

        assert_eq!(secret1, secret2);
        assert_eq!(secret1, "cached_value");

        // Clear cache
        manager.clear_cache().await;

        // Third call - should retrieve from env (new value)
        let secret3 = manager
            .get_secret(SecretType::LLMApiKey("test".to_string()))
            .await
            .unwrap();

        assert_eq!(secret3, "new_value");

        // Cleanup
        std::env::remove_var("TEST_API_KEY");
    }
}
