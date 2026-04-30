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
        #[arg(long, default_value = "http://localhost:3001")]
        server_url: String,

        /// URL do endpoint OpenAI-compatible principal (SecureLLM Bridge API)
        #[arg(long, default_value = "http://localhost:8081")]
        ml_api_url: String,
    },

    /// Executa health checks no servidor
    Test {
        /// Endpoint REST para testes
        #[arg(long, default_value = "http://localhost:3001")]
        rest_endpoint: String,

        /// Endpoint gRPC para testes
        #[arg(long, default_value = "http://localhost:3001")]
        grpc_endpoint: String,

        /// Renderiza o relatório em JSON para automação
        #[arg(long)]
        json: bool,
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

    /// Diagnóstica o ambiente de desenvolvimento
    Doctor {
        /// URL do servidor REST para verificar conectividade
        #[arg(long, default_value = "http://localhost:3001")]
        server_url: String,

        /// URL do gateway LLM principal para verificar
        #[arg(long, default_value = "http://localhost:8081")]
        ml_api_url: String,

        /// Renderiza o relatório em JSON para automação
        #[arg(long)]
        json: bool,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        let mut args: Vec<String> = std::env::args().collect();

        if should_default_to_server(&args) {
            args.insert(1, "server".to_string());
        }

        Self::parse_from(args)
    }
}

fn should_default_to_server(args: &[String]) -> bool {
    match args.get(1).map(String::as_str) {
        None => true,
        Some(arg) if arg.starts_with('-') => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn clap_configuration_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_server_defaults() {
        let cli = Cli::try_parse_from(["neoland", "server"]).expect("server parses");
        match cli.command {
            Commands::Server { grpc_port, rest_port } => {
                assert_eq!(grpc_port, 50051);
                assert_eq!(rest_port, 3001);
            },
            _ => panic!("expected server command"),
        }
        assert_eq!(cli.log_level, "info");
    }

    #[test]
    fn parses_test_command_with_json_and_custom_endpoints() {
        let cli = Cli::try_parse_from([
            "neoland",
            "test",
            "--rest-endpoint",
            "http://127.0.0.1:3003",
            "--grpc-endpoint",
            "http://127.0.0.1:50053",
            "--json",
        ])
        .expect("test parses");

        match cli.command {
            Commands::Test { rest_endpoint, grpc_endpoint, json } => {
                assert_eq!(rest_endpoint, "http://127.0.0.1:3003");
                assert_eq!(grpc_endpoint, "http://127.0.0.1:50053");
                assert!(json);
            },
            _ => panic!("expected test command"),
        }
    }

    #[test]
    fn parses_doctor_with_global_log_level_after_subcommand() {
        let cli = Cli::try_parse_from([
            "neoland",
            "doctor",
            "--server-url",
            "http://localhost:4000",
            "--ml-api-url",
            "http://localhost:9000",
            "--json",
            "--log-level",
            "debug",
        ])
        .expect("doctor parses");

        match cli.command {
            Commands::Doctor { server_url, ml_api_url, json } => {
                assert_eq!(server_url, "http://localhost:4000");
                assert_eq!(ml_api_url, "http://localhost:9000");
                assert!(json);
            },
            _ => panic!("expected doctor command"),
        }
        assert_eq!(cli.log_level, "debug");
    }

    #[test]
    fn parses_global_log_level_before_subcommand() {
        let cli = Cli::try_parse_from(["neoland", "--log-level", "trace", "client"])
            .expect("client parses");

        match cli.command {
            Commands::Client { server_url, ml_api_url } => {
                assert_eq!(server_url, "http://[::1]:50051");
                assert_eq!(ml_api_url, "http://localhost:8081");
            },
            _ => panic!("expected client command"),
        }
        assert_eq!(cli.log_level, "trace");
    }

    #[test]
    fn defaults_to_server_when_no_subcommand_is_provided() {
        let args = vec!["neoland".to_string()];
        assert!(should_default_to_server(&args));
    }

    #[test]
    fn defaults_to_server_when_only_global_flags_are_provided() {
        let args = vec!["neoland".to_string(), "--log-level".to_string(), "debug".to_string()];
        assert!(should_default_to_server(&args));

        let mut with_default = args.clone();
        with_default.insert(1, "server".to_string());
        let cli = Cli::try_parse_from(with_default).expect("server parses with global flags");

        match cli.command {
            Commands::Server { grpc_port, rest_port } => {
                assert_eq!(grpc_port, 50051);
                assert_eq!(rest_port, 3001);
            },
            _ => panic!("expected server command"),
        }
        assert_eq!(cli.log_level, "debug");
    }

    #[test]
    fn does_not_default_when_subcommand_is_explicit() {
        let args = vec!["neoland".to_string(), "doctor".to_string()];
        assert!(!should_default_to_server(&args));
    }
}
