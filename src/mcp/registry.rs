use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;

use super::{
    client::McpClient,
    types::{CallToolRequest, CallToolResult, Tool},
};

pub struct McpRegistry {
    client: Arc<McpClient>,
    tools: Vec<Tool>,
}

impl McpRegistry {
    pub async fn new(client: McpClient) -> Result<Self> {
        let client = Arc::new(client);
        let tools = client.list_tools().await?;
        tracing::info!(count = tools.len(), "MCP tools discovered");
        Ok(Self { client, tools })
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }

    pub async fn call(&self, name: impl Into<String>, args: Value) -> Result<CallToolResult> {
        self.client
            .call_tool(CallToolRequest { name: name.into(), arguments: args })
            .await
    }
}
