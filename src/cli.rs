use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "neoland")]
#[command(version)]
#[command(about = "Neoland — Autonomous AI Engineering Platform", long_about = None)]
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
        /// Porta gRPC (fallback: NEOLAND_GRPC_PORT, neoland.toml; default 50051)
        #[arg(long)]
        grpc_port: Option<u16>,

        /// Porta REST API (fallback: NEOLAND_REST_PORT, neoland.toml; default 3001)
        #[arg(long)]
        rest_port: Option<u16>,

        /// Caminho para o bundle estático do Web Console (Leptos WASM)
        #[arg(long, env = "NEOLAND_WEB_DIST_DIR")]
        web_dist: Option<String>,
    },

    /// Inicia o cliente TUI (Terminal User Interface)
    Client {
        /// URL do servidor REST
        #[arg(long, env = "NEOLAND_SERVER_URL", default_value = "http://localhost:3001")]
        server_url: String,

        /// URL do servidor gRPC (usado pelo fallback direto de LLM)
        #[arg(long, env = "NEOLAND_GRPC_URL", default_value = "http://localhost:50051")]
        grpc_url: String,

        /// URL do gateway LLM principal
        #[arg(
            long = "neoland-gateway-url",
            env = "NEOLAND_GATEWAY_URL",
            default_value = "http://localhost:8080"
        )]
        neoland_gateway_url: String,
    },

    /// Executa health checks no servidor
    Test {
        /// Endpoint REST para testes
        #[arg(long, default_value = "http://localhost:3001")]
        rest_endpoint: String,

        /// Endpoint gRPC para testes
        #[arg(long, default_value = "http://localhost:50051")]
        grpc_endpoint: String,

        /// Renderiza o relatório em JSON
        #[arg(long)]
        json: bool,
    },

    /// Reinicia o servidor
    Restart {
        /// Porta gRPC do servidor
        #[arg(long, default_value = "50051")]
        grpc_port: u16,

        /// Porta REST do servidor
        #[arg(long, default_value = "3001")]
        rest_port: u16,
    },

    /// Diagnóstica o ambiente
    Doctor {
        /// URL do servidor REST
        #[arg(long, default_value = "http://localhost:3001")]
        server_url: String,

        /// URL do gateway LLM principal
        #[arg(long = "neoland-gateway-url", default_value = "http://localhost:8080")]
        neoland_gateway_url: String,

        /// Renderiza o relatório em JSON
        #[arg(long)]
        json: bool,
    },

    /// Abre a landing page no browser
    Site {
        /// URL base do servidor
        #[arg(long, default_value = "http://localhost:3001")]
        server_url: String,
    },

    /// Abre o Web Console no browser
    Web {
        /// URL base do servidor
        #[arg(long, default_value = "http://localhost:3001")]
        server_url: String,
    },

    /// Abre a documentação da API (Swagger UI) no browser
    Docs {
        /// URL base do servidor
        #[arg(long, default_value = "http://localhost:3001")]
        server_url: String,
    },

    /// Aplica as migrations do banco (embutidas de migrations/)
    Migrate {
        /// URL do Postgres (fallback: NEOLAND_DATABASE_URL, DATABASE_URL)
        #[arg(long)]
        database_url: Option<String>,
    },

    /// Gera certificados mTLS para desenvolvimento local
    GenCerts {
        /// Dias de validade dos certificados
        #[arg(long, default_value = "365")]
        days: u16,

        /// Common Name do servidor
        #[arg(long, default_value = "neoland.local")]
        cn: String,
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
    // `neoland --help` / `--version` devem mostrar o help/versão da raiz
    // (lista de subcomandos), não ser reescritos para `neoland server --help`.
    let wants_help_or_version = args
        .iter()
        .skip(1)
        .any(|arg| matches!(arg.as_str(), "-h" | "--help" | "-V" | "--version"));
    if wants_help_or_version {
        return false;
    }

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
            // Sem flags: os campos ficam None e a resolução final (env > TOML >
            // default) acontece via Config no bin. web_dist pode vir do env
            // NEOLAND_WEB_DIST_DIR do ambiente de teste, então não é assertado.
            Commands::Server { grpc_port, rest_port, web_dist: _ } => {
                assert_eq!(grpc_port, None);
                assert_eq!(rest_port, None);
            },
            _ => panic!("expected server command"),
        }
        assert_eq!(cli.log_level, "info");
    }

    #[test]
    fn server_flags_override_defaults() {
        let cli = Cli::try_parse_from([
            "neoland",
            "server",
            "--grpc-port",
            "6000",
            "--rest-port",
            "6001",
        ])
        .expect("server parses");
        match cli.command {
            Commands::Server { grpc_port, rest_port, .. } => {
                assert_eq!(grpc_port, Some(6000));
                assert_eq!(rest_port, Some(6001));
            },
            _ => panic!("expected server command"),
        }
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
            "--neoland-gateway-url",
            "http://localhost:9000",
            "--json",
            "--log-level",
            "debug",
        ])
        .expect("doctor parses");

        match cli.command {
            Commands::Doctor { server_url, neoland_gateway_url, json } => {
                assert_eq!(server_url, "http://localhost:4000");
                assert_eq!(neoland_gateway_url, "http://localhost:9000");
                assert!(json);
            },
            _ => panic!("expected doctor command"),
        }
        assert_eq!(cli.log_level, "debug");
    }

    #[test]
    fn parses_global_log_level_before_subcommand() {
        // Isolate from NEOLAND_SERVER_URL / NEOLAND_GATEWAY_URL env vars that
        // the devShell sets — we only care that log-level and the subcommand
        // are parsed correctly, not the specific URL values.
        let cli = Cli::try_parse_from([
            "neoland",
            "--log-level",
            "trace",
            "client",
            "--server-url",
            "http://localhost:3001",
            "--neoland-gateway-url",
            "http://localhost:8080",
        ])
        .expect("client parses");

        match cli.command {
            Commands::Client { server_url, grpc_url, neoland_gateway_url } => {
                assert_eq!(server_url, "http://localhost:3001");
                assert_eq!(grpc_url, "http://localhost:50051");
                assert_eq!(neoland_gateway_url, "http://localhost:8080");
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
            Commands::Server { grpc_port, rest_port, web_dist: _ } => {
                assert_eq!(grpc_port, None);
                assert_eq!(rest_port, None);
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

    #[test]
    fn does_not_default_when_help_or_version_is_requested() {
        for flag in ["-h", "--help", "-V", "--version"] {
            let args = vec!["neoland".to_string(), flag.to_string()];
            assert!(!should_default_to_server(&args), "flag {flag} must reach root parser");
        }

        // Help depois de flags globais também deve mostrar o help da raiz.
        let args = vec![
            "neoland".to_string(),
            "--log-level".to_string(),
            "debug".to_string(),
            "--help".to_string(),
        ];
        assert!(!should_default_to_server(&args));
    }

    #[test]
    fn root_help_lists_subcommands() {
        let mut cmd = Cli::command();
        let help = cmd.render_long_help().to_string();
        for sub in ["server", "client", "test", "restart", "doctor", "gen-certs"] {
            assert!(help.contains(sub), "root help must list `{sub}`");
        }
    }
}
