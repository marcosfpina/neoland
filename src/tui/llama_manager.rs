use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct LlamaFlags {
    pub flash_attn: bool,
    pub no_kv_offload: bool,
    pub cont_batching: bool,
    pub embeddings: bool,
    pub metrics: bool,
    pub ctx_size: String,
    pub gpu_layers: String,
    pub threads: String,
    pub port: String,
}

impl Default for LlamaFlags {
    fn default() -> Self {
        Self {
            flash_attn: true,
            no_kv_offload: true,
            cont_batching: true,
            embeddings: true,
            metrics: true,
            ctx_size: "8192".into(),
            gpu_layers: "40".into(),
            threads: "12".into(),
            port: "8081".into(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum LlamaManagerPhase {
    Browsing,
    Configuring,
    Running,
}

#[derive(Clone)]
pub struct LlamaManagerState {
    pub phase: LlamaManagerPhase,
    pub available_models: Vec<String>,
    pub selected_index: usize,
    pub flags: LlamaFlags,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub is_running: bool,
    pub child_process: Arc<Mutex<Option<Child>>>,
}

impl Default for LlamaManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl LlamaManagerState {
    pub fn new() -> Self {
        Self {
            phase: LlamaManagerPhase::Browsing,
            available_models: Vec::new(),
            selected_index: 0,
            flags: LlamaFlags::default(),
            logs: Arc::new(Mutex::new(Vec::new())),
            is_running: false,
            child_process: Arc::new(Mutex::new(None)),
        }
    }
}
