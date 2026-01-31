// Unified LLM Client - Local First Strategy
// Abstração unificada para ml-offload-api + securellm-bridge

use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::{
    llm::SecureLLMProxy,
    ml_offload::{ChatCompletionRequest, ChatMessage, MLOffloadClient},
    secrets::SecretsManager,
};

/// Estratégia de roteamento para LLM requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Prioriza ml-offload local, fallback para SecureLLM externo
    LocalFirst,
    /// Prioriza SecureLLM externo, fallback para ml-offload local
    ExternalFirst,
    /// (Não implementado) Alterna baseado em latência
    LoadBalanced,
}

/// Cliente LLM unificado com roteamento inteligente
pub struct UnifiedLLMClient {
    ml_offload: Option<MLOffloadClient>,
    securellm: Option<SecureLLMProxy>,
    strategy: RoutingStrategy,
}

impl UnifiedLLMClient {
    /// Cria novo cliente com estratégia LocalFirst (padrão)
    ///
    /// Phase 1.2: Now requires SecretsManager for secure API key loading
    pub async fn new_local_first(
        ml_offload_url: String,
        secrets_manager: Arc<SecretsManager>,
        securellm_provider: Option<(&str, Option<String>)>,
    ) -> Result<Self> {
        let ml_offload = match MLOffloadClient::new(ml_offload_url) {
            Ok(client) => {
                info!("ML-Offload client initialized");
                Some(client)
            },
            Err(e) => {
                warn!("Failed to initialize ML-Offload client: {}", e);
                None
            },
        };

        let securellm = if let Some((provider, api_key)) = securellm_provider {
            match SecureLLMProxy::new(provider, secrets_manager.clone(), api_key).await {
                Ok(proxy) => {
                    info!(provider = provider, "SecureLLM proxy initialized");
                    Some(proxy)
                },
                Err(e) => {
                    warn!("Failed to initialize SecureLLM proxy: {}", e);
                    None
                },
            }
        } else {
            None
        };

        if ml_offload.is_none() && securellm.is_none() {
            anyhow::bail!("No LLM backend available (both ml-offload and securellm failed)");
        }

        Ok(Self { ml_offload, securellm, strategy: RoutingStrategy::LocalFirst })
    }

    /// Envia chat request com roteamento baseado na estratégia
    pub async fn chat(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        match self.strategy {
            RoutingStrategy::LocalFirst => {
                self.chat_local_first(prompt, temperature, max_tokens).await
            },
            RoutingStrategy::ExternalFirst => {
                self.chat_external_first(prompt, temperature, max_tokens).await
            },
            RoutingStrategy::LoadBalanced => {
                anyhow::bail!("LoadBalanced strategy not yet implemented")
            },
        }
    }

    /// LocalFirst: Tenta ml-offload → fallback SecureLLM
    async fn chat_local_first(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        // Try ml-offload first (baixa latência)
        if let Some(ml) = &self.ml_offload {
            match self.try_ml_offload(ml, prompt, temperature, max_tokens).await {
                Ok(response) => {
                    info!("Response from ml-offload (local)");
                    return Ok(response);
                },
                Err(e) => {
                    warn!("ml-offload failed: {}, trying SecureLLM fallback", e);
                },
            }
        }

        // Fallback to SecureLLM (external, audited)
        if let Some(sec) = &self.securellm {
            let response = sec.send_secure(prompt).await.context("SecureLLM fallback failed")?;
            info!(provider = sec.provider(), "Response from SecureLLM (external)");
            return Ok(response);
        }

        anyhow::bail!("All LLM backends failed")
    }

    /// ExternalFirst: Tenta SecureLLM → fallback ml-offload
    async fn chat_external_first(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        // Try SecureLLM first (máxima qualidade + audit)
        if let Some(sec) = &self.securellm {
            match sec.send_secure(prompt).await {
                Ok(response) => {
                    info!(provider = sec.provider(), "Response from SecureLLM (external)");
                    return Ok(response);
                },
                Err(e) => {
                    warn!("SecureLLM failed: {}, trying ml-offload fallback", e);
                },
            }
        }

        // Fallback to ml-offload (local, fast)
        if let Some(ml) = &self.ml_offload {
            let response = self
                .try_ml_offload(ml, prompt, temperature, max_tokens)
                .await
                .context("ml-offload fallback failed")?;
            info!("Response from ml-offload (local)");
            return Ok(response);
        }

        anyhow::bail!("All LLM backends failed")
    }

    /// Helper: Try ml-offload API
    async fn try_ml_offload(
        &self,
        ml: &MLOffloadClient,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let request = ChatCompletionRequest {
            model: "auto".into(), // ml-offload selects best backend
            messages: vec![ChatMessage { role: "user".into(), content: prompt.to_string() }],
            temperature,
            max_tokens,
            stream: None,
            stop: None,
            top_p: None,
        };

        let response = ml.chat_completion(request).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            anyhow::bail!("Empty response from ml-offload")
        }
    }

    /// Health check for both backends
    pub async fn health_check(&self) -> HealthStatus {
        let ml_healthy = if let Some(ml) = &self.ml_offload {
            ml.health().await.is_ok()
        } else {
            false
        };

        let sec_healthy = if let Some(sec) = &self.securellm {
            sec.health_check().await.unwrap_or(false)
        } else {
            false
        };

        HealthStatus { ml_offload: ml_healthy, securellm: sec_healthy }
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub ml_offload: bool,
    pub securellm: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_first_creation() {
        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let result = UnifiedLLMClient::new_local_first(
            "http://localhost:9000".to_string(),
            secrets_manager,
            None, // No SecureLLM
        )
        .await;

        // Should succeed if ml-offload client can be created
        if let Ok(client) = result {
            assert_eq!(client.strategy, RoutingStrategy::LocalFirst);
        }
    }
}
