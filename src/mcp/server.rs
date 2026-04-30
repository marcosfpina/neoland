use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot, RwLock};
use serde_json::Value;
use anyhow::{Result, bail};

use super::types::{Tool, CallToolResult};
use uuid::Uuid;

#[derive(Debug)]
pub enum BreakpointResolution {
    Approve,
    Reject,
    Steer(String),
}

#[derive(Debug)]
pub struct BreakpointRequest {
    pub session_id: Uuid,
    pub tool_name: String,
    pub args_summary: String,
    pub resolve_tx: oneshot::Sender<BreakpointResolution>,
}

#[async_trait::async_trait]
pub trait NativeTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    
    /// The core execution logic of the tool
    async fn execute(&self, args: Value) -> Result<String>;
}

/// The Native MCP Server runs inside Neoland
/// It routes tool calls and handles the Breakpoint interruptions.
#[derive(Clone)]
pub struct NativeMcpServer {
    tools: Arc<HashMap<String, Box<dyn NativeTool>>>,
    breakpoint_tx: mpsc::Sender<BreakpointRequest>,
}

impl NativeMcpServer {
    pub fn new(tools: Vec<Box<dyn NativeTool>>, breakpoint_tx: mpsc::Sender<BreakpointRequest>) -> Self {
        let mut map = HashMap::new();
        for t in tools {
            map.insert(t.name().to_string(), t);
        }
        Self {
            tools: Arc::new(map),
            breakpoint_tx,
        }
    }

    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().map(|t| Tool {
            name: t.name().to_string(),
            description: t.description().to_string(),
            
        }).collect()
    }

    pub async fn call_tool(&self, session_id: Uuid, name: &str, args: Value) -> Result<CallToolResult> {
        let tool = self.tools.get(name).ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
        
        let args_summary = args.to_string();
        let (resolve_tx, resolve_rx) = oneshot::channel();
        
        let req = BreakpointRequest {
            session_id,
            tool_name: name.to_string(),
            args_summary: args_summary.clone(),
            resolve_tx,
        };

        // Envia pedido de aprovação para o Orquestrador/UI
        if self.breakpoint_tx.send(req).await.is_err() {
            return Ok(CallToolResult {
                text: "Agent control plane is offline. Tool execution aborted.".to_string(),
                is_error: true,
            });
        }

        // Bloqueia a execução dessa Tool até o Humano clicar na TUI
        let resolution = resolve_rx.await.unwrap_or(BreakpointResolution::Reject);

        match resolution {
            BreakpointResolution::Approve => {
                // Humano autorizou, executa a ferramenta de verdade
                match tool.execute(args).await {
                    Ok(text) => Ok(CallToolResult { text, is_error: false }),
                    Err(e) => Ok(CallToolResult { text: e.to_string(), is_error: true }),
                }
            },
            BreakpointResolution::Reject => {
                Ok(CallToolResult {
                    text: "Execution rejected by human operator.".to_string(),
                    is_error: true,
                })
            },
            BreakpointResolution::Steer(instruction) => {
                Ok(CallToolResult {
                    text: format!("Execution rejected by human operator. Follow this instruction instead: {}", instruction),
                    is_error: true,
                })
            }
        }
    }
}
