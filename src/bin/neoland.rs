use std::process::Command;

use cli::{Cli, Commands};
use neoland::{cli, config::Config, logging, server};
use tracing::Level;

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse_args();

    // Phase 4.2: Setup structured logging
    let log_level = match cli.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    // Initialize logging (respects LOG_FORMAT env var if set)
    if std::env::var("LOG_FORMAT").is_ok() {
        // Use environment configuration
        if let Err(e) = logging::init_from_env() {
            eprintln!("⚠️  Failed to initialize logging: {}", e);
            std::process::exit(1);
        }
    } else {
        // Use CLI-specified level with default format
        let config = logging::LogConfig { level: log_level, ..Default::default() };
        if let Err(e) = logging::init_logging(config) {
            eprintln!("⚠️  Failed to initialize logging: {}", e);
            std::process::exit(1);
        }
    }

    // Match subcommands
    match cli.command {
        Commands::Server { grpc_port, rest_port } => {
            if let Err(e) = server::run_server(grpc_port, rest_port).await {
                eprintln!("❌ Server error: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Client { server_url, ml_api_url } => {
            if let Err(e) = neoland::tui::run_client(&server_url, &ml_api_url).await {
                eprintln!("❌ Erro no cliente TUI: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Test { rest_endpoint, grpc_endpoint } => {
            run_health_check(&rest_endpoint, &grpc_endpoint).await;
        },

        Commands::Restart { grpc_port, rest_port } => {
            restart_server(grpc_port, rest_port).await;
        },

        Commands::Doctor { server_url, ml_api_url } => {
            run_doctor(&server_url, &ml_api_url).await;
        },
    }
}

/// Executa health checks no servidor
async fn run_health_check(rest_endpoint: &str, grpc_endpoint: &str) {
    use tracing::{error, info};

    info!("🧪 Executando health checks...");
    info!("");

    // Test 1: REST health endpoint
    info!("1️⃣ Testando REST API...");
    match reqwest::get(format!("{}/health", rest_endpoint)).await {
        Ok(response) => {
            if response.status().is_success() {
                info!("   ✅ REST OK");
            } else {
                error!("   ❌ REST retornou status: {}", response.status());
            }
        },
        Err(e) => error!("   ❌ Erro ao conectar: {}", e),
    }

    info!("");

    // Test 2: gRPC connection (simple check)
    info!("2️⃣ Verificando gRPC endpoint...");
    info!("   📡 Endpoint: {}", grpc_endpoint);
    info!("   ℹ️  Para teste completo, execute:");
    info!("      nix develop --command cargo test --release test_grpc_chat_stream");

    info!("");

    // Test 3: Check processes
    info!("3️⃣ Verificando processos...");
    let output = Command::new("ps").args(["aux"]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("neoland") {
                info!("   ✅ Servidor rodando");
            } else {
                error!("   ❌ Servidor não encontrado");
            }
        },
        Err(e) => error!("   ❌ Erro ao verificar processos: {}", e),
    }

    info!("");
    info!("✅ Health check concluído");
}

/// Reinicia o servidor (mata processo antigo e inicia novo)
async fn restart_server(grpc_port: u16, rest_port: u16) {
    use tracing::{info, warn};

    info!("🔄 Reiniciando servidor Neoland...");

    // 1. Find and kill old process
    let output = Command::new("ps").args(["aux"]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("neoland") && !line.contains("grep") {
                    // Extract PID (second column)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        let pid = parts[1];
                        info!("🛑 Parando servidor antigo (PID: {})...", pid);
                        let _ = Command::new("kill").arg(pid).output();
                        std::thread::sleep(std::time::Duration::from_secs(2));
                    }
                    break;
                }
            }
        },
        Err(e) => warn!("⚠️  Erro ao buscar processo: {}", e),
    }

    // 2. Start new server
    info!("🚀 Iniciando novo servidor...");
    if let Err(e) = server::run_server(grpc_port, rest_port).await {
        eprintln!("❌ Erro ao iniciar servidor: {}", e);
        std::process::exit(1);
    }
}

/// Diagnóstica o ambiente — neoland doctor
async fn run_doctor(server_url: &str, ml_api_url: &str) {
    let cfg = Config::load();
    let mut any_error = false;

    eprintln!("🩺 neoland doctor\n");

    // 1. Nix development environment
    let in_nix = std::env::var("IN_NIX_SHELL").is_ok()
        || std::env::var("FLAKE_ROOT").is_ok()
        || std::env::var("NIX_BUILD_TOP").is_ok();
    if in_nix {
        eprintln!("  ✅ Nix develop environment detected");
    } else {
        eprintln!("  ⚠  Nix develop shell not detected (run `nix develop`)");
    }

    // 2. Server accessible
    let health_url = format!("{}/health", server_url);
    match reqwest::get(&health_url).await {
        Ok(resp) if resp.status().is_success() => {
            eprintln!("  ✅ Server accessible at {} (status: {})", server_url, resp.status());
        },
        Ok(resp) => {
            eprintln!("  ⚠  Server at {} returned status {}", server_url, resp.status());
        },
        Err(e) => {
            eprintln!("  ❌ Server not reachable at {} — {}", server_url, e);
            eprintln!("       Hint: start the server with `neoland server`");
            any_error = true;
        },
    }

    // 3. ml-offload accessible
    let ml_health_url = format!("{}/health", ml_api_url);
    match reqwest::get(&ml_health_url).await {
        Ok(resp) if resp.status().is_success() => {
            eprintln!("  ✅ ml-offload accessible at {}", ml_api_url);
        },
        Ok(resp) => {
            eprintln!("  ⚠  ml-offload at {} returned status {}", ml_api_url, resp.status());
        },
        Err(_) => {
            eprintln!("  ⚠  ml-offload not reachable at {} (optional — gRPC fallback available)", ml_api_url);
        },
    }

    // 4. Vault configured
    let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_else(|_| cfg.vault.addr.clone());
    if std::env::var("VAULT_ADDR").is_ok() {
        eprintln!("  ✅ VAULT_ADDR set: {}", vault_addr);
    } else {
        eprintln!("  ⚠  VAULT_ADDR not set — using default: {}", vault_addr);
        eprintln!("       Hint: `export VAULT_ADDR=http://localhost:8200`");
    }

    // 5. RUST_LOG defined
    if let Ok(v) = std::env::var("RUST_LOG") {
        eprintln!("  ✅ RUST_LOG set: {}", v);
    } else {
        eprintln!("  ⚠  RUST_LOG not set — using CLI --log-level (default: info)");
    }

    // 6. Config file found
    let config_paths = [
        std::path::PathBuf::from("neoland.toml"),
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".config").join("neoland").join("config.toml")
        },
    ];
    let found_config = config_paths.iter().find(|p| p.exists());
    if let Some(path) = found_config {
        eprintln!("  ✅ Config file found: {}", path.display());
    } else {
        eprintln!("  ⚠  No config file found (./neoland.toml or ~/.config/neoland/config.toml)");
        eprintln!("       Using built-in defaults — all options are optional");
    }

    eprintln!();
    if any_error {
        eprintln!("❌ Some checks failed. See hints above.");
        std::process::exit(1);
    } else {
        eprintln!("✅ Environment looks good (warnings above are non-fatal).");
    }
}
