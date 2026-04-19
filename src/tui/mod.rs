// TUI module for Neoland
// Terminal User Interface using ratatui + crossterm

pub mod app;
pub mod events;
pub mod presets;
pub mod ui;

use std::io;

use anyhow::Result;
use app::{AppState, ConnectionStatus};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::{handle_events, AppEvent};
use ratatui::{backend::CrosstermBackend, Terminal};
use ui::render;

/// Executa o cliente TUI
pub async fn run_client(server_url: &str, ml_api_url: &str) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = AppState::new(server_url.to_string(), ml_api_url.to_string());

    // Health check before entering main loop
    check_server_health(&mut app).await;
    terminal.draw(|f| render(f, &app))?;

    // Main event loop
    loop {
        terminal.draw(|f| render(f, &app))?;

        if let Some(event) = handle_events(&mut app)? {
            match event {
                AppEvent::Quit => break,
                AppEvent::SendMessage => {
                    if !app.input_buffer.is_empty() {
                        // Send message to server
                        app.is_thinking = true;
                        terminal.draw(|f| render(f, &app))?;
                        send_message_to_server(&mut app).await?;
                        app.is_thinking = false;
                    }
                },
                AppEvent::ClearChat => {
                    app.messages.clear();
                    app.add_system_message("🗑️ Chat limpo");
                },
                AppEvent::ApplyPreset(preset_name) => {
                    app.apply_preset(&preset_name);
                },
                AppEvent::ToggleSidebar => {
                    app.sidebar_visible = !app.sidebar_visible;
                },
                AppEvent::Input(c) => {
                    app.input_buffer.push(c);
                },
                AppEvent::Backspace => {
                    app.input_buffer.pop();
                },
                AppEvent::ScrollUp => {
                    if app.scroll_offset > 0 {
                        app.scroll_offset -= 1;
                    }
                },
                AppEvent::ScrollDown => {
                    app.scroll_offset += 1;
                },
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Verifica o servidor antes de iniciar o TUI e actualiza app.connection_status
async fn check_server_health(app: &mut AppState) {
    let health_url = format!("{}/health", app.server_url.replace("[::1]", "localhost"));
    match reqwest::get(&health_url).await {
        Ok(resp) if resp.status().is_success() => {
            app.connection_status = ConnectionStatus::Connected;
            app.add_system_message(&format!(
                "✅ Servidor acessível em {} — pronto",
                app.server_url
            ));
        },
        Ok(resp) => {
            app.connection_status = ConnectionStatus::Degraded;
            app.add_system_message(&format!(
                "⚠ Servidor em {} retornou status {} — pode estar degradado",
                app.server_url,
                resp.status()
            ));
        },
        Err(e) => {
            app.connection_status = ConnectionStatus::Offline;
            app.add_system_message(&format!(
                "❌ Servidor não acessível em {} ({})\n  Execute `neoland server` para iniciar",
                app.server_url, e
            ));
        },
    }
}

/// Envia mensagem usando UnifiedLLMClient (LocalFirst strategy)
async fn send_message_to_server(app: &mut AppState) -> Result<()> {
    let message = app.input_buffer.clone();
    app.add_user_message(&message);
    app.input_buffer.clear();

    let req_start = std::time::Instant::now();

    // Initialize SecretsManager (Phase 1.2)
    let secrets_manager = match crate::secrets::SecretsManager::new().await {
        Ok(sm) => std::sync::Arc::new(sm),
        Err(e) => {
            tracing::error!(error = %e, "TUI: failed to initialize SecretsManager");
            app.add_system_message(&format!("❌ Failed to initialize secrets manager: {}", e));
            return Ok(());
        },
    };

    // Load SecureLLM API key from environment (if available)
    let securellm_provider = std::env::var("SECURELLM_PROVIDER").ok().map(|provider| {
        let api_key = std::env::var(format!("{}_API_KEY", provider.to_uppercase())).ok();
        (provider.leak() as &str, api_key)
    });

    // Create UnifiedLLMClient with LocalFirst strategy (Phase 1.2: now async)
    let client = match crate::llm::UnifiedLLMClient::new_local_first(
        app.ml_api_url.clone(),
        secrets_manager,
        securellm_provider,
    )
    .await
    {
        Ok(client) => client,
        Err(e) => {
            tracing::error!(error = %e, "TUI: failed to initialize UnifiedLLMClient");
            app.add_system_message(&format!("❌ Failed to initialize LLM client: {}", e));
            return Ok(());
        },
    };

    // Send chat request (LocalFirst: ml-offload → SecureLLM fallback)
    match client
        .chat(&message, Some(app.config.temperature), Some(app.config.max_tokens as u32))
        .await
    {
        Ok(response) => {
            app.last_latency_ms = req_start.elapsed().as_millis() as u64;
            // Rough token estimate (words * 1.3)
            let tok_estimate = (response.split_whitespace().count() as f64 * 1.3) as u64;
            app.session_tokens = app.session_tokens.saturating_add(tok_estimate);
            app.active_backend = Some("ml-offload/cloud".to_string());
            app.add_assistant_message(&response);
        },
        Err(e) => {
            tracing::warn!(error = %e, "TUI: UnifiedLLMClient failed, trying gRPC fallback");
            app.add_system_message(&format!(
                "⚠ ml-offload/cloud backends failed — {}\n  Tentando gRPC local ({})...",
                e, app.server_url
            ));

            // Last resort: Try local gRPC server directly
            match try_grpc_fallback(app, message).await {
                Ok(()) => {
                    app.last_latency_ms = req_start.elapsed().as_millis() as u64;
                    app.active_backend = Some("gRPC local".to_string());
                },
                Err(grpc_err) => {
                    app.connection_status = ConnectionStatus::Offline;
                    app.add_system_message(&format!(
                        "⚠ gRPC local [{}]: {}\n❌ Todos os backends falharam. Execute `neoland \
                         doctor` para diagnóstico.",
                        app.server_url, grpc_err
                    ));
                },
            }
        },
    }

    Ok(())
}

/// Last resort fallback: Direct gRPC connection
async fn try_grpc_fallback(app: &mut AppState, message: String) -> Result<()> {
    use crate::llamachat::{llama_service_client::LlamaServiceClient, ChatRequest};

    let mut client = LlamaServiceClient::connect(app.server_url.clone()).await?;

    let config = app.config.clone();
    let request = ChatRequest {
        prompt: message.clone(),
        model_id: "qwen-1.8b".to_string(),
        use_local: true,
        temperature: Some(config.temperature),
        top_p: Some(config.top_p),
        max_tokens: Some(config.max_tokens),
        repetition_penalty: Some(config.repetition_penalty),
        typical_p: Some(config.typical_p),
        epsilon_cutoff: Some(config.epsilon_cutoff),
        eta_cutoff: Some(config.eta_cutoff),
        tail_free_sampling: Some(config.tail_free_sampling),
        top_a: Some(config.top_a),
        context_top_k: Some(config.context_top_k),
        context_similarity_threshold: Some(config.context_similarity_threshold),
        disable_context: Some(config.disable_context),
        system_prompt: if !config.system_prompt.is_empty() {
            Some(config.system_prompt.clone())
        } else {
            None
        },
        enable_commands: Some(config.enable_commands),
        allowed_commands: config
            .allowed_commands
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        session_id: None,
        streaming: Some(true),
    };

    // Stream response — accumulate via pending_message for incremental display
    let mut stream = client.chat_stream(request).await?.into_inner();
    app.pending_message = Some(String::new());

    while let Ok(Some(chunk)) = stream.message().await {
        if chunk.content == "[[METADATA_UPDATE]]" {
            continue;
        }
        if let Some(ref mut pending) = app.pending_message {
            pending.push_str(&chunk.content);
        }
    }

    // Finalize: move pending_message → messages
    if let Some(full_response) = app.pending_message.take() {
        if !full_response.is_empty() {
            let tok_estimate = (full_response.split_whitespace().count() as f64 * 1.3) as u64;
            app.session_tokens = app.session_tokens.saturating_add(tok_estimate);
            app.add_assistant_message(&full_response);
        }
    }
    Ok(())
}
