/// Command system — typed `/command [args...]` à la K9s.
///
/// Parsed from user input that starts with `/`.
/// Returns `None` if the input is not a valid command
/// (allowing normal Send flow).
use super::app::LlmProvider;

// ── Command enum ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Show help overlay (same as `?`).
    Help,
    /// Show reasoning chain for the latest stage.
    Why,
    /// Cancel the active task.
    Cancel,
    /// List queued tasks.
    Queue,
    /// Remove task N from queue.
    Dequeue(usize),
    /// Switch active provider by name.
    Provider(String),
    /// Apply a preset by name.
    Preset(String),
    /// Exit (with confirmation if tasks active).
    Exit,
    /// Clear output.
    Clear,
    /// Search output for term.
    Search(String),
    /// Send a steer message (alias for the steer action).
    Steer(String),
    /// Switch the color theme by name.
    Theme(String),
    /// Switch the streaming reveal mode by name.
    Stream(String),
    /// Start a new conversation session.
    NewSession,
    /// Unknown / malformed command.
    Unknown(String),
}

// ── Parser ──────────────────────────────────────────────────────────────────

/// Parse a string starting with `/` into a `Command`.
///
/// Sorted list of completable command names (no leading `/`).
/// Used by `AppState::tab_complete_command` for Tab-completion.
pub const COMPLETABLE_COMMANDS: &[&str] = &[
    "cancel", "clear", "dequeue", "exit", "help", "new",
    "preset", "provider", "queue", "search", "steer", "stream", "theme", "why",
];

/// Returns `None` if the input does not start with `/`.
/// Returns `Command::Unknown(args)` for unrecognized commands.
pub fn parse_command(input: &str) -> Option<Command> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let rest = &trimmed[1..]; // strip leading '/'

    // Split into command word and optional args.
    let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).map(|s| s.trim()).unwrap_or("");

    let command = match cmd.as_str() {
        "help" | "h" | "?" => Command::Help,
        "why" | "w" => Command::Why,
        "cancel" | "c" | "x" => Command::Cancel,
        "queue" | "q" => Command::Queue,
        "dequeue" | "dq" => {
            if args.is_empty() {
                Command::Unknown("usage: /dequeue <N> — number from /queue".into())
            } else {
                match args.parse::<usize>() {
                    Ok(n) if n > 0 => Command::Dequeue(n),
                    _ => Command::Unknown(format!("invalid position: {args}")),
                }
            }
        },
        "provider" | "model" => {
            if args.is_empty() {
                let names: Vec<&str> = LlmProvider::ALL.iter().map(|p| p.label()).collect();
                Command::Unknown(format!(
                    "usage: /provider <name>. Available: {}",
                    names.join(", ")
                ))
            } else {
                Command::Provider(args.to_string())
            }
        },
        "preset" => {
            if args.is_empty() {
                Command::Unknown(
                    "usage: /preset <name>. Available: balanced, creative, precise, research, safe"
                        .into(),
                )
            } else {
                Command::Preset(args.to_string())
            }
        },
        "exit" | "quit" => Command::Exit,
        "new" => Command::NewSession,
        "clear" | "cls" => Command::Clear,
        "search" | "find" | "s" => {
            if args.is_empty() {
                Command::Unknown("usage: /search <term>".into())
            } else {
                Command::Search(args.to_string())
            }
        },
        "steer" | "st" => {
            if args.is_empty() {
                Command::Unknown("usage: /steer <message>".into())
            } else {
                Command::Steer(args.to_string())
            }
        },
        "theme" => {
            if args.is_empty() {
                Command::Unknown(
                    "usage: /theme <name>. Available: tokyo-night, neon-glass, high-contrast, monochrome"
                        .into(),
                )
            } else {
                Command::Theme(args.to_string())
            }
        },
        "stream" | "streaming" => {
            if args.is_empty() {
                Command::Unknown(
                    "usage: /stream <mode>. Available: line, typewriter, thinking".into(),
                )
            } else {
                Command::Stream(args.to_string())
            }
        },
        other => Command::Unknown(format!("unknown command: /{other}. Try /help")),
    };

    Some(command)
}

// ── Help text ───────────────────────────────────────────────────────────────

pub const HELP_TEXT: &str = "\
 ── Commands ──────────────────────────────────────────────────
  /help, /h, /?     Show this help
  /why, /w          Show reasoning chain for latest stage
  /cancel, /c       Cancel active task
  /queue, /q        List queued tasks
  /dequeue <N>      Remove task N from queue
  /provider <name>  Switch provider (deepseek, gemini, groq, ...)
  /preset <name>    Apply preset (balanced, creative, precise, ...)
  /exit             Quit (confirms if tasks active)
  /new              Start a new conversation session
  /clear, /cls      Clear output
  /search <term>    Search output
  /steer <msg>      Send steering instruction
  /theme <name>     Switch theme (tokyo-night, neon-glass, ...)
  /stream <mode>    Reveal mode (line, typewriter, thinking)

 ── Keybindings ───────────────────────────────────────────────
  Tab / Shift+Tab   Cycle panels
  Enter             Send task / submit
  Shift+Enter       Steer task (multi-line)
  ^p                Toggle pipeline view
  ^x                Cancel task
  ^6                Cycle provider
  ^l                Clear screen
  ^c                Quit
  Esc               Cancel / close
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_a_command() {
        assert_eq!(parse_command("hello"), None);
        assert_eq!(parse_command(""), None);
        assert_eq!(parse_command("  plain text  "), None);
    }

    #[test]
    fn help_variants() {
        assert_eq!(parse_command("/help"), Some(Command::Help));
        assert_eq!(parse_command("/h"), Some(Command::Help));
        assert_eq!(parse_command("/?"), Some(Command::Help));
        assert_eq!(parse_command("  /help  "), Some(Command::Help));
    }

    #[test]
    fn cancel_variants() {
        assert_eq!(parse_command("/cancel"), Some(Command::Cancel));
        assert_eq!(parse_command("/c"), Some(Command::Cancel));
        assert_eq!(parse_command("/x"), Some(Command::Cancel));
    }

    #[test]
    fn exit_variants() {
        assert_eq!(parse_command("/exit"), Some(Command::Exit));
        assert_eq!(parse_command("/quit"), Some(Command::Exit));
    }

    #[test]
    fn clear_variants() {
        assert_eq!(parse_command("/clear"), Some(Command::Clear));
        assert_eq!(parse_command("/cls"), Some(Command::Clear));
    }

    #[test]
    fn steer_with_args() {
        assert_eq!(
            parse_command("/steer focus on tests"),
            Some(Command::Steer("focus on tests".into()))
        );
        assert_eq!(parse_command("/st use verbose"), Some(Command::Steer("use verbose".into())));
    }

    #[test]
    fn steer_missing_args() {
        assert!(matches!(parse_command("/steer"), Some(Command::Unknown(_))));
    }

    #[test]
    fn search_with_term() {
        assert_eq!(parse_command("/search error"), Some(Command::Search("error".into())));
        assert_eq!(parse_command("/find timeout"), Some(Command::Search("timeout".into())));
        assert_eq!(parse_command("/s panic"), Some(Command::Search("panic".into())));
    }

    #[test]
    fn dequeue_valid() {
        assert_eq!(parse_command("/dequeue 3"), Some(Command::Dequeue(3)));
        assert_eq!(parse_command("/dq 1"), Some(Command::Dequeue(1)));
    }

    #[test]
    fn dequeue_invalid() {
        assert!(matches!(parse_command("/dequeue"), Some(Command::Unknown(_))));
        assert!(matches!(parse_command("/dequeue abc"), Some(Command::Unknown(_))));
    }

    #[test]
    fn unknown_command() {
        assert!(matches!(parse_command("/foobar"), Some(Command::Unknown(_))));
    }

    #[test]
    fn provider_without_args_shows_usage() {
        assert!(matches!(parse_command("/provider"), Some(Command::Unknown(_))));
    }

    #[test]
    fn provider_with_name() {
        assert_eq!(parse_command("/provider gemini"), Some(Command::Provider("gemini".into())));
    }

    #[test]
    fn preset_with_name() {
        assert_eq!(parse_command("/preset creative"), Some(Command::Preset("creative".into())));
    }

    #[test]
    fn theme_with_name() {
        assert_eq!(parse_command("/theme neon-glass"), Some(Command::Theme("neon-glass".into())));
    }

    #[test]
    fn theme_without_args_shows_usage() {
        assert!(matches!(parse_command("/theme"), Some(Command::Unknown(_))));
    }
}
