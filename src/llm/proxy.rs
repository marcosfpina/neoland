// SecureLLM Bridge Integration - Real Implementation
// Proxy seguro para LLMs remotos com auditoria e rate limiting

use std::sync::Arc;

use anyhow::{Context, Result};
use securellm_core::{LLMProvider, Message, MessageContent, MessageRole, Request};
use securellm_providers::{
    deepseek::{DeepSeekConfig, DeepSeekProvider},
    gemini::{GeminiConfig, GeminiProvider},
    groq::{GroqConfig, GroqProvider},
    llamacpp::LlamaCppProvider,
};
use tracing::{info, warn};

use crate::secrets::{SecretType, SecretsManager};

/// SecureLLM Proxy com audit e rate limiting
pub struct SecureLLMProxy {
    provider: Arc<dyn LLMProvider>,
    provider_name: String,
    #[allow(dead_code)] // Will be used in Phase 1.3 for key rotation
    secrets_manager: Arc<SecretsManager>,
}

impl std::fmt::Debug for SecureLLMProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureLLMProxy")
            .field("provider_name", &self.provider_name)
            .finish()
    }
}

impl SecureLLMProxy {
    /// Cria nova instância do proxy com provider real
    ///
    /// Phase 1.2: Now uses SecretsManager for secure API key retrieval
    pub async fn new(
        provider_name: &str,
        secrets_manager: Arc<SecretsManager>,
        api_key: Option<String>,
    ) -> Result<Self> {
        let api_key = match api_key {
            Some(key) => key,
            None => Self::load_api_key(provider_name, &secrets_manager).await?,
        };

        let provider: Arc<dyn LLMProvider> = match provider_name {
            "deepseek" => {
                let config = DeepSeekConfig::new(api_key).with_logging(true);
                let provider =
                    DeepSeekProvider::new(config).context("Failed to create DeepSeek provider")?;
                Arc::new(provider)
            },
            "llamacpp" => {
                // LlamaCPP runs locally, no API key needed
                // Default port: 8081, can be configured via api_key param (e.g., "8081:model-name")
                let (port, model) = if api_key.contains(':') {
                    let parts: Vec<&str> = api_key.split(':').collect();
                    let port = parts[0].parse().unwrap_or(8081);
                    let model = parts.get(1).unwrap_or(&"local-model").to_string();
                    (port, model)
                } else {
                    (8081, "local-model".to_string())
                };
                let provider = LlamaCppProvider::new(port, model)
                    .context("Failed to create LlamaCPP provider")?;
                Arc::new(provider)
            },
            "gemini" => {
                let config = GeminiConfig::new(api_key);
                let provider =
                    GeminiProvider::new(config).context("Failed to create Gemini provider")?;
                Arc::new(provider)
            },
            "groq" => {
                let config = GroqConfig::new(api_key);
                let provider =
                    GroqProvider::new(config).context("Failed to create Groq provider")?;
                Arc::new(provider)
            },
            _ => anyhow::bail!(
                "Unsupported provider: {}. Supported: deepseek, llamacpp, gemini, groq",
                provider_name
            ),
        };

        // Validate configuration
        provider.validate_config().context("Provider configuration validation failed")?;

        info!(
            provider = provider_name,
            vault_available = secrets_manager.is_vault_available(),
            "SecureLLM Proxy initialized successfully"
        );

        Ok(Self { provider, provider_name: provider_name.to_string(), secrets_manager })
    }

    /// Load API key from SecretsManager (Vault or environment variable
    /// fallback)
    ///
    /// Phase 1.2: Replaced direct env var access with SecretsManager
    async fn load_api_key(provider: &str, secrets_manager: &SecretsManager) -> Result<String> {
        // LlamaCPP is local and doesn't require an API key
        if provider == "llamacpp" {
            return Ok("8081:local-model".to_string()); // Default config
        }

        secrets_manager
            .get_secret(SecretType::LLMApiKey(provider.to_string()))
            .await
            .with_context(|| {
                format!(
                    "Failed to load API key for {}. Configure Vault or set {}_API_KEY environment \
                     variable",
                    provider,
                    provider.to_uppercase()
                )
            })
    }

    /// Envia requisição segura com auditoria integrada
    pub async fn send_secure(&self, prompt: &str) -> Result<String> {
        // Build request with audit metadata
        let request = Request::new(&self.provider_name, "deepseek-chat")
            .with_temperature(0.7)
            .with_max_tokens(1000)
            .add_message(Message {
                role: MessageRole::User,
                content: MessageContent::Text(prompt.to_string()),
                name: None,
                metadata: None,
            });

        // Log request (sanitized)
        info!(
            provider = %self.provider_name,
            model = %request.model,
            request_id = %request.id,
            prompt_len = prompt.len(),
            "Sending secure LLM request"
        );

        // Send request through provider
        let response =
            self.provider.send_request(request).await.context("SecureLLM request failed")?;

        // Log response metrics
        info!(
            provider = %self.provider_name,
            request_id = %response.request_id,
            tokens_used = response.usage.total_tokens,
            processing_time_ms = response.metadata.processing_time_ms,
            "Received secure LLM response"
        );

        // Extract text from response
        if let Some(choice) = response.choices.first() {
            if let MessageContent::Text(text) = &choice.message.content {
                return Ok(text.clone());
            }
        }

        anyhow::bail!("Empty response from SecureLLM provider")
    }

    /// Check provider health
    pub async fn health_check(&self) -> Result<bool> {
        match self.provider.health_check().await {
            Ok(health) => {
                let is_healthy = matches!(health.status, securellm_core::HealthStatus::Healthy);

                info!(
                    provider = %self.provider_name,
                    status = ?health.status,
                    latency_ms = ?health.latency_ms,
                    "Provider health check completed"
                );

                Ok(is_healthy)
            },
            Err(e) => {
                warn!(
                    provider = %self.provider_name,
                    error = %e,
                    "Provider health check failed"
                );
                Ok(false)
            },
        }
    }

    /// Get provider name
    pub fn provider(&self) -> &str {
        &self.provider_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_api_key_missing() {
        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let result = SecureLLMProxy::load_api_key("nonexistent_provider", &secrets_manager).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("NONEXISTENT_PROVIDER_API_KEY"));
    }

    #[tokio::test]
    async fn test_proxy_creation_no_key() {
        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let result = SecureLLMProxy::new("deepseek", secrets_manager, None).await;
        // Should fail if DEEPSEEK_API_KEY not set in env or Vault
        if std::env::var("DEEPSEEK_API_KEY").is_err() {
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_unsupported_provider() {
        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let result =
            SecureLLMProxy::new("invalid_provider", secrets_manager, Some("test".into())).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported provider"));
    }

    #[tokio::test]
    async fn test_proxy_with_env_var() {
        std::env::set_var("TEST_PROVIDER_API_KEY", "test_key_12345");

        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let api_key = SecureLLMProxy::load_api_key("test_provider", &secrets_manager).await;

        assert!(api_key.is_ok());
        assert_eq!(api_key.unwrap(), "test_key_12345");

        std::env::remove_var("TEST_PROVIDER_API_KEY");
    }
}
