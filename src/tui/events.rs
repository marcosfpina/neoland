/// Key processing result — returned by process_key in mod.rs
use super::commands::Command;

#[derive(Debug)]
pub enum Action {
    None,
    Quit,
    Send(String),       // legacy LLM chat path (kept for fallback)
    SubmitTask(String), // agent workstation: POST /v1/agents/task
    QueueTask(String),  // queue a follow-up task while another is running
    SteerTask(String),  // agent workstation: POST /v1/agents/session/:id/steer
    TogglePipeline,     // ^p
    CancelTask,         // ^x
    CycleProvider,      // ^6 — cycle through deepseek/gemini/groq/llamacpp/local
    FocusNextPanel,     // tab
    FocusPrevPanel,     // shift+tab
    ResolveBreakpoint {
        resolution: String,
        instruction: Option<String>,
    }, /* Enter/Esc during
                         * Breakpoint */
    /// Execute a parsed `/` command.
    ExecuteCommand(Command),
}
