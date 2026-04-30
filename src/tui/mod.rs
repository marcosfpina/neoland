use std::{io, sync::Arc, time::Duration};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt as _;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

pub mod app;
pub mod events;
pub mod llama_manager;
pub mod llama_manager_logic;
pub mod presets;
pub mod ui;

use app::{AppState, ConnectionStatus, StageStatus};
use events::Action;
use ui::render;

// ── Agent stream events (TUI-internal) ───────────────────────────────

enum AgentStreamEvent {
    StageStarted { stage: String },
    StageDone { stage: String, confidence: Option<f32>, latency_ms: u64 },
    StageSkipped { stage: String },
    StageOutput { stage: String, content: String },
    ToolCallStarted { tool: String, args_summary: String },
    ToolCallDone { tool: String, duration_ms: u64 },
    ToolCallFailed { tool: String },
    BreakpointHit { tool: String, args_summary: String },
    AdrCheckpoint { status: String, title: String },
    PipelineDone { latency_ms: u64 },
    PipelineError { error: String },
    FinalResult { rationale: String, adr_title: String, adr_status: String },
    SteeringReceived { message: String },
}

// ── Legacy LLM channel events (kept for fallback) ─────────────────────

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

    let (key_tx, mut key_rx) = mpsc::channel::<Event>(64);
    let (llm_tx, mut llm_rx) = mpsc::channel::<LlmEvent>(16);
    let (agent_tx, mut agent_rx) = mpsc::channel::<AgentStreamEvent>(64);

    std::thread::spawn(move || {
        while let Ok(ev) = event::read() {
            if key_tx.blocking_send(ev).is_err() {
                break;
            }
        }
    });

    let api_key = std::env::var("NEOLAND_API_KEY").unwrap_or_default();
    let mut app = AppState::new(server_url.to_string(), ml_api_url.to_string());
    check_server_health(&mut app).await;

    let mut tick = tokio::time::interval(Duration::from_millis(80));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let models = llama_manager_logic::scan_models().await;
    app.llama_state.available_models = models;

    loop {
        terminal.draw(|f| render(f, &mut app))?;

        tokio::select! {
            biased;

            // ── Agent pipeline events ─────────────────────────────────
            Some(ev) = agent_rx.recv() => {
                match ev {
                    AgentStreamEvent::StageStarted { stage } => {
                        app.update_stage(&stage, StageStatus::Running, None);
                    }
                    AgentStreamEvent::StageDone { stage, confidence, latency_ms } => {
                        app.update_stage(&stage, StageStatus::Done { latency_ms }, confidence);
                    }
                    AgentStreamEvent::StageSkipped { stage } => {
                        app.update_stage(&stage, StageStatus::Skipped, None);
                    }
                    AgentStreamEvent::StageOutput { stage, content } => {
                        app.set_stage_output(&stage, content);
                    }
                    AgentStreamEvent::ToolCallStarted { tool, args_summary } => {
                        app.add_tool_call(tool, args_summary);
                    }
                    AgentStreamEvent::ToolCallDone { tool, duration_ms } => {
                        app.finish_tool_call(&tool, duration_ms);
                    }
                    AgentStreamEvent::BreakpointHit { tool, args_summary } => {
                        app.trigger_breakpoint(tool, args_summary);
                    }
                    AgentStreamEvent::ToolCallFailed { tool } => {
                        app.fail_tool_call(&tool);
                    }
                    AgentStreamEvent::AdrCheckpoint { status, title } => {
                        app.adr_status = Some(status);
                        app.adr_title = Some(title);
                    }
                    AgentStreamEvent::PipelineDone { latency_ms } => {
                        app.last_latency_ms = latency_ms;
                        if let Some(id) = app.active_task_id {
                            app.complete_task(id, true);
                        }
                        spawn_next_queued_task(&mut app, &agent_tx, &api_key);
                    }
                    AgentStreamEvent::PipelineError { error } => {
                        app.output_text = format!("error: {}", error);
                        if let Some(id) = app.active_task_id {
                            app.complete_task(id, false);
                        }
                        spawn_next_queued_task(&mut app, &agent_tx, &api_key);
                    }
                    AgentStreamEvent::FinalResult { rationale, adr_title, adr_status } => {
                        app.output_text = rationale;
                        if !adr_title.is_empty() {
                            app.adr_title = Some(adr_title);
                        }
                        if !adr_status.is_empty() {
                            app.adr_status = Some(adr_status);
                        }
                        app.auto_scroll = true;
                    }
                    AgentStreamEvent::SteeringReceived { message } => {
                        app.output_text = format!("{}\n\n[STEERING]: {}", app.output_text, message);
                    }
                }
            }

            // ── Legacy LLM events ─────────────────────────────────────
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

            // ── Keyboard ──────────────────────────────────────────────
            Some(key_ev) = key_rx.recv() => {
                match key_ev {
                    Event::Key(key)
                        if key.kind == KeyEventKind::Press
                            || key.kind == KeyEventKind::Repeat =>
                    {
                        match process_key(&mut app, key.code, key.modifiers) {
                            Action::Quit => break,

                            Action::SubmitTask(task) => {
                                let (task_id, session_id) = app.enqueue_task(task.clone());
                                app.start_task(task_id);
                                let tx = agent_tx.clone();
                                let srv = app.server_url.clone();
                                let key = api_key.clone();
                                tokio::spawn(async move {
                                    run_agent_task(task, session_id, srv, key, tx).await;
                                });
                            }

                            Action::QueueTask(task) => {
                                let (task_id, _) = app.enqueue_task(task);
                                let task_short = task_id.to_string()[..6].to_string();
                                app.output_text = if app.output_text.is_empty() {
                                    format!("[queue] task {} queued and waiting for dispatch.", task_short)
                                } else {
                                    format!("{}\n\n[queue] task {} queued and waiting for dispatch.", app.output_text, task_short)
                                };
                                app.auto_scroll = true;
                            }
                            Action::ResolveBreakpoint { resolution, instruction } => {
                                let srv = app.server_url.clone();
                                let session = app.active_session;
                                let key = api_key.clone();
                                app.pending_breakpoint = None; // Hide UI instantly
                                tokio::spawn(async move {
                                    if let Err(e) = post_breakpoint_resolve(&srv, &key, session, &resolution, instruction.as_deref()).await {
                                        eprintln!("Failed to resolve breakpoint: {}", e);
                                    }
                                });
                            }

                            Action::SteerTask(msg) => {
                                let srv = app.server_url.clone();
                                let key = api_key.clone();
                                let session = app.active_session;
                                tokio::spawn(async move {
                                    if let Err(e) = post_agent_steer(&srv, &key, session, &msg).await {
                                        eprintln!("Failed to steer task: {}", e);
                                    }
                                });
                            }

                            Action::Send(msg) => {
                                app.is_thinking = true;
                                app.auto_scroll = true;
                                let tx = llm_tx.clone();
                                let ml_url = app.ml_api_url.clone();
                                let srv_url = app.server_url.clone();
                                let cfg = app.config.clone();
                                tokio::spawn(run_llm(msg, cfg, ml_url, srv_url, tx));
                            }

                            Action::TogglePipeline => {
                                app.pipeline_visible = !app.pipeline_visible;
                            }

                            Action::FocusNextPanel => {
                                app.focus_next_panel();
                            }

                            Action::FocusPrevPanel => {
                                app.focus_prev_panel();
                            }

                            Action::OpenMatrix => {
                                let _ = std::process::Command::new("xdg-open")
                                    .arg("http://localhost:3000")
                                    .spawn();
                            }

                            Action::ToggleLlamaManager => {
                                app.mode = match app.mode {
                                    app::AppMode::Workstation => app::AppMode::LlamaManager,
                                    app::AppMode::LlamaManager => app::AppMode::Workstation,
                                };
                            }

                            Action::LlamaManagerRun => {
                                if app.mode == app::AppMode::LlamaManager && !app.llama_state.is_running {
                                    let index = app.llama_state.selected_index;
                                    if index < app.llama_state.available_models.len() {
                                        let model = app.llama_state.available_models[index].clone();
                                        app.llama_state.is_running = true;
                                        let flags = app.llama_state.flags.clone();
                                        let logs = app.llama_state.logs.clone();
                                        let child_process = app.llama_state.child_process.clone();
                                        tokio::spawn(async move {
                                            if let Err(e) = llama_manager_logic::run_llama_server(model, flags, logs, child_process).await {
                                                eprintln!("Failed to run llama server: {}", e);
                                            }
                                        });
                                    }
                                }
                            }

                            Action::LlamaManagerStop => {
                                if app.mode == app::AppMode::LlamaManager && app.llama_state.is_running {
                                    app.llama_state.is_running = false;
                                    let logs = app.llama_state.logs.clone();
                                    let child_process = app.llama_state.child_process.clone();
                                    tokio::spawn(async move {
                                        if let Err(e) = llama_manager_logic::stop_llama_server(child_process, logs).await {
                                            eprintln!("Failed to stop llama server: {}", e);
                                        }
                                    });
                                }
                            }

                            Action::CancelTask => {
                                if let Some(id) = app.active_task_id {
                                    app.complete_task(id, false);
                                    app.output_text = "task cancelled".to_string();
                                }
                            }

                            Action::None => {}
                        }
                    }
                    _ => {}
                }
            }

            _ = tick.tick() => {
                app.tick = app.tick.wrapping_add(1);
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}

// ── Key → Action ──────────────────────────────────────────────────────

fn process_key(app: &mut AppState, code: KeyCode, mods: KeyModifiers) -> Action {
    if app.mode == app::AppMode::LlamaManager {
        match (code, mods) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Action::Quit,
            (KeyCode::Char('l'), KeyModifiers::ALT) => return Action::ToggleLlamaManager,
            (KeyCode::Esc, _) => return Action::ToggleLlamaManager,
            (KeyCode::Char('r'), _) | (KeyCode::Enter, _) => return Action::LlamaManagerRun,
            (KeyCode::Char('s'), _) | (KeyCode::Char('c'), _) => return Action::LlamaManagerStop,
            (KeyCode::Up, _) => {
                if app.llama_state.selected_index > 0 {
                    app.llama_state.selected_index -= 1;
                }
                return Action::None;
            },
            (KeyCode::Down, _) => {
                if app.llama_state.selected_index + 1 < app.llama_state.available_models.len() {
                    app.llama_state.selected_index += 1;
                }
                return Action::None;
            },
            _ => return Action::None,
        }
    }

    match (code, mods) {
        // ── Quit ──────────────────────────────────────────────────────
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Action::Quit,

        // ── Multi-line input ──────────────────────────────────────────
        (KeyCode::Enter, KeyModifiers::SHIFT) => {
            app.insert_char('\n');
        },

        // ── Submit task ───────────────────────────────────────────────
        (KeyCode::Enter, _) if app.pending_breakpoint.is_some() => {
            let msg = std::mem::take(&mut app.input_buffer);
            app.cursor_pos = 0;
            if msg.trim().is_empty() {
                return Action::ResolveBreakpoint { resolution: "approve".to_string(), instruction: None };
            } else {
                app.history_commit(msg.clone());
                return Action::ResolveBreakpoint { resolution: "steer".to_string(), instruction: Some(msg) };
            }
        },
        (KeyCode::Esc, _) if app.pending_breakpoint.is_some() => {
            app.input_buffer.clear();
            app.cursor_pos = 0;
            return Action::ResolveBreakpoint { resolution: "reject".to_string(), instruction: None };
        },

        (KeyCode::Enter, KeyModifiers::CONTROL) if !app.input_buffer.is_empty() => {
            let msg = std::mem::take(&mut app.input_buffer);
            app.cursor_pos = 0;
            app.history_commit(msg.clone());
            return Action::QueueTask(msg);
        },
        (KeyCode::Enter, _) if !app.input_buffer.is_empty() => {
            let msg = std::mem::take(&mut app.input_buffer);
            app.cursor_pos = 0;
            app.history_commit(msg.clone());
            if app.active_task_id.is_none() {
                return Action::SubmitTask(msg);
            } else {
                return Action::SteerTask(msg);
            }
        },

        // ── Agent keybindings ─────────────────────────────────────────
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => return Action::TogglePipeline,
        (KeyCode::Char('m'), KeyModifiers::CONTROL) => return Action::OpenMatrix,
        (KeyCode::Char('x'), KeyModifiers::CONTROL) => return Action::CancelTask,
        (KeyCode::Char('l'), KeyModifiers::ALT) => return Action::ToggleLlamaManager,

        // ── Clear ─────────────────────────────────────────────────────
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

        // ── Focus / Esc ───────────────────────────────────────────────
        (KeyCode::Tab, KeyModifiers::SHIFT) | (KeyCode::BackTab, _) => {
            return Action::FocusPrevPanel;
        },
        (KeyCode::Tab, _) => return Action::FocusNextPanel,
        (KeyCode::Esc, _) if !app.input_buffer.is_empty() => {
            app.input_buffer.clear();
            app.cursor_pos = 0;
        },

        // ── Ctrl/Alt combos ───────────────────────────────────────────
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

        // ── History navigation (shell-like) ──────────────────────────
        (KeyCode::Up, KeyModifiers::NONE) => {
            app.history_prev();
        },
        (KeyCode::Down, KeyModifiers::NONE) => {
            app.history_next();
        },

        // ── Scroll (PageUp/PageDown + Shift+arrow) ────────────────────
        (KeyCode::Char('g'), KeyModifiers::NONE) if app.input_buffer.is_empty() => {
            app.auto_scroll = true;
        },
        (KeyCode::Up, KeyModifiers::SHIFT) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(3);
            app.auto_scroll = false;
        },
        (KeyCode::Down, KeyModifiers::SHIFT) => {
            app.scroll_offset = app.scroll_offset.saturating_add(3);
        },
        (KeyCode::PageUp, _) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
            app.auto_scroll = false;
        },
        (KeyCode::PageDown, _) => {
            app.scroll_offset = app.scroll_offset.saturating_add(10);
        },

        // ── Generic char input ────────────────────────────────────────
        (KeyCode::Char(c), m) if m == KeyModifiers::NONE || m == KeyModifiers::SHIFT => {
            app.insert_char(c)
        },
        (KeyCode::Backspace, KeyModifiers::NONE) => app.backspace(),
        (KeyCode::Delete, _) if app.cursor_pos < app.input_buffer.len() => {
            app.input_buffer.remove(app.cursor_pos);
        },
        (KeyCode::Left, KeyModifiers::NONE) => app.cursor_left(),
        (KeyCode::Right, KeyModifiers::NONE) => app.cursor_right(),
        (KeyCode::Home, _) => app.cursor_pos = 0,
        (KeyCode::End, _) => app.cursor_pos = app.input_buffer.len(),

        _ => {},
    }

    Action::None
}

// ── Agent task runner (spawned per submission) ────────────────────────

async fn run_agent_task(
    task: String,
    session_id: uuid::Uuid,
    server_url: String,
    api_key: String,
    tx: mpsc::Sender<AgentStreamEvent>,
) {
    // Subscribe to SSE in the background (events arrive before POST returns)
    let sse_url = format!("{}/v1/agents/events/{}", server_url, session_id);
    let sse_tx = tx.clone();
    let sse_key = api_key.clone();
    tokio::spawn(async move { subscribe_sse(sse_url, sse_key, sse_tx).await });

    // POST /v1/agents/task — blocks until pipeline completes
    let result = post_agent_task(&server_url, &api_key, &task, session_id).await;
    match result {
        Ok(data) => {
            let rationale = data
                .get("tech_leader")
                .and_then(|tl| tl.get("rationale"))
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string();
            let adr_title = data
                .get("tech_leader")
                .and_then(|tl| tl.get("adr_title"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let adr_status = data
                .get("tech_leader")
                .and_then(|tl| tl.get("decision"))
                .and_then(|d| d.as_str())
                .map(|s| format!("{:?}", s).to_lowercase())
                .unwrap_or_default();
            tx.send(AgentStreamEvent::FinalResult { rationale, adr_title, adr_status })
                .await
                .ok();
        },
        Err(e) => {
            tx.send(AgentStreamEvent::PipelineError { error: e.to_string() }).await.ok();
        },
    }
}

fn spawn_next_queued_task(
    app: &mut AppState,
    agent_tx: &mpsc::Sender<AgentStreamEvent>,
    api_key: &str,
) {
    if app.active_task_id.is_some() {
        return;
    }

    let Some((task_id, session_id, task)) = app.next_queued_task() else {
        return;
    };

    app.start_task(task_id);

    let tx = agent_tx.clone();
    let srv = app.server_url.clone();
    let key = api_key.to_string();

    tokio::spawn(async move {
        run_agent_task(task, session_id, srv, key, tx).await;
    });
}

async fn post_agent_steer(
    server_url: &str,
    api_key: &str,
    session_id: uuid::Uuid,
    message: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let url =
        format!("{}/v1/agents/session/{}/steer", server_url.trim_end_matches('/'), session_id);

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "message": message,
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Steer error: {}", err));
    }

    Ok(())
}

async fn post_agent_task(
    server_url: &str,
    api_key: &str,
    task: &str,
    session_id: uuid::Uuid,
) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/agents/task", server_url))
        .header("X-API-Key", api_key)
        .json(&serde_json::json!({"task": task, "session_id": session_id}))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("server returned {}", resp.status());
    }
    Ok(resp.json::<serde_json::Value>().await?)
}

async fn post_breakpoint_resolve(
    server_url: &str,
    api_key: &str,
    session_id: uuid::Uuid,
    resolution: &str,
    instruction: Option<&str>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/agents/session/{}/breakpoint/resolve", server_url, session_id))
        .header("X-API-Key", api_key)
        .json(&serde_json::json!({"resolution": resolution, "instruction": instruction}))
        .send()
        .await?;
    if !resp.status().is_success() { anyhow::bail!("server returned {}", resp.status()); }
    Ok(())
}

// ── SSE subscriber ────────────────────────────────────────────────────

async fn subscribe_sse(url: String, api_key: String, tx: mpsc::Sender<AgentStreamEvent>) {
    let client = reqwest::Client::new();
    let resp = match client.get(&url).header("X-API-Key", &api_key).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return,
    };

    let mut stream = resp.bytes_stream();
    let mut buf = String::new();

    while let Some(Ok(bytes)) = stream.next().await {
        buf.push_str(&String::from_utf8_lossy(&bytes));
        while let Some(pos) = buf.find("\n\n") {
            let event_text = buf[..pos].to_string();
            buf.drain(..pos + 2);
            for line in event_text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(evt) = parse_sse_event(&val) {
                            if tx.send(evt).await.is_err() {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn parse_sse_event(val: &serde_json::Value) -> Option<AgentStreamEvent> {
    match val["type"].as_str()? {
        "stage_started" => {
            Some(AgentStreamEvent::StageStarted { stage: val["stage"].as_str()?.to_string() })
        },
        "stage_done" => Some(AgentStreamEvent::StageDone {
            stage: val["stage"].as_str()?.to_string(),
            confidence: val["confidence"].as_f64().map(|f| f as f32),
            latency_ms: val["latency_ms"].as_u64().unwrap_or(0),
        }),
        "stage_skipped" => {
            Some(AgentStreamEvent::StageSkipped { stage: val["stage"].as_str()?.to_string() })
        },
        "stage_output" => Some(AgentStreamEvent::StageOutput {
            stage: val["stage"].as_str()?.to_string(),
            content: val["content"].as_str().unwrap_or("").to_string(),
        }),
        "tool_call_started" => Some(AgentStreamEvent::ToolCallStarted {
            tool: val["tool"].as_str()?.to_string(),
            args_summary: val["args_summary"].as_str().unwrap_or("").to_string(),
        }),
        "tool_call_done" => Some(AgentStreamEvent::ToolCallDone {
            tool: val["tool"].as_str()?.to_string(),
            duration_ms: val["duration_ms"].as_u64().unwrap_or(0),
        }),
        "tool_call_failed" => {
            Some(AgentStreamEvent::ToolCallFailed { tool: val["tool"].as_str()?.to_string() })
        },
        "breakpoint_hit" => Some(AgentStreamEvent::BreakpointHit {
            tool: val["tool"].as_str()?.to_string(),
            args_summary: val["args_summary"].as_str().unwrap_or("").to_string(),
        }),
        "adr_checkpoint" => Some(AgentStreamEvent::AdrCheckpoint {
            status: val["status"].as_str().unwrap_or("").to_string(),
            title: val["title"].as_str().unwrap_or("").to_string(),
        }),
        "pipeline_done" => Some(AgentStreamEvent::PipelineDone {
            latency_ms: val["latency_ms"].as_u64().unwrap_or(0),
        }),
        "pipeline_error" => Some(AgentStreamEvent::PipelineError {
            error: val["error"].as_str().unwrap_or("Unknown error").to_string(),
        }),
        "steering_received" => Some(AgentStreamEvent::SteeringReceived {
            message: val["message"].as_str().unwrap_or("").to_string(),
        }),
        _ => None,
    }
}

// ── Legacy LLM task (kept for direct ML API access) ──────────────────

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
        },
        Ok(r) => {
            app.connection_status = ConnectionStatus::Degraded;
            app.output_text =
                format!("server {} returned {} (degraded)", app.server_url, r.status());
        },
        Err(_) => {
            app.connection_status = ConnectionStatus::Offline;
            app.output_text =
                format!("cannot reach {} — run `neoland server` to start", app.server_url);
        },
    }
}
