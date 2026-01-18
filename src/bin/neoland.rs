use llamachat_poc::{cli, server};
use cli::{Cli, Commands};
use clap::Parser;
use std::process::Command;

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse_args();
    
    // Setup logging
    let log_filter = match cli.log_level.as_str() {
        "trace" => "trace",
        "debug" => "debug",
        "info" => "info",
        "warn" => "warn",
        "error" => "error",
        _ => "info",
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_target(false)
        .with_thread_ids(false)
        .init();
    
    // Match subcommands
    match cli.command {
        Commands::Server { grpc_port, rest_port } => {
            if let Err(e) = server::run_server(grpc_port, rest_port).await {
                eprintln!("❌ Server error: {}", e);
                std::process::exit(1);
            }
        }
        
        Commands::Client { server_url, ml_api_url } => {
            if let Err(e) = llamachat_poc::tui::run_client(&server_url, &ml_api_url).await {
                eprintln!("❌ Erro no cliente TUI: {}", e);
                std::process::exit(1);
            }
        }
        
        Commands::Test { rest_endpoint, grpc_endpoint } => {
            run_health_check(&rest_endpoint, &grpc_endpoint).await;
        }
        
        Commands::Restart { grpc_port, rest_port } => {
            restart_server(grpc_port, rest_port).await;
        }
    }
}

/// Executa health checks no servidor
async fn run_health_check(rest_endpoint: &str, grpc_endpoint: &str) {
    use tracing::{info, error};
    
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
        }
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
    let output = Command::new("ps")
        .args(&["aux"])
        .output();
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("llamachat-server") {
                info!("   ✅ Servidor rodando");
            } else {
                error!("   ❌ Servidor não encontrado");
            }
        }
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
    let output = Command::new("ps")
        .args(&["aux"])
        .output();
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("llamachat-server") && !line.contains("grep") {
                    // Extract PID (second column)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        let pid = parts[1];
                        info!("🛑 Parando servidor antigo (PID: {})...", pid);
                        let _ = Command::new("kill")
                            .arg(pid)
                            .output();
                        std::thread::sleep(std::time::Duration::from_secs(2));
                    }
                    break;
                }
            }
        }
        Err(e) => warn!("⚠️  Erro ao buscar processo: {}", e),
    }
    
    // 2. Start new server
    info!("🚀 Iniciando novo servidor...");
    if let Err(e) = server::run_server(grpc_port, rest_port).await {
        eprintln!("❌ Erro ao iniciar servidor: {}", e);
        std::process::exit(1);
    }
}
