// ML Offload API - OpenAI-Compatible Data Models
use serde::{Deserialize, Serialize};

// =============================================================================
// Chat Completions (OpenAI Compatible)
// =============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// =============================================================================
// Models List (OpenAI Compatible)
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsListResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

// =============================================================================
// Batch Processing (TensorForge API)
// =============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct BatchRequest {
    pub requests: Vec<ChatCompletionRequest>,
    pub priority: String, // e.g. "high", "normal"
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchResponse {
    pub batch_id: String,
    pub status: String,
}

// =============================================================================
// WebSocket Telemetry (TensorForge API)
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum TensorForgeWsEvent {
    #[serde(rename = "vram_update")]
    VramUpdate { free_gb: f64 },
    #[serde(rename = "inference_complete")]
    InferenceComplete { request_id: String },
}

// =============================================================================
// Health & Status
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    #[serde(default)]
    pub uptime_seconds: u64,
    #[serde(default)]
    pub active_backends: u32,
}
