use cli::{Cli, Commands};
use neoland::{
    cli,
    commands::{
        collect_doctor_report, collect_health_check_report, parse_log_level, render_doctor_report,
        render_health_check_report, render_restart_report, restart_server, SystemCommandRuntime,
    },
    config::Config,
    logging, server,
};

fn open_browser(url: &str) {
    let url = url.to_string();
    // Try xdg-open (Linux), open (macOS), start (Windows)
    if std::process::Command::new("xdg-open").arg(&url).spawn().is_ok() {
        return;
    }
    if std::process::Command::new("open").arg(&url).spawn().is_ok() {
        return;
    }
    if std::process::Command::new("cmd").args(["/c", "start", &url]).spawn().is_ok() {
        return;
    }
    eprintln!("→ Abra no browser: {url}");
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse_args();

    // Phase 4.2: Setup structured logging
    let log_level = parse_log_level(&cli.log_level);

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

    let runtime = SystemCommandRuntime;

    // Match subcommands
    match cli.command {
        Commands::Server { grpc_port, rest_port, web_dist } => {
            // Precedência: flag CLI > env (NEOLAND_*) > neoland.toml > default.
            // Config::load() já aplica env sobre TOML; a flag só entra se passada.
            let config = Config::load();
            let grpc_port = grpc_port.unwrap_or(config.server.grpc_port);
            let rest_port = rest_port.unwrap_or(config.server.rest_port);
            let web_dist = web_dist.unwrap_or_else(|| config.server.web_dist_dir.clone());
            if let Err(e) = server::run_server(grpc_port, rest_port, &web_dist).await {
                eprintln!("❌ Server error: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Client { server_url, grpc_url, neoland_gateway_url } => {
            if let Err(e) =
                neoland::tui::run_client(&server_url, &grpc_url, &neoland_gateway_url).await
            {
                eprintln!("❌ Erro no cliente TUI: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Test { rest_endpoint, grpc_endpoint, json } => {
            let report =
                collect_health_check_report(&runtime, &rest_endpoint, &grpc_endpoint).await;
            println!("{}", render_health_check_report(&report, json));
            if report.has_errors() {
                std::process::exit(1);
            }
        },

        Commands::Restart { grpc_port, rest_port } => {
            match restart_server(&runtime, grpc_port, rest_port).await {
                Ok(report) => println!("{}", render_restart_report(&report)),
                Err(e) => {
                    eprintln!("❌ Failed to restart server: {}", e);
                    std::process::exit(1);
                },
            }
        },

        Commands::Doctor { server_url, neoland_gateway_url, json } => {
            let config = Config::load();
            let report =
                collect_doctor_report(&runtime, &config, &server_url, &neoland_gateway_url).await;
            println!("{}", render_doctor_report(&report, json));
            if report.has_errors() {
                std::process::exit(1);
            }
        },

        Commands::Site { server_url } => {
            open_browser(&format!("{}/landing.html", server_url.trim_end_matches('/')));
        },

        Commands::Web { server_url } => {
            open_browser(&server_url);
        },

        Commands::Docs { server_url } => {
            open_browser(&format!("{}/swagger-ui/", server_url.trim_end_matches('/')));
        },

        Commands::Migrate { database_url } => {
            let url = database_url
                .or_else(|| std::env::var("NEOLAND_DATABASE_URL").ok())
                .or_else(|| std::env::var("DATABASE_URL").ok());
            let Some(url) = url else {
                eprintln!(
                    "❌ Banco não configurado — use --database-url, NEOLAND_DATABASE_URL ou DATABASE_URL"
                );
                std::process::exit(1);
            };
            match neoland::storage::run_migrations(&url).await {
                Ok(()) => println!("✅ Migrations aplicadas"),
                Err(e) => {
                    eprintln!("❌ Falha ao aplicar migrations: {e}");
                    std::process::exit(1);
                },
            }
        },

        Commands::Config { action } => match action {
            neoland::cli::ConfigAction::Show { json } => {
                let cfg = Config::load().redacted();
                let rendered = if json {
                    serde_json::to_string_pretty(&cfg).expect("config serializes to JSON")
                } else {
                    toml::to_string_pretty(&cfg).expect("config serializes to TOML")
                };
                println!("{rendered}");
            },
            neoland::cli::ConfigAction::Validate => {
                let mut failed = false;
                for path in Config::candidate_paths() {
                    if !path.exists() {
                        continue;
                    }
                    let parse = std::fs::read_to_string(&path)
                        .map_err(|e| e.to_string())
                        .and_then(|c| toml::from_str::<Config>(&c).map_err(|e| e.to_string()));
                    match parse {
                        Ok(_) => println!("✅ {} — parse OK", path.display()),
                        Err(e) => {
                            failed = true;
                            println!("❌ {} — {e}", path.display());
                        },
                    }
                }
                let problems = Config::load().validate();
                for p in &problems {
                    println!("⚠️  {p}");
                }
                if problems.is_empty() && !failed {
                    println!("✅ Config efetiva válida");
                } else {
                    std::process::exit(1);
                }
            },
            neoland::cli::ConfigAction::Init { force } => {
                let path = std::path::Path::new("neoland.toml");
                if path.exists() && !force {
                    eprintln!("❌ neoland.toml já existe (use --force para sobrescrever)");
                    std::process::exit(1);
                }
                let body =
                    toml::to_string_pretty(&Config::default()).expect("default config serializes");
                let header = "# Neoland — configuração\n# Precedência: flag CLI > env NEOLAND_* > este arquivo > defaults\n# Referência completa de variáveis de ambiente: .env.example\n\n";
                if let Err(e) = std::fs::write(path, format!("{header}{body}")) {
                    eprintln!("❌ Falha ao escrever neoland.toml: {e}");
                    std::process::exit(1);
                }
                println!("✅ neoland.toml gerado");
            },
        },

        Commands::GenCerts { days, cn } => {
            let status = std::process::Command::new("bash")
                .arg("scripts/gen-certs.sh")
                .arg("--auto")
                .arg("--days")
                .arg(days.to_string())
                .arg("--cn")
                .arg(&cn)
                .status()
                .unwrap_or_else(|_| {
                    eprintln!("❌ scripts/gen-certs.sh não encontrado. Rode do raiz do projeto.");
                    std::process::exit(1);
                });
            if !status.success() {
                std::process::exit(1);
            }
        },
    }
}
