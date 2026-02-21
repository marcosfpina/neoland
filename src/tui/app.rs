// TUI App State
use chrono::{DateTime, Utc};

use super::presets::QueryConfig;

#[derive(Clone, PartialEq)]
pub enum ConnectionStatus {
    /// Health check not yet performed
    Unknown,
    /// Server is reachable and healthy
    Connected,
    /// Server responded but with degraded health
    Degraded,
    /// Server is not reachable
    Offline,
}

#[derive(Clone)]
pub struct AppState {
    pub messages: Vec<ChatMessage>,
    pub input_buffer: String,
    pub config: QueryConfig,
    pub sidebar_visible: bool,
    pub server_url: String,
    pub ml_api_url: String,
    pub scroll_offset: usize,
    pub is_thinking: bool,
    /// Server connectivity status (updated on health check + each request)
    pub connection_status: ConnectionStatus,
    /// Name of the last backend that successfully responded
    pub active_backend: Option<String>,
    /// Cumulative output tokens in this session
    pub session_tokens: u64,
    /// Latency (ms) of the last completed request
    pub last_latency_ms: u64,
    /// Partial assistant message being streamed (token-by-token accumulation)
    pub pending_message: Option<String>,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl AppState {
    pub fn new(server_url: String, ml_api_url: String) -> Self {
        Self {
            messages: vec![ChatMessage {
                role: MessageRole::System,
                content: format!("Neoland TUI | Checking server at {}...", server_url),
                timestamp: Utc::now(),
            }],
            input_buffer: String::new(),
            config: QueryConfig::default(),
            sidebar_visible: true,
            server_url,
            ml_api_url,
            scroll_offset: 0,
            is_thinking: false,
            connection_status: ConnectionStatus::Unknown,
            active_backend: None,
            session_tokens: 0,
            last_latency_ms: 0,
            pending_message: None,
        }
    }

    pub fn apply_preset(&mut self, preset_name: &str) {
        self.config = match preset_name {
            "balanced" => QueryConfig::balanced(),
            "creative" => QueryConfig::creative(),
            "precise" => QueryConfig::precise(),
            "research" => QueryConfig::research(),
            "safe" => QueryConfig::safe(),
            _ => QueryConfig::default(),
        };
        self.add_system_message(&format!("✅ Preset aplicado: {}", preset_name.to_uppercase()));
    }

    pub fn add_system_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }
}
