use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::{AppState, ConnectionStatus, StageStatus, TaskStatus, ToolStatus};
use super::presets::QueryConfig;

// ── Palette — Tokyo Night Storm + Hyprland accent ─────────────────────────────

pub mod colors {
    use ratatui::style::Color;
    pub const FG: Color = Color::Rgb(192, 202, 245); // #c0caf5  foreground
    pub const FG_DIM: Color = Color::Rgb(169, 177, 214); // #a9b1d6  dim foreground
    pub const PRIMARY: Color = Color::Rgb(122, 162, 247); // #7aa2f7  blue  (active border)
    pub const CYAN: Color = Color::Rgb(125, 207, 255); // #7dcfff  tools label
    pub const ACCENT: Color = Color::Rgb(187, 154, 247); // #bb9af7  purple (pipeline)
    pub const SUCCESS: Color = Color::Rgb(158, 206, 106); // #9ece6a  green  (done, connected)
    pub const WARNING: Color = Color::Rgb(255, 158, 100); // #ff9e64  orange (deferred)
    pub const ERROR: Color = Color::Rgb(247, 118, 142); // #f7768e  red    (failed, offline)
    pub const MUTED: Color = Color::Rgb(86, 95, 137); // #565f89  muted text
    pub const BORDER: Color = Color::Rgb(65, 72, 104); // #414868  inactive border
    pub const CURSOR_BG: Color = Color::Rgb(26, 27, 38); // #1a1b26  cursor bg
}

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// ── Top-level render ──────────────────────────────────────────────────────────

pub fn render(f: &mut Frame<'_>, app: &mut AppState) {
    let area = f.area();
    let [main, input] = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area);

    render_main(f, main, app);
    render_input(f, input, app);
}

// ── Main panel ────────────────────────────────────────────────────────────────

fn render_main(f: &mut Frame<'_>, area: ratatui::layout::Rect, app: &mut AppState) {
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);

    let lines = build_main_lines(app);
    let total = count_visual_lines(&lines, inner_w);

    if app.auto_scroll {
        app.scroll_offset = total.saturating_sub(inner_h as usize) as u16;
    }
    app.scroll_offset = app.scroll_offset.min(total.saturating_sub(inner_h as usize) as u16);

    // Status bar — waybar pill style
    let (dot, dot_color) = match app.connection_status {
        ConnectionStatus::Connected => ("●", colors::SUCCESS),
        ConnectionStatus::Degraded => ("◐", colors::WARNING),
        ConnectionStatus::Offline => ("○", colors::ERROR),
        ConnectionStatus::Unknown => ("◌", colors::MUTED),
    };

    let mut status: Vec<Span> = vec![Span::styled(
        format!(" {} ", dot),
        Style::default().fg(dot_color).add_modifier(Modifier::BOLD),
    )];
    if app.last_latency_ms > 0 {
        status.push(Span::styled(" · ", Style::default().fg(colors::BORDER)));
        status.push(Span::styled(
            format!("{}ms ", app.last_latency_ms),
            Style::default().fg(colors::MUTED),
        ));
    }

    let scroll_hint = if app.auto_scroll {
        Line::from(Span::styled(" ↓ end ", Style::default().fg(colors::BORDER)))
    } else {
        Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(colors::PRIMARY)),
            Span::styled("scroll  g:end ", Style::default().fg(colors::MUTED)),
        ])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::BORDER))
        .title(
            Line::from(Span::styled(
                " ◆ neoland ",
                Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD),
            ))
            .left_aligned(),
        )
        .title(Line::from(status).right_aligned())
        .title_bottom(scroll_hint.right_aligned());

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.scroll_offset, 0)),
        area,
    );
}

fn build_main_lines(app: &AppState) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];

    // ── TASKS ─────────────────────────────────────────────────────────
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("TASKS", Style::default().fg(colors::MUTED).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    if app.tasks.is_empty() {
        lines.push(Line::from(Span::styled(
            "  no tasks yet — type a description and press Enter",
            Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
        )));
    } else {
        for task in &app.tasks {
            let (bullet, bullet_color) = match task.status {
                TaskStatus::Running => (spin, colors::ACCENT),
                TaskStatus::Done => ("✓", colors::SUCCESS),
                TaskStatus::Failed => ("✗", colors::ERROR),
                TaskStatus::Queued => ("○", colors::MUTED),
            };
            let status_label = match task.status {
                TaskStatus::Running => "running",
                TaskStatus::Done => "done   ",
                TaskStatus::Failed => "failed ",
                TaskStatus::Queued => "queued ",
            };
            let id_short = task.id.to_string()[..6].to_string();
            let desc = task.description.chars().take(45).collect::<String>();
            let padding = " ".repeat(47usize.saturating_sub(desc.chars().count()));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    bullet.to_string(),
                    Style::default().fg(bullet_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(format!("[{}]", id_short), Style::default().fg(colors::MUTED)),
                Span::raw(" "),
                Span::styled(desc, Style::default().fg(colors::FG)),
                Span::raw(padding),
                Span::styled(status_label.to_string(), Style::default().fg(bullet_color)),
            ]));
        }
    }

    // ── Divider ───────────────────────────────────────────────────────
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ─────────────────────────────────────────────────────────────────",
        Style::default().fg(colors::BORDER),
    )));
    lines.push(Line::from(""));

    // ── PIPELINE ──────────────────────────────────────────────────────
    if app.pipeline_visible || !app.pipeline_stages.is_empty() {
        let active_id = app
            .active_task_id
            .map(|id| format!("  [{}]", &id.to_string()[..6]))
            .unwrap_or_default();

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "▸ pipeline",
                Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(active_id, Style::default().fg(colors::MUTED)),
        ]));
        lines.push(Line::from(""));

        if app.pipeline_stages.is_empty() {
            lines.push(Line::from(Span::styled(
                "    no pipeline running",
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

                let conf_str = stage
                    .confidence
                    .map(|c| format!("{:.2}", c))
                    .unwrap_or_else(|| "    ".to_string());

                let latency_str = match &stage.status {
                    StageStatus::Done { latency_ms } => format!("{}ms", latency_ms),
                    StageStatus::Running => "running...".to_string(),
                    StageStatus::Skipped => "skipped".to_string(),
                    StageStatus::Failed => "failed".to_string(),
                    StageStatus::Pending => String::new(),
                };

                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        icon.to_string(),
                        Style::default().fg(icon_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(format!("{:<14}", stage.name), Style::default().fg(colors::FG)),
                    Span::styled(format!("{:<8}", conf_str), Style::default().fg(colors::MUTED)),
                    Span::styled(latency_str, Style::default().fg(colors::MUTED)),
                ]));

                // Transparent thinking — show first 2 lines of agent output
                if let Some(output) = &stage.output {
                    for text in output.lines().take(2) {
                        let truncated: String = text.chars().take(72).collect();
                        lines.push(Line::from(vec![
                            Span::raw("      "),
                            Span::styled("┊ ", Style::default().fg(colors::BORDER)),
                            Span::styled(
                                truncated,
                                Style::default().fg(colors::MUTED).add_modifier(Modifier::DIM),
                            ),
                        ]));
                    }
                }
            }
        }
        lines.push(Line::from(""));
    }

    // ── TOOLS ─────────────────────────────────────────────────────────
    if !app.tool_calls.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("▸ tools", Style::default().fg(colors::CYAN).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        for tool in &app.tool_calls {
            let (icon, icon_color): (&str, ratatui::style::Color) = match &tool.status {
                ToolStatus::Done { .. } => ("✓", colors::SUCCESS),
                ToolStatus::Running => (spin, colors::ACCENT),
                ToolStatus::Failed => ("✗", colors::ERROR),
            };
            let duration_str = match &tool.status {
                ToolStatus::Done { duration_ms } => format!("{}ms", duration_ms),
                ToolStatus::Running => "running...".to_string(),
                ToolStatus::Failed => "failed".to_string(),
            };
            let args = tool.args_summary.chars().take(22).collect::<String>();

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    icon.to_string(),
                    Style::default().fg(icon_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(format!("{:<18}", tool.name), Style::default().fg(colors::FG)),
                Span::styled(format!("{:<24}", args), Style::default().fg(colors::MUTED)),
                Span::styled(duration_str, Style::default().fg(colors::MUTED)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // ── OUTPUT ────────────────────────────────────────────────────────
    {
        let active_id = app
            .active_task_id
            .map(|id| format!("  [{}]", &id.to_string()[..6]))
            .unwrap_or_default();

        let adr_suffix = match (&app.adr_status, &app.adr_title) {
            (Some(status), Some(title)) => {
                let color = match status.as_str() {
                    "accepted" => colors::SUCCESS,
                    "rejected" => colors::ERROR,
                    _ => colors::WARNING,
                };
                Some((format!("  {} — {}", status, title), color))
            },
            _ => None,
        };

        let mut header = vec![
            Span::raw("  "),
            Span::styled(
                "▸ output",
                Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD),
            ),
            Span::styled(active_id, Style::default().fg(colors::MUTED)),
        ];
        if let Some((label, color)) = adr_suffix {
            header.push(Span::styled(label, Style::default().fg(color)));
        }
        lines.push(Line::from(header));
        lines.push(Line::from(""));

        if app.output_text.is_empty() && app.active_task_id.is_some() {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(spin.to_string(), Style::default().fg(colors::ACCENT)),
                Span::styled(
                    " pipeline running...",
                    Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
                ),
            ]));
        } else if app.output_text.is_empty() && app.tasks.is_empty() {
            lines.push(Line::from(Span::styled(
                "    submit a task above to see the pipeline output here",
                Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
            )));
        } else {
            for text_line in app.output_text.lines() {
                lines.push(Line::from(Span::styled(
                    format!("    {}", text_line),
                    Style::default().fg(colors::FG_DIM),
                )));
            }
        }
    }

    lines.push(Line::from(""));
    lines
}

// ── Input bar ─────────────────────────────────────────────────────────────────

fn render_input(f: &mut Frame<'_>, area: ratatui::layout::Rect, app: &AppState) {
    let preset = preset_name(&app.config);
    let busy = app.active_task_id.is_some();

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
        .title_bottom(
            Line::from(Span::styled(
                " ↵ task  ^p pipeline  ^m matrix  ^x cancel  ^c quit ",
                Style::default().fg(colors::MUTED),
            ))
            .right_aligned(),
        );

    f.render_widget(
        Paragraph::new(build_input_line(&app.input_buffer, app.cursor_pos, busy)).block(block),
        area,
    );
}

fn build_input_line(buf: &str, cursor: usize, busy: bool) -> Line<'static> {
    if busy {
        return Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "pipeline running — ^x to cancel",
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn count_visual_lines(lines: &[Line<'_>], width: u16) -> usize {
    if width == 0 {
        return lines.len();
    }
    let w = width as usize;
    lines
        .iter()
        .map(|l| {
            let len: usize = l.spans.iter().map(|s| s.content.chars().count()).sum();
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
