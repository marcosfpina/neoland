use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "neoland")]
#[command(version = "0.1.0")]
#[command(about = "Neoland AI Assistant - CLI unificado para servidor e cliente", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Nível de logging (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info")]
    pub log_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inicia o servidor gRPC + REST
    Server {
        /// Porta gRPC (IPv6)
        #[arg(long, default_value = "50051")]
        grpc_port: u16,
        
        /// Porta REST API
        #[arg(long, default_value = "3001")]
        rest_port: u16,
    },
    
    /// Inicia o cliente TUI (Terminal User Interface)
    Client {
        /// URL do servidor gRPC
        #[arg(long, default_value = "http://[::1]:50051")]
        server_url: String,

        /// URL do endpoint OpenAI-compatible (ml-offload-api ou llama.cpp)
        #[arg(long, default_value = "http://localhost:8080")]
        ml_api_url: String,
    },
    
    /// Executa health checks no servidor
    Test {
        /// Endpoint REST para testes
        #[arg(long, default_value = "http://localhost:3001")]
        rest_endpoint: String,
        
        /// Endpoint gRPC para testes
        #[arg(long, default_value = "http://[::1]:50051")]
        grpc_endpoint: String,
    },
    
    /// Reinicia o servidor (mata processo antigo e inicia novo)
    Restart {
        /// Porta gRPC do servidor a reiniciar
        #[arg(long, default_value = "50051")]
        grpc_port: u16,
        
        /// Porta REST do servidor a reiniciar
        #[arg(long, default_value = "3001")]
        rest_port: u16,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
