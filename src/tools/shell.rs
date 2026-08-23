use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;

use crate::config::ShellToolConfig;
use crate::mcp::server::NativeTool;

/// Metacaracteres que compõem ou redirecionam comandos. No modo restrito
/// (allowlist não-vazia) a presença de qualquer um invalida a checagem do
/// primeiro token — `permitido; proibido` passaria pela lista.
const SHELL_METACHARS: [&str; 9] = [";", "&&", "||", "|", "$(", "`", ">", "<", "&"];

#[derive(Default)]
pub struct RunShellCommand {
    config: ShellToolConfig,
}

impl RunShellCommand {
    pub fn new(config: ShellToolConfig) -> Self {
        Self { config }
    }

    /// Erro se a allowlist estiver ativa e o comando não a satisfizer.
    /// Lista vazia = sem restrição (comportamento histórico).
    fn check_allowlist(&self, command: &str) -> Result<()> {
        if self.config.allowlist.is_empty() {
            return Ok(());
        }

        if let Some(meta) = SHELL_METACHARS.iter().find(|m| command.contains(**m)) {
            anyhow::bail!(
                "Command rejected: shell metacharacter {meta:?} is not allowed while an allowlist is active"
            );
        }

        let program = command.split_whitespace().next().unwrap_or_default();
        if !self.config.allowlist.iter().any(|allowed| allowed == program) {
            anyhow::bail!(
                "Command rejected: {program:?} is not in the allowlist ({:?})",
                self.config.allowlist
            );
        }

        Ok(())
    }
}

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

        self.check_allowlist(command)?;

        // kill_on_drop: no timeout o future é descartado e o processo morre
        // junto. Sem isso o comando continuaria rodando órfão.
        let child = Command::new("bash")
            .arg("-c")
            .arg(command)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let timeout = std::time::Duration::from_secs(self.config.timeout_secs);
        let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
            Ok(result) => result?,
            Err(_) => {
                anyhow::bail!(
                    "Command timed out after {}s and was killed",
                    self.config.timeout_secs
                )
            },
        };

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

    // These tests document the contract of the tool: the original
    // behaviour (stdout, STDERR marker, exit status, argument errors) plus
    // the two limits added on top — timeout and optional allowlist.

    #[test]
    fn metadata_is_stable() {
        let tool = RunShellCommand::default();
        assert_eq!(tool.name(), "run_shell_command");
        assert!(!tool.description().is_empty());
        let schema = tool.input_schema();
        assert_eq!(schema["required"][0], "command");
        assert_eq!(schema["properties"]["command"]["type"], "string");
    }

    #[tokio::test]
    async fn executes_command_and_returns_stdout() {
        let out = RunShellCommand::default()
            .execute(serde_json::json!({"command": "echo shell-tool-ok"}))
            .await
            .expect("echo must succeed");
        assert!(out.contains("shell-tool-ok"));
    }

    #[tokio::test]
    async fn stderr_is_appended_with_marker() {
        let out = RunShellCommand::default()
            .execute(serde_json::json!({"command": "echo out; echo err >&2"}))
            .await
            .expect("command must succeed");
        assert!(out.contains("out"));
        assert!(out.contains("--- STDERR ---"));
        assert!(out.contains("err"));
    }

    #[tokio::test]
    async fn nonzero_exit_with_no_output_reports_status() {
        let out = RunShellCommand::default()
            .execute(serde_json::json!({"command": "exit 3"}))
            .await
            .expect("execute returns Ok even on nonzero exit");
        assert!(out.contains("exit status: 3"), "got: {out}");
    }

    #[tokio::test]
    async fn missing_command_argument_is_an_error() {
        let err = RunShellCommand::default()
            .execute(serde_json::json!({"cmd": "echo wrong-key"}))
            .await
            .expect_err("missing 'command' must error");
        assert!(err.to_string().contains("Missing 'command'"));
    }

    #[tokio::test]
    async fn non_string_command_is_an_error() {
        let err = RunShellCommand::default()
            .execute(serde_json::json!({"command": 42}))
            .await
            .expect_err("non-string 'command' must error");
        assert!(err.to_string().contains("Missing 'command'"));
    }

    // ── Timeout ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn command_exceeding_timeout_is_killed_and_errors() {
        let tool = RunShellCommand::new(ShellToolConfig { timeout_secs: 1, allowlist: vec![] });
        let started = std::time::Instant::now();
        let err = tool
            .execute(serde_json::json!({"command": "sleep 60"}))
            .await
            .expect_err("a command past the timeout must error, not hang");
        assert!(err.to_string().contains("timed out"), "got: {err}");
        assert!(
            started.elapsed() < std::time::Duration::from_secs(10),
            "must return at the timeout, not wait for the command"
        );
    }

    #[tokio::test]
    async fn command_within_timeout_still_succeeds() {
        let tool = RunShellCommand::new(ShellToolConfig { timeout_secs: 10, allowlist: vec![] });
        let out = tool
            .execute(serde_json::json!({"command": "echo fast-enough"}))
            .await
            .expect("a quick command must not be affected by the timeout");
        assert!(out.contains("fast-enough"));
    }

    // ── Allowlist ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn empty_allowlist_permits_everything() {
        // Default config keeps the historical behaviour.
        let out = RunShellCommand::default()
            .execute(serde_json::json!({"command": "echo unrestricted"}))
            .await
            .expect("empty allowlist must not restrict");
        assert!(out.contains("unrestricted"));
    }

    #[tokio::test]
    async fn allowlist_permits_listed_program() {
        let tool = RunShellCommand::new(ShellToolConfig {
            timeout_secs: 30,
            allowlist: vec!["echo".to_string()],
        });
        let out = tool
            .execute(serde_json::json!({"command": "echo allowed"}))
            .await
            .expect("listed program must run");
        assert!(out.contains("allowed"));
    }

    #[tokio::test]
    async fn allowlist_rejects_unlisted_program() {
        let tool = RunShellCommand::new(ShellToolConfig {
            timeout_secs: 30,
            allowlist: vec!["echo".to_string()],
        });
        let err = tool
            .execute(serde_json::json!({"command": "cat /etc/passwd"}))
            .await
            .expect_err("unlisted program must be rejected");
        assert!(err.to_string().contains("not in the allowlist"), "got: {err}");
    }

    #[tokio::test]
    async fn allowlist_rejects_chaining_past_a_listed_program() {
        // Without this, `echo x; cat /etc/passwd` would pass a first-token check.
        let tool = RunShellCommand::new(ShellToolConfig {
            timeout_secs: 30,
            allowlist: vec!["echo".to_string()],
        });
        for command in
            ["echo ok; cat /etc/passwd", "echo ok && id", "echo $(id)", "echo ok > /tmp/x"]
        {
            let err = tool.execute(serde_json::json!({ "command": command })).await.unwrap_err();
            assert!(
                err.to_string().contains("metacharacter"),
                "command {command:?} must be rejected for chaining, got: {err}"
            );
        }
    }
}
