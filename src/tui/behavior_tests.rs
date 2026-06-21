//! Behavioral tests — simulate user interactions via `process_key` and assert
//! on both state mutations and rendered output. No server, no event threads.
//!
//! Run:  cargo test --lib tui::behavior_tests

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};

use super::{
    app::{AppState, StageStatus},
    events::Action,
    process_key,
    ui::render,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

use super::app::NotificationLevel;
use super::commands::Command;

fn app() -> AppState {
    AppState::new("http://x".into(), "http://y".into())
}

fn key(app: &mut AppState, code: KeyCode, mods: KeyModifiers) -> Action {
    process_key(app, code, mods)
}

fn press(app: &mut AppState, code: KeyCode) -> Action {
    let action = key(app, code, KeyModifiers::NONE);
    apply(app, &action);
    action
}

fn type_str(app: &mut AppState, s: &str) {
    for c in s.chars() {
        press(app, KeyCode::Char(c));
    }
}

/// Apply the synchronous, non-async parts of the main-loop action dispatch.
/// Skips network actions (SubmitTask, SteerTask, etc.) — tests verify state
/// mutations and renders, not server interactions.
fn apply(app: &mut AppState, action: &Action) {
    use Action::*;
    match action {
        FocusNextPanel => app.focus_next_panel(),
        FocusPrevPanel => app.focus_prev_panel(),
        CancelTask => {
            if let Some(id) = app.active_task_id {
                app.complete_task(id, false);
            }
        },
        CycleProvider => app.cycle_provider(),
        ExecuteCommand(cmd) => apply_command(app, cmd),
        _ => {},
    }
}

fn apply_command(app: &mut AppState, cmd: &Command) {
    match cmd {
        Command::Help => app.show_help = true,
        Command::Why => {
            if app.pipeline_stages.is_empty() {
                app.add_notification(
                    NotificationLevel::Info,
                    "nenhum pipeline ativo para inspecionar".to_string(),
                );
            } else {
                app.why_mode = true;
            }
        },
        Command::Clear => {
            app.messages.clear();
            app.scroll_offsets = [0; 3];
        },
        Command::NewSession => app.new_session(),
        Command::Name(slot, name) => app.set_agent_name(slot - 1, name.clone()),
        Command::Theme(name) => {
            if let Some(t) = super::app::Theme::from_label(name) {
                app.theme = t;
            }
        },
        _ => {},
    }
}

fn rendered(app: &mut AppState) -> String {
    let mut t = Terminal::new(TestBackend::new(120, 30)).unwrap();
    t.draw(|f| render(f, app)).unwrap();
    t.backend().buffer().content().iter().map(|c| c.symbol()).collect()
}

fn visible(app: &mut AppState, text: &str) -> bool {
    rendered(app).contains(text)
}

// ── Tab completion ────────────────────────────────────────────────────────────

#[test]
fn tab_completes_single_match() {
    let mut a = app();
    type_str(&mut a, "/prov");
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.input_buffer, "/provider ");
}

#[test]
fn tab_on_ambiguous_prefix_cycles_and_notifies() {
    let mut a = app();
    type_str(&mut a, "/s"); // matches: search, steer, stream
    press(&mut a, KeyCode::Tab);
    assert!(!a.notifications.is_empty(), "should notify with match list");
    assert!(a.input_buffer.starts_with('/'));
}

#[test]
fn tab_on_no_match_is_noop() {
    let mut a = app();
    type_str(&mut a, "/zzz");
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.input_buffer, "/zzz"); // unchanged
}

#[test]
fn tab_without_slash_changes_panel() {
    let mut a = app();
    let before = a.focused_panel;
    press(&mut a, KeyCode::Tab);
    assert_ne!(a.focused_panel, before);
}

// ── Breakpoint Y/N one-keystroke ─────────────────────────────────────────────

#[test]
fn breakpoint_y_approves_without_enter() {
    let mut a = app();
    a.trigger_breakpoint("read_file".into(), "auth.rs".into());
    let action = press(&mut a, KeyCode::Char('y'));
    assert!(
        matches!(action, Action::ResolveBreakpoint { ref resolution, .. } if resolution == "approve"),
        "expected approve, got {:?}",
        action
    );
}

#[test]
fn breakpoint_capital_y_also_approves() {
    let mut a = app();
    a.trigger_breakpoint("tool".into(), "args".into());
    let action = key(&mut a, KeyCode::Char('Y'), KeyModifiers::NONE);
    assert!(
        matches!(action, Action::ResolveBreakpoint { ref resolution, .. } if resolution == "approve")
    );
}

#[test]
fn breakpoint_n_rejects_without_enter() {
    let mut a = app();
    a.trigger_breakpoint("rm".into(), "*.rs".into());
    let action = press(&mut a, KeyCode::Char('n'));
    assert!(
        matches!(action, Action::ResolveBreakpoint { ref resolution, .. } if resolution == "reject")
    );
}

#[test]
fn y_with_text_in_buffer_does_not_resolve() {
    // If user has typed something, Y is just a character
    let mut a = app();
    a.trigger_breakpoint("tool".into(), "args".into());
    type_str(&mut a, "steer: ");
    let action = press(&mut a, KeyCode::Char('y'));
    assert!(!matches!(action, Action::ResolveBreakpoint { .. }));
}

// ── Why-mode ──────────────────────────────────────────────────────────────────

#[test]
fn why_command_enters_why_mode() {
    let mut a = app();
    // start_task populates pipeline_stages — required for /why to activate
    let (id, _) = a.enqueue_task("test".into());
    a.start_task(id);
    a.update_stage("junior", StageStatus::Done { latency_ms: 100 }, Some(0.9));
    type_str(&mut a, "/why");
    press(&mut a, KeyCode::Enter);
    assert!(a.why_mode, "why_mode should be true after /why");
}

#[test]
fn why_with_empty_pipeline_shows_notification() {
    let mut a = app();
    type_str(&mut a, "/why");
    press(&mut a, KeyCode::Enter);
    assert!(!a.why_mode, "why_mode stays false when no stages");
    assert!(!a.notifications.is_empty());
}

#[test]
fn esc_exits_why_mode() {
    let mut a = app();
    a.why_mode = true;
    press(&mut a, KeyCode::Esc);
    assert!(!a.why_mode);
}

// ── Context hints in render ───────────────────────────────────────────────────

#[test]
fn idle_hints_show_send_and_tab() {
    let mut a = app();
    let out = rendered(&mut a);
    assert!(out.contains("enviar"), "idle hint should mention 'enviar'");
    assert!(out.contains("painéis"), "idle hint should mention 'painéis'");
}

#[test]
fn hints_show_steer_when_task_active() {
    let mut a = app();
    let (id, _) = a.enqueue_task("task".into());
    a.start_task(id);
    assert!(visible(&mut a, "steer"), "hints should mention 'steer' when task active");
}

#[test]
fn hints_show_aprovar_at_breakpoint() {
    let mut a = app();
    a.trigger_breakpoint("tool".into(), "args".into());
    assert!(visible(&mut a, "aprovar"), "hints should mention 'aprovar' at breakpoint");
}

#[test]
fn hints_show_complete_hint_on_slash_input() {
    let mut a = app();
    type_str(&mut a, "/");
    assert!(visible(&mut a, "completar"), "hints should mention 'completar' when typing /");
}

// ── Session naming ────────────────────────────────────────────────────────────

#[test]
fn session_named_from_task_truncated_at_24() {
    let mut a = app();
    a.name_active_session_from("Refatorar módulo de autenticação completa do sistema");
    let name = &a.sessions[0].name;
    assert!(name.ends_with('…'), "should end with ellipsis");
    // "Refatorar módulo de auten" is 25 chars, so we take 24 + …
    let base: String = name.chars().filter(|&c| c != '…').collect();
    assert!(base.chars().count() <= 24, "base name too long: {}", base.chars().count());
}

#[test]
fn short_task_name_has_no_ellipsis() {
    let mut a = app();
    a.name_active_session_from("Fix bug");
    assert_eq!(a.sessions[0].name, "Fix bug");
}

// ── /name command — agent nickname ────────────────────────────────────────────

#[test]
fn set_agent_name_updates_active_stage() {
    let mut a = app();
    let (id, _) = a.enqueue_task("task".into());
    a.start_task(id);
    a.set_agent_name(0, "Kronos".into());
    assert_eq!(a.pipeline_stages[0].nickname, "Kronos");
}

#[test]
fn set_agent_name_out_of_range_is_noop() {
    let mut a = app();
    let (id, _) = a.enqueue_task("task".into());
    a.start_task(id);
    a.set_agent_name(99, "Bad".into()); // should not panic
    assert_ne!(a.pipeline_stages[0].nickname, "Bad");
}

// ── Nicknames are not job titles ──────────────────────────────────────────────

#[test]
fn render_does_not_contain_job_titles() {
    let mut a = app();
    let (id, _) = a.enqueue_task("test task".into());
    a.start_task(id);
    a.update_stage("junior", StageStatus::Done { latency_ms: 100 }, Some(0.9));
    let out = rendered(&mut a);
    assert!(!out.contains("junior"), "render should not show 'junior'");
    assert!(!out.contains("senior"), "render should not show 'senior'");
    assert!(!out.contains("architect"), "render should not show 'architect'");
    // tech-leader is hyphenated — harder to match but let's check
    assert!(!out.contains("tech-leader"), "render should not show 'tech-leader'");
}
