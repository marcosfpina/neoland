use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use ratatui::style::Color;

use super::{
    app::{
        AppState, ConnectionStatus, LlmProvider, MessageRole, NotificationLevel, Panel,
        SessionStatus, StageStatus, StreamMode, Theme, ToolStatus,
    },
    commands::HELP_TEXT,
    presets::QueryConfig,
};

/// Runtime color palette. Swappable per [`Theme`]; all render code reads from a
/// `Palette` value (obtained via `app.theme.palette()`) instead of consts, so
/// `/theme` re-colors the whole UI live.
#[derive(Clone, Copy)]
pub struct Palette {
    pub fg: Color,
    pub fg_dim: Color,
    pub primary: Color,
    pub cyan: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
    pub border: Color,
    pub glass_bg: Color,
    pub glass_bg_dim: Color,
    pub cursor_bg: Color,
    // ── Conversation bubbles / code ──────────────────────────────────────
    pub user_bubble_bg: Color,
    pub user_bubble_fg: Color,
    pub agent_bubble_bg: Color,
    pub agent_bubble_fg: Color,
    pub bubble_border: Color,
    pub code_bg: Color,
}

impl Theme {
    pub fn palette(&self) -> Palette {
        match self {
            // Tokyo Night — the project's original palette.
            Theme::TokyoNight => Palette {
                fg: Color::Rgb(192, 202, 245),
                fg_dim: Color::Rgb(169, 177, 214),
                primary: Color::Rgb(122, 162, 247),
                cyan: Color::Rgb(125, 207, 255),
                accent: Color::Rgb(187, 154, 247),
                success: Color::Rgb(158, 206, 106),
                warning: Color::Rgb(255, 158, 100),
                error: Color::Rgb(247, 118, 142),
                muted: Color::Rgb(54, 59, 84),
                border: Color::Rgb(41, 46, 66),
                glass_bg: Color::Rgb(22, 22, 30),
                glass_bg_dim: Color::Rgb(16, 16, 20),
                cursor_bg: Color::Rgb(26, 27, 38),
                user_bubble_bg: Color::Rgb(40, 52, 87),
                user_bubble_fg: Color::Rgb(192, 202, 245),
                agent_bubble_bg: Color::Rgb(30, 33, 51),
                agent_bubble_fg: Color::Rgb(192, 202, 245),
                bubble_border: Color::Rgb(122, 162, 247),
                code_bg: Color::Rgb(16, 16, 20),
            },
            // Neon Glass — deep indigo glassmorphism with cyan/magenta glow (mockup).
            Theme::NeonGlass => Palette {
                fg: Color::Rgb(220, 225, 245),
                fg_dim: Color::Rgb(160, 165, 200),
                primary: Color::Rgb(139, 124, 247),
                cyan: Color::Rgb(90, 220, 255),
                accent: Color::Rgb(210, 130, 255),
                success: Color::Rgb(120, 230, 180),
                warning: Color::Rgb(255, 180, 120),
                error: Color::Rgb(255, 110, 150),
                muted: Color::Rgb(70, 60, 110),
                border: Color::Rgb(90, 70, 140),
                glass_bg: Color::Rgb(20, 16, 34),
                glass_bg_dim: Color::Rgb(14, 11, 26),
                cursor_bg: Color::Rgb(30, 24, 48),
                user_bubble_bg: Color::Rgb(44, 34, 74),
                user_bubble_fg: Color::Rgb(225, 220, 250),
                agent_bubble_bg: Color::Rgb(32, 26, 56),
                agent_bubble_fg: Color::Rgb(220, 225, 245),
                bubble_border: Color::Rgb(210, 130, 255),
                code_bg: Color::Rgb(14, 11, 26),
            },
            // High-contrast — strong, accessible, sharp user/agent separation.
            Theme::HighContrast => Palette {
                fg: Color::Rgb(255, 255, 255),
                fg_dim: Color::Rgb(200, 200, 200),
                primary: Color::Rgb(0, 200, 255),
                cyan: Color::Rgb(0, 255, 255),
                accent: Color::Rgb(255, 220, 0),
                success: Color::Rgb(0, 255, 120),
                warning: Color::Rgb(255, 170, 0),
                error: Color::Rgb(255, 60, 60),
                muted: Color::Rgb(120, 120, 120),
                border: Color::Rgb(200, 200, 200),
                glass_bg: Color::Rgb(0, 0, 0),
                glass_bg_dim: Color::Rgb(10, 10, 10),
                cursor_bg: Color::Rgb(40, 40, 40),
                user_bubble_bg: Color::Rgb(0, 80, 120),
                user_bubble_fg: Color::Rgb(255, 255, 255),
                agent_bubble_bg: Color::Rgb(60, 45, 0),
                agent_bubble_fg: Color::Rgb(255, 255, 255),
                bubble_border: Color::Rgb(255, 220, 0),
                code_bg: Color::Rgb(20, 20, 20),
            },
            // Monochrome — grayscale, content-focused, zero color distraction.
            Theme::Monochrome => Palette {
                fg: Color::Rgb(230, 230, 230),
                fg_dim: Color::Rgb(170, 170, 170),
                primary: Color::Rgb(200, 200, 200),
                cyan: Color::Rgb(210, 210, 210),
                accent: Color::Rgb(240, 240, 240),
                success: Color::Rgb(185, 185, 185),
                warning: Color::Rgb(205, 205, 205),
                error: Color::Rgb(150, 150, 150),
                muted: Color::Rgb(90, 90, 90),
                border: Color::Rgb(110, 110, 110),
                glass_bg: Color::Rgb(18, 18, 18),
                glass_bg_dim: Color::Rgb(12, 12, 12),
                cursor_bg: Color::Rgb(40, 40, 40),
                user_bubble_bg: Color::Rgb(50, 50, 50),
                user_bubble_fg: Color::Rgb(235, 235, 235),
                agent_bubble_bg: Color::Rgb(32, 32, 32),
                agent_bubble_fg: Color::Rgb(225, 225, 225),
                bubble_border: Color::Rgb(140, 140, 140),
                code_bg: Color::Rgb(24, 24, 24),
            },
        }
    }
}

// Spinners de altíssima fidelidade (Braille)
const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render(f: &mut Frame<'_>, app: &mut AppState) {
    let screen_area = f.area();
    let pal = app.theme.palette();

    // O grande truque de "4K / Premium": Centering
    let max_width = 120;
    let ui_area = if screen_area.width > max_width {
        let h_pad = (screen_area.width - max_width) / 2;
        Rect::new(screen_area.x + h_pad, screen_area.y, max_width, screen_area.height)
    } else {
        screen_area
    };

    // Glassmorphism
    f.render_widget(Clear, ui_area);
    let bg_block = Block::default().style(Style::default().bg(pal.glass_bg));
    f.render_widget(bg_block, ui_area);

    // ── Confirm quit overlay ───────────────────────────────────────────
    if app.confirm_quit {
        render_normal(f, ui_area, app);
        render_confirm_quit(f, screen_area, &pal);
        return;
    }

    // ── Help overlay ───────────────────────────────────────────────────
    if app.show_help {
        render_help_overlay(f, screen_area, &pal);
        return; // help covers full screen
    }

    render_normal(f, ui_area, app);
}

/// Normal TUI layout (no overlays).
fn render_normal(f: &mut Frame<'_>, ui_area: Rect, app: &mut AppState) {
    let notif_height: u16 = if app.notifications.is_empty() { 0 } else { 1 };
    let [header, notif, canvas, input] = Layout::vertical([
        Constraint::Length(2),            // Header
        Constraint::Length(notif_height), // Notification bar
        Constraint::Min(0),               // 3-column canvas
        Constraint::Length(4),            // Floating input (2 lines: text + hints)
    ])
    .areas(ui_area);

    render_header(f, header, app);
    if notif_height > 0 {
        render_notification_bar(f, notif, app);
    }
    render_canvas(f, canvas, app);
    render_floating_input(f, input, app);
}

/// Full-screen help overlay — triggered by `?` or `/help`.
fn render_help_overlay(f: &mut Frame<'_>, area: Rect, pal: &Palette) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(pal.cyan).bg(pal.glass_bg));

    let help = Paragraph::new(HELP_TEXT)
        .block(block)
        .style(Style::default().fg(pal.fg))
        .wrap(Wrap { trim: false });

    f.render_widget(Clear, area);
    f.render_widget(help, area);
}

/// Quit confirmation overlay — shown when `confirm_quit` is true.
fn render_confirm_quit(f: &mut Frame<'_>, area: Rect, pal: &Palette) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            " ⚠  Sair do Neoland? ",
            Style::default().fg(pal.warning).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "   Existem tarefas em andamento ou na fila.",
            Style::default().fg(pal.fg_dim),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "   ^c ou /exit novamente para confirmar",
            Style::default().fg(pal.muted),
        )),
        Line::from(Span::styled("   Esc para cancelar", Style::default().fg(pal.muted))),
        Line::from(""),
    ];

    let block = Block::default()
        .title(" Confirmação ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(pal.warning).bg(pal.glass_bg));

    let para = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(pal.fg))
        .alignment(ratatui::layout::Alignment::Center);

    // Centered box ~50% width, ~8 lines tall
    let w = area.width.min(60);
    let h = 11;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect::new(x, y, w, h);

    f.render_widget(Clear, popup);
    f.render_widget(para, popup);
}

fn render_header(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let pal = app.theme.palette();
    let (dot, dot_color) = match app.connection_status {
        ConnectionStatus::Connected => ("󰤨", pal.success),
        ConnectionStatus::Degraded => ("󰤯", pal.warning),
        ConnectionStatus::Offline => ("󰤭", pal.error),
        ConnectionStatus::Unknown => ("󰤣", pal.muted),
    };

    let provider_label = app.active_provider.label();
    let (provider_icon, provider_color) = match app.active_provider {
        LlmProvider::Local => ("󰍉", pal.success),
        LlmProvider::Deepseek => ("󱁆", pal.cyan),
        LlmProvider::Gemini => ("󰊭", pal.primary),
        LlmProvider::Groq => ("󰚌", pal.accent),
        LlmProvider::Llamacpp => ("󰏗", pal.warning),
    };

    let active_id = app
        .active_task_id
        .map(|id| format!(" 󰡱 {}", &id.to_string()[..6]))
        .unwrap_or_default();

    let header_spans = vec![
        // Moon logo + brand (mockup).
        Span::styled("  󰽥 Neoland ", Style::default().fg(pal.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(pal.muted)),
        Span::styled(format!("{} ", dot), Style::default().fg(dot_color)),
        Span::styled(" │ ", Style::default().fg(pal.muted)),
        Span::styled(
            format!("{} {} ", provider_icon, provider_label),
            Style::default().fg(provider_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(pal.muted)),
        Span::styled(
            format!("󰔎 {}ms", app.last_latency_ms),
            Style::default().fg(pal.fg_dim).add_modifier(Modifier::ITALIC),
        ),
        Span::styled(active_id, Style::default().fg(pal.accent)),
        Span::styled(
            format!("  {}", panel_indicator(app.focused_panel)),
            Style::default().fg(pal.muted),
        ),
    ];

    // Right-aligned action icons (user, settings) — mirrors the mockup.
    let right_spans = vec![
        Span::styled("󰀄 ", Style::default().fg(pal.cyan)),
        Span::styled(" 󰢻  ", Style::default().fg(pal.accent)),
    ];

    // Bottom border on the full header, then left/right paragraphs inside.
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(pal.muted))
        .style(Style::default().bg(pal.glass_bg));
    f.render_widget(block, area);

    let [left, right] = Layout::horizontal([Constraint::Min(0), Constraint::Length(8)]).areas(area);
    f.render_widget(Paragraph::new(Line::from(header_spans)), left);
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(ratatui::layout::Alignment::Right),
        right,
    );
}

/// Notification bar — renders the most recent notification as a colored line.
fn render_notification_bar(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let pal = app.theme.palette();
    if let Some(n) = app.notifications.first() {
        let (icon, color, label) = match n.level {
            NotificationLevel::Error => ("󰅖", pal.error, "Error"),
            NotificationLevel::Warning => ("󰀦", pal.warning, "Warning"),
            NotificationLevel::Info => ("󰋼", pal.primary, "Info"),
            NotificationLevel::Success => ("󰸞", pal.success, "Success"),
        };
        let msg: String = n.message.chars().take(area.width.saturating_sub(12) as usize).collect();
        let spans = vec![
            Span::raw("  "),
            Span::styled(
                format!("{} {} ", icon, label),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("│ ", Style::default().fg(pal.muted)),
            Span::styled(msg, Style::default().fg(pal.fg)),
        ];
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(pal.glass_bg));
        f.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
    }
}

/// Returns a panel label for the header bar indicator.
fn panel_indicator(panel: Panel) -> &'static str {
    match panel {
        Panel::Sessions => "󰙯 Sessions",
        Panel::Conversation => "󰭹 Conversation",
        Panel::Reasoning => "󰒝 Reasoning",
    }
}

/// Three-column layout (always visible): sessions | conversation | reasoning.
/// The focused column gets a highlighted border; the others are muted.
fn render_canvas(f: &mut Frame<'_>, area: Rect, app: &mut AppState) {
    let [left, center, right] = Layout::horizontal([
        Constraint::Length(28), // Sessions sidebar
        Constraint::Min(0),     // Conversation
        Constraint::Length(34), // Reasoning
    ])
    .areas(area);

    render_sessions_panel(f, left, app);
    render_conversation_panel(f, center, app);
    render_reasoning_panel(f, right, app);
}

/// Wrap content lines in a bordered block with a panel title.
/// Scroll offset is per-panel (`app.scroll_offsets[panel.index()]`); the border
/// is highlighted when `panel` is focused.
fn render_panel_content(
    f: &mut Frame<'_>,
    area: Rect,
    app: &mut AppState,
    panel: Panel,
    title: &str,
    attention: bool,
    mut lines: Vec<Line<'_>>,
) {
    let pal = app.theme.palette();
    let focused = app.focused_panel == panel;
    let idx = panel.index();
    let inner_w = area.width.saturating_sub(4);
    let view_h = area.height.saturating_sub(2) as usize; // subtract borders
    let total = count_visual_lines(&lines, inner_w);

    let max_off = total.saturating_sub(view_h) as u16;
    if app.auto_scroll {
        app.scroll_offsets[idx] = max_off;
    }
    app.scroll_offsets[idx] = app.scroll_offsets[idx].min(max_off);
    let offset = app.scroll_offsets[idx];

    // ── Scroll indicators ──────────────────────────────────────────────
    let scroll = offset as usize;
    let visible_end = scroll.saturating_add(view_h);
    let above = scroll;
    let below = total.saturating_sub(visible_end);
    if above > 0 || below > 0 {
        let mut parts: Vec<String> = Vec::new();
        if above > 0 {
            parts.push(format!("▲ {} more", above));
        }
        if below > 0 {
            parts.push(format!("▼ {} more", below));
        }
        let indicator = format!(" {} ", parts.join(" · "));
        lines.push(Line::from(vec![Span::styled(indicator, Style::default().fg(pal.muted))]));
    }

    // Attention pulse: alternate warning ↔ border every tick so the panel
    // "calls" for the user's eye even when not focused.
    let pulsing = attention && !focused && app.tick % 2 == 0;
    let border_color = if focused {
        pal.primary
    } else if pulsing {
        pal.warning
    } else if attention {
        pal.border
    } else {
        pal.muted
    };
    let title_color = if focused {
        pal.primary
    } else if attention {
        pal.warning
    } else {
        pal.fg_dim
    };
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(title_color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(pal.glass_bg));

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((offset, 0)),
        area,
    );
}

/// Sessions sidebar — "＋ New Chat" plus the list of conversations (active +
/// past), each with a status dot, name and relative time. The active session is
/// highlighted; navigate with ↑/↓ when this column is focused.
fn render_sessions_panel(f: &mut Frame<'_>, area: Rect, app: &mut AppState) {
    let pal = app.theme.palette();
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
    let now = chrono::Utc::now();
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // "New Chat" affordance.
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("✚ New Chat", Style::default().fg(pal.cyan).add_modifier(Modifier::BOLD)),
        Span::styled("  ^n", Style::default().fg(pal.muted)),
    ]));
    lines.push(Line::from(""));

    let name_w = (area.width as usize).saturating_sub(11).clamp(6, 16);
    for (i, s) in app.sessions.iter().enumerate() {
        let (dot, dot_color) = match s.last_status {
            SessionStatus::Active => (spin, pal.accent),
            SessionStatus::Done => ("●", pal.success),
            SessionStatus::Failed => ("●", pal.error),
            SessionStatus::Idle => ("○", pal.muted),
        };
        let active = i == app.active_session_idx;
        let name: String = {
            let n: String = s.name.chars().take(name_w).collect();
            if s.name.chars().count() > name_w {
                format!("{}…", n)
            } else {
                n
            }
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            if active {
                Span::styled("󰅂 ", Style::default().fg(pal.primary))
            } else {
                Span::raw("  ")
            },
            Span::styled(format!("{} ", dot), Style::default().fg(dot_color)),
            Span::styled(
                name,
                Style::default().fg(if active { pal.fg } else { pal.fg_dim }).add_modifier(
                    if active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("      "),
            Span::styled(
                super::sessions::relative_time(s.updated_at, now),
                Style::default().fg(pal.muted).add_modifier(Modifier::ITALIC),
            ),
        ]));
        lines.push(Line::from(""));
    }

    render_panel_content(f, area, app, Panel::Sessions, " 󰙯 Sessions ", false, lines);
}

/// Pipeline/reasoning panel. In normal mode shows the live pipeline tree with
/// confidence badges, elapsed timers and RWA anchors. When `app.why_mode` is
/// true, renders the full reasoning chain coloured by stage role (Esc to exit).
fn render_reasoning_panel(f: &mut Frame<'_>, area: Rect, app: &mut AppState) {
    let pal = app.theme.palette();
    let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
    let mut lines: Vec<Line<'_>> = Vec::new();
    lines.push(Line::from(""));

    // ── Compute attention (any done/failed stage with confidence < 50%) ──
    let low_confidence = app.pipeline_stages.iter().any(|s| {
        matches!(s.status, StageStatus::Done { .. } | StageStatus::Failed)
            && s.confidence.map(|c| c < 0.5).unwrap_or(false)
    });
    let title = if low_confidence && app.focused_panel != Panel::Reasoning {
        " 󰒝 Reasoning ⚠ "
    } else if app.why_mode {
        " 󰒝 Reasoning — /why "
    } else {
        " 󰒝 Reasoning "
    };

    // ── Why-mode: full reasoning chain coloured by stage ─────────────────
    if app.why_mode {
        lines.push(Line::from(vec![Span::styled(
            "  ── Reasoning Chain ─────────────────",
            Style::default().fg(pal.muted),
        )]));
        lines.push(Line::from(""));

        // Per-stage colours mirror pipeline roles
        let stage_color = |name: &str| match name {
            "junior" => pal.cyan,
            "senior" => pal.primary,
            "architect" => pal.accent,
            "tech-leader" => pal.success,
            _ => pal.fg_dim,
        };

        for stage in &app.pipeline_stages {
            let conf_str = stage
                .confidence
                .map(|c| format!(" ✓ {:.0}%", c * 100.0))
                .unwrap_or_default();
            let prov_str = stage
                .provenance
                .as_deref()
                .map(|p| format!("  ⛓ {}", p))
                .unwrap_or_default();

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("🧠 {}  ", stage.name),
                    Style::default().fg(stage_color(stage.name)).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}{}{}", stage.status.icon(), conf_str, prov_str),
                    Style::default().fg(pal.fg_dim),
                ),
            ]));

            if let Some(ref out) = stage.output {
                for text in out.lines().take(6) {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled("│  ", Style::default().fg(pal.muted)),
                        Span::styled(text.to_string(), Style::default().fg(pal.fg_dim)),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }

        if let (Some(status), Some(title_adr)) = (&app.adr_status, &app.adr_title) {
            let color = match status.as_str() {
                "approve" | "accepted" => pal.success,
                "reject" | "rejected" => pal.error,
                _ => pal.warning,
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("ADR {}  ", status),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(title_adr.to_string(), Style::default().fg(pal.fg_dim)),
            ]));
        }
        lines.push(Line::from(vec![Span::styled(
            "  ── Esc para fechar ─────────────────",
            Style::default().fg(pal.muted),
        )]));

        return render_panel_content(
            f, area, app, Panel::Reasoning, title, false, lines,
        );
    }

    // ── Normal mode: live pipeline tree ──────────────────────────────────
    let active_task = app.tasks.iter().find(|t| app.active_task_id == Some(t.id));

    if let Some(task) = active_task {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                task.description.clone(),
                Style::default().fg(pal.fg).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Iterate with index so we can compute confidence delta from prev stage
        let stages: Vec<_> = app.pipeline_stages.iter().collect();
        for (i, stage) in stages.iter().enumerate() {
            let (s_icon, s_color) = match stage.status {
                StageStatus::Running => (spin, pal.primary),
                StageStatus::Done { .. } => ("󰄬", pal.muted),
                StageStatus::Failed => ("󰅖", pal.error),
                StageStatus::Skipped => ("󰜎", pal.muted),
                StageStatus::Pending => ("·", pal.muted),
            };

            let confidence_badge = stage.confidence.map(|c| confidence_span(c, &pal));

            // Confidence delta relative to the previous done/skipped stage
            let delta_span: Option<Span> = if let Some(cur_c) = stage.confidence {
                // Find last preceding stage that has confidence
                let prev_c = stages[..i]
                    .iter()
                    .rev()
                    .find_map(|s| s.confidence);
                prev_c.map(|p| {
                    let d = (cur_c - p) * 100.0;
                    let (arrow, color) = if d >= 0.0 {
                        (format!("▲{:.0} ", d), pal.success)
                    } else {
                        (format!("▼{:.0} ", d.abs()), pal.error)
                    };
                    Span::styled(arrow, Style::default().fg(color))
                })
            } else {
                None
            };

            let mut spans: Vec<Span> = vec![
                Span::raw("    "),
                Span::styled("├─ ", Style::default().fg(pal.muted)),
                Span::styled(format!("{} ", s_icon), Style::default().fg(s_color)),
                Span::styled(
                    format!("{} ", stage.name),
                    Style::default()
                        .fg(if stage.status == StageStatus::Running { pal.fg } else { pal.fg_dim }),
                ),
            ];
            if let Some(badge) = confidence_badge {
                spans.push(badge);
            }
            if let Some(delta) = delta_span {
                spans.push(delta);
            }
            // Elapsed timer on running stage
            if matches!(stage.status, StageStatus::Running) {
                if let Some(started) = stage.started_at {
                    let secs = started.elapsed().as_secs_f32();
                    spans.push(Span::styled(
                        format!("{:.1}s ", secs),
                        Style::default().fg(pal.muted),
                    ));
                }
            }
            // RWA provenance anchor
            if let Some(ref anchor) = stage.provenance {
                spans.push(Span::styled(
                    format!("⛓ {} ", anchor),
                    Style::default().fg(pal.cyan).add_modifier(Modifier::BOLD),
                ));
            }
            lines.push(Line::from(spans));

            // Tool calls + condensed output for the running stage
            if stage.status == StageStatus::Running {
                for tool in &app.tool_calls {
                    let (t_icon, t_color) = match tool.status {
                        ToolStatus::Running => (spin, pal.cyan),
                        ToolStatus::Done { .. } => ("󰄬", pal.muted),
                        ToolStatus::Failed => ("󰅖", pal.error),
                    };
                    let args: String = tool.args_summary.chars().take(60).collect();
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled("│  ", Style::default().fg(pal.muted)),
                        Span::styled("├─ ", Style::default().fg(pal.muted)),
                        Span::styled(format!("{} ", t_icon), Style::default().fg(t_color)),
                        Span::styled(format!("{} ", tool.name), Style::default().fg(pal.muted)),
                        Span::styled(
                            args,
                            Style::default().fg(pal.muted).add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
                if let Some(output) = &stage.output {
                    for text in output.lines().take(2) {
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled("│  ", Style::default().fg(pal.muted)),
                            Span::styled("│ ", Style::default().fg(pal.muted)),
                            Span::styled(text.to_string(), Style::default().fg(pal.fg_dim)),
                        ]));
                    }
                }
            }
        }

        // ADR footer
        if let (Some(status), Some(adr_title)) = (&app.adr_status, &app.adr_title) {
            let color = match status.as_str() {
                "approve" | "accepted" => pal.success,
                "reject" | "rejected" => pal.error,
                "escalate" => pal.accent,
                _ => pal.warning,
            };
            let anchored = app.pipeline_stages.iter().any(|s| s.provenance.is_some());
            lines.push(Line::from(""));
            let mut adr_spans = vec![
                Span::raw("    "),
                Span::styled("╰─ ", Style::default().fg(pal.muted)),
                Span::styled(
                    format!("󰡱 ADR {} ", status),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ];
            if anchored {
                adr_spans.push(Span::styled(
                    "✓ verified ",
                    Style::default().fg(pal.cyan).add_modifier(Modifier::BOLD),
                ));
            }
            lines.push(Line::from(adr_spans));
            lines.push(Line::from(vec![
                Span::raw("       "),
                Span::styled(adr_title.to_string(), Style::default().fg(pal.fg_dim)),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "󰬺 Sem tarefa ativa.",
                Style::default().fg(pal.muted).add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    render_panel_content(f, area, app, Panel::Reasoning, title, low_confidence, lines);
}

/// Conversation panel — renders the message history as themed chat bubbles.
/// User messages hug the right; agent messages the left (with an avatar);
/// system messages are muted center lines. Fenced ``` blocks get a code bg.
fn render_conversation_panel(f: &mut Frame<'_>, area: Rect, app: &mut AppState) {
    let pal = app.theme.palette();
    let inner_w = (area.width as usize).saturating_sub(4).max(16);
    let mut lines: Vec<Line<'static>> = Vec::new();

    if app.messages.is_empty() {
        render_welcome(&mut lines, app, &pal);
    } else {
        lines.push(Line::from(""));
        let term_lower = app.search_term.as_ref().map(|t| t.to_lowercase());
        for msg in &app.messages {
            let highlight = term_lower
                .as_ref()
                .map(|t| msg.content.to_lowercase().contains(t))
                .unwrap_or(false);
            push_bubble(&mut lines, &msg.role, &msg.content, inner_w, &pal, highlight);
        }

        // In-progress reveal (typewriter/line) or a "thinking" bubble while the
        // pipeline runs in ThinkingReveal mode.
        if app.streaming {
            let mut text = app.streamed_text();
            text.push('▋'); // cursor
            push_bubble(&mut lines, &MessageRole::Assistant, &text, inner_w, &pal, false);
        } else if app.awaiting_response() && app.stream_mode == StreamMode::ThinkingReveal {
            let spin = SPINNER[(app.tick as usize) % SPINNER.len()];
            push_bubble(
                &mut lines,
                &MessageRole::Assistant,
                &format!("{} pensando…", spin),
                inner_w,
                &pal,
                false,
            );
        }
    }

    render_panel_content(f, area, app, Panel::Conversation, " 󰭹 Conversation ", false, lines);
}

/// Branded welcome card shown when the conversation is empty.
/// Centered logo + tagline + a few onboarding hints — the "front door" of the TUI.
fn render_welcome(lines: &mut Vec<Line<'static>>, app: &AppState, pal: &Palette) {
    let center = ratatui::layout::Alignment::Center;
    let push_c = |lines: &mut Vec<Line<'static>>, spans: Vec<Span<'static>>| {
        lines.push(Line::from(spans).alignment(center));
    };

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    // Moon logo.
    push_c(
        lines,
        vec![Span::styled("󰽥", Style::default().fg(pal.accent).add_modifier(Modifier::BOLD))],
    );
    lines.push(Line::from(""));
    // Brand + tagline.
    push_c(
        lines,
        vec![
            Span::styled("Welcome to ", Style::default().fg(pal.fg_dim)),
            Span::styled("Neoland", Style::default().fg(pal.cyan).add_modifier(Modifier::BOLD)),
        ],
    );
    push_c(
        lines,
        vec![Span::styled(
            "Performance e máximo alinhamento no trabalho.",
            Style::default().fg(pal.fg_dim).add_modifier(Modifier::ITALIC),
        )],
    );
    lines.push(Line::from(""));
    push_c(
        lines,
        vec![Span::styled("How can I assist you today?", Style::default().fg(pal.fg))],
    );
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Onboarding hints.
    let hint = |label: &'static str, desc: &'static str| -> Vec<Span<'static>> {
        vec![
            Span::styled(format!("{:>12}", label), Style::default().fg(pal.primary)),
            Span::styled("  ", Style::default()),
            Span::styled(desc, Style::default().fg(pal.fg_dim)),
        ]
    };
    push_c(lines, hint("Enter", "enviar uma instrução para o agente"));
    push_c(lines, hint("Tab", "alternar foco entre as colunas"));
    push_c(lines, hint("/help", "todos os comandos e atalhos"));
    push_c(lines, hint("/theme", "trocar o tema de cores"));
    lines.push(Line::from(""));

    // Current context footer (theme + provider).
    push_c(
        lines,
        vec![
            Span::styled("tema ", Style::default().fg(pal.muted)),
            Span::styled(app.theme.label(), Style::default().fg(pal.accent)),
            Span::styled("  ·  ", Style::default().fg(pal.muted)),
            Span::styled("provider ", Style::default().fg(pal.muted)),
            Span::styled(app.active_provider.label(), Style::default().fg(pal.success)),
        ],
    );
}

/// Append a chat bubble (a box of styled lines) to `lines`.
///
/// - `User` → right-aligned, `user_bubble_*` colors.
/// - `Assistant` → left-aligned with a `N` avatar label, `agent_bubble_*`.
/// - `System` → a centered muted line (no box).
///
/// Fenced ``` code blocks render with `code_bg`. When `highlight` is set
/// (a `/search` hit) the border is drawn in `warning`.
fn push_bubble(
    lines: &mut Vec<Line<'static>>,
    role: &MessageRole,
    content: &str,
    inner_w: usize,
    pal: &Palette,
    highlight: bool,
) {
    // System messages: simple centered muted line, no bubble.
    if matches!(role, MessageRole::System) {
        lines.push(
            Line::from(Span::styled(
                format!("─ {} ─", content),
                Style::default().fg(pal.muted).add_modifier(Modifier::ITALIC),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        );
        lines.push(Line::from(""));
        return;
    }

    let is_user = matches!(role, MessageRole::User);
    let (bub_bg, bub_fg) = if is_user {
        (pal.user_bubble_bg, pal.user_bubble_fg)
    } else {
        (pal.agent_bubble_bg, pal.agent_bubble_fg)
    };
    let border_color = if highlight {
        pal.warning
    } else {
        pal.bubble_border
    };

    let indent = 2usize;
    let max_box = inner_w.saturating_sub(indent + 1).max(14);
    let text_w = max_box.saturating_sub(4).max(8); // minus 2 borders + 2 padding

    // Wrap content, tracking ``` fences (code is truncated, not word-wrapped).
    let mut wrapped: Vec<(String, bool)> = Vec::new();
    let mut in_code = false;
    for raw in content.lines() {
        if raw.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if raw.is_empty() {
            wrapped.push((String::new(), in_code));
            continue;
        }
        if in_code {
            wrapped.push((raw.chars().take(text_w).collect(), true));
        } else {
            let mut cur = String::new();
            for word in raw.split(' ') {
                if !cur.is_empty() && cur.chars().count() + word.chars().count() + 1 > text_w {
                    wrapped.push((std::mem::take(&mut cur), false));
                }
                if !cur.is_empty() {
                    cur.push(' ');
                }
                cur.push_str(word);
            }
            wrapped.push((cur, false));
        }
    }
    if wrapped.is_empty() {
        wrapped.push((String::new(), false));
    }

    let bw = wrapped.iter().map(|(t, _)| t.chars().count()).max().unwrap_or(1).max(1);
    let box_w = bw + 4; // ╭ + space + text + space + ╮
    let pad_left = if is_user {
        inner_w.saturating_sub(box_w + 1)
    } else {
        indent
    };
    let lead = " ".repeat(pad_left);

    // Role label above the bubble.
    let label = if is_user {
        Line::from(Span::styled("you 󰀄", Style::default().fg(pal.fg_dim)))
            .alignment(ratatui::layout::Alignment::Right)
    } else {
        Line::from(vec![
            Span::raw(lead.clone()),
            Span::styled("󰚩 Neoland", Style::default().fg(pal.accent).add_modifier(Modifier::BOLD)),
        ])
    };
    lines.push(label);

    // Top border.
    lines.push(Line::from(vec![
        Span::raw(lead.clone()),
        Span::styled(format!("╭{}╮", "─".repeat(bw + 2)), Style::default().fg(border_color)),
    ]));
    // Content lines.
    for (text, is_code) in &wrapped {
        let cell_bg = if *is_code { pal.code_bg } else { bub_bg };
        let cell_fg = if *is_code { pal.cyan } else { bub_fg };
        let padded = format!(" {}{} ", text, " ".repeat(bw - text.chars().count()));
        lines.push(Line::from(vec![
            Span::raw(lead.clone()),
            Span::styled("│", Style::default().fg(border_color)),
            Span::styled(padded, Style::default().fg(cell_fg).bg(cell_bg)),
            Span::styled("│", Style::default().fg(border_color)),
        ]));
    }
    // Bottom border.
    lines.push(Line::from(vec![
        Span::raw(lead),
        Span::styled(format!("╰{}╯", "─".repeat(bw + 2)), Style::default().fg(border_color)),
    ]));
    lines.push(Line::from(""));
}

fn render_floating_input(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let pal = app.theme.palette();
    let buf = &app.input_buffer;
    let cursor = app.cursor_pos;
    let busy = app.active_task_id.is_some();
    let preset = preset_name(&app.config);
    let provider = app.active_provider.label();

    let (p_icon, p_color) = match app.active_provider {
        LlmProvider::Local => ("L", pal.success),
        LlmProvider::Deepseek => ("D", pal.cyan),
        LlmProvider::Gemini => ("G", pal.primary),
        LlmProvider::Groq => ("R", pal.accent),
        LlmProvider::Llamacpp => ("C", pal.warning),
    };

    let mut spans = vec![
        Span::styled(
            format!(" {} {} ", p_icon, provider),
            Style::default().fg(p_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(pal.muted)),
        Span::styled(format!(" 󰢱 {} ", preset), Style::default().fg(pal.muted)),
        Span::styled("│ ", Style::default().fg(pal.muted)),
        Span::styled(
            if busy { "󰑮 " } else { "  " },
            Style::default().fg(if busy { pal.accent } else { pal.primary }),
        ),
    ];

    if buf.is_empty() {
        spans.push(Span::styled(
            if busy {
                "Aguardando intervenção (Steer)..."
            } else {
                "Digite sua instrução para o Neoland..."
            },
            Style::default().fg(pal.muted).add_modifier(Modifier::ITALIC),
        ));
    } else {
        let before = buf[..cursor].to_owned();
        let at_char = buf[cursor..].chars().next();
        let after_start = cursor + at_char.map(|c| c.len_utf8()).unwrap_or(0);
        let after = buf[after_start..].to_owned();

        spans.push(Span::raw(before));
        match at_char {
            Some(c) => spans
                .push(Span::styled(c.to_string(), Style::default().bg(pal.fg).fg(pal.cursor_bg))),
            None => spans.push(Span::styled(" ", Style::default().bg(pal.fg))),
        };
        spans.push(Span::raw(after));
    }

    // ── Context-sensitive hint line (coloured: keys accent, text muted) ───
    let hint_spans: Vec<Span> = {
        // Each hint entry is (key_label, description) pairs joined by "·"
        let entries: &[(&str, &str)] = if app.search_term.is_some() {
            &[("n", "próximo"), ("N", "anterior"), ("Esc", "limpar")]
        } else if app.why_mode {
            &[("Esc", "fechar why")]
        } else if app.pending_breakpoint.is_some() {
            &[("Y", "aprovar"), ("N", "rejeitar"), ("/steer", "instruções")]
        } else if busy {
            &[("Enter", "steer"), ("Ctrl+X", "cancelar"), ("/why", "raciocínio")]
        } else if buf.starts_with('/') {
            &[("Tab", "completar"), ("/help", "ver tudo")]
        } else {
            &[("Enter", "enviar"), ("Tab", "painéis"), ("?", "help")]
        };
        let mut s: Vec<Span> = vec![Span::raw("  ")];
        for (i, (key, desc)) in entries.iter().enumerate() {
            if i > 0 {
                s.push(Span::styled("  ·  ", Style::default().fg(pal.muted)));
            }
            s.push(Span::styled(
                key.to_string(),
                Style::default().fg(pal.accent).add_modifier(Modifier::BOLD),
            ));
            s.push(Span::styled(
                format!(" {}", desc),
                Style::default().fg(pal.muted),
            ));
        }
        s
    };

    let padded_area = Rect::new(area.x + 2, area.y, area.width.saturating_sub(4), area.height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if app.pending_breakpoint.is_some() {
            pal.warning
        } else if busy {
            pal.accent
        } else {
            pal.muted
        }))
        .style(Style::default().bg(pal.glass_bg_dim));

    f.render_widget(
        Paragraph::new(vec![Line::from(spans), Line::from(hint_spans)]).block(block),
        padded_area,
    );
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
        "balanced"
    }
}

/// Create a confidence badge span with color-coding:
/// - Green  (SUCCESS) if >= 0.70
/// - Yellow (WARNING) if >= 0.50
/// - Red    (ERROR)   if < 0.50
fn confidence_span(confidence: f32, pal: &Palette) -> Span<'static> {
    let pct = (confidence * 100.0).round() as u8;
    let color = if confidence >= 0.70 {
        pal.success
    } else if confidence >= 0.50 {
        pal.warning
    } else {
        pal.error
    };
    Span::styled(format!("[{}%] ", pct), Style::default().fg(color).add_modifier(Modifier::BOLD))
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, Terminal};

    use super::*;

    /// Collect the full TestBackend buffer into a single string of cell symbols.
    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect()
    }

    fn render_to_string(app: &mut AppState, w: u16, h: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        buffer_text(&terminal)
    }

    #[test]
    fn renders_three_columns_and_header() {
        let mut app = AppState::new("http://x".into(), "http://y".into());
        let text = render_to_string(&mut app, 120, 40);
        assert!(text.contains("Neoland"), "header brand missing");
        assert!(text.contains("Sessions"), "sessions column missing");
        assert!(text.contains("Conversation"), "conversation column missing");
        assert!(text.contains("Reasoning"), "reasoning column missing");
    }

    #[test]
    fn renders_conversation_bubbles() {
        let mut app = AppState::new("http://x".into(), "http://y".into());
        app.add_user_message("optimize this script");
        app.add_assistant_message("Use multiprocessing:\n```python\nPool().map(f, xs)\n```");
        let text = render_to_string(&mut app, 120, 40);
        assert!(text.contains("optimize this script"), "user bubble missing");
        assert!(text.contains("Neoland"), "agent avatar/brand missing");
        assert!(text.contains("multiprocessing"), "agent bubble missing");
    }

    #[test]
    fn welcome_shown_when_empty() {
        let mut app = AppState::new("http://x".into(), "http://y".into());
        let text = render_to_string(&mut app, 120, 40);
        assert!(text.contains("Welcome to"), "welcome card missing");
        assert!(text.contains("assist you today"), "welcome tagline missing");
    }

    /// Visual preview tool — renders representative states to stdout. Kept as an
    /// ignored test so it never runs in CI but is always available for a quick
    /// look (and for OSS docs/README captures):
    ///   cargo test --lib tui::ui::tests::dump_visual -- --ignored --nocapture
    #[test]
    #[ignore = "visual preview — run with --ignored --nocapture"]
    fn dump_visual() {
        fn dump(app: &mut AppState, w: u16, h: u16, title: &str) {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            terminal.draw(|f| render(f, app)).unwrap();
            let buf = terminal.backend().buffer().clone();
            println!("\n=== {} ===", title);
            for y in 0..buf.area.height {
                let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
                println!("│{}│", row);
            }
        }

        use crate::tui::app::{Session, SessionStatus, StageStatus};
        let mut app = AppState::new("http://x".into(), "http://y".into());
        app.theme = Theme::NeonGlass;
        dump(&mut app, 120, 28, "WELCOME (empty conversation)");

        // Populate a few sessions for the sidebar.
        let now = chrono::Utc::now();
        let mk = |name: &str, st: SessionStatus, mins: i64| {
            let mut s = Session::new(name.into());
            s.last_status = st;
            s.updated_at = now - chrono::Duration::minutes(mins);
            s
        };
        app.sessions = vec![
            mk("Refatorar módulo de auth", SessionStatus::Active, 0),
            mk("Revisar pipeline DSPy", SessionStatus::Done, 18),
            mk("Otimizar query do storage", SessionStatus::Failed, 130),
            mk("Brainstorm de arquitetura", SessionStatus::Idle, 1500),
        ];
        app.active_session_idx = 0;

        app.add_user_message("Otimiza esse script Python pra performance");
        app.add_assistant_message(
            "Claro! Sugiro multiprocessing:\n```python\nfrom multiprocessing import Pool\nwith Pool() as p:\n    results = p.map(work, items)\n```\nParaleliza o trabalho entre os cores.",
        );
        // A live pipeline in the reasoning column with RWA anchors.
        let (t1, _) = app.enqueue_task("Refatorar módulo de auth".into());
        app.start_task(t1);
        app.add_user_message("Otimiza esse script Python pra performance");
        app.add_assistant_message("Sugiro multiprocessing:\n```python\nPool().map(f, xs)\n```");
        app.update_stage("junior", StageStatus::Done { latency_ms: 120 }, Some(0.92));
        app.set_stage_provenance("junior", "0xA3F".into());
        app.update_stage("senior", StageStatus::Done { latency_ms: 200 }, Some(0.88));
        app.set_stage_provenance("senior", "0x7C1".into());
        app.update_stage("architect", StageStatus::Running, None);
        app.adr_status = Some("accepted".into());
        app.adr_title = Some("Auth middleware".into());
        dump(&mut app, 120, 28, "FULL (sessions + bubbles + reasoning w/ RWA)");

        // Mid-stream typewriter reveal.
        app.messages.clear();
        app.add_user_message("explica o pipeline");
        app.stream_mode = StreamMode::Typewriter;
        app.stream_target = "O pipeline tem 4 estágios: junior, senior, architect e tech-leader, cada um com confidence ancorada.".into();
        app.stream_revealed = 38;
        app.streaming = true;
        dump(&mut app, 120, 20, "STREAMING (typewriter, mid-reveal)");
    }

    #[test]
    fn renders_without_panic_when_narrow() {
        // Narrow terminal must not panic (side columns + Min(0) center clamp).
        let mut app = AppState::new("http://x".into(), "http://y".into());
        let _ = render_to_string(&mut app, 40, 20);
    }

    #[test]
    fn theme_switch_changes_rendered_colors() {
        let mut app = AppState::new("http://x".into(), "http://y".into());
        app.theme = Theme::TokyoNight;
        let mut t1 = Terminal::new(TestBackend::new(120, 40)).unwrap();
        t1.draw(|f| render(f, &mut app)).unwrap();
        let buf_tokyo = t1.backend().buffer().clone();

        app.theme = Theme::Monochrome;
        let mut t2 = Terminal::new(TestBackend::new(120, 40)).unwrap();
        t2.draw(|f| render(f, &mut app)).unwrap();
        let buf_mono = t2.backend().buffer().clone();

        assert_ne!(buf_tokyo, buf_mono, "theme change should alter rendered styles");
    }
}
