//! vLLM OpenAI-compatible client.
//!
//! Communicates with a vLLM inference server via its `/v1/chat/completions`
//! endpoint. vLLM exposes an OpenAI-compatible REST API, making it a drop-in
//! replacement for high-throughput GPU-accelerated inference.
//!
//! Configurable via:
//!   - `NEOLAND_VLLM_URL`  (default: http://localhost:8000)
//!   - `NEOLAND_VLLM_MODEL` (default: the model loaded by the vLLM server)
//!
//! v0.0.1 #6 — Optional vLLM backend alongside llama.cpp

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

// ── Request / Response types (OpenAI-compatible subset) ───────────────────

#[derive(Debug, Clone, Serialize)]
struct VllmChatRequest {
    model: String,
    messages: Vec<VllmMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VllmMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct VllmChatResponse {
    choices: Vec<VllmChoice>,
    #[allow(dead_code)]
    usage: Option<VllmUsage>,
}

#[derive(Debug, Deserialize)]
struct VllmChoice {
    message: VllmMessage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct VllmUsage {
    total_tokens: u32,
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct VllmModelList {
    data: Vec<VllmModelEntry>,
}

#[derive(Debug, Deserialize)]
struct VllmModelEntry {
    id: String,
}

// ── Client ────────────────────────────────────────────────────────────────

/// OpenAI-compatible client targeting a vLLM inference server.
///
/// Supports the standard `/v1/chat/completions` endpoint plus vLLM-specific
/// health checks at `/health`.
#[derive(Debug, Clone)]
pub struct VllmClient {
    /// Base URL (e.g. `http://localhost:8000`) — no trailing slash.
    base_url: String,
    /// Model name to pass in requests (e.g. `meta-llama/Llama-3.1-8B-Instruct`).
    model: String,
    /// Reusable HTTP client (connection pool, timeouts).
    http: Client,
}

impl VllmClient {
    /// Create a new vLLM client.
    ///
    /// `base_url` should be the root of the vLLM server, e.g.
    /// `http://gpu-node-1:8000`. The `/v1/chat/completions` path is appended
    /// automatically.
    pub fn new(base_url: String, model: String) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("vLLM HTTP client builder");

        Self { base_url: base_url.trim_end_matches('/').to_string(), model, http }
    }

    /// Send a chat completion request and return the assistant's text response.
    ///
    /// This is a **synchronous** (non-streaming) call — best suited for
    /// agent pipelines where the full response is needed before proceeding.
    pub async fn chat(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let mut messages = Vec::with_capacity(2);

        if let Some(sys) = system_prompt {
            messages.push(VllmMessage { role: "system".to_string(), content: sys.to_string() });
        }
        messages.push(VllmMessage { role: "user".to_string(), content: prompt.to_string() });

        let request = VllmChatRequest {
            model: self.model.clone(),
            messages,
            temperature,
            max_tokens,
            top_p: None,
            stream: false,
        };

        let url = format!("{}/v1/chat/completions", self.base_url);

        debug!(
            url = %url,
            model = %self.model,
            prompt_len = prompt.len(),
            "Sending vLLM chat request"
        );

        let response: VllmChatResponse = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("vLLM HTTP request failed")?
            .error_for_status()
            .context("vLLM returned error status")?
            .json()
            .await
            .context("Failed to parse vLLM response JSON")?;

        let text = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("vLLM returned empty choices array"))?;

        info!(
            model = %self.model,
            response_len = text.len(),
            tokens = ?response.usage.as_ref().map(|u| u.total_tokens),
            "vLLM chat completed"
        );

        Ok(text)
    }

    /// Check whether the vLLM server is reachable and healthy.
    ///
    /// vLLM exposes `/health` which returns HTTP 200 when ready.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        match self.http.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
            Ok(resp) => {
                let healthy = resp.status().is_success();
                debug!(url = %url, healthy, "vLLM health check");
                Ok(healthy)
            },
            Err(e) => {
                warn!(url = %url, error = %e, "vLLM health check failed");
                Ok(false)
            },
        }
    }

    /// Discover available models from the vLLM server.
    ///
    /// Calls `/v1/models` (OpenAI-compatible) and returns the list of model IDs.
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/v1/models", self.base_url);
        let list: VllmModelList = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to list vLLM models")?
            .json()
            .await
            .context("Failed to parse vLLM model list")?;

        Ok(list.data.into_iter().map(|e| e.id).collect())
    }

    /// Return the configured model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Return the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction_strips_trailing_slash() {
        let client = VllmClient::new("http://localhost:8000/".into(), "test-model".into());
        assert_eq!(client.base_url(), "http://localhost:8000");
        assert_eq!(client.model(), "test-model");
    }

    #[test]
    fn test_client_preserves_custom_port() {
        let client = VllmClient::new("http://10.0.0.5:9999".into(), "llama-3".into());
        assert_eq!(client.base_url(), "http://10.0.0.5:9999");
    }

    #[test]
    fn test_client_base_url_no_trailing_slash() {
        let client = VllmClient::new("https://vllm.example.com".into(), "mistral".into());
        assert_eq!(client.base_url(), "https://vllm.example.com");
    }

    #[tokio::test]
    async fn test_health_check_no_server() {
        // Use a port unlikely to have anything listening
        let client = VllmClient::new("http://127.0.0.1:19999".into(), "model".into());
        let healthy = client.health_check().await.expect("health_check should not error");
        assert!(!healthy, "No server at port 19999, should be unhealthy");
    }
}
