use std::{io, sync::Arc, time::Duration};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

pub mod app;
pub mod events;
pub mod presets;
pub mod ui;

use app::{AppState, ConnectionStatus};
use events::Action;
use ui::render;

// ── LLM channel events ────────────────────────────────────────────────

enum LlmEvent {
    Chunk(String),
    Done { content: String, latency_ms: u64, tokens: u64, backend: String },
    StreamDone { latency_ms: u64, tokens: u64, backend: String },
    Error(String),
}

// ── Entry point ───────────────────────────────────────────────────────

pub async fn run_client(server_url: &str, ml_api_url: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Keyboard events: blocking thread → async channel
    let (key_tx, mut key_rx) = mpsc::channel::<Event>(64);
    let (llm_tx, mut llm_rx) = mpsc::channel::<LlmEvent>(16);

    std::thread::spawn(move || {
        while let Ok(ev) = event::read() {
            if key_tx.blocking_send(ev).is_err() {
                break;
            }
        }
    });

    let mut app = AppState::new(server_url.to_string(), ml_api_url.to_string());
    check_server_health(&mut app).await;

    let mut tick = tokio::time::interval(Duration::from_millis(80));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        terminal.draw(|f| render(f, &mut app))?;

        tokio::select! {
            biased;

            Some(ev) = llm_rx.recv() => {
                match ev {
                    LlmEvent::Chunk(s) => {
                        match app.pending_message {
                            Some(ref mut p) => p.push_str(&s),
                            None => app.pending_message = Some(s),
                        }
                        app.auto_scroll = true;
                    }
                    LlmEvent::Done { content, latency_ms, tokens, backend } => {
                        app.pending_message = None;
                        app.is_thinking = false;
                        app.add_assistant_message(&content);
                        app.last_latency_ms = latency_ms;
                        app.session_tokens = app.session_tokens.saturating_add(tokens);
                        app.active_backend = Some(backend);
                    }
                    LlmEvent::StreamDone { latency_ms, tokens, backend } => {
                        if let Some(msg) = app.pending_message.take() {
                            if !msg.is_empty() {
                                app.add_assistant_message(&msg);
                            }
                        }
                        app.is_thinking = false;
                        app.last_latency_ms = latency_ms;
                        app.session_tokens = app.session_tokens.saturating_add(tokens);
                        app.active_backend = Some(backend);
                    }
                    LlmEvent::Error(e) => {
                        app.is_thinking = false;
                        app.pending_message = None;
                        app.add_system_message(&format!("error: {}", e));
                    }
                }
            }

            Some(key_ev) = key_rx.recv() => {
                match key_ev {
                    Event::Key(key)
                        if key.kind == KeyEventKind::Press
                            || key.kind == KeyEventKind::Repeat =>
                    {
                        match process_key(&mut app, key.code, key.modifiers) {
                            Action::Quit => break,
                            Action::Send(msg) => {
                                app.is_thinking = true;
                                app.auto_scroll = true;
                                let tx = llm_tx.clone();
                                let ml_url = app.ml_api_url.clone();
                                let srv_url = app.server_url.clone();
                                let cfg = app.config.clone();
                                tokio::spawn(run_llm(msg, cfg, ml_url, srv_url, tx));
                            }
                            Action::None => {}
                        }
                    }
                    _ => {}
                }
            }

            _ = tick.tick() => {
                if app.is_thinking {
                    app.tick = app.tick.wrapping_add(1);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}

// ── Key → Action ──────────────────────────────────────────────────────

fn process_key(app: &mut AppState, code: KeyCode, mods: KeyModifiers) -> Action {
    match (code, mods) {
        // ── Quit ──────────────────────────────────────────────────────
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Action::Quit,
        (KeyCode::Esc, _) => return Action::Quit,

        // ── Send ──────────────────────────────────────────────────────
        (KeyCode::Enter, _) if !app.input_buffer.is_empty() && !app.is_thinking => {
            let msg = std::mem::take(&mut app.input_buffer);
            app.cursor_pos = 0;
            app.add_user_message(&msg);
            return Action::Send(msg);
        },

        // ── Clear chat ────────────────────────────────────────────────
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
            app.messages.clear();
            app.scroll_offset = 0;
            app.auto_scroll = true;
        },

        // ── Presets ───────────────────────────────────────────────────
        (KeyCode::Char('1'), KeyModifiers::CONTROL) => app.apply_preset("balanced"),
        (KeyCode::Char('2'), KeyModifiers::CONTROL) => app.apply_preset("creative"),
        (KeyCode::Char('3'), KeyModifiers::CONTROL) => app.apply_preset("precise"),
        (KeyCode::Char('4'), KeyModifiers::CONTROL) => app.apply_preset("research"),
        (KeyCode::Char('5'), KeyModifiers::CONTROL) => app.apply_preset("safe"),

        // ── Sidebar ───────────────────────────────────────────────────
        (KeyCode::Tab, _) => app.sidebar_visible = !app.sidebar_visible,

        // ── Ctrl/Alt combos (must be before generic Char arm) ─────────
        (KeyCode::Left, KeyModifiers::CONTROL) | (KeyCode::Char('b'), KeyModifiers::ALT) => {
            app.cursor_word_left()
        },
        (KeyCode::Right, KeyModifiers::CONTROL) | (KeyCode::Char('f'), KeyModifiers::ALT) => {
            app.cursor_word_right()
        },
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => app.cursor_pos = 0,
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => app.cursor_pos = app.input_buffer.len(),
        (KeyCode::Backspace, KeyModifiers::CONTROL)
        | (KeyCode::Char('w'), KeyModifiers::CONTROL) => app.delete_word_back(),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.input_buffer.drain(..app.cursor_pos);
            app.cursor_pos = 0;
        },
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            app.input_buffer.truncate(app.cursor_pos);
        },

        // ── Scroll to bottom when buffer empty (vim-g) ────────────────
        (KeyCode::Char('g'), KeyModifiers::NONE) if app.input_buffer.is_empty() => {
            app.auto_scroll = true;
        },

        // ── Generic char input ────────────────────────────────────────
        (KeyCode::Char(c), m) if m == KeyModifiers::NONE || m == KeyModifiers::SHIFT => {
            app.insert_char(c)
        },

        // ── Editing ───────────────────────────────────────────────────
        (KeyCode::Backspace, KeyModifiers::NONE) => app.backspace(),
        (KeyCode::Delete, _) if app.cursor_pos < app.input_buffer.len() => {
            app.input_buffer.remove(app.cursor_pos);
        },

        // ── Cursor movement ───────────────────────────────────────────
        (KeyCode::Left, KeyModifiers::NONE) => app.cursor_left(),
        (KeyCode::Right, KeyModifiers::NONE) => app.cursor_right(),
        (KeyCode::Home, _) => app.cursor_pos = 0,
        (KeyCode::End, _) => app.cursor_pos = app.input_buffer.len(),

        // ── Chat scroll ───────────────────────────────────────────────
        (KeyCode::Up, _) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(3);
            app.auto_scroll = false;
        },
        (KeyCode::Down, _) => {
            app.scroll_offset = app.scroll_offset.saturating_add(3);
        },
        (KeyCode::PageUp, _) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
            app.auto_scroll = false;
        },
        (KeyCode::PageDown, _) => {
            app.scroll_offset = app.scroll_offset.saturating_add(10);
        },

        _ => {},
    }

    Action::None
}

// ── LLM task (spawned per request) ───────────────────────────────────

async fn run_llm(
    message: String,
    config: presets::QueryConfig,
    ml_api_url: String,
    server_url: String,
    tx: mpsc::Sender<LlmEvent>,
) {
    let start = std::time::Instant::now();

    let secrets = match crate::secrets::SecretsManager::new().await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tx.send(LlmEvent::Error(format!("secrets: {}", e))).await.ok();
            return;
        },
    };

    let securellm_provider = std::env::var("SECURELLM_PROVIDER").ok().map(|p| {
        let key = std::env::var(format!("{}_API_KEY", p.to_uppercase())).ok();
        (p.leak() as &str, key)
    });

    let client = match crate::llm::UnifiedLLMClient::new_local_first(
        ml_api_url,
        secrets,
        securellm_provider,
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            tx.send(LlmEvent::Error(format!("llm client: {}", e))).await.ok();
            return;
        },
    };

    match client
        .chat(&message, Some(config.temperature), Some(config.max_tokens as u32))
        .await
    {
        Ok(response) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let tokens = (response.split_whitespace().count() as f64 * 1.3) as u64;
            tx.send(LlmEvent::Done {
                content: response,
                latency_ms,
                tokens,
                backend: "ml-offload".to_string(),
            })
            .await
            .ok();
        },
        Err(e) => {
            tracing::warn!(error = %e, "TUI: UnifiedLLMClient failed, trying gRPC fallback");
            if let Err(grpc_err) =
                try_grpc_fallback(&message, &server_url, &config, &tx, start).await
            {
                tx.send(LlmEvent::Error(format!("all backends failed: {}", grpc_err)))
                    .await
                    .ok();
            }
        },
    }
}

async fn try_grpc_fallback(
    message: &str,
    server_url: &str,
    config: &presets::QueryConfig,
    tx: &mpsc::Sender<LlmEvent>,
    start: std::time::Instant,
) -> Result<()> {
    use crate::llamachat::{llama_service_client::LlamaServiceClient, ChatRequest};

    let mut client = LlamaServiceClient::connect(server_url.to_string()).await?;

    let request = ChatRequest {
        prompt: message.to_string(),
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
        system_prompt: if config.system_prompt.is_empty() {
            None
        } else {
            Some(config.system_prompt.clone())
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

    let mut stream = client.chat_stream(request).await?.into_inner();
    let mut total = String::new();

    while let Ok(Some(chunk)) = stream.message().await {
        if chunk.content == "[[METADATA_UPDATE]]" || chunk.content.is_empty() {
            continue;
        }
        total.push_str(&chunk.content);
        tx.send(LlmEvent::Chunk(chunk.content)).await.ok();
    }

    let latency_ms = start.elapsed().as_millis() as u64;
    let tokens = (total.split_whitespace().count() as f64 * 1.3) as u64;

    tx.send(LlmEvent::StreamDone { latency_ms, tokens, backend: "gRPC".to_string() })
        .await
        .ok();

    Ok(())
}

// ── Server health check ───────────────────────────────────────────────

async fn check_server_health(app: &mut AppState) {
    let url = format!("{}/health", app.server_url.replace("[::1]", "localhost"));
    match reqwest::get(&url).await {
        Ok(r) if r.status().is_success() => {
            app.connection_status = ConnectionStatus::Connected;
            app.add_system_message(&format!("connected → {}", app.server_url));
        },
        Ok(r) => {
            app.connection_status = ConnectionStatus::Degraded;
            app.add_system_message(&format!(
                "server {} returned {} (degraded)",
                app.server_url,
                r.status()
            ));
        },
        Err(_) => {
            app.connection_status = ConnectionStatus::Offline;
            app.add_system_message(&format!(
                "cannot reach {} — run `neoland server` to start",
                app.server_url
            ));
        },
    }
}
