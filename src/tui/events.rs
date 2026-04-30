// Key processing result — returned by process_key in mod.rs
pub enum Action {
    None,
    Quit,
    Send(String),       // legacy LLM chat path (kept for fallback)
    SubmitTask(String), // agent workstation: POST /v1/agents/task
    QueueTask(String),  // queue a follow-up task while another is running
    SteerTask(String),  // agent workstation: POST /v1/agents/session/:id/steer
    TogglePipeline,     // ^p
    OpenMatrix,         // ^m
    CancelTask,         // ^x
    FocusNextPanel,     // tab
    FocusPrevPanel,     // shift+tab
    ToggleLlamaManager, // ^l
    LlamaManagerRun,    // r or enter
    LlamaManagerStop,   // s or ctrl+c
    ResolveBreakpoint { resolution: String, instruction: Option<String> }, // Enter/Esc during Breakpoint
}
