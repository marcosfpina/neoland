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
