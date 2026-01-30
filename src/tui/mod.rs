// TUI module for Neoland
// Terminal User Interface using ratatui + crossterm

pub mod app;
pub mod ui;
pub mod events;
pub mod presets;

use app::AppState;
use events::{handle_events, AppEvent};
use ui::render;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

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
                }
                AppEvent::ClearChat => {
                    app.messages.clear();
                    app.add_system_message("🗑️ Chat limpo");
                }
                AppEvent::ApplyPreset(preset_name) => {
                    app.apply_preset(&preset_name);
                }
                AppEvent::ToggleSidebar => {
                    app.sidebar_visible = !app.sidebar_visible;
                }
                AppEvent::Input(c) => {
                    app.input_buffer.push(c);
                }
                AppEvent::Backspace => {
                    app.input_buffer.pop();
                }
                AppEvent::ScrollUp => {
                    if app.scroll_offset > 0 {
                        app.scroll_offset -= 1;
                    }
                }
                AppEvent::ScrollDown => {
                    app.scroll_offset += 1;
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Envia mensagem usando UnifiedLLMClient (LocalFirst strategy)
async fn send_message_to_server(app: &mut AppState) -> Result<()> {

    let message = app.input_buffer.clone();
    app.add_user_message(&message);
    app.input_buffer.clear();

    // Initialize SecretsManager (Phase 1.2)
    let secrets_manager = match crate::secrets::SecretsManager::new().await {
        Ok(sm) => std::sync::Arc::new(sm),
        Err(e) => {
            app.add_system_message(&format!("❌ Failed to initialize secrets manager: {}", e));
            return Ok(());
        }
    };

    // Load SecureLLM API key from environment (if available)
    let securellm_provider = std::env::var("SECURELLM_PROVIDER")
        .ok()
        .map(|provider| {
            let api_key = std::env::var(format!("{}_API_KEY", provider.to_uppercase())).ok();
            (provider.leak() as &str, api_key)
        });

    // Create UnifiedLLMClient with LocalFirst strategy (Phase 1.2: now async)
    let client = match crate::llm::UnifiedLLMClient::new_local_first(
        app.ml_api_url.clone(),
        secrets_manager,
        securellm_provider,
    ).await {
        Ok(client) => client,
        Err(e) => {
            app.add_system_message(&format!("❌ Failed to initialize LLM client: {}", e));
            return Ok(());
        }
    };

    // Send chat request (LocalFirst: ml-offload → SecureLLM fallback)
    match client.chat(
        &message,
        Some(app.config.temperature),
        Some(app.config.max_tokens as u32),
    ).await {
        Ok(response) => {
            app.add_assistant_message(&response);
        }
        Err(e) => {
            app.add_system_message(&format!("❌ All LLM backends failed: {}", e));
            
            // Last resort: Try local gRPC server directly
            if let Err(grpc_err) = try_grpc_fallback(app, message).await {
                app.add_system_message(&format!("❌ gRPC fallback also failed: {}", grpc_err));
            }
        }
    }

    Ok(())
}

/// Last resort fallback: Direct gRPC connection
async fn try_grpc_fallback(app: &mut AppState, message: String) -> Result<()> {
    use crate::llamachat::llama_service_client::LlamaServiceClient;
    use crate::llamachat::ChatRequest;

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

    // Stream response
    let mut stream = client.chat_stream(request).await?.into_inner();
    let mut response_text = String::new();

    while let Ok(Some(chunk)) = stream.message().await {
        if chunk.content == "[[METADATA_UPDATE]]" {
            continue;
        }
        response_text.push_str(&chunk.content);
        // TODO: Update UI in real-time (need streaming support in AppState)
    }

    app.add_assistant_message(&response_text);
    Ok(())
}
