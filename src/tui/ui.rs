use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::{AppState, ConnectionStatus, MessageRole};
use super::presets::QueryConfig;

// ── Palette — Tokyo Night Storm + Hyprland accent ────────────────────────────

pub mod colors {
    use ratatui::style::Color;
    pub const FG: Color = Color::Rgb(192, 202, 245); // #c0caf5  foreground
    pub const FG_DIM: Color = Color::Rgb(169, 177, 214); // #a9b1d6  dim foreground (AI text)
    pub const PRIMARY: Color = Color::Rgb(122, 162, 247); // #7aa2f7  blue  (active border)
    pub const CYAN: Color = Color::Rgb(125, 207, 255); // #7dcfff  user label
    pub const ACCENT: Color = Color::Rgb(187, 154, 247); // #bb9af7  purple (preset badge, spinner)
    pub const SUCCESS: Color = Color::Rgb(158, 206, 106); // #9ece6a  green  (ai label, connected)
    pub const WARNING: Color = Color::Rgb(255, 158, 100); // #ff9e64  orange (degraded)
    pub const ERROR: Color = Color::Rgb(247, 118, 142); // #f7768e  red    (offline)
    pub const MUTED: Color = Color::Rgb(86, 95, 137); // #565f89  muted text
    pub const BORDER: Color = Color::Rgb(65, 72, 104); // #414868  inactive border
    pub const CURSOR_BG: Color = Color::Rgb(26, 27, 38); // #1a1b26  cursor char bg (= true bg)
}

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// ── Top-level render ──────────────────────────────────────────────────────────

pub fn render(f: &mut Frame<'_>, app: &mut AppState) {
    let area = f.area();
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(area);

    let content_area = if app.sidebar_visible {
        let split = Layout::horizontal([Constraint::Percentage(72), Constraint::Percentage(28)])
            .split(chunks[0]);
        render_sidebar(f, split[1], app);
        split[0]
    } else {
        chunks[0]
    };

    render_chat(f, content_area, app);
    render_input(f, chunks[1], app);
}

// ── Chat panel ────────────────────────────────────────────────────────────────

fn render_chat(f: &mut Frame<'_>, area: ratatui::layout::Rect, app: &mut AppState) {
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);

    let lines = build_chat_lines(app);
    let total = count_visual_lines(&lines, inner_w);

    if app.auto_scroll {
        app.scroll_offset = total.saturating_sub(inner_h as usize) as u16;
    }
    app.scroll_offset = app.scroll_offset.min(total.saturating_sub(inner_h as usize) as u16);

    // Waybar-style status: ● · gpt-4o-mini · 1.2k tok · 42ms
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
    if let Some(ref b) = app.active_backend {
        status.push(Span::styled(" · ", Style::default().fg(colors::BORDER)));
        status.push(Span::styled(b.clone(), Style::default().fg(colors::ACCENT)));
    }
    if app.session_tokens > 0 {
        status.push(Span::styled(" · ", Style::default().fg(colors::BORDER)));
        status.push(Span::styled(
            format!("{}tok", app.session_tokens),
            Style::default().fg(colors::MUTED),
        ));
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

fn build_chat_lines(app: &AppState) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for msg in &app.messages {
        let time = msg.timestamp.format("%H:%M").to_string();
        match msg.role {
            MessageRole::User => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        "◆ you",
                        Style::default().fg(colors::CYAN).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("   {}", time), Style::default().fg(colors::MUTED)),
                ]));
                for text in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text),
                        Style::default().fg(colors::FG),
                    )));
                }
                lines.push(Line::from(""));
            },
            MessageRole::Assistant => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        "◆ ai",
                        Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("    {}", time), Style::default().fg(colors::MUTED)),
                ]));
                for text in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text),
                        Style::default().fg(colors::FG_DIM),
                    )));
                }
                lines.push(Line::from(""));
            },
            MessageRole::System => {
                for text in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text),
                        Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
                    )));
                }
                lines.push(Line::from(""));
            },
        }
    }

    // In-flight streaming / thinking
    if let Some(ref pending) = app.pending_message {
        let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("◆ ai", Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled(spin, Style::default().fg(colors::ACCENT)),
        ]));
        for text in pending.lines() {
            lines.push(Line::from(Span::styled(
                format!("    {}", text),
                Style::default().fg(colors::FG_DIM),
            )));
        }
        lines.push(Line::from(""));
    } else if app.is_thinking {
        let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("◆ ai", Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled(spin, Style::default().fg(colors::ACCENT)),
            Span::styled(
                " generating",
                Style::default().fg(colors::MUTED).add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    lines
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

fn render_sidebar(f: &mut Frame<'_>, area: ratatui::layout::Rect, app: &AppState) {
    let (conn_label, conn_color) = match app.connection_status {
        ConnectionStatus::Connected => ("connected", colors::SUCCESS),
        ConnectionStatus::Degraded => ("degraded", colors::WARNING),
        ConnectionStatus::Offline => ("offline", colors::ERROR),
        ConnectionStatus::Unknown => ("checking", colors::MUTED),
    };

    let preset = preset_name(&app.config);

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  session",
            Style::default().fg(colors::MUTED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  status   ", Style::default().fg(colors::MUTED)),
            Span::styled(conn_label.to_string(), Style::default().fg(conn_color)),
        ]),
    ];

    if let Some(ref b) = app.active_backend {
        lines.push(Line::from(vec![
            Span::styled("  backend  ", Style::default().fg(colors::MUTED)),
            Span::styled(b.clone(), Style::default().fg(colors::FG)),
        ]));
    }
    if app.session_tokens > 0 {
        lines.push(Line::from(vec![
            Span::styled("  tokens   ", Style::default().fg(colors::MUTED)),
            Span::styled(app.session_tokens.to_string(), Style::default().fg(colors::FG)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  latency  ", Style::default().fg(colors::MUTED)),
            Span::styled(format!("{}ms", app.last_latency_ms), Style::default().fg(colors::FG)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  preset",
        Style::default().fg(colors::MUTED).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  active   ", Style::default().fg(colors::MUTED)),
        Span::styled(
            preset.to_string(),
            Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  temp     ", Style::default().fg(colors::MUTED)),
        Span::styled(format!("{:.2}", app.config.temperature), Style::default().fg(colors::FG)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  top_p    ", Style::default().fg(colors::MUTED)),
        Span::styled(format!("{:.2}", app.config.top_p), Style::default().fg(colors::FG)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  max_tok  ", Style::default().fg(colors::MUTED)),
        Span::styled(app.config.max_tokens.to_string(), Style::default().fg(colors::FG)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ^1–5 preset  tab hide",
        Style::default().fg(colors::BORDER),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::BORDER))
        .title(
            Line::from(Span::styled(" info ", Style::default().fg(colors::MUTED))).left_aligned(),
        );

    f.render_widget(Paragraph::new(lines).block(block), area);
}

// ── Input bar ─────────────────────────────────────────────────────────────────

fn render_input(f: &mut Frame<'_>, area: ratatui::layout::Rect, app: &AppState) {
    let preset = preset_name(&app.config);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY))
        .title(
            Line::from(Span::styled(
                format!(" [ {} ] ", preset),
                Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD),
            ))
            .left_aligned(),
        )
        .title_bottom(
            Line::from(Span::styled(
                " ↵ send  ^l clear  ^c quit ",
                Style::default().fg(colors::MUTED),
            ))
            .right_aligned(),
        );

    f.render_widget(
        Paragraph::new(build_input_line(&app.input_buffer, app.cursor_pos)).block(block),
        area,
    );
}

fn build_input_line(buf: &str, cursor: usize) -> Line<'static> {
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
