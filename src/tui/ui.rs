use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::{AppState, AppMode, ConnectionStatus, Panel, StageStatus, TaskStatus, ToolStatus};
use super::presets::QueryConfig;

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
    pub const MUTED: Color = Color::Rgb(86, 95, 137);
    pub const BORDER: Color = Color::Rgb(65, 72, 104);
    pub const CURSOR_BG: Color = Color::Rgb(26, 27, 38);
}

const SPINNER: &[&str] = &["o", "O", "0", "O"];

pub fn render(f: &mut Frame<'_>, app: &mut AppState) {
    if app.mode == AppMode::LlamaManager {
        render_llama_manager(f, app);
        return;
    }

    let area = f.area();
    let [header, body, input] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .areas(area);

    render_header(f, header, app);
    render_workstation(f, body, app);
    render_input(f, input, app);
}

fn render_header(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let (dot, dot_color, status) = match app.connection_status {
        ConnectionStatus::Connected => ("●", colors::SUCCESS, "connected"),
        ConnectionStatus::Degraded => ("◐", colors::WARNING, "degraded"),
        ConnectionStatus::Offline => ("○", colors::ERROR, "offline"),
        ConnectionStatus::Unknown => ("◌", colors::MUTED, "unknown"),
    };

    let active_task = app
        .active_task_id
        .map(|id| id.to_string()[..6].to_string())
        .unwrap_or_else(|| "idle".to_string());
    let backend = app.active_backend.as_deref().unwrap_or("agent-pipeline");

    let header = Line::from(vec![
        Span::styled(" neoland ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("  ", Style::default()),
        Span::styled(dot, Style::default().fg(dot_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}  ", status), Style::default().fg(colors::FG_DIM)),
        Span::styled("backend ", Style::default().fg(colors::MUTED)),
        Span::styled(format!("{}  ", backend), Style::default().fg(colors::FG)),
        Span::styled("task ", Style::default().fg(colors::MUTED)),
        Span::styled(format!("{}  ", active_task), Style::default().fg(colors::ACCENT)),
        Span::styled("latency ", Style::default().fg(colors::MUTED)),
        Span::styled(format!("{}ms  ", app.last_latency_ms), Style::default().fg(colors::FG)),
        Span::styled("tokens ", Style::default().fg(colors::MUTED)),
        Span::styled(app.session_tokens.to_string(), Style::default().fg(colors::FG)),
    ]);

    f.render_widget(
        Paragraph::new(header).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors::BORDER)),
        ),
        area,
    );
}

fn render_workstation(f: &mut Frame<'_>, area: Rect, app: &mut AppState) {
    let [left, center, right] = Layout::horizontal([
        Constraint::Length(32),
        Constraint::Min(44),
        Constraint::Min(40),
    ])
    .areas(area);

    let [tasks_area, obs_area] =
        Layout::vertical([Constraint::Percentage(58), Constraint::Percentage(42)]).areas(left);
    let [pipeline_area, tools_area] =
        Layout::vertical([Constraint::Percentage(74), Constraint::Percentage(26)]).areas(center);

    render_tasks_panel(f, tasks_area, app);
    render_observability_panel(f, obs_area, app);
    render_pipeline_panel(f, pipeline_area, app);
    render_tools_panel(f, tools_area, app);
    render_output_panel(f, right, app);
}

fn render_llama_manager(f: &mut Frame<'_>, app: &mut AppState) {
    let area = f.area();
    let [browser, logs] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            " [Llama Manager] ",
            Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (Press 'r' to run, 's' to stop, 'Esc' to exit) ",
            Style::default().fg(colors::FG_DIM),
        ),
    ]));
    lines.push(Line::from(""));

    for (i, model) in app.llama_state.available_models.iter().enumerate() {
        let is_selected = i == app.llama_state.selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        let style = if is_selected {
            Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors::FG)
        };
        lines.push(Line::from(Span::styled(format!("{}{}", prefix, model), style)));
    }

    let status_str = if app.llama_state.is_running { "RUNNING" } else { "STOPPED" };
    let status_color = if app.llama_state.is_running {
        colors::SUCCESS
    } else {
        colors::ERROR
    };

    let browser_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::BORDER))
        .title(Span::styled(
            format!(" Models [{}] ", status_str),
            Style::default().fg(status_color),
        ));

    f.render_widget(Paragraph::new(lines).block(browser_block), browser);

    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::BORDER))
        .title(Span::styled(" Logs ", Style::default().fg(colors::CYAN)));

    let mut log_lines = Vec::new();
    if let Ok(locked_logs) = app.llama_state.logs.try_lock() {
        for log in locked_logs.iter().rev().take(logs.height as usize) {
            log_lines.push(Line::from(Span::raw(log.clone())));
        }
    }
    log_lines.reverse();

    f.render_widget(Paragraph::new(log_lines).block(log_block), logs);
}

fn render_tasks_panel(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
    let (queued, running, done, failed) = app.task_counts();
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("q ", Style::default().fg(colors::MUTED)),
        Span::styled(queued.to_string(), Style::default().fg(colors::FG).add_modifier(Modifier::BOLD)),
        Span::styled("  r ", Style::default().fg(colors::MUTED)),
        Span::styled(running.to_string(), Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("  ok ", Style::default().fg(colors::MUTED)),
        Span::styled(done.to_string(), Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled("  fail ", Style::default().fg(colors::MUTED)),
        Span::styled(failed.to_string(), Style::default().fg(colors::ERROR).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    if app.tasks.is_empty() {
        lines.push(Line::from(Span::styled(
            "No tasks yet.\nDescribe a task below and press Enter.",
            Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
        )));
    } else {
        for task in app.tasks.iter().rev().take(area.height.saturating_sub(4) as usize) {
            let (icon, color, status) = match task.status {
                TaskStatus::Queued => ("○", colors::MUTED, "queued"),
                TaskStatus::Running => (spin, colors::ACCENT, "running"),
                TaskStatus::Done => ("✓", colors::SUCCESS, "done"),
                TaskStatus::Failed => ("✗", colors::ERROR, "failed"),
            };
            let desc: String = task.description.chars().take(24).collect();
            let active = app.active_task_id == Some(task.id);

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", icon),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{}] ", &task.id.to_string()[..6]),
                    Style::default().fg(if active { colors::PRIMARY } else { colors::FG_DIM }),
                ),
                Span::styled(desc, Style::default().fg(colors::FG)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(status, Style::default().fg(color)),
                Span::styled("  s:", Style::default().fg(colors::BORDER)),
                Span::styled(
                    task.session_id.to_string()[..6].to_string(),
                    Style::default().fg(colors::FG_DIM),
                ),
            ]));
            lines.push(Line::from(""));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block(" Tasks ", app.focused_panel == Panel::Tasks, colors::PRIMARY))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_observability_panel(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let (dot, dot_color, status) = match app.connection_status {
        ConnectionStatus::Connected => ("●", colors::SUCCESS, "connected"),
        ConnectionStatus::Degraded => ("◐", colors::WARNING, "degraded"),
        ConnectionStatus::Offline => ("○", colors::ERROR, "offline"),
        ConnectionStatus::Unknown => ("◌", colors::MUTED, "unknown"),
    };

    let backend = app.active_backend.as_deref().unwrap_or("agent-pipeline");
    let active_task = app
        .active_task_id
        .map(|id| id.to_string()[..6].to_string())
        .unwrap_or_else(|| "idle".to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled(dot, Style::default().fg(dot_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {}", status), Style::default().fg(colors::FG)),
        ]),
        Line::from(vec![
            Span::styled("backend ", Style::default().fg(colors::MUTED)),
            Span::styled(backend, Style::default().fg(colors::FG)),
        ]),
        Line::from(vec![
            Span::styled("session ", Style::default().fg(colors::MUTED)),
            Span::styled(
                app.active_session.to_string()[..8].to_string(),
                Style::default().fg(colors::FG_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("active  ", Style::default().fg(colors::MUTED)),
            Span::styled(active_task, Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("latency ", Style::default().fg(colors::MUTED)),
            Span::styled(format!("{}ms", app.last_latency_ms), Style::default().fg(colors::FG)),
        ]),
        Line::from(vec![
            Span::styled("tokens  ", Style::default().fg(colors::MUTED)),
            Span::styled(app.session_tokens.to_string(), Style::default().fg(colors::FG)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Tab cycles panels. Enter steers active work. Ctrl+Enter queues follow-up work.",
            Style::default().fg(colors::FG_DIM).add_modifier(Modifier::DIM),
        )),
    ];

    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block(
                " Observability ",
                app.focused_panel == Panel::Observability,
                colors::CYAN,
            ))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_pipeline_panel(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
    let mut lines = Vec::new();

    if app.pipeline_stages.is_empty() {
        lines.push(Line::from(Span::styled(
            "No active pipeline.\nSubmit a task to populate stage progress here.",
            Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
        )));
    } else {
        for stage in &app.pipeline_stages {
            let (icon, icon_color): (&str, ratatui::style::Color) = match &stage.status {
                StageStatus::Done { .. } => ("✓", colors::SUCCESS),
                StageStatus::Running => (spin, colors::ACCENT),
                StageStatus::Skipped => ("○", colors::MUTED),
                StageStatus::Failed => ("✗", colors::ERROR),
                StageStatus::Pending => ("·", colors::BORDER),
            };

            let confidence = stage
                .confidence
                .map(|value| format!("{:.2}", value))
                .unwrap_or_else(|| "--".to_string());

            let timing = match &stage.status {
                StageStatus::Done { latency_ms } => format!("{}ms", latency_ms),
                StageStatus::Running => "running".to_string(),
                StageStatus::Skipped => "skipped".to_string(),
                StageStatus::Failed => "failed".to_string(),
                StageStatus::Pending => "pending".to_string(),
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", icon),
                    Style::default().fg(icon_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:<14}", stage.name), Style::default().fg(colors::FG)),
                Span::styled(format!("{:<6}", confidence), Style::default().fg(colors::FG_DIM)),
                Span::styled(timing, Style::default().fg(colors::MUTED)),
            ]));

            if let Some(output) = &stage.output {
                for text in output.lines().take(3) {
                    let truncated: String =
                        text.chars().take(area.width.saturating_sub(8) as usize).collect();
                    lines.push(Line::from(vec![
                        Span::styled("  | ", Style::default().fg(colors::BORDER)),
                        Span::styled(
                            truncated,
                            Style::default().fg(colors::MUTED).add_modifier(Modifier::DIM),
                        ),
                    ]));
                }
            }

            lines.push(Line::from(""));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block(
                " Pipeline Live View ",
                app.focused_panel == Panel::Pipeline,
                colors::ACCENT,
            ))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_tools_panel(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
    let mut lines = Vec::new();

    if app.tool_calls.is_empty() {
        lines.push(Line::from(Span::styled(
            "No tool calls yet.",
            Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
        )));
    } else {
        for tool in app
            .tool_calls
            .iter()
            .rev()
            .take(area.height.saturating_sub(2) as usize)
        {
            let (icon, icon_color): (&str, ratatui::style::Color) = match &tool.status {
                ToolStatus::Done { .. } => ("✓", colors::SUCCESS),
                ToolStatus::Running => (spin, colors::ACCENT),
                ToolStatus::Failed => ("✗", colors::ERROR),
            };
            let duration = match &tool.status {
                ToolStatus::Done { duration_ms } => format!("{}ms", duration_ms),
                ToolStatus::Running => "running".to_string(),
                ToolStatus::Failed => "failed".to_string(),
            };
            let args: String = tool.args_summary.chars().take(28).collect();

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", icon),
                    Style::default().fg(icon_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:<16}", tool.name), Style::default().fg(colors::FG)),
                Span::styled(duration, Style::default().fg(colors::FG_DIM)),
            ]));
            lines.push(Line::from(Span::styled(
                format!("  {}", args),
                Style::default().fg(colors::MUTED).add_modifier(Modifier::DIM),
            )));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block(" Tool Telemetry ", false, colors::CYAN))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_output_panel(f: &mut Frame<'_>, area: Rect, app: &mut AppState) {
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
    let mut lines = Vec::new();

    if let (Some(status), Some(title)) = (&app.adr_status, &app.adr_title) {
        let color = match status.as_str() {
            "approve" | "accepted" => colors::SUCCESS,
            "reject" | "rejected" => colors::ERROR,
            "escalate" => colors::ACCENT,
            _ => colors::WARNING,
        };
        lines.push(Line::from(vec![
            Span::styled("decision ", Style::default().fg(colors::MUTED)),
            Span::styled(status.clone(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(Span::styled(
            title.clone(),
            Style::default().fg(colors::FG).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
    }

    if app.output_text.is_empty() && app.active_task_id.is_some() {
        lines.push(Line::from(vec![
            Span::styled(spin, Style::default().fg(colors::ACCENT)),
            Span::styled(
                " waiting for final rationale...",
                Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
            ),
        ]));
    } else if app.output_text.is_empty() {
        lines.push(Line::from(Span::styled(
            "Output will show final rationale, ADR state, and steering feedback.",
            Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
        )));
    } else {
        for text_line in app.output_text.lines() {
            lines.push(Line::from(Span::styled(
                text_line.to_string(),
                Style::default().fg(colors::FG_DIM),
            )));
        }
    }

    let total = count_visual_lines(&lines, inner_w);
    if app.auto_scroll {
        app.scroll_offset = total.saturating_sub(inner_h as usize) as u16;
    }
    app.scroll_offset = app.scroll_offset.min(total.saturating_sub(inner_h as usize) as u16);

    let hint = if app.auto_scroll {
        " auto "
    } else {
        " shift+up/down scroll "
    };

    let block = panel_block(" Output ", app.focused_panel == Panel::Output, colors::PRIMARY)
        .title_bottom(Line::from(Span::styled(hint, Style::default().fg(colors::MUTED))).right_aligned());

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.scroll_offset, 0)),
        area,
    );
}

fn render_input(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let preset = preset_name(&app.config);
    let busy = app.active_task_id.is_some();

    let bottom_hint = if busy {
        " enter steer  ctrl+enter queue  shift+enter newline  tab focus "
    } else {
        " enter task  ctrl+enter queue  shift+enter newline  tab focus "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if busy {
            colors::ACCENT
        } else {
            colors::PRIMARY
        }))
        .title(
            Line::from(Span::styled(
                format!(" [ {} ] ", preset),
                Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD),
            ))
            .left_aligned(),
        )
        .title_bottom(Line::from(Span::styled(bottom_hint, Style::default().fg(colors::MUTED))).right_aligned());

    f.render_widget(Paragraph::new(build_input_line(app)).block(block), area);
}

fn build_input_line(app: &AppState) -> Line<'static> {
    let buf = &app.input_buffer;
    let cursor = app.cursor_pos;
    let busy = app.active_task_id.is_some();

    if buf.is_empty() && busy {
        return Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "steer the active run or press Ctrl+Enter to queue a new task",
                Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
            ),
        ]);
    }

    if buf.is_empty() {
        return Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "describe a task for the agent workstation",
                Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
            ),
        ]);
    }

    let before = buf[..cursor].to_owned();
    let at_char = buf[cursor..].chars().next();
    let after_start = cursor + at_char.map(|c| c.len_utf8()).unwrap_or(0);
    let after = buf[after_start..].to_owned();

    let cursor_span = match at_char {
        Some(c) => Span::styled(
            c.to_string(),
            Style::default()
                .bg(colors::PRIMARY)
                .fg(colors::CURSOR_BG)
                .add_modifier(Modifier::BOLD),
        ),
        None => Span::styled(" ", Style::default().bg(colors::PRIMARY).fg(colors::CURSOR_BG)),
    };

    Line::from(vec![Span::raw(" "), Span::raw(before), cursor_span, Span::raw(after)])
}

fn panel_block<'a>(title: &'a str, focused: bool, accent: ratatui::style::Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { accent } else { colors::BORDER }))
        .title(Span::styled(
            title,
            Style::default()
                .fg(if focused { accent } else { colors::FG_DIM })
                .add_modifier(Modifier::BOLD),
        ))
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
