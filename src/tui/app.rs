use chrono::{DateTime, Utc};
use std::time::Instant;
use uuid::Uuid;

use super::presets::QueryConfig;

// ── Theme selection ────────────────────────────────────────────────────
// The enum lives here (plain data); the concrete `Palette` (ratatui colors)
// and the per-theme mapping live in `ui.rs` (`Theme::palette`).

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Theme {
    TokyoNight,
    NeonGlass,
    HighContrast,
    Monochrome,
}

impl Theme {
    pub const ALL: &'static [Theme] =
        &[Theme::TokyoNight, Theme::NeonGlass, Theme::HighContrast, Theme::Monochrome];

    /// Canonical kebab-case name (used by `/theme`, prefs persistence).
    pub fn label(&self) -> &'static str {
        match self {
            Theme::TokyoNight => "tokyo-night",
            Theme::NeonGlass => "neon-glass",
            Theme::HighContrast => "high-contrast",
            Theme::Monochrome => "monochrome",
        }
    }

    /// Parse a theme name, accepting a few aliases. Case-insensitive.
    pub fn from_label(s: &str) -> Option<Theme> {
        match s.trim().to_lowercase().replace([' ', '_'], "-").as_str() {
            "tokyo-night" | "tokyo" | "tokyonight" => Some(Theme::TokyoNight),
            "neon-glass" | "neon" | "glass" => Some(Theme::NeonGlass),
            "high-contrast" | "contrast" | "hc" => Some(Theme::HighContrast),
            "monochrome" | "mono" => Some(Theme::Monochrome),
            _ => None,
        }
    }
}

// ── Streaming mode ─────────────────────────────────────────────────────
// How the agent's response is revealed in the conversation. The reveal is a
// render-time animation over the already-received text, driven by the tick.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StreamMode {
    LineByLine,
    Typewriter,
    ThinkingReveal,
}

impl StreamMode {
    pub const ALL: &'static [StreamMode] =
        &[StreamMode::LineByLine, StreamMode::Typewriter, StreamMode::ThinkingReveal];

    pub fn label(&self) -> &'static str {
        match self {
            StreamMode::LineByLine => "line",
            StreamMode::Typewriter => "typewriter",
            StreamMode::ThinkingReveal => "thinking",
        }
    }

    pub fn from_label(s: &str) -> Option<StreamMode> {
        match s.trim().to_lowercase().replace([' ', '_'], "-").as_str() {
            "line" | "line-by-line" | "lines" => Some(StreamMode::LineByLine),
            "typewriter" | "type" | "typing" | "char" => Some(StreamMode::Typewriter),
            "thinking" | "thinking-reveal" | "reveal" | "think" => Some(StreamMode::ThinkingReveal),
            _ => None,
        }
    }
}

// ── LLM provider selection ─────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LlmProvider {
    Local, // ml-offload only, no external key needed
    Deepseek,
    Gemini,
    Groq,
    Llamacpp,
}

impl LlmProvider {
    pub const ALL: &'static [LlmProvider] = &[
        LlmProvider::Local,
        LlmProvider::Deepseek,
        LlmProvider::Gemini,
        LlmProvider::Groq,
        LlmProvider::Llamacpp,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            LlmProvider::Local => "local",
            LlmProvider::Deepseek => "deepseek",
            LlmProvider::Gemini => "gemini",
            LlmProvider::Groq => "groq",
            LlmProvider::Llamacpp => "llamacpp",
        }
    }

    pub fn next(&self) -> LlmProvider {
        let pos = Self::ALL.iter().position(|p| p == self).unwrap_or(0);
        Self::ALL[(pos + 1) % Self::ALL.len()]
    }
}

// ── Agent workstation types
// ───────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum TaskStatus {
    Queued,
    Running,
    Done,
    Failed,
    WaitingForBreakpoint,
}

#[derive(Clone)]
pub struct Task {
    pub id: Uuid,
    pub session_id: Uuid,
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

impl StageStatus {
    /// Static glyph for the stage status (Running uses a static arrow — the
    /// animated spinner is applied at render time, not here).
    pub fn icon(&self) -> &'static str {
        match self {
            StageStatus::Pending => "·",
            StageStatus::Running => "▸",
            StageStatus::Done { .. } => "󰄬",
            StageStatus::Skipped => "󰜎",
            StageStatus::Failed => "󰅖",
        }
    }

    /// Human-readable label for the stage status.
    pub fn label(&self) -> &'static str {
        match self {
            StageStatus::Pending => "pending",
            StageStatus::Running => "running",
            StageStatus::Done { .. } => "done",
            StageStatus::Skipped => "skipped",
            StageStatus::Failed => "failed",
        }
    }
}

/// Pool of character names drawn randomly at task start.
/// Internal stage keys ("junior", "senior", etc.) remain unchanged for SSE
/// matching; these nicknames are purely a display persona — no job titles.
pub const STAGE_NICKNAMES: &[&str] = &[
    "Vega", "Orion", "Nova", "Atlas", "Coda", "Wren", "Kael", "Mira", "Zeph", "Nox", "Flux",
    "Rook", "Drift", "Lux", "Sage", "Apex", "Void", "Echo", "Ghost", "Cipher", "Storm", "Root",
    "Lyra", "Dusk", "Fern", "Gale", "Haze", "Jade", "Fuse", "Crest", "Pike", "Rune", "Sable",
    "Thorn", "Vale", "Ward",
];

#[derive(Clone)]
pub struct PipelineStage {
    /// Internal key matching SSE event `stage` field (e.g. "junior").
    pub name: &'static str,
    /// Display persona — a random character name from `STAGE_NICKNAMES`.
    pub nickname: String,
    pub status: StageStatus,
    pub confidence: Option<f32>,
    pub output: Option<String>,
    /// Provenance anchor (RWA token / signed attestation ref) backing this
    /// stage's decision. `None` until the backend emits a chain anchor; the
    /// reasoning column shows `⛓ <ref>` when present.
    pub provenance: Option<String>,
    /// Wall-clock moment when this stage entered Running state — used to
    /// render elapsed time next to the spinner (e.g. "Vega ▸ 4.2s").
    pub started_at: Option<Instant>,
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Panel {
    Sessions,     // left sidebar
    Conversation, // center
    Reasoning,    // right
}

impl Panel {
    /// Stable index for per-panel state (e.g. scroll offsets).
    /// Order matches the left→center→right column layout.
    pub fn index(&self) -> usize {
        match self {
            Panel::Sessions => 0,
            Panel::Conversation => 1,
            Panel::Reasoning => 2,
        }
    }
}

// ── Sessions ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SessionStatus {
    Active,
    Done,
    Failed,
    Idle,
}

/// A persisted conversation. The active session's messages live in
/// `AppState.messages` (the working buffer) and are synced back here on
/// switch / persist.
#[derive(Clone)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<ChatMessage>,
    pub last_status: SessionStatus,
}

impl Session {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Session {
            id: Uuid::new_v4(),
            name,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            last_status: SessionStatus::Idle,
        }
    }
}

/// Default name given to a fresh session until the first task renames it.
pub const DEFAULT_SESSION_NAME: &str = "Nova sessão";

/// Pick 4 unique character nicknames from `STAGE_NICKNAMES` using the task
/// UUID bytes as a deterministic entropy source — no `rand` crate needed.
/// Same task → same names, so `/why` always shows the same cast.
fn pick_stage_nicknames(task_id: Uuid) -> [String; 4] {
    let bytes = task_id.as_bytes();
    let n = STAGE_NICKNAMES.len();
    let mut chosen: Vec<usize> = Vec::with_capacity(4);
    let mut b = 0usize;
    while chosen.len() < 4 {
        let idx = (bytes[b % 16] as usize ^ (b >> 4)) % n;
        if !chosen.contains(&idx) {
            chosen.push(idx);
        }
        b += 1;
    }
    [
        STAGE_NICKNAMES[chosen[0]].to_string(),
        STAGE_NICKNAMES[chosen[1]].to_string(),
        STAGE_NICKNAMES[chosen[2]].to_string(),
        STAGE_NICKNAMES[chosen[3]].to_string(),
    ]
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
    pub neoland_gateway_url: String,
    pub scroll_offsets: [u16; 3], // per-panel scroll, indexed by Panel::index()
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
    pub pending_breakpoint: Option<(String, String)>,
    pub adr_status: Option<String>,
    pub active_session: Uuid,
    pub input_history: Vec<String>,
    pub history_idx: Option<usize>,
    pub active_provider: LlmProvider,
    pub theme: Theme,
    // ── Streaming reveal ──────────────────────────────────────────────
    pub stream_mode: StreamMode,
    pub streaming: bool,
    pub stream_target: String,
    pub stream_revealed: usize, // chars of stream_target currently revealed
    pub show_help: bool,
    pub confirm_quit: bool,
    // ── Search ────────────────────────────────────────────────────────
    pub search_term: Option<String>,
    pub search_matches: Vec<usize>,
    pub search_idx: usize,
    // ── Notifications ─────────────────────────────────────────────────
    pub notifications: Vec<Notification>,
    // ── Sessions ──────────────────────────────────────────────────────
    pub sessions: Vec<Session>,
    pub active_session_idx: usize,
    /// When true, the reasoning panel renders the full `/why` chain instead of
    /// the live pipeline tree. Cleared by Esc or on new task start.
    pub why_mode: bool,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

// ── Notification system ──────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum NotificationLevel {
    Error,
    Warning,
    Info,
    Success,
}

#[derive(Clone)]
pub struct Notification {
    pub level: NotificationLevel,
    pub message: String,
    pub dismissable: bool,
    pub created_at: Instant,
}

impl AppState {
    pub fn new(server_url: String, neoland_gateway_url: String) -> Self {
        // Respect SECURELLM_PROVIDER env var as initial provider, default to local
        let active_provider = match std::env::var("SECURELLM_PROVIDER").as_deref() {
            Ok("deepseek") => LlmProvider::Deepseek,
            Ok("gemini") => LlmProvider::Gemini,
            Ok("groq") => LlmProvider::Groq,
            Ok("llamacpp") => LlmProvider::Llamacpp,
            _ => LlmProvider::Local,
        };

        // Load persisted preferences (theme, etc.) — defaults if absent.
        let prefs = super::prefs::load();
        let theme = prefs.theme.as_deref().and_then(Theme::from_label).unwrap_or(Theme::TokyoNight);
        let stream_mode = prefs
            .stream_mode
            .as_deref()
            .and_then(StreamMode::from_label)
            .unwrap_or(StreamMode::Typewriter);

        // Load persisted sessions (most-recent first); ensure at least one.
        let mut sessions = super::sessions::load_all();
        if sessions.is_empty() {
            sessions.push(Session::new(DEFAULT_SESSION_NAME.to_string()));
        }
        let messages = sessions[0].messages.clone();

        Self {
            messages,
            pending_message: None,
            input_buffer: String::new(),
            cursor_pos: 0,
            config: QueryConfig::default(),
            sidebar_visible: false,
            server_url,
            neoland_gateway_url,
            scroll_offsets: [0; 3],
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
            focused_panel: Panel::Conversation,
            adr_title: None,
            pending_breakpoint: None,
            adr_status: None,
            active_session: Uuid::new_v4(),
            input_history: Vec::new(),
            history_idx: None,
            active_provider,
            theme,
            stream_mode,
            streaming: false,
            stream_target: String::new(),
            stream_revealed: 0,
            show_help: false,
            confirm_quit: false,
            search_term: None,
            search_matches: Vec::new(),
            search_idx: 0,
            notifications: Vec::new(),
            sessions,
            active_session_idx: 0,
            why_mode: false,
        }
    }

    pub fn cycle_provider(&mut self) {
        self.active_provider = self.active_provider.next();
        self.add_system_message(&format!("provider → {}", self.active_provider.label()));
    }

    /// Switch theme and persist the preference.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.save_prefs();
    }

    /// Switch streaming mode and persist the preference.
    pub fn set_stream_mode(&mut self, mode: StreamMode) {
        self.stream_mode = mode;
        self.save_prefs();
    }

    /// Persist current preferences (theme, stream mode), merging into any
    /// existing file so unrelated keys are preserved.
    pub fn save_prefs(&self) {
        let mut prefs = super::prefs::load();
        prefs.theme = Some(self.theme.label().to_string());
        prefs.stream_mode = Some(self.stream_mode.label().to_string());
        super::prefs::save(&prefs);
    }

    // ── Streaming reveal ──────────────────────────────────────────────

    /// Begin revealing the agent's response. For `ThinkingReveal` the spinner
    /// already played during the pipeline, so the message is committed at once;
    /// otherwise an animated reveal starts (advanced by the tick loop).
    pub fn begin_stream(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        if matches!(self.stream_mode, StreamMode::ThinkingReveal) {
            self.add_assistant_message(&text);
            self.streaming = false;
            self.persist_active_session();
            return;
        }
        self.stream_target = text;
        self.stream_revealed = 0;
        self.streaming = true;
        self.auto_scroll = true;
    }

    /// Advance the reveal by one tick. Commits the message once fully revealed.
    pub fn advance_stream(&mut self) {
        if !self.streaming {
            return;
        }
        let chars: Vec<char> = self.stream_target.chars().collect();
        let total = chars.len();
        match self.stream_mode {
            StreamMode::Typewriter => {
                self.stream_revealed = (self.stream_revealed + 8).min(total);
            },
            StreamMode::LineByLine => {
                let mut i = self.stream_revealed;
                while i < total && chars[i] != '\n' {
                    i += 1;
                }
                if i < total {
                    i += 1; // consume the newline
                }
                self.stream_revealed = i;
            },
            StreamMode::ThinkingReveal => {
                self.stream_revealed = total;
            },
        }
        self.auto_scroll = true;
        if self.stream_revealed >= total {
            let text = std::mem::take(&mut self.stream_target);
            self.add_assistant_message(&text);
            self.streaming = false;
            self.stream_revealed = 0;
            self.persist_active_session();
        }
    }

    /// The portion of the streaming text revealed so far.
    pub fn streamed_text(&self) -> String {
        self.stream_target.chars().take(self.stream_revealed).collect()
    }

    /// True when a task is running and the user's message has no response yet
    /// (drives the "thinking" bubble in `ThinkingReveal` mode).
    pub fn awaiting_response(&self) -> bool {
        self.active_task_id.is_some()
            && !self.streaming
            && self.messages.last().map(|m| m.role == MessageRole::User).unwrap_or(false)
    }

    // ── Agent workstation methods ─────────────────────────────────────

    pub fn enqueue_task(&mut self, description: String) -> (Uuid, Uuid) {
        let id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        self.tasks.push(Task {
            id,
            session_id,
            description,
            status: TaskStatus::Queued,
            created_at: Utc::now(),
        });
        (id, session_id)
    }

    pub fn start_task(&mut self, id: Uuid) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.status = TaskStatus::Running;
            self.active_session = t.session_id;
        }
        self.active_task_id = Some(id);

        // Pick 4 unique character nicknames — user overrides from prefs take
        // precedence; random pool fills any unset slots.
        let prefs = super::prefs::load();
        let nicks = pick_stage_nicknames(id);
        let effective = |i: usize, random: &str| -> String {
            prefs.agent_names[i].clone().unwrap_or_else(|| random.to_string())
        };
        let mk = |name, nick: String| PipelineStage {
            name,
            nickname: nick.to_string(),
            status: StageStatus::Pending,
            confidence: None,
            output: None,
            provenance: None,
            started_at: None,
        };
        self.pipeline_stages = vec![
            mk("junior", effective(0, &nicks[0])),
            mk("senior", effective(1, &nicks[1])),
            mk("architect", effective(2, &nicks[2])),
            mk("tech-leader", effective(3, &nicks[3])),
        ];
        self.tool_calls.clear();
        self.output_text.clear();
        self.adr_title = None;
        self.adr_status = None;
        self.why_mode = false;
        self.auto_scroll = true;
    }

    pub fn update_stage(&mut self, name: &str, status: StageStatus, confidence: Option<f32>) {
        let normalized = name.replace('_', "-");
        if let Some(s) = self.pipeline_stages.iter_mut().find(|s| s.name == normalized) {
            if matches!(status, StageStatus::Running) && s.started_at.is_none() {
                s.started_at = Some(Instant::now());
            }
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

    /// Attach a provenance anchor (RWA token / attestation ref) to a stage.
    pub fn set_stage_provenance(&mut self, stage: &str, anchor: String) {
        let normalized = stage.replace('_', "-");
        if let Some(s) = self.pipeline_stages.iter_mut().find(|s| s.name == normalized) {
            s.provenance = Some(anchor);
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

    pub fn focus_next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Sessions => Panel::Conversation,
            Panel::Conversation => Panel::Reasoning,
            Panel::Reasoning => Panel::Sessions,
        };
    }

    pub fn focus_prev_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Sessions => Panel::Reasoning,
            Panel::Reasoning => Panel::Conversation,
            Panel::Conversation => Panel::Sessions,
        };
    }

    /// Scroll offset of the currently focused panel (mutable).
    pub fn focused_scroll_mut(&mut self) -> &mut u16 {
        &mut self.scroll_offsets[self.focused_panel.index()]
    }

    // ── Sessions ──────────────────────────────────────────────────────

    pub fn active_session(&self) -> &Session {
        &self.sessions[self.active_session_idx]
    }

    /// Mirror the live message buffer back into the active session record.
    fn sync_active_session(&mut self) {
        let idx = self.active_session_idx;
        if let Some(s) = self.sessions.get_mut(idx) {
            s.messages = self.messages.clone();
            s.updated_at = Utc::now();
        }
    }

    /// Sync the live buffer into the active session and persist it to disk.
    pub fn persist_active_session(&mut self) {
        self.sync_active_session();
        if let Some(s) = self.sessions.get(self.active_session_idx) {
            super::sessions::save(s);
        }
    }

    /// Start a fresh session (persisting the current one first).
    pub fn new_session(&mut self) {
        self.persist_active_session();
        self.sessions.insert(0, Session::new(DEFAULT_SESSION_NAME.to_string()));
        self.active_session_idx = 0;
        self.messages.clear();
        self.output_text.clear();
        self.pipeline_stages.clear();
        self.adr_title = None;
        self.adr_status = None;
        self.auto_scroll = true;
    }

    /// Switch to session `idx`, loading its messages into the live buffer.
    pub fn switch_session(&mut self, idx: usize) {
        if idx >= self.sessions.len() || idx == self.active_session_idx {
            return;
        }
        self.persist_active_session();
        self.active_session_idx = idx;
        self.messages = self.sessions[idx].messages.clone();
        self.auto_scroll = true;
    }

    pub fn select_session_next(&mut self) {
        if self.sessions.len() > 1 {
            let n = (self.active_session_idx + 1) % self.sessions.len();
            self.switch_session(n);
        }
    }

    pub fn select_session_prev(&mut self) {
        if self.sessions.len() > 1 {
            let n = if self.active_session_idx == 0 {
                self.sessions.len() - 1
            } else {
                self.active_session_idx - 1
            };
            self.switch_session(n);
        }
    }

    /// Name the active session from its first task description, if still unnamed.
    /// Truncates to 24 chars with a trailing "…" so the sidebar stays readable.
    pub fn name_active_session_from(&mut self, task: &str) {
        if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
            if s.name == DEFAULT_SESSION_NAME || s.name.is_empty() {
                let name: String = task.chars().take(24).collect();
                s.name = if task.chars().count() > 24 {
                    format!("{}…", name)
                } else {
                    name
                };
            }
        }
    }

    /// Set a custom nickname for pipeline slot `slot` (0-based).
    /// Persists to `prefs.json` so it survives restarts. Also renames the
    /// stage inline if the pipeline is currently active.
    pub fn set_agent_name(&mut self, slot: usize, name: String) {
        let mut prefs = super::prefs::load();
        if slot < 4 {
            prefs.agent_names[slot] = Some(name.clone());
            super::prefs::save(&prefs);
            if let Some(stage) = self.pipeline_stages.get_mut(slot) {
                stage.nickname = name;
            }
        }
    }

    /// Inject a compact task-completion summary into the conversation so the
    /// cadence has a clear milestone marker. Called after `PipelineDone`.
    pub fn inject_task_summary(&mut self) {
        let adr_part = match (&self.adr_status, &self.adr_title) {
            (Some(status), Some(title)) => format!("  ·  ADR {} — {}", status, title),
            (Some(status), None) => format!("  ·  ADR {}", status),
            _ => String::new(),
        };
        let avg_conf = {
            let done: Vec<f32> = self
                .pipeline_stages
                .iter()
                .filter_map(|s| {
                    if matches!(s.status, StageStatus::Done { .. }) {
                        s.confidence
                    } else {
                        None
                    }
                })
                .collect();
            if done.is_empty() {
                String::new()
            } else {
                format!("  ·  avg {:.0}% conf", done.iter().sum::<f32>() / done.len() as f32)
            }
        };
        let latency_part = if self.last_latency_ms > 0 {
            format!("  ·  {:.1}s", self.last_latency_ms as f32 / 1000.0)
        } else {
            String::new()
        };
        self.add_system_message(&format!(
            "✓  Pipeline concluído{}{}{}",
            adr_part, avg_conf, latency_part
        ));
    }

    /// Tab-complete a `/command` prefix in `input_buffer`. Returns `true` when
    /// the buffer was changed. On a single match, appends a trailing space ready
    /// for args. On multiple matches, cycles alphabetically and fires a
    /// notification listing candidates.
    pub fn tab_complete_command(&mut self) -> bool {
        if !self.input_buffer.starts_with('/') {
            return false;
        }
        let partial = self.input_buffer[1..].to_lowercase();
        if partial.contains(' ') {
            return false; // don't complete inside args
        }
        use crate::tui::commands::COMPLETABLE_COMMANDS;
        let matches: Vec<&str> = COMPLETABLE_COMMANDS
            .iter()
            .copied()
            .filter(|cmd| cmd.starts_with(partial.as_str()))
            .collect();
        match matches.len() {
            0 => false,
            1 => {
                self.input_buffer = format!("/{} ", matches[0]);
                self.cursor_pos = self.input_buffer.len();
                true
            },
            _ => {
                // Cycle: find next command alphabetically after current partial
                let next = matches
                    .iter()
                    .find(|&&cmd| cmd > partial.as_str())
                    .or_else(|| matches.first())
                    .copied()
                    .unwrap_or(matches[0]);
                self.input_buffer = format!("/{}", next);
                self.cursor_pos = self.input_buffer.len();
                let list = matches.join(", /");
                self.add_notification(
                    NotificationLevel::Info,
                    format!("{} matches: /{}", matches.len(), list),
                );
                true
            },
        }
    }

    pub fn set_active_session_status(&mut self, status: SessionStatus) {
        if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
            s.last_status = status;
        }
    }

    pub fn next_queued_task(&self) -> Option<(Uuid, Uuid, String)> {
        self.tasks
            .iter()
            .find(|t| t.status == TaskStatus::Queued)
            .map(|t| (t.id, t.session_id, t.description.clone()))
    }

    pub fn task_counts(&self) -> (usize, usize, usize, usize) {
        let mut queued = 0;
        let mut running = 0;
        let mut done = 0;
        let mut failed = 0;

        for task in &self.tasks {
            match task.status {
                TaskStatus::Queued => queued += 1,
                TaskStatus::Running => running += 1,
                TaskStatus::Done => done += 1,
                TaskStatus::Failed => failed += 1,
                TaskStatus::WaitingForBreakpoint => running += 1,
            }
        }

        (queued, running, done, failed)
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

    pub fn trigger_breakpoint(&mut self, tool: String, args: String) {
        self.pending_breakpoint = Some((tool, args));
        if let Some(id) = self.active_task_id {
            if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
                t.status = TaskStatus::WaitingForBreakpoint;
            }
        }
        // Clear input to prepare for [Y/N] or Steer
        self.input_buffer.clear();
        self.cursor_pos = 0;
    }

    // ── Notification methods ─────────────────────────────────────────

    /// Add a notification that auto-expires after 8 seconds.
    pub fn add_notification(&mut self, level: NotificationLevel, message: impl Into<String>) {
        self.notifications.push(Notification {
            level,
            message: message.into(),
            dismissable: true,
            created_at: Instant::now(),
        });
        // Keep at most 5 visible
        if self.notifications.len() > 5 {
            self.notifications.remove(0);
        }
    }

    /// Dismiss the oldest notification.
    pub fn dismiss_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.notifications.remove(0);
        }
    }

    /// Remove expired notifications (older than 8s).
    pub fn dismiss_expired_notifications(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|n| now.duration_since(n.created_at).as_secs() < 8);
    }

    // ── Search methods ───────────────────────────────────────────────

    /// Set search term and compute matching line indices in output_text.
    pub fn perform_search(&mut self, term: &str) {
        if term.is_empty() {
            self.clear_search();
            return;
        }
        let lower = term.to_lowercase();
        self.search_term = Some(term.to_string());
        self.search_matches = self
            .output_text
            .lines()
            .enumerate()
            .filter(|(_, line)| line.to_lowercase().contains(&lower))
            .map(|(i, _)| i)
            .collect();
        self.search_idx = 0;
    }

    /// Move to the next search match.
    pub fn next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_idx = (self.search_idx + 1) % self.search_matches.len();
    }

    /// Move to the previous search match.
    pub fn prev_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_idx = if self.search_idx == 0 {
            self.search_matches.len() - 1
        } else {
            self.search_idx - 1
        };
    }

    /// Clear search state.
    pub fn clear_search(&mut self) {
        self.search_term = None;
        self.search_matches.clear();
        self.search_idx = 0;
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

    // ── theme ─────────────────────────────────────────────────────────────────

    #[test]
    fn theme_from_label_accepts_aliases() {
        assert_eq!(Theme::from_label("neon-glass"), Some(Theme::NeonGlass));
        assert_eq!(Theme::from_label("Neon Glass"), Some(Theme::NeonGlass));
        assert_eq!(Theme::from_label("mono"), Some(Theme::Monochrome));
        assert_eq!(Theme::from_label("TOKYO"), Some(Theme::TokyoNight));
        assert_eq!(Theme::from_label("nope"), None);
    }

    #[test]
    fn theme_label_roundtrips() {
        for t in Theme::ALL {
            assert_eq!(Theme::from_label(t.label()), Some(*t));
        }
    }

    // ── streaming ───────────────────────────────────────────────────────────

    #[test]
    fn stream_mode_label_roundtrips() {
        for m in StreamMode::ALL {
            assert_eq!(StreamMode::from_label(m.label()), Some(*m));
        }
        assert_eq!(StreamMode::from_label("type"), Some(StreamMode::Typewriter));
        assert_eq!(StreamMode::from_label("line-by-line"), Some(StreamMode::LineByLine));
    }

    #[test]
    fn typewriter_reveals_then_commits() {
        let mut a = AppState::new("u".into(), "m".into());
        a.stream_mode = StreamMode::Typewriter;
        a.begin_stream("hello world".into()); // 11 chars
        assert!(a.streaming);
        // 11 chars / 8 per tick → 2 ticks to finish.
        a.advance_stream();
        assert_eq!(a.streamed_text(), "hello wo");
        a.advance_stream();
        assert!(!a.streaming, "should finish");
        assert_eq!(a.messages.last().unwrap().content, "hello world");
    }

    #[test]
    fn line_by_line_reveals_one_line_per_tick() {
        let mut a = AppState::new("u".into(), "m".into());
        a.stream_mode = StreamMode::LineByLine;
        a.begin_stream("a\nb\nc".into());
        a.advance_stream();
        assert_eq!(a.streamed_text(), "a\n");
        a.advance_stream();
        assert_eq!(a.streamed_text(), "a\nb\n");
        a.advance_stream();
        assert!(!a.streaming);
        assert_eq!(a.messages.last().unwrap().content, "a\nb\nc");
    }

    #[test]
    fn thinking_reveal_commits_immediately() {
        let mut a = AppState::new("u".into(), "m".into());
        a.stream_mode = StreamMode::ThinkingReveal;
        a.begin_stream("done".into());
        assert!(!a.streaming, "thinking mode commits at once");
        assert_eq!(a.messages.last().unwrap().content, "done");
    }

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
