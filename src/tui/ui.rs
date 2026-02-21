// TUI Rendering (ratatui widgets)
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{AppState, ConnectionStatus, MessageRole};

// Tokyo Night Palette
pub mod colors {
    use ratatui::style::Color;

    pub const BG: Color = Color::Rgb(26, 27, 38);
    pub const FG: Color = Color::Rgb(192, 202, 245);
    pub const PRIMARY: Color = Color::Rgb(122, 162, 247);
    pub const ACCENT: Color = Color::Rgb(187, 154, 247);
    pub const SUCCESS: Color = Color::Rgb(158, 206, 106);
    pub const WARNING: Color = Color::Rgb(255, 158, 100);
    pub const ERROR: Color = Color::Rgb(247, 118, 142);
    pub const MUTED: Color = Color::Rgb(86, 95, 137);
}

pub fn render(f: &mut Frame<'_>, app: &AppState) {
    let size = f.area();

    // Main layout: Header | Chat | Input
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Chat area
            Constraint::Length(3), // Input bar
        ])
        .split(size);

    render_header(f, main_chunks[0], app);

    // Chat area + optional sidebar
    if app.sidebar_visible {
        let chat_sidebar = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // Chat
                Constraint::Percentage(30), // Sidebar
            ])
            .split(main_chunks[1]);

        render_chat(f, chat_sidebar[0], app);
        render_sidebar(f, chat_sidebar[1], app);
    } else {
        render_chat(f, main_chunks[1], app);
    }

    render_input(f, main_chunks[2], app);
}

fn render_header(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let (conn_label, conn_color): (&str, Color) = match app.connection_status {
        ConnectionStatus::Connected => ("●", colors::SUCCESS),
        ConnectionStatus::Degraded => ("◐", colors::WARNING),
        ConnectionStatus::Offline => ("○", colors::ERROR),
        ConnectionStatus::Unknown => ("◌", colors::MUTED),
    };

    let thinking_label = if app.is_thinking {
        " ⏳ Thinking..."
    } else {
        ""
    };

    let backend_label = match &app.active_backend {
        Some(b) => format!(" [{}]", b),
        None => String::new(),
    };

    let session_label = if app.session_tokens > 0 {
        format!(" {}tok {}ms", app.session_tokens, app.last_latency_ms)
    } else {
        String::new()
    };

    let spans = vec![
        Span::raw(" 🚀 Neoland TUI "),
        Span::styled(conn_label, Style::default().fg(conn_color).add_modifier(Modifier::BOLD)),
        Span::styled(backend_label, Style::default().fg(colors::ACCENT)),
        Span::styled(session_label, Style::default().fg(colors::MUTED)),
        Span::styled(thinking_label, Style::default().fg(colors::WARNING)),
        Span::styled(
            format!(" | Preset: {} ", get_preset_name(&app.config)),
            Style::default().fg(colors::PRIMARY),
        ),
    ];

    let header = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::MUTED)),
        );

    f.render_widget(header, area);
}

fn render_chat(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let mut messages: Vec<ListItem> = app
        .messages
        .iter()
        .skip(app.scroll_offset)
        .map(|msg| {
            let (prefix, color) = match msg.role {
                MessageRole::User => ("👤 VOCÊ", colors::PRIMARY),
                MessageRole::Assistant => ("🤖 AI", colors::SUCCESS),
                MessageRole::System => ("⚙️  SYSTEM", colors::WARNING),
            };

            let time = msg.timestamp.format("%H:%M:%S").to_string();
            let content = vec![
                Line::from(vec![
                    Span::styled(format!("[{}] ", time), Style::default().fg(colors::MUTED)),
                    Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(Span::styled(msg.content.clone(), Style::default().fg(colors::FG))),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    // Show streaming message in progress (if any)
    if let Some(ref pending) = app.pending_message {
        let streaming_content = vec![
            Line::from(vec![
                Span::styled(
                    "🤖 AI",
                    Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ▍", Style::default().fg(colors::ACCENT)),
            ]),
            Line::from(Span::styled(pending.clone(), Style::default().fg(colors::FG))),
            Line::from(""),
        ];
        messages.push(ListItem::new(streaming_content));
    }

    let chat_list = List::new(messages)
        .block(
            Block::default()
                .title(" 💬 Chat ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::MUTED)),
        )
        .style(Style::default().fg(colors::FG));

    f.render_widget(chat_list, area);
}

fn render_sidebar(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let config = &app.config;

    let conn_str = match app.connection_status {
        ConnectionStatus::Connected => "✅ Connected",
        ConnectionStatus::Degraded => "⚠ Degraded",
        ConnectionStatus::Offline => "❌ Offline",
        ConnectionStatus::Unknown => "◌ Checking...",
    };

    let backend_str = app.active_backend.as_deref().unwrap_or("—");

    let info = vec![
        format!("🔗 Sessão"),
        format!(""),
        format!(" Status: {}", conn_str),
        format!(" Backend: {}", backend_str),
        format!(" Tokens: {}", app.session_tokens),
        format!(" Latência: {}ms", app.last_latency_ms),
        format!(""),
        format!("📊 Config"),
        format!(""),
        format!(" Temperature: {:.2}", config.temperature),
        format!(" Top P: {:.2}", config.top_p),
        format!(" Max Tokens: {}", config.max_tokens),
        format!(" Context Top K: {}", config.context_top_k),
        format!(""),
        format!("⌨️  Atalhos"),
        format!(""),
        format!(" Ctrl+1-5: Presets"),
        format!(" Ctrl+L: Limpar"),
        format!(" Enter: Enviar"),
        format!(" Tab: Sidebar"),
        format!(" Esc: Sair"),
        format!(" j/k: Scroll"),
    ];

    let sidebar = Paragraph::new(info.join("\n"))
        .style(Style::default().fg(colors::FG))
        .block(
            Block::default()
                .title(" ℹ️  Info ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::MUTED)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(sidebar, area);
}

fn render_input(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let input = Paragraph::new(app.input_buffer.clone())
        .style(Style::default().fg(colors::FG))
        .block(
            Block::default()
                .title(" ✏️  Input (Enter para enviar) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY)),
        );

    f.render_widget(input, area);
}

fn get_preset_name(config: &super::presets::QueryConfig) -> &'static str {
    if config.temperature == 0.7 && config.max_tokens == 600 {
        "BALANCEADO"
    } else if config.temperature == 1.5 {
        "CRIATIVO"
    } else if config.temperature == 0.3 {
        "PRECISO"
    } else if config.context_top_k == 8 {
        "PESQUISA"
    } else if !config.enable_commands {
        "SEGURO"
    } else {
        "CUSTOM"
    }
}
