use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::{
    app::{AppMode, AppState, ConnectionStatus, StageStatus, TaskStatus, ToolStatus},
    presets::QueryConfig,
};

pub mod colors {
    use ratatui::style::Color;

    pub const FG: Color = Color::Rgb(192, 202, 245);
    pub const FG_DIM: Color = Color::Rgb(169, 177, 214);
    pub const PRIMARY: Color = Color::Rgb(122, 162, 247);
    pub const CYAN: Color = Color::Rgb(125, 207, 255);
    pub const ACCENT: Color = Color::Rgb(187, 154, 247);
    pub const SUCCESS: Color = Color::Rgb(158, 206, 106);
    pub const WARNING: Color = Color::Rgb(255, 158, 100);
    pub const ERROR: Color = Color::Rgb(247, 118, 142);
    pub const MUTED: Color = Color::Rgb(54, 59, 84); // Cinza muito escuro para linhas guia
    pub const BORDER: Color = Color::Rgb(41, 46, 66);
    // Para o "Glassmorphism", vamos forçar um background escuro na janela flutuante
    pub const GLASS_BG: Color = Color::Rgb(22, 22, 30);
    pub const GLASS_BG_DIM: Color = Color::Rgb(16, 16, 20);
    pub const CURSOR_BG: Color = Color::Rgb(26, 27, 38);
}

// Spinners de altíssima fidelidade (Braille)
const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render(f: &mut Frame<'_>, app: &mut AppState) {
    if app.mode == AppMode::LlamaManager {
        render_llama_manager(f, app);
        return;
    }

    let screen_area = f.area();

    // O grande truque de "4K / Premium": Centering
    // Se o terminal for largo, nós desenhamos uma coluna central perfeita,
    // como o Raycast, Spotlight ou modo Zen do Obsidian.
    let max_width = 120;
    let ui_area = if screen_area.width > max_width {
        let h_pad = (screen_area.width - max_width) / 2;
        Rect::new(screen_area.x + h_pad, screen_area.y, max_width, screen_area.height)
    } else {
        screen_area
    };

    // Glassmorphism: Limpamos o fundo do terminal atrás do nosso painel e pintamos
    // um fundo escuro
    f.render_widget(Clear, ui_area);
    let bg_block = Block::default().style(Style::default().bg(colors::GLASS_BG));
    f.render_widget(bg_block, ui_area);

    let [header, canvas, _input_spacer, input] = Layout::vertical([
        Constraint::Length(2), // Header top
        Constraint::Min(0),    // Main Timeline
        Constraint::Length(1), // Spacer invisível
        Constraint::Length(3), // Floating Input
    ])
    .areas(ui_area);

    render_header(f, header, app);
    render_canvas(f, canvas, app);
    render_floating_input(f, input, app);
}

fn render_header(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let (dot, dot_color) = match app.connection_status {
        ConnectionStatus::Connected => ("󰤨", colors::SUCCESS),
        ConnectionStatus::Degraded => ("󰤯", colors::WARNING),
        ConnectionStatus::Offline => ("󰤭", colors::ERROR),
        ConnectionStatus::Unknown => ("󰤣", colors::MUTED),
    };

    let backend = app.active_backend.as_deref().unwrap_or("pipeline");
    let active_id = app
        .active_task_id
        .map(|id| format!(" 󰡱 {}", &id.to_string()[..6]))
        .unwrap_or_default();

    // Uma header finíssima
    let header_spans = vec![
        Span::styled(
            "  󰚌 neoland ",
            Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(colors::MUTED)),
        Span::styled(format!("{} ", dot), Style::default().fg(dot_color)),
        Span::styled(format!("{} ", backend), Style::default().fg(colors::FG_DIM)),
        Span::styled(" │ ", Style::default().fg(colors::MUTED)),
        Span::styled(
            format!("󰔎 {}ms", app.last_latency_ms),
            Style::default().fg(colors::FG_DIM).add_modifier(Modifier::ITALIC),
        ),
        Span::styled(active_id, Style::default().fg(colors::ACCENT)),
    ];

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(colors::MUTED))
        .style(Style::default().bg(colors::GLASS_BG));

    f.render_widget(Paragraph::new(Line::from(header_spans)).block(block), area);
}

fn render_canvas(f: &mut Frame<'_>, area: Rect, app: &mut AppState) {
    let mut lines = Vec::new();
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];

    // Top spacing
    lines.push(Line::from(""));

    if app.tasks.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                "󰛁 Aguardando instruções. Digite abaixo para iniciar...",
                Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
            ),
        ]));
    } else {
        for task in &app.tasks {
            let (icon, color) = match task.status {
                TaskStatus::Running => (spin, colors::ACCENT),
                TaskStatus::Done => ("󰄬", colors::SUCCESS),
                TaskStatus::Failed => ("󰅖", colors::ERROR),
                TaskStatus::Queued => ("󰔟", colors::MUTED),
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{} ", icon),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", &task.id.to_string()[..6]),
                    Style::default().fg(colors::MUTED),
                ),
                Span::styled(
                    &task.description,
                    Style::default()
                        .fg(if task.status == TaskStatus::Running {
                            colors::FG
                        } else {
                            colors::FG_DIM
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]));

            let is_active = app.active_task_id == Some(task.id);
            let is_last = Some(task.id) == app.tasks.last().map(|t| t.id);

            if is_active || is_last {
                // Árvore de Pipeline High-Fidelity
                for stage in &app.pipeline_stages {
                    let (s_icon, s_color) = match stage.status {
                        StageStatus::Running => (spin, colors::PRIMARY),
                        StageStatus::Done { .. } => ("󰄬", colors::MUTED),
                        StageStatus::Failed => ("󰅖", colors::ERROR),
                        StageStatus::Skipped => ("󰜎", colors::MUTED),
                        StageStatus::Pending => ("·", colors::MUTED),
                    };

                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled("│  ", Style::default().fg(colors::MUTED)),
                        Span::styled(format!("{} ", s_icon), Style::default().fg(s_color)),
                        Span::styled(
                            format!("{} ", stage.name),
                            Style::default().fg(colors::FG_DIM),
                        ),
                    ]));

                    if stage.status == StageStatus::Running {
                        for tool in &app.tool_calls {
                            let (t_icon, t_color) = match tool.status {
                                ToolStatus::Running => (spin, colors::CYAN),
                                ToolStatus::Done { .. } => ("󰄬", colors::MUTED),
                                ToolStatus::Failed => ("󰅖", colors::ERROR),
                            };
                            let args: String = tool.args_summary.chars().take(60).collect();

                            lines.push(Line::from(vec![
                                Span::raw("      "),
                                Span::styled("│    ", Style::default().fg(colors::MUTED)),
                                Span::styled("├─ ", Style::default().fg(colors::MUTED)),
                                Span::styled(format!("{} ", t_icon), Style::default().fg(t_color)),
                                Span::styled(
                                    format!("{} ", tool.name),
                                    Style::default().fg(colors::MUTED),
                                ),
                                Span::styled(
                                    format!("{} ", args),
                                    Style::default()
                                        .fg(colors::MUTED)
                                        .add_modifier(Modifier::ITALIC),
                                ),
                            ]));
                        }

                        if let Some(output) = &stage.output {
                            for text in output.lines().take(5) {
                                lines.push(Line::from(vec![
                                    Span::raw("      "),
                                    Span::styled("│    ", Style::default().fg(colors::MUTED)),
                                    Span::styled("│  ", Style::default().fg(colors::MUTED)),
                                    Span::styled(
                                        text.to_string(),
                                        Style::default().fg(colors::FG_DIM),
                                    ),
                                ]));
                            }
                        }
                    }
                }

                // Saída final
                if !app.output_text.is_empty() {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled("│", Style::default().fg(colors::MUTED)),
                    ]));
                    for line in app.output_text.lines() {
                        lines.push(Line::from(vec![
                            Span::raw("      "),
                            Span::styled("│  ", Style::default().fg(colors::MUTED)),
                            Span::styled(line.to_string(), Style::default().fg(colors::FG)),
                        ]));
                    }
                }

                // Decisão ADR
                if let (Some(status), Some(title)) = (&app.adr_status, &app.adr_title) {
                    let color = match status.as_str() {
                        "approve" | "accepted" => colors::SUCCESS,
                        "reject" | "rejected" => colors::ERROR,
                        "escalate" => colors::ACCENT,
                        _ => colors::WARNING,
                    };
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled("│", Style::default().fg(colors::MUTED)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled("╰─ ", Style::default().fg(colors::MUTED)),
                        Span::styled(
                            "󰡱 ADR ",
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(title.to_string(), Style::default().fg(colors::FG_DIM)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled("╰─", Style::default().fg(colors::MUTED)),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }
    }

    let inner_w = area.width.saturating_sub(4);
    let total = count_visual_lines(&lines, inner_w);

    if app.auto_scroll {
        app.scroll_offset = total.saturating_sub(area.height as usize) as u16;
    }
    app.scroll_offset = app.scroll_offset.min(total.saturating_sub(area.height as usize) as u16);

    let block = Block::default()
        .style(Style::default().bg(colors::GLASS_BG))
        .padding(ratatui::widgets::Padding::horizontal(2));

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.scroll_offset, 0)),
        area,
    );
}

fn render_floating_input(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let buf = &app.input_buffer;
    let cursor = app.cursor_pos;
    let busy = app.active_task_id.is_some();
    let preset = preset_name(&app.config);

    let mut spans = vec![
        Span::styled(
            format!(" 󰢱 {} ", preset),
            Style::default().fg(colors::MUTED).add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(colors::MUTED)),
        Span::styled(
            if busy { "󰑮 " } else { "  " },
            Style::default().fg(if busy {
                colors::ACCENT
            } else {
                colors::PRIMARY
            }),
        ),
    ];

    if buf.is_empty() {
        spans.push(Span::styled(
            if busy {
                "Aguardando intervenção (Steer)..."
            } else {
                "Digite sua instrução para o Neoland..."
            },
            Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
        ));
    } else {
        let before = buf[..cursor].to_owned();
        let at_char = buf[cursor..].chars().next();
        let after_start = cursor + at_char.map(|c| c.len_utf8()).unwrap_or(0);
        let after = buf[after_start..].to_owned();

        spans.push(Span::raw(before));
        match at_char {
            Some(c) => spans.push(Span::styled(
                c.to_string(),
                Style::default().bg(colors::FG).fg(colors::CURSOR_BG),
            )),
            None => spans.push(Span::styled(" ", Style::default().bg(colors::FG))),
        };
        spans.push(Span::raw(after));
    }

    // Input "Flutuante" no bottom com rounded borders de alta qualidade e margin
    // lateral
    let padded_area = Rect::new(area.x + 2, area.y, area.width.saturating_sub(4), area.height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if busy { colors::ACCENT } else { colors::MUTED }))
        .style(Style::default().bg(colors::GLASS_BG_DIM));

    f.render_widget(Paragraph::new(Line::from(spans)).block(block), padded_area);
}

fn render_llama_manager(f: &mut Frame<'_>, app: &mut AppState) {
    let area = f.area();
    let [browser, logs] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            " 󰚌 Llama Manager ",
            Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ r: run │ s: stop │ Esc: exit ", Style::default().fg(colors::MUTED)),
    ]));
    lines.push(Line::from(""));

    for (i, model) in app.llama_state.available_models.iter().enumerate() {
        let is_selected = i == app.llama_state.selected_index;
        let prefix = if is_selected { " 󰄾 " } else { "   " };
        let style = if is_selected {
            Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors::FG)
        };
        lines.push(Line::from(Span::styled(format!("{}{}", prefix, model), style)));
    }

    let status_str = if app.llama_state.is_running {
        "󰤨 ONLINE"
    } else {
        "󰤭 OFFLINE"
    };
    let status_color = if app.llama_state.is_running {
        colors::SUCCESS
    } else {
        colors::MUTED
    };

    let browser_block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(format!(" {} ", status_str), Style::default().fg(status_color)))
        .padding(ratatui::widgets::Padding::horizontal(2))
        .style(Style::default().bg(colors::GLASS_BG));

    f.render_widget(Paragraph::new(lines).block(browser_block), browser);

    let log_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(colors::MUTED))
        .title(Span::styled(" 󰈐 Logs ", Style::default().fg(colors::CYAN)))
        .padding(ratatui::widgets::Padding::horizontal(2))
        .style(Style::default().bg(colors::GLASS_BG));

    let mut log_lines = Vec::new();
    if let Ok(locked_logs) = app.llama_state.logs.try_lock() {
        for log in locked_logs.iter().rev().take(logs.height as usize) {
            log_lines.push(Line::from(Span::raw(log.clone())));
        }
    }
    log_lines.reverse();

    f.render_widget(Paragraph::new(log_lines).block(log_block), logs);
}

fn count_visual_lines(lines: &[Line<'_>], width: u16) -> usize {
    if width == 0 {
        return lines.len();
    }
    let w = width as usize;
    lines
        .iter()
        .map(|line| {
            let len: usize = line.spans.iter().map(|span| span.content.chars().count()).sum();
            if len == 0 {
                1
            } else {
                len.div_ceil(w)
            }
        })
        .sum()
}

fn preset_name(cfg: &QueryConfig) -> &'static str {
    if cfg.temperature == 0.7 && cfg.max_tokens == 600 {
        "balanced"
    } else if cfg.temperature == 1.5 {
        "creative"
    } else if cfg.temperature == 0.3 {
        "precise"
    } else if cfg.context_top_k == 8 {
        "research"
    } else if !cfg.enable_commands {
        "safe"
    } else {
        "custom"
    }
}
