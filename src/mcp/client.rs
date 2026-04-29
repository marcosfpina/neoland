use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

use super::types::{CallToolRequest, CallToolResult, JsonRpcRequest, JsonRpcResponse, Tool};

struct Inner {
    stdin: BufWriter<ChildStdin>,
    lines: tokio::io::Lines<BufReader<ChildStdout>>,
}

pub struct McpClient {
    inner: Mutex<Inner>,
    next_id: AtomicU64,
    _child: Child,
}

impl McpClient {
    pub async fn spawn(binary: &str) -> Result<Self> {
        let mut child = Command::new(binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("MCP process has no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("MCP process has no stdout"))?;

        let client = Self {
            inner: Mutex::new(Inner {
                stdin: BufWriter::new(stdin),
                lines: BufReader::new(stdout).lines(),
            }),
            next_id: AtomicU64::new(1),
            _child: child,
        };

        client.initialize().await?;
        Ok(client)
    }

    async fn send(&self, method: &'static str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = JsonRpcRequest::new(id, method, params);
        let mut line = serde_json::to_string(&req)?;
        line.push('\n');

        let mut g = self.inner.lock().await;
        g.stdin.write_all(line.as_bytes()).await?;
        g.stdin.flush().await?;

        loop {
            let raw =
                g.lines.next_line().await?.ok_or_else(|| anyhow!("MCP process closed stdout"))?;
            if raw.trim().is_empty() {
                continue;
            }
            let resp: JsonRpcResponse = match serde_json::from_str(&raw) {
                Ok(r) => r,
                Err(_) => continue, // ignore malformed lines (e.g. log output)
            };
            if resp.id != Some(id) {
                continue; // notification or response for a different id
            }
            if let Some(err) = resp.error {
                return Err(anyhow!("MCP error {}: {}", err.code, err.message));
            }
            return Ok(resp.result.unwrap_or(Value::Null));
        }
    }

    async fn notify(&self, method: &'static str) -> Result<()> {
        let msg = json!({ "jsonrpc": "2.0", "method": method, "params": {} });
        let mut line = serde_json::to_string(&msg)?;
        line.push('\n');
        let mut g = self.inner.lock().await;
        g.stdin.write_all(line.as_bytes()).await?;
        g.stdin.flush().await?;
        Ok(())
    }

    async fn initialize(&self) -> Result<()> {
        self.send(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "neoland", "version": "0.1.0" }
            })),
        )
        .await?;
        self.notify("notifications/initialized").await?;
        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let result = self.send("tools/list", None).await?;
        let tools: Vec<Tool> = serde_json::from_value(result["tools"].clone().take())?;
        Ok(tools)
    }

    pub async fn call_tool(&self, req: CallToolRequest) -> Result<CallToolResult> {
        let result = self
            .send("tools/call", Some(json!({ "name": req.name, "arguments": req.arguments })))
            .await?;

        let is_error = result["isError"].as_bool().unwrap_or(false);
        let text = result["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|c| c["text"].as_str())
            .unwrap_or("")
            .to_string();

        Ok(CallToolResult { text, is_error })
    }
}
