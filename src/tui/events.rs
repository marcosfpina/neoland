// TUI Event Handling
use super::app::AppState;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Quit,
    SendMessage,
    ClearChat,
    ApplyPreset(String),
    ToggleSidebar,
    Input(char),
    Backspace,
    ScrollUp,
    ScrollDown,
}

pub fn handle_events(_app: &mut AppState) -> Result<Option<AppEvent>> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            return Ok(match (key.code, key.modifiers) {
                // Quit
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(AppEvent::Quit),
                (KeyCode::Esc, _) => Some(AppEvent::Quit),

                // Send message (Ctrl+Enter or plain Enter)
                (KeyCode::Enter, KeyModifiers::CONTROL) => Some(AppEvent::SendMessage),
                (KeyCode::Enter, KeyModifiers::NONE) => Some(AppEvent::SendMessage),

                // Clear chat
                (KeyCode::Char('l'), KeyModifiers::CONTROL) => Some(AppEvent::ClearChat),

                // Presets (Ctrl+1-5)
                (KeyCode::Char('1'), KeyModifiers::CONTROL) => {
                    Some(AppEvent::ApplyPreset("balanced".into()))
                }
                (KeyCode::Char('2'), KeyModifiers::CONTROL) => {
                    Some(AppEvent::ApplyPreset("creative".into()))
                }
                (KeyCode::Char('3'), KeyModifiers::CONTROL) => {
                    Some(AppEvent::ApplyPreset("precise".into()))
                }
                (KeyCode::Char('4'), KeyModifiers::CONTROL) => {
                    Some(AppEvent::ApplyPreset("research".into()))
                }
                (KeyCode::Char('5'), KeyModifiers::CONTROL) => {
                    Some(AppEvent::ApplyPreset("safe".into()))
                }

                // Toggle sidebar
                (KeyCode::Tab, _) => Some(AppEvent::ToggleSidebar),

                // Text input
                (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                    Some(AppEvent::Input(c))
                }

                // Backspace
                (KeyCode::Backspace, _) => Some(AppEvent::Backspace),

                // Scroll
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(AppEvent::ScrollUp),
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(AppEvent::ScrollDown),

                _ => None,
            });
        }
    }
    Ok(None)
}
