use std::{path::PathBuf, process::Command, time::Duration};

use async_trait::async_trait;
use serde::Serialize;
use tonic::transport::Endpoint;
use tracing::Level;

use crate::{config::Config, server};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckState {
    Ok,
    Warning,
    Error,
}

impl CheckState {
    fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub name: String,
    pub state: CheckState,
    pub detail: String,
    pub hint: Option<String>,
}

impl CheckResult {
    fn ok(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { name: name.into(), state: CheckState::Ok, detail: detail.into(), hint: None }
    }

    fn warning(
        name: impl Into<String>,
        detail: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            state: CheckState::Warning,
            detail: detail.into(),
            hint: Some(hint.into()),
        }
    }

    fn error(name: impl Into<String>, detail: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: CheckState::Error,
            detail: detail.into(),
            hint: Some(hint.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub ok: bool,
    pub checks: Vec<CheckResult>,
}

impl DoctorReport {
    fn from_checks(checks: Vec<CheckResult>) -> Self {
        let ok = !checks.iter().any(|check| check.state == CheckState::Error);
        Self { ok, checks }
    }

    pub fn has_errors(&self) -> bool {
        !self.ok
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HealthCheckReport {
    pub ok: bool,
    pub checks: Vec<CheckResult>,
}

impl HealthCheckReport {
    fn from_checks(checks: Vec<CheckResult>) -> Self {
        let ok = !checks.iter().any(|check| check.state == CheckState::Error);
        Self { ok, checks }
    }

    pub fn has_errors(&self) -> bool {
        !self.ok
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct RestartReport {
    pub killed_pid: Option<String>,
    pub process_scan_warning: Option<String>,
    pub kill_warning: Option<String>,
}

#[async_trait]
pub trait CommandRuntime {
    async fn http_get_status(&self, url: &str) -> Result<u16, String>;
    async fn grpc_connect(&self, endpoint: &str) -> Result<(), String>;
    async fn run_server(&self, grpc_port: u16, rest_port: u16, web_dist_dir: &str) -> Result<(), String>;
    async fn check_db(&self, url: &str) -> Result<(), String>;
    fn env_var(&self, key: &str) -> Option<String>;
    fn config_paths(&self) -> Vec<PathBuf>;
    fn process_list(&self) -> Result<String, String>;
    fn kill_process(&self, pid: &str) -> Result<(), String>;
    fn sleep(&self, duration: Duration);
}

pub struct SystemCommandRuntime;

#[async_trait]
impl CommandRuntime for SystemCommandRuntime {
    async fn http_get_status(&self, url: &str) -> Result<u16, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|err| err.to_string())?;

        let response = client.get(url).send().await.map_err(|err| err.to_string())?;
        Ok(response.status().as_u16())
    }

    async fn grpc_connect(&self, endpoint: &str) -> Result<(), String> {
        let endpoint =
            Endpoint::from_shared(endpoint.to_string()).map_err(|err| err.to_string())?;
        endpoint
            .connect_timeout(Duration::from_secs(3))
            .connect()
            .await
            .map(|_| ())
            .map_err(|err| err.to_string())
    }

    async fn run_server(&self, grpc_port: u16, rest_port: u16, web_dist_dir: &str) -> Result<(), String> {
        server::run_server(grpc_port, rest_port, web_dist_dir).await.map_err(|err| err.to_string())
    }

    async fn check_db(&self, url: &str) -> Result<(), String> {
        use sqlx::postgres::PgPoolOptions;
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .connect(url)
            .await
            .map_err(|err| err.to_string())?;

        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|err| err.to_string())
    }

    fn env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        default_config_paths(self.env_var("HOME"))
    }

    fn process_list(&self) -> Result<String, String> {
        let output = Command::new("ps").args(["aux"]).output().map_err(|err| err.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    fn kill_process(&self, pid: &str) -> Result<(), String> {
        let output = Command::new("kill").arg(pid).output().map_err(|err| err.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                Err(format!("kill exited with status {}", output.status))
            } else {
                Err(stderr)
            }
        }
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

pub fn parse_log_level(value: &str) -> Level {
    match value {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

pub async fn collect_health_check_report(
    runtime: &impl CommandRuntime,
    rest_endpoint: &str,
    grpc_endpoint: &str,
) -> HealthCheckReport {
    let rest_health_url = format!("{}/health", rest_endpoint.trim_end_matches('/'));

    let rest_check = match runtime.http_get_status(&rest_health_url).await {
        Ok(status) if (200..300).contains(&status) => {
            CheckResult::ok("REST API", format!("Healthy at {} (status {})", rest_endpoint, status))
        },
        Ok(status) => CheckResult::error(
            "REST API",
            format!("{} returned status {}", rest_health_url, status),
            "Start the server with `neoland server` and re-run the check.",
        ),
        Err(err) => CheckResult::error(
            "REST API",
            format!("Could not reach {}", rest_health_url),
            format!("Connection error: {err}"),
        ),
    };

    let grpc_check = match runtime.grpc_connect(grpc_endpoint).await {
        Ok(()) => CheckResult::ok("gRPC", format!("Connected to {}", grpc_endpoint)),
        Err(err) => CheckResult::error(
            "gRPC",
            format!("Could not connect to {}", grpc_endpoint),
            format!("Connection error: {err}"),
        ),
    };

    let process_check = match runtime.process_list() {
        Ok(processes) => match find_neoland_pid(&processes) {
            Some(pid) => {
                CheckResult::ok("Process", format!("Found neoland process with PID {}", pid))
            },
            None => CheckResult::error(
                "Process",
                "No running neoland process found".to_string(),
                "Launch the server with `neoland server`.",
            ),
        },
        Err(err) => CheckResult::error(
            "Process",
            "Could not inspect running processes".to_string(),
            format!("ps error: {err}"),
        ),
    };

    HealthCheckReport::from_checks(vec![rest_check, grpc_check, process_check])
}

pub async fn collect_doctor_report(
    runtime: &impl CommandRuntime,
    config: &Config,
    server_url: &str,
    neoland_gateway_url: &str,
) -> DoctorReport {
    let mut checks = Vec::new();

    // 1. Nix Environment
    let in_nix = runtime.env_var("IN_NIX_SHELL").is_some()
        || runtime.env_var("FLAKE_ROOT").is_some()
        || runtime.env_var("NIX_BUILD_TOP").is_some();

    if in_nix {
        checks.push(CheckResult::ok("Nix shell", "Development shell detected"));
    } else {
        checks.push(CheckResult::warning(
            "Nix shell",
            "Development shell not detected",
            "Run `nix develop` for the supported toolchain.",
        ));
    }

    // 2. Server Connectivity (REST)
    let server_health_url = format!("{}/health", server_url.trim_end_matches('/'));
    match runtime.http_get_status(&server_health_url).await {
        Ok(status) if (200..300).contains(&status) => {
            checks.push(CheckResult::ok(
                "Server (REST)",
                format!("Server reachable at {} (status {})", server_url, status),
            ));
        },
        Ok(status) => {
            checks.push(CheckResult::warning(
                "Server (REST)",
                format!("Server returned status {}", status),
                "Check `neoland server` logs if this is unexpected.",
            ));
        },
        Err(err) => {
            checks.push(CheckResult::error(
                "Server (REST)",
                format!("Server not reachable at {}", server_url),
                format!("Start the server with `neoland server` ({err})"),
            ));
        },
    }

    // 3. gRPC Connectivity
    // Note: The CLI arg for server_url is usually the REST port.
    // We try to infer gRPC port from default or config if needed, but for
    // simplicity we use a hardcoded fallback or look for it in the config.
    let grpc_url = "http://[::1]:50051"; // Default gRPC endpoint
    match runtime.grpc_connect(grpc_url).await {
        Ok(()) => {
            checks
                .push(CheckResult::ok("Server (gRPC)", format!("gRPC reachable at {}", grpc_url)));
        },
        Err(err) => {
            checks.push(CheckResult::warning(
                "Server (gRPC)",
                format!("gRPC not reachable at {}", grpc_url),
                format!("Check if server is running with gRPC enabled ({err})"),
            ));
        },
    }

    // 4. LLM Gateway Connectivity (SecureLLM Bridge API)
    let gateway_candidates = build_health_candidates(neoland_gateway_url, true);
    match probe_http_candidates(runtime, &gateway_candidates).await {
        ProbeResult::Ok { url, status } => {
            checks.push(CheckResult::ok(
                "LLM Gateway",
                format!(
                    "SecureLLM API reachable at {} via {} (status {})",
                    neoland_gateway_url, url, status
                ),
            ));
        },
        ProbeResult::Status { url, status } => {
            checks.push(CheckResult::error(
                "LLM Gateway",
                format!("SecureLLM API returned status {} at {}", status, url),
                "Check if `securellm-api-server` is running and exposing `/api/health`.",
            ));
        },
        ProbeResult::Err { url, error } => {
            checks.push(CheckResult::error(
                "LLM Gateway",
                format!("SecureLLM API not reachable at {}", neoland_gateway_url),
                format!("Expected health at {} ({error})", url),
            ));
        },
    }

    // 5. Inference bridge (optional, but diagnosed when declared)
    if let Some(ml_ops_url) = runtime.env_var("ML_OPS_API_URL") {
        let ml_ops_candidates = build_health_candidates(&ml_ops_url, false);
        match probe_http_candidates(runtime, &ml_ops_candidates).await {
            ProbeResult::Ok { url, status } => {
                checks.push(CheckResult::ok(
                    "Inference Bridge",
                    format!(
                        "ml-ops-api reachable at {} via {} (status {})",
                        ml_ops_url, url, status
                    ),
                ));
            },
            ProbeResult::Status { url, status } => {
                checks.push(CheckResult::warning(
                    "Inference Bridge",
                    format!("ml-ops-api returned status {} at {}", status, url),
                    "Check if `ml-ops-api` is running and exposing `/health`.",
                ));
            },
            ProbeResult::Err { url, error } => {
                checks.push(CheckResult::warning(
                    "Inference Bridge",
                    format!("ml-ops-api not reachable at {}", ml_ops_url),
                    format!("Expected health at {} ({error})", url),
                ));
            },
        }
    }

    // 6. Local backend (optional, but diagnosed when declared)
    if let Some(llamacpp_url) = runtime.env_var("LLAMACPP_URL") {
        let llamacpp_candidates = build_health_candidates(&llamacpp_url, false);
        match probe_http_candidates(runtime, &llamacpp_candidates).await {
            ProbeResult::Ok { url, status } => {
                checks.push(CheckResult::ok(
                    "llama.cpp",
                    format!(
                        "llama.cpp reachable at {} via {} (status {})",
                        llamacpp_url, url, status
                    ),
                ));
            },
            ProbeResult::Status { url, status } => {
                checks.push(CheckResult::warning(
                    "llama.cpp",
                    format!("llama.cpp returned status {} at {}", status, url),
                    "Check if `llama-server` is running on the configured upstream port.",
                ));
            },
            ProbeResult::Err { url, error } => {
                checks.push(CheckResult::warning(
                    "llama.cpp",
                    format!("llama.cpp not reachable at {}", llamacpp_url),
                    format!("Expected health at {} ({error})", url),
                ));
            },
        }
    }

    // 7. DSPy Agent Pipeline
    let dspy_url = runtime
        .env_var("NEOLAND_DSPY_URL")
        .unwrap_or_else(|| config.agents.dspy_url.clone());
    let dspy_health_url = format!("{}/health", dspy_url.trim_end_matches('/'));
    match runtime.http_get_status(&dspy_health_url).await {
        Ok(status) if (200..300).contains(&status) => {
            checks.push(CheckResult::ok(
                "DSPy Pipeline",
                format!("Agent pipeline reachable at {} (status {})", dspy_url, status),
            ));
        },
        Ok(status) => {
            checks.push(CheckResult::warning(
                "DSPy Pipeline",
                format!("Agent pipeline returned status {} at {}", status, dspy_url),
                "Check if `just agents-start` (uvicorn on :8001) is running.",
            ));
        },
        Err(err) => {
            checks.push(CheckResult::warning(
                "DSPy Pipeline",
                format!("Agent pipeline not reachable at {}", dspy_url),
                format!("Start with `just agents-start` or set NEOLAND_DSPY_URL ({err})"),
            ));
        },
    }

    // 8. Vault Connectivity
    let vault_addr = runtime.env_var("VAULT_ADDR").unwrap_or_else(|| config.vault.addr.clone());
    let vault_health_url = format!("{}/v1/sys/health", vault_addr.trim_end_matches('/'));
    match runtime.http_get_status(&vault_health_url).await {
        Ok(status) if (200..300).contains(&status) || status == 429 => {
            // Vault health returns 200 (initialized, unsealed), 429 (unsealed, standby),
            // 472 (disaster recovery), 501 (not initialized), 503 (sealed)
            checks.push(CheckResult::ok("Vault", format!("Vault reachable at {}", vault_addr)));
        },
        Ok(status) => {
            checks.push(CheckResult::warning(
                "Vault",
                format!("Vault at {} returned status {}", vault_addr, status),
                "Vault may be sealed or not initialized.",
            ));
        },
        Err(err) => {
            checks.push(CheckResult::warning(
                "Vault",
                format!("Vault not reachable at {}", vault_addr),
                format!("Check if Vault is running ({err})"),
            ));
        },
    }

    // 8. PostgreSQL / pgvector Connectivity
    let db_url = runtime.env_var("DATABASE_URL").or_else(|| {
        if config.database.url.is_empty() {
            None
        } else {
            Some(config.database.url.clone())
        }
    });

    if let Some(url) = db_url {
        match runtime.check_db(&url).await {
            Ok(()) => {
                checks.push(CheckResult::ok("Database", "PostgreSQL connected successfully"));
            },
            Err(err) => {
                checks.push(CheckResult::error(
                    "Database",
                    "Failed to connect to PostgreSQL",
                    format!("Check DATABASE_URL and server status ({err})"),
                ));
            },
        }
    } else {
        checks.push(CheckResult::warning(
            "Database",
            "DATABASE_URL not set",
            "PostgreSQL/pgvector will be disabled. Fallback to in-memory store.",
        ));
    }

    // 9. Environment & Config
    if let Some(value) = runtime.env_var("RUST_LOG") {
        checks.push(CheckResult::ok("RUST_LOG", format!("Using {}", value)));
    } else {
        checks.push(CheckResult::warning(
            "RUST_LOG",
            "Not set".to_string(),
            "Use `RUST_LOG=debug` for richer troubleshooting logs.",
        ));
    }

    let config_paths = runtime.config_paths();
    let found_config = config_paths.iter().find(|path| path.exists());
    if let Some(path) = found_config {
        checks.push(CheckResult::ok("Config file", format!("Found {}", path.display())));
    } else {
        checks.push(CheckResult::warning(
            "Config file",
            "No config file found".to_string(),
            "Optional paths: ./neoland.toml or ~/.config/neoland/config.toml",
        ));
    }

    DoctorReport::from_checks(checks)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProbeResult {
    Ok { url: String, status: u16 },
    Status { url: String, status: u16 },
    Err { url: String, error: String },
}

fn build_health_candidates(base_url: &str, prefer_api_prefix: bool) -> Vec<String> {
    let base_url = base_url.trim_end_matches('/');
    let mut candidates = Vec::with_capacity(2);

    if prefer_api_prefix {
        candidates.push(format!("{base_url}/api/health"));
        candidates.push(format!("{base_url}/health"));
    } else {
        candidates.push(format!("{base_url}/health"));
        candidates.push(format!("{base_url}/api/health"));
    }

    candidates
}

async fn probe_http_candidates(
    runtime: &impl CommandRuntime,
    candidates: &[String],
) -> ProbeResult {
    let mut fallback: Option<ProbeResult> = None;

    for url in candidates {
        match runtime.http_get_status(url).await {
            Ok(status) if (200..300).contains(&status) => {
                return ProbeResult::Ok { url: url.clone(), status };
            },
            Ok(status) => {
                if fallback.is_none() {
                    fallback = Some(ProbeResult::Status { url: url.clone(), status });
                }
            },
            Err(error) => {
                if fallback.is_none() {
                    fallback = Some(ProbeResult::Err { url: url.clone(), error });
                }
            },
        }
    }

    fallback.unwrap_or_else(|| ProbeResult::Err {
        url: "<no-candidates>".to_string(),
        error: "no health candidates configured".to_string(),
    })
}

pub async fn restart_server(
    runtime: &impl CommandRuntime,
    grpc_port: u16,
    rest_port: u16,
) -> Result<RestartReport, String> {
    let mut report = RestartReport::default();

    match runtime.process_list() {
        Ok(processes) => {
            if let Some(pid) = find_neoland_pid(&processes) {
                match runtime.kill_process(&pid) {
                    Ok(()) => {
                        report.killed_pid = Some(pid);
                        runtime.sleep(Duration::from_secs(2));
                    },
                    Err(err) => {
                        report.kill_warning = Some(err);
                    },
                }
            }
        },
        Err(err) => {
            report.process_scan_warning = Some(err);
        },
    }

    runtime.run_server(grpc_port, rest_port, "web/dist").await?;
    Ok(report)
}

pub fn render_health_check_report(report: &HealthCheckReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(report).expect("health report should serialize");
    }

    render_report(
        "neoland test",
        report.ok,
        &report.checks,
        "Health checks passed.",
        "Health checks failed.",
    )
}

pub fn render_doctor_report(report: &DoctorReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(report).expect("doctor report should serialize");
    }

    render_report(
        "neoland doctor",
        report.ok,
        &report.checks,
        "Environment looks good. Warnings above are non-fatal.",
        "Some checks failed. See hints above.",
    )
}

pub fn render_restart_report(report: &RestartReport) -> String {
    let mut lines = vec!["neoland restart".to_string(), String::new()];

    if let Some(pid) = &report.killed_pid {
        lines.push(format!("  [ok] Process: terminated neoland PID {}", pid));
    } else {
        lines.push("  [warn] Process: no running neoland process was found".to_string());
    }

    if let Some(warning) = &report.process_scan_warning {
        lines.push(format!("  [warn] Process scan: {}", warning));
    }

    if let Some(warning) = &report.kill_warning {
        lines.push(format!("  [warn] Kill: {}", warning));
    }

    lines.push(String::new());
    lines.push("Restarted server command path.".to_string());
    lines.join("\n")
}

pub fn find_neoland_pid(processes: &str) -> Option<String> {
    find_pid_in_lines(processes.lines().filter(|line| line.contains("neoland server"))).or_else(
        || {
            find_pid_in_lines(
                processes
                    .lines()
                    .filter(|line| line.contains("neoland") && !line.contains("grep")),
            )
        },
    )
}

pub fn default_config_paths(home: Option<String>) -> Vec<PathBuf> {
    let home = home.unwrap_or_else(|| ".".to_string());

    vec![
        PathBuf::from("neoland.toml"),
        PathBuf::from(home).join(".config").join("neoland").join("config.toml"),
    ]
}

fn render_report(
    title: &str,
    ok: bool,
    checks: &[CheckResult],
    success_summary: &str,
    error_summary: &str,
) -> String {
    let mut lines = vec![title.to_string(), String::new()];

    for check in checks {
        lines.push(format!("  [{}] {}: {}", check.state.label(), check.name, check.detail));
        if let Some(hint) = &check.hint {
            lines.push(format!("        hint: {}", hint));
        }
    }

    lines.push(String::new());
    lines.push(if ok { success_summary } else { error_summary }.to_string());
    lines.join("\n")
}

fn find_pid_in_lines<'a>(mut lines: impl Iterator<Item = &'a str>) -> Option<String> {
    lines.find_map(|line| line.split_whitespace().nth(1).map(|pid| pid.to_string()))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs,
        sync::{Arc, Mutex},
    };

    use tempfile::TempDir;

    use super::*;

    #[derive(Clone)]
    struct MockRuntime {
        http_statuses: HashMap<String, Result<u16, String>>,
        grpc_statuses: HashMap<String, Result<(), String>>,
        env: HashMap<String, String>,
        config_paths: Vec<PathBuf>,
        processes: Result<String, String>,
        server_error: Option<String>,
        killed_pids: Arc<Mutex<Vec<String>>>,
        started_servers: Arc<Mutex<Vec<(u16, u16)>>>,
    }

    impl Default for MockRuntime {
        fn default() -> Self {
            Self {
                http_statuses: HashMap::new(),
                grpc_statuses: HashMap::new(),
                env: HashMap::new(),
                config_paths: Vec::new(),
                processes: Ok(String::new()),
                server_error: None,
                killed_pids: Arc::new(Mutex::new(Vec::new())),
                started_servers: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl CommandRuntime for MockRuntime {
        async fn http_get_status(&self, url: &str) -> Result<u16, String> {
            self.http_statuses
                .get(url)
                .cloned()
                .unwrap_or_else(|| Err(format!("unexpected http url: {url}")))
        }

        async fn grpc_connect(&self, endpoint: &str) -> Result<(), String> {
            self.grpc_statuses
                .get(endpoint)
                .cloned()
                .unwrap_or_else(|| Err(format!("unexpected grpc endpoint: {endpoint}")))
        }

        async fn run_server(&self, grpc_port: u16, rest_port: u16, _web_dist_dir: &str) -> Result<(), String> {
            self.started_servers.lock().unwrap().push((grpc_port, rest_port));
            self.server_error.clone().map_or(Ok(()), Err)
        }

        async fn check_db(&self, _url: &str) -> Result<(), String> {
            Ok(())
        }

        fn env_var(&self, key: &str) -> Option<String> {
            self.env.get(key).cloned()
        }

        fn config_paths(&self) -> Vec<PathBuf> {
            self.config_paths.clone()
        }

        fn process_list(&self) -> Result<String, String> {
            self.processes.clone()
        }

        fn kill_process(&self, pid: &str) -> Result<(), String> {
            self.killed_pids.lock().unwrap().push(pid.to_string());
            Ok(())
        }

        fn sleep(&self, _duration: Duration) {}
    }

    fn find_check<'a>(checks: &'a [CheckResult], name: &str) -> &'a CheckResult {
        checks.iter().find(|check| check.name == name).expect("missing check")
    }

    #[test]
    fn parse_log_level_defaults_to_info_for_unknown_values() {
        assert_eq!(parse_log_level("trace"), Level::TRACE);
        assert_eq!(parse_log_level("debug"), Level::DEBUG);
        assert_eq!(parse_log_level("warn"), Level::WARN);
        assert_eq!(parse_log_level("unexpected"), Level::INFO);
    }

    #[test]
    fn default_config_paths_include_project_and_home_locations() {
        let paths = default_config_paths(Some("/tmp/home".to_string()));
        assert_eq!(paths[0], PathBuf::from("neoland.toml"));
        assert_eq!(paths[1], PathBuf::from("/tmp/home/.config/neoland/config.toml"));
    }

    #[test]
    fn find_neoland_pid_prefers_server_processes() {
        let processes = "\
user 1000 0.0 0.1 123 456 pts/1 Sl+ 00:00 cargo test\nuser 4242 0.0 0.1 123 456 pts/2 Sl+ 00:00 \
                         /tmp/target/debug/neoland server\nuser 5252 0.0 0.1 123 456 pts/3 Sl+ \
                         00:00 /tmp/target/debug/neoland client\n";

        assert_eq!(find_neoland_pid(processes).as_deref(), Some("4242"));
    }

    #[tokio::test]
    async fn doctor_report_marks_server_error_and_gateway_error() {
        let runtime = MockRuntime {
            http_statuses: HashMap::from([
                (
                    "http://localhost:3001/health".to_string(),
                    Err("connection refused".to_string()),
                ),
                (
                    "http://localhost:8080/api/health".to_string(),
                    Err("connection refused".to_string()),
                ),
                (
                    "http://localhost:8080/health".to_string(),
                    Err("connection refused".to_string()),
                ),
                (
                    "http://localhost:8200/v1/sys/health".to_string(),
                    Err("connection refused".to_string()),
                ),
            ]),
            grpc_statuses: HashMap::from([(
                "http://[::1]:50051".to_string(),
                Err("connection refused".to_string()),
            )]),
            config_paths: vec![PathBuf::from("missing-project"), PathBuf::from("missing-home")],
            ..Default::default()
        };

        let report = collect_doctor_report(
            &runtime,
            &Config::default(),
            "http://localhost:3001",
            "http://localhost:8080",
        )
        .await;

        assert!(report.has_errors());
        assert_eq!(find_check(&report.checks, "Server (REST)").state, CheckState::Error);
        assert_eq!(find_check(&report.checks, "Server (gRPC)").state, CheckState::Warning);
        assert_eq!(find_check(&report.checks, "LLM Gateway").state, CheckState::Error);
        assert_eq!(find_check(&report.checks, "Vault").state, CheckState::Warning);
    }

    #[tokio::test]
    async fn doctor_report_detects_nix_shell_and_internal_llm_layers() {
        let temp_dir = TempDir::new().expect("temp dir");
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[server]\ngrpc_port = 50051\n").expect("config file");

        let runtime = MockRuntime {
            http_statuses: HashMap::from([
                ("http://localhost:3001/health".to_string(), Ok(200)),
                ("http://localhost:8080/api/health".to_string(), Ok(200)),
                ("http://localhost:8083/health".to_string(), Ok(200)),
                ("http://localhost:5001/health".to_string(), Ok(200)),
                ("http://localhost:8200/v1/sys/health".to_string(), Ok(200)),
            ]),
            grpc_statuses: HashMap::from([("http://[::1]:50051".to_string(), Ok(()))]),
            env: HashMap::from([
                ("IN_NIX_SHELL".to_string(), "1".to_string()),
                ("VAULT_ADDR".to_string(), "http://localhost:8200".to_string()),
                ("RUST_LOG".to_string(), "debug".to_string()),
                ("ML_OPS_API_URL".to_string(), "http://localhost:8083".to_string()),
                ("LLAMACPP_URL".to_string(), "http://localhost:5001".to_string()),
            ]),
            config_paths: vec![config_path.clone()],
            ..Default::default()
        };

        let report = collect_doctor_report(
            &runtime,
            &Config::default(),
            "http://localhost:3001",
            "http://localhost:8080",
        )
        .await;

        assert!(report.ok);
        assert_eq!(find_check(&report.checks, "Nix shell").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "Server (REST)").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "Server (gRPC)").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "LLM Gateway").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "Inference Bridge").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "llama.cpp").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "Vault").state, CheckState::Ok);
        assert_eq!(
            find_check(&report.checks, "Config file").detail,
            format!("Found {}", config_path.display())
        );
    }

    #[tokio::test]
    async fn health_check_probes_rest_grpc_and_processes() {
        let runtime = MockRuntime {
            http_statuses: HashMap::from([("http://localhost:3001/health".to_string(), Ok(200))]),
            grpc_statuses: HashMap::from([("http://[::1]:50051".to_string(), Ok(()))]),
            processes: Ok("user 4242 0.0 0.1 123 456 pts/2 Sl+ 00:00 /tmp/target/debug/neoland \
                           server\n"
                .to_string()),
            ..Default::default()
        };

        let report =
            collect_health_check_report(&runtime, "http://localhost:3001", "http://[::1]:50051")
                .await;

        assert!(report.ok);
        assert_eq!(find_check(&report.checks, "REST API").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "gRPC").state, CheckState::Ok);
        assert_eq!(find_check(&report.checks, "Process").state, CheckState::Ok);
    }

    #[tokio::test]
    async fn restart_server_kills_existing_process_before_starting() {
        let runtime = MockRuntime {
            processes: Ok("user 31337 0.0 0.1 123 456 pts/2 Sl+ 00:00 /tmp/target/debug/neoland \
                           server\n"
                .to_string()),
            ..Default::default()
        };

        let report = restart_server(&runtime, 50051, 3001).await.expect("restart succeeds");

        assert_eq!(report.killed_pid.as_deref(), Some("31337"));
        assert_eq!(runtime.killed_pids.lock().unwrap().as_slice(), ["31337"]);
        assert_eq!(runtime.started_servers.lock().unwrap().as_slice(), [(50051, 3001)]);
    }
}
