// Key processing result — returned by process_key in mod.rs
pub enum Action {
    None,
    Quit,
    Send(String),       // legacy LLM chat path (kept for fallback)
    SubmitTask(String), // agent workstation: POST /v1/agents/task
    SteerTask(String),  // agent workstation: POST /v1/agents/session/:id/steer
    TogglePipeline,     // ^p
    OpenMatrix,         // ^m
    CancelTask,         // ^x
    ToggleLlamaManager, // ^l
    LlamaManagerRun,    // r or enter
    LlamaManagerStop,   // s or ctrl+c
}
