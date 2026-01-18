// SecureLLM Bridge Integration - Real Implementation
// Proxy seguro para LLMs remotos com auditoria e rate limiting

use anyhow::{Context, Result};
use securellm_core::{
    LLMProvider, Request, Message, MessageRole, MessageContent,
};
use securellm_providers::deepseek::{DeepSeekConfig, DeepSeekProvider};
use std::sync::Arc;
use tracing::{info, warn};

/// SecureLLM Proxy com audit e rate limiting
pub struct SecureLLMProxy {
    provider: Arc<dyn LLMProvider>,
    provider_name: String,
}

impl SecureLLMProxy {
    /// Cria nova instância do proxy com provider real
    pub fn new(provider_name: &str, api_key: Option<String>) -> Result<Self> {
        let api_key = match api_key {
            Some(key) => key,
            None => Self::load_api_key(provider_name)?,
        };

        let provider: Arc<dyn LLMProvider> = match provider_name {
            "deepseek" => {
                let config = DeepSeekConfig::new(api_key)
                    .with_logging(true); // Enable logging for audit
                let provider = DeepSeekProvider::new(config)
                    .context("Failed to create DeepSeek provider")?;
                Arc::new(provider)
            }
            // TODO: Add OpenAI, Anthropic, Ollama when needed
            _ => anyhow::bail!("Unsupported provider: {}. Supported: deepseek", provider_name),
        };

        // Validate configuration
        provider.validate_config()
            .context("Provider configuration validation failed")?;

        info!(
            provider = provider_name,
            "SecureLLM Proxy initialized successfully"
        );

        Ok(Self {
            provider,
            provider_name: provider_name.to_string(),
        })
    }

    /// Load API key from environment variable
    fn load_api_key(provider: &str) -> Result<String> {
        let var_name = format!("{}_API_KEY", provider.to_uppercase());
        std::env::var(&var_name)
            .with_context(|| format!(
                "Missing API key for {}. Set {} environment variable",
                provider, var_name
            ))
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
        let response = self.provider
            .send_request(request)
            .await
            .context("SecureLLM request failed")?;

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
                let is_healthy = matches!(
                    health.status,
                    securellm_core::HealthStatus::Healthy
                );
                
                info!(
                    provider = %self.provider_name,
                    status = ?health.status,
                    latency_ms = ?health.latency_ms,
                    "Provider health check completed"
                );
                
                Ok(is_healthy)
            }
            Err(e) => {
                warn!(
                    provider = %self.provider_name,
                    error = %e,
                    "Provider health check failed"
                );
                Ok(false)
            }
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

    #[test]
    fn test_load_api_key_missing() {
        let result = SecureLLMProxy::load_api_key("deepseek");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DEEPSEEK_API_KEY"));
    }

    #[tokio::test]
    async fn test_proxy_creation_no_key() {
        let result = SecureLLMProxy::new("deepseek", None);
        // Should fail if DEEPSEEK_API_KEY not set
        if std::env::var("DEEPSEEK_API_KEY").is_err() {
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_unsupported_provider() {
        let result = SecureLLMProxy::new("invalid_provider", Some("test".into()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported provider"));
    }
}
