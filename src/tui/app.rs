use chrono::{DateTime, Utc};

use super::presets::QueryConfig;

#[derive(Clone, PartialEq)]
pub enum ConnectionStatus {
    Unknown,
    Connected,
    Degraded,
    Offline,
}

#[derive(Clone)]
pub struct AppState {
    pub messages: Vec<ChatMessage>,
    pub pending_message: Option<String>,
    pub input_buffer: String,
    pub cursor_pos: usize,
    pub config: QueryConfig,
    pub sidebar_visible: bool,
    pub server_url: String,
    pub ml_api_url: String,
    pub scroll_offset: u16,
    pub auto_scroll: bool,
    pub is_thinking: bool,
    pub connection_status: ConnectionStatus,
    pub active_backend: Option<String>,
    pub session_tokens: u64,
    pub last_latency_ms: u64,
    pub tick: u8,
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
            messages: Vec::new(),
            pending_message: None,
            input_buffer: String::new(),
            cursor_pos: 0,
            config: QueryConfig::default(),
            sidebar_visible: false,
            server_url,
            ml_api_url,
            scroll_offset: 0,
            auto_scroll: true,
            is_thinking: false,
            connection_status: ConnectionStatus::Unknown,
            active_backend: None,
            session_tokens: 0,
            last_latency_ms: 0,
            tick: 0,
        }
    }

    pub fn apply_preset(&mut self, name: &str) {
        self.config = match name {
            "balanced" => QueryConfig::balanced(),
            "creative" => QueryConfig::creative(),
            "precise" => QueryConfig::precise(),
            "research" => QueryConfig::research(),
            "safe" => QueryConfig::safe(),
            _ => QueryConfig::default(),
        };
        self.add_system_message(&format!("preset → {}", name));
    }

    pub fn add_system_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: content.to_string(),
            timestamp: Utc::now(),
        });
        self.auto_scroll = true;
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
        self.auto_scroll = true;
    }

    // ── Cursor helpers (all byte-offset safe) ──────────────────────────

    pub fn cursor_left(&mut self) {
        self.cursor_pos = self.input_buffer[..self.cursor_pos]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
    }

    pub fn cursor_right(&mut self) {
        let rest = &self.input_buffer[self.cursor_pos..];
        self.cursor_pos += rest.char_indices().nth(1).map(|(i, _)| i).unwrap_or(rest.len());
    }

    pub fn cursor_word_left(&mut self) {
        let s = self.input_buffer[..self.cursor_pos].trim_end_matches(char::is_whitespace);
        self.cursor_pos = s.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
    }

    pub fn cursor_word_right(&mut self) {
        let rest = &self.input_buffer[self.cursor_pos..];
        let after_word = rest.trim_start_matches(|c: char| !c.is_whitespace());
        let after_space = after_word.trim_start_matches(char::is_whitespace);
        self.cursor_pos = self.input_buffer.len() - after_space.len();
    }

    pub fn insert_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let new_pos = self.input_buffer[..self.cursor_pos]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input_buffer.drain(new_pos..self.cursor_pos);
        self.cursor_pos = new_pos;
    }

    pub fn delete_word_back(&mut self) {
        let new_pos = {
            let s = self.input_buffer[..self.cursor_pos].trim_end_matches(char::is_whitespace);
            s.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0)
        };
        self.input_buffer.drain(new_pos..self.cursor_pos);
        self.cursor_pos = new_pos;
    }
}
