use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;
use tokio::process::Command;
use crate::mcp::server::NativeTool;

pub struct RunShellCommand;

#[async_trait]
impl NativeTool for RunShellCommand {
    fn name(&self) -> &'static str {
        "run_shell_command"
    }

    fn description(&self) -> &'static str {
        "Execute a shell command locally on the Neoland host machine. Returns stdout/stderr."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The exact bash command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;

        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
            .await?;

        let mut out = String::from_utf8_lossy(&output.stdout).to_string();
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        
        if !err.is_empty() {
            out.push_str("\n--- STDERR ---\n");
            out.push_str(&err);
        }

        if out.is_empty() {
            return Ok(format!("Command exited with status {}", output.status));
        }
        
        Ok(out)
    }
}
