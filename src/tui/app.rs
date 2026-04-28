use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::llama_manager::LlamaManagerState;
use super::presets::QueryConfig;

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Workstation,
    LlamaManager,
}

// ── Agent workstation types ───────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum TaskStatus {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq)]
pub enum StageStatus {
    Pending,
    Running,
    Done { latency_ms: u64 },
    Skipped,
    Failed,
}

#[derive(Clone)]
pub struct PipelineStage {
    pub name: &'static str,
    pub status: StageStatus,
    pub confidence: Option<f32>,
    pub output: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum ToolStatus {
    Running,
    Done { duration_ms: u64 },
    Failed,
}

#[derive(Clone)]
pub struct ToolCall {
    pub name: String,
    pub args_summary: String,
    pub status: ToolStatus,
}

#[derive(Clone, PartialEq)]
pub enum Panel {
    Tasks,
    Pipeline,
    Output,
}

#[derive(Clone, PartialEq)]
pub enum ConnectionStatus {
    Unknown,
    Connected,
    Degraded,
    Offline,
}

#[derive(Clone)]
pub struct AppState {
    // ── Legacy chat fields (kept for fallback) ────────────────────────
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
    // ── Agent workstation ─────────────────────────────────────────────
    pub tasks: Vec<Task>,
    pub pipeline_stages: Vec<PipelineStage>,
    pub tool_calls: Vec<ToolCall>,
    pub active_task_id: Option<Uuid>,
    pub output_text: String,
    pub pipeline_visible: bool,
    pub focused_panel: Panel,
    pub adr_title: Option<String>,
    pub adr_status: Option<String>,
    pub active_session: Uuid,
    pub input_history: Vec<String>,
    pub history_idx: Option<usize>,
    pub mode: AppMode,
    pub llama_state: LlamaManagerState,
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
            tasks: Vec::new(),
            pipeline_stages: Vec::new(),
            tool_calls: Vec::new(),
            active_task_id: None,
            output_text: String::new(),
            pipeline_visible: true,
            focused_panel: Panel::Output,
            adr_title: None,
            adr_status: None,
            active_session: Uuid::new_v4(),
            input_history: Vec::new(),
            history_idx: None,
            mode: AppMode::Workstation,
            llama_state: LlamaManagerState::new(),
        }
    }

    // ── Agent workstation methods ─────────────────────────────────────

    pub fn enqueue_task(&mut self, description: String) -> Uuid {
        let id = Uuid::new_v4();
        self.tasks.push(Task {
            id,
            description,
            status: TaskStatus::Queued,
            created_at: Utc::now(),
        });
        id
    }

    pub fn start_task(&mut self, id: Uuid) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.status = TaskStatus::Running;
        }
        self.active_task_id = Some(id);
        self.pipeline_stages = vec![
            PipelineStage {
                name: "junior",
                status: StageStatus::Pending,
                confidence: None,
                output: None,
            },
            PipelineStage {
                name: "senior",
                status: StageStatus::Pending,
                confidence: None,
                output: None,
            },
            PipelineStage {
                name: "architect",
                status: StageStatus::Pending,
                confidence: None,
                output: None,
            },
            PipelineStage {
                name: "tech-leader",
                status: StageStatus::Pending,
                confidence: None,
                output: None,
            },
        ];
        self.tool_calls.clear();
        self.output_text.clear();
        self.adr_title = None;
        self.adr_status = None;
        self.auto_scroll = true;
    }

    pub fn update_stage(&mut self, name: &str, status: StageStatus, confidence: Option<f32>) {
        let normalized = name.replace('_', "-");
        if let Some(s) = self.pipeline_stages.iter_mut().find(|s| s.name == normalized) {
            s.status = status;
            if confidence.is_some() {
                s.confidence = confidence;
            }
        }
    }

    pub fn set_stage_output(&mut self, stage: &str, content: String) {
        let normalized = stage.replace('_', "-");
        if let Some(s) = self.pipeline_stages.iter_mut().find(|s| s.name == normalized) {
            s.output = Some(content);
        }
    }

    pub fn history_prev(&mut self) {
        if self.input_history.is_empty() {
            return;
        }
        match self.history_idx {
            None => {
                let idx = self.input_history.len() - 1;
                self.history_idx = Some(idx);
                self.input_buffer = self.input_history[idx].clone();
                self.cursor_pos = self.input_buffer.len();
            },
            Some(0) => {},
            Some(idx) => {
                let new_idx = idx - 1;
                self.history_idx = Some(new_idx);
                self.input_buffer = self.input_history[new_idx].clone();
                self.cursor_pos = self.input_buffer.len();
            },
        }
    }

    pub fn history_next(&mut self) {
        match self.history_idx {
            None => {},
            Some(idx) if idx + 1 >= self.input_history.len() => {
                self.history_idx = None;
                self.input_buffer.clear();
                self.cursor_pos = 0;
            },
            Some(idx) => {
                let new_idx = idx + 1;
                self.history_idx = Some(new_idx);
                self.input_buffer = self.input_history[new_idx].clone();
                self.cursor_pos = self.input_buffer.len();
            },
        }
    }

    pub fn history_commit(&mut self, text: String) {
        if !text.is_empty() && self.input_history.last().map(|s| s.as_str()) != Some(&text) {
            self.input_history.push(text);
        }
        self.history_idx = None;
    }

    pub fn complete_task(&mut self, id: Uuid, ok: bool) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.status = if ok {
                TaskStatus::Done
            } else {
                TaskStatus::Failed
            };
        }
        if self.active_task_id == Some(id) {
            self.active_task_id = None;
        }
    }

    pub fn add_tool_call(&mut self, name: String, args_summary: String) {
        self.tool_calls
            .push(ToolCall { name, args_summary, status: ToolStatus::Running });
    }

    pub fn finish_tool_call(&mut self, name: &str, duration_ms: u64) {
        if let Some(t) = self
            .tool_calls
            .iter_mut()
            .rev()
            .find(|t| t.name == name && t.status == ToolStatus::Running)
        {
            t.status = ToolStatus::Done { duration_ms };
        }
    }

    pub fn fail_tool_call(&mut self, name: &str) {
        if let Some(t) = self
            .tool_calls
            .iter_mut()
            .rev()
            .find(|t| t.name == name && t.status == ToolStatus::Running)
        {
            t.status = ToolStatus::Failed;
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

// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn app_with(buf: &str, cursor: usize) -> AppState {
        let mut a = AppState::new("http://localhost:3000".into(), "http://localhost:8001".into());
        a.input_buffer = buf.to_owned();
        a.cursor_pos = cursor;
        a
    }

    // ── cursor_left ──────────────────────────────────────────────────────────

    #[test]
    fn cursor_left_from_end() {
        let mut a = app_with("hello", 5);
        a.cursor_left();
        assert_eq!(a.cursor_pos, 4);
    }

    #[test]
    fn cursor_left_clamps_at_zero() {
        let mut a = app_with("hi", 0);
        a.cursor_left();
        assert_eq!(a.cursor_pos, 0);
    }

    #[test]
    fn cursor_left_multibyte() {
        // "é" = 2 bytes (U+00E9)
        let mut a = app_with("aé", 3); // cursor after "é"
        a.cursor_left();
        assert_eq!(a.cursor_pos, 1); // now after "a", before "é"
    }

    // ── cursor_right ─────────────────────────────────────────────────────────

    #[test]
    fn cursor_right_from_start() {
        let mut a = app_with("hello", 0);
        a.cursor_right();
        assert_eq!(a.cursor_pos, 1);
    }

    #[test]
    fn cursor_right_clamps_at_end() {
        let mut a = app_with("hi", 2);
        a.cursor_right();
        assert_eq!(a.cursor_pos, 2);
    }

    #[test]
    fn cursor_right_multibyte() {
        // "中" = 3 bytes (U+4E2D)
        let mut a = app_with("中x", 0);
        a.cursor_right();
        assert_eq!(a.cursor_pos, 3);
        a.cursor_right();
        assert_eq!(a.cursor_pos, 4);
    }

    // ── cursor_word_left / right ──────────────────────────────────────────────

    #[test]
    fn cursor_word_left_simple() {
        let mut a = app_with("foo bar", 7); // end
        a.cursor_word_left();
        assert_eq!(a.cursor_pos, 4); // start of "bar"
    }

    #[test]
    fn cursor_word_left_trailing_spaces() {
        let mut a = app_with("foo   ", 6);
        a.cursor_word_left();
        assert_eq!(a.cursor_pos, 0); // skips spaces, then "foo" → 0
    }

    #[test]
    fn cursor_word_left_at_start() {
        let mut a = app_with("word", 4);
        a.cursor_word_left();
        assert_eq!(a.cursor_pos, 0);
    }

    #[test]
    fn cursor_word_right_simple() {
        let mut a = app_with("foo bar", 0);
        a.cursor_word_right();
        assert_eq!(a.cursor_pos, 4); // after "foo " → start of "bar"
    }

    #[test]
    fn cursor_word_right_at_end() {
        let mut a = app_with("foo", 3);
        a.cursor_word_right();
        assert_eq!(a.cursor_pos, 3);
    }

    // ── insert_char ──────────────────────────────────────────────────────────

    #[test]
    fn insert_at_end() {
        let mut a = app_with("hi", 2);
        a.insert_char('!');
        assert_eq!(a.input_buffer, "hi!");
        assert_eq!(a.cursor_pos, 3);
    }

    #[test]
    fn insert_at_middle() {
        let mut a = app_with("hllo", 1);
        a.insert_char('e');
        assert_eq!(a.input_buffer, "hello");
        assert_eq!(a.cursor_pos, 2);
    }

    #[test]
    fn insert_multibyte() {
        let mut a = app_with("ac", 1);
        a.insert_char('é');
        assert_eq!(a.input_buffer, "aéc");
        assert_eq!(a.cursor_pos, 3); // 1 + 2 bytes for é
    }

    // ── backspace ────────────────────────────────────────────────────────────

    #[test]
    fn backspace_deletes_previous_char() {
        let mut a = app_with("hello", 5);
        a.backspace();
        assert_eq!(a.input_buffer, "hell");
        assert_eq!(a.cursor_pos, 4);
    }

    #[test]
    fn backspace_at_zero_is_noop() {
        let mut a = app_with("hi", 0);
        a.backspace();
        assert_eq!(a.input_buffer, "hi");
        assert_eq!(a.cursor_pos, 0);
    }

    #[test]
    fn backspace_multibyte() {
        let mut a = app_with("aé", 3); // cursor after "é" (byte 3)
        a.backspace();
        assert_eq!(a.input_buffer, "a");
        assert_eq!(a.cursor_pos, 1);
    }

    #[test]
    fn backspace_at_middle() {
        let mut a = app_with("hello", 3); // cursor after 'l' (index 3)
        a.backspace();
        assert_eq!(a.input_buffer, "helo");
        assert_eq!(a.cursor_pos, 2);
    }

    // ── delete_word_back ─────────────────────────────────────────────────────

    #[test]
    fn delete_word_back_single_word() {
        let mut a = app_with("hello", 5);
        a.delete_word_back();
        assert_eq!(a.input_buffer, "");
        assert_eq!(a.cursor_pos, 0);
    }

    #[test]
    fn delete_word_back_last_word() {
        let mut a = app_with("foo bar", 7);
        a.delete_word_back();
        assert_eq!(a.input_buffer, "foo ");
        assert_eq!(a.cursor_pos, 4);
    }

    #[test]
    fn delete_word_back_trailing_spaces() {
        let mut a = app_with("foo   ", 6);
        a.delete_word_back();
        assert_eq!(a.input_buffer, "");
        assert_eq!(a.cursor_pos, 0);
    }

    #[test]
    fn delete_word_back_at_zero_is_noop() {
        let mut a = app_with("hi", 0);
        a.delete_word_back();
        assert_eq!(a.input_buffer, "hi");
        assert_eq!(a.cursor_pos, 0);
    }

    // ── auto_scroll behaviour ─────────────────────────────────────────────────

    #[test]
    fn add_system_message_sets_auto_scroll() {
        let mut a = AppState::new("u".into(), "m".into());
        a.auto_scroll = false;
        a.add_system_message("info");
        assert!(a.auto_scroll);
        assert_eq!(a.messages.len(), 1);
    }

    #[test]
    fn add_assistant_message_sets_auto_scroll() {
        let mut a = AppState::new("u".into(), "m".into());
        a.auto_scroll = false;
        a.add_assistant_message("response");
        assert!(a.auto_scroll);
    }

    #[test]
    fn add_user_message_does_not_change_auto_scroll() {
        let mut a = AppState::new("u".into(), "m".into());
        a.auto_scroll = false;
        a.add_user_message("prompt");
        assert!(!a.auto_scroll);
    }

    // ── apply_preset ──────────────────────────────────────────────────────────

    #[test]
    fn apply_preset_balanced() {
        let mut a = AppState::new("u".into(), "m".into());
        a.apply_preset("balanced");
        assert!((a.config.temperature - 0.7).abs() < f32::EPSILON);
        assert_eq!(a.config.max_tokens, 600);
    }

    #[test]
    fn apply_preset_creative_sets_high_temp() {
        let mut a = AppState::new("u".into(), "m".into());
        a.apply_preset("creative");
        assert!(a.config.temperature > 1.0);
    }

    #[test]
    fn apply_preset_safe_disables_commands() {
        let mut a = AppState::new("u".into(), "m".into());
        a.apply_preset("safe");
        assert!(!a.config.enable_commands);
    }

    #[test]
    fn apply_preset_unknown_falls_back_to_default() {
        let mut a = AppState::new("u".into(), "m".into());
        a.apply_preset("unknown_preset");
        assert!((a.config.temperature - 0.7).abs() < f32::EPSILON);
    }

    // ── set_stage_output ──────────────────────────────────────────────────────

    #[test]
    fn set_stage_output_updates_matching_stage() {
        let mut a = AppState::new("u".into(), "m".into());
        a.start_task(Uuid::new_v4());
        a.set_stage_output("junior", "hypothesis text".into());
        let s = a.pipeline_stages.iter().find(|s| s.name == "junior").unwrap();
        assert_eq!(s.output.as_deref(), Some("hypothesis text"));
    }

    #[test]
    fn set_stage_output_normalizes_underscore() {
        let mut a = AppState::new("u".into(), "m".into());
        a.start_task(Uuid::new_v4());
        a.set_stage_output("tech_leader", "rationale".into());
        let s = a.pipeline_stages.iter().find(|s| s.name == "tech-leader").unwrap();
        assert_eq!(s.output.as_deref(), Some("rationale"));
    }

    // ── history navigation ────────────────────────────────────────────────────

    #[test]
    fn history_commit_and_prev() {
        let mut a = AppState::new("u".into(), "m".into());
        a.history_commit("first task".into());
        a.history_commit("second task".into());
        a.history_prev();
        assert_eq!(a.input_buffer, "second task");
        a.history_prev();
        assert_eq!(a.input_buffer, "first task");
    }

    #[test]
    fn history_next_clears_buffer() {
        let mut a = AppState::new("u".into(), "m".into());
        a.history_commit("task one".into());
        a.history_prev();
        a.history_next();
        assert_eq!(a.history_idx, None);
        assert_eq!(a.input_buffer, "");
    }

    #[test]
    fn history_commit_deduplicates_consecutive() {
        let mut a = AppState::new("u".into(), "m".into());
        a.history_commit("same".into());
        a.history_commit("same".into());
        assert_eq!(a.input_history.len(), 1);
    }

    #[test]
    fn history_prev_on_empty_is_noop() {
        let mut a = AppState::new("u".into(), "m".into());
        a.history_prev();
        assert_eq!(a.history_idx, None);
    }
}
