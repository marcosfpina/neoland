// ML Offload API HTTP Client (OpenAI-Compatible)
use super::models::*;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

pub struct MLOffloadClient {
    client: Client,
    base_url: String,
}

impl MLOffloadClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
    }

    /// Check if API is available
    pub async fn health(&self) -> Result<HealthStatus> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to ml-offload-api")?;

        response
            .json()
            .await
            .context("Failed to parse health response")
    }

    /// List all available models (OpenAI compatible)
    pub async fn list_models(&self) -> Result<ModelsListResponse> {
        let url = format!("{}/v1/models", self.base_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }

    /// Perform chat completion (OpenAI compatible)
    pub async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send chat completion request")?;

        response
            .json()
            .await
            .context("Failed to parse chat completion response")
    }

    /// Perform streaming chat completion (SSE)
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
        mut callback: impl FnMut(String),
    ) -> Result<()> {
        request.stream = Some(true);
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        let response = self.client.post(&url).json(&request).send().await?;

        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk).to_string();
            
            // Parse SSE format: "data: {json}\n\n"
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let json_str = &line[6..]; // Skip "data: "
                    if json_str == "[DONE]" {
                        break;
                    }
                    
                    // Parse chunk and extract content
                    if let Ok(chunk_data) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(content) = chunk_data["choices"][0]["delta"]["content"].as_str() {
                            callback(content.to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requer ml-offload-api rodando
    async fn test_health_check() {
        let client = MLOffloadClient::new("http://localhost:9000".into());
        let health = client.health().await;
        assert!(health.is_ok(), "ml-offload-api should be healthy");
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_models() {
        let client = MLOffloadClient::new("http://localhost:9000".into());
        let models = client.list_models().await;
        assert!(models.is_ok(), "Should list models");
    }

    #[tokio::test]
    #[ignore]
    async fn test_chat_completion() {
        let client = MLOffloadClient::new("http://localhost:9000".into());
        let request = ChatCompletionRequest {
            model: "auto".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "Hello!".into(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: None,
            stop: None,
            top_p: None,
        };
        
        let response = client.chat_completion(request).await;
        assert!(response.is_ok(), "Should complete chat");
    }
}
