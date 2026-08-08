use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
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
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;

        let output = Command::new("bash").arg("-c").arg(command).output().await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    // These tests document the CURRENT contract of the tool. Known gaps
    // (tracked in the roadmap, not silently "fixed" here): no timeout and
    // no allowlist — arbitrary bash behind auth + human breakpoint.

    #[test]
    fn metadata_is_stable() {
        let tool = RunShellCommand;
        assert_eq!(tool.name(), "run_shell_command");
        assert!(!tool.description().is_empty());
        let schema = tool.input_schema();
        assert_eq!(schema["required"][0], "command");
        assert_eq!(schema["properties"]["command"]["type"], "string");
    }

    #[tokio::test]
    async fn executes_command_and_returns_stdout() {
        let out = RunShellCommand
            .execute(serde_json::json!({"command": "echo shell-tool-ok"}))
            .await
            .expect("echo must succeed");
        assert!(out.contains("shell-tool-ok"));
    }

    #[tokio::test]
    async fn stderr_is_appended_with_marker() {
        let out = RunShellCommand
            .execute(serde_json::json!({"command": "echo out; echo err >&2"}))
            .await
            .expect("command must succeed");
        assert!(out.contains("out"));
        assert!(out.contains("--- STDERR ---"));
        assert!(out.contains("err"));
    }

    #[tokio::test]
    async fn nonzero_exit_with_no_output_reports_status() {
        let out = RunShellCommand
            .execute(serde_json::json!({"command": "exit 3"}))
            .await
            .expect("execute returns Ok even on nonzero exit");
        assert!(out.contains("exit status: 3"), "got: {out}");
    }

    #[tokio::test]
    async fn missing_command_argument_is_an_error() {
        let err = RunShellCommand
            .execute(serde_json::json!({"cmd": "echo wrong-key"}))
            .await
            .expect_err("missing 'command' must error");
        assert!(err.to_string().contains("Missing 'command'"));
    }

    #[tokio::test]
    async fn non_string_command_is_an_error() {
        let err = RunShellCommand
            .execute(serde_json::json!({"command": 42}))
            .await
            .expect_err("non-string 'command' must error");
        assert!(err.to_string().contains("Missing 'command'"));
    }
}
