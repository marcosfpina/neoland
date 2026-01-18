// QueryConfig presets - compartilhado entre GTK4 e TUI

#[derive(Clone)]
pub struct QueryConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub typical_p: f32,
    pub epsilon_cutoff: f32,
    pub eta_cutoff: f32,
    pub tail_free_sampling: f32,
    pub top_a: f32,
    pub max_tokens: i32,
    pub repetition_penalty: f32,
    pub context_top_k: i32,
    pub context_similarity_threshold: f32,
    pub disable_context: bool,
    pub system_prompt: String,
    pub enable_commands: bool,
    pub allowed_commands: String,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

impl QueryConfig {
    /// Preset: Balanced (Default)
    pub fn balanced() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            typical_p: 1.0,
            epsilon_cutoff: 0.0,
            eta_cutoff: 0.0,
            tail_free_sampling: 1.0,
            top_a: 0.0,
            max_tokens: 600,
            repetition_penalty: 1.1,
            context_top_k: 2,
            context_similarity_threshold: 0.3,
            disable_context: false,
            system_prompt: String::new(),
            enable_commands: true,
            allowed_commands: String::new(),
        }
    }

    /// Preset: Creative (High temperature, diverse sampling)
    pub fn creative() -> Self {
        Self {
            temperature: 1.5,
            top_p: 0.95,
            typical_p: 0.95,
            max_tokens: 800,
            context_top_k: 1,
            context_similarity_threshold: 0.4,
            ..Self::balanced()
        }
    }

    /// Preset: Precise (Low temperature, focused)
    pub fn precise() -> Self {
        Self {
            temperature: 0.3,
            top_p: 0.7,
            typical_p: 0.8,
            max_tokens: 400,
            repetition_penalty: 1.3,
            context_top_k: 5,
            context_similarity_threshold: 0.4,
            ..Self::balanced()
        }
    }

    /// Preset: Research (Maximum RAG context)
    pub fn research() -> Self {
        Self {
            temperature: 0.5,
            top_p: 0.85,
            typical_p: 0.9,
            max_tokens: 800,
            repetition_penalty: 1.2,
            context_top_k: 8,
            context_similarity_threshold: 0.2,
            ..Self::balanced()
        }
    }

    /// Preset: Safe (No commands, moderate)
    pub fn safe() -> Self {
        Self {
            temperature: 0.6,
            top_p: 0.85,
            max_tokens: 500,
            context_top_k: 3,
            enable_commands: false,
            ..Self::balanced()
        }
    }
}
