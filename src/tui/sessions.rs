//! Session persistence — one JSON file per session under
//! `$XDG_CONFIG_HOME/neoland/sessions/` (fallback `$HOME/.config/neoland/`).
//!
//! Timestamps are stored as RFC3339 strings via a DTO (`SessionRecord`) so we
//! don't need chrono's `serde` feature.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::app::{ChatMessage, MessageRole, Session, SessionStatus};

// ── Persistence DTOs ──────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct MessageRecord {
    role: String,
    content: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct SessionRecord {
    id: String,
    name: String,
    created_at: String,
    updated_at: String,
    last_status: String,
    messages: Vec<MessageRecord>,
}

fn role_to_str(r: &MessageRole) -> &'static str {
    match r {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}

fn role_from_str(s: &str) -> MessageRole {
    match s {
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        _ => MessageRole::System,
    }
}

fn status_to_str(s: SessionStatus) -> &'static str {
    match s {
        SessionStatus::Active => "active",
        SessionStatus::Done => "done",
        SessionStatus::Failed => "failed",
        SessionStatus::Idle => "idle",
    }
}

fn status_from_str(s: &str) -> SessionStatus {
    match s {
        "active" => SessionStatus::Active,
        "done" => SessionStatus::Done,
        "failed" => SessionStatus::Failed,
        _ => SessionStatus::Idle,
    }
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

impl SessionRecord {
    fn from_session(s: &Session) -> Self {
        SessionRecord {
            id: s.id.to_string(),
            name: s.name.clone(),
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
            last_status: status_to_str(s.last_status).to_string(),
            messages: s
                .messages
                .iter()
                .map(|m| MessageRecord {
                    role: role_to_str(&m.role).to_string(),
                    content: m.content.clone(),
                    timestamp: m.timestamp.to_rfc3339(),
                })
                .collect(),
        }
    }

    fn into_session(self) -> Session {
        Session {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            name: self.name,
            created_at: parse_ts(&self.created_at),
            updated_at: parse_ts(&self.updated_at),
            last_status: status_from_str(&self.last_status),
            messages: self
                .messages
                .into_iter()
                .map(|m| ChatMessage {
                    role: role_from_str(&m.role),
                    content: m.content,
                    timestamp: parse_ts(&m.timestamp),
                })
                .collect(),
        }
    }
}

// ── Paths ─────────────────────────────────────────────────────────────────

fn sessions_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("neoland").join("sessions"))
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Load all sessions, most-recently-updated first. Returns empty on any error.
/// Disabled under `cfg!(test)` so unit tests stay isolated from real disk state.
pub fn load_all() -> Vec<Session> {
    if cfg!(test) {
        return Vec::new();
    }
    let Some(dir) = sessions_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut sessions: Vec<Session> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|s| serde_json::from_str::<SessionRecord>(&s).ok())
        .map(SessionRecord::into_session)
        .collect();
    sessions.sort_by_key(|session| std::cmp::Reverse(session.updated_at));
    sessions
}

/// Persist a single session (best-effort). No-op under `cfg!(test)`.
pub fn save(session: &Session) {
    if cfg!(test) {
        return;
    }
    let Some(dir) = sessions_dir() else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    let record = SessionRecord::from_session(session);
    if let Ok(json) = serde_json::to_string_pretty(&record) {
        let _ = std::fs::write(dir.join(format!("{}.json", session.id)), json);
    }
}

/// Human-friendly "time ago" label (e.g. "agora", "5m", "3h", "2d").
pub fn relative_time(then: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = (now - then).num_seconds().max(0);
    if secs < 10 {
        "agora".to_string()
    } else if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_roundtrip_preserves_fields() {
        let mut s = Session::new("Refatorar auth".into());
        s.last_status = SessionStatus::Done;
        s.messages.push(ChatMessage {
            role: MessageRole::User,
            content: "oi".into(),
            timestamp: Utc::now(),
        });
        s.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: "olá".into(),
            timestamp: Utc::now(),
        });

        let rec = SessionRecord::from_session(&s);
        let back = rec.into_session();
        assert_eq!(back.id, s.id);
        assert_eq!(back.name, "Refatorar auth");
        assert_eq!(back.last_status, SessionStatus::Done);
        assert_eq!(back.messages.len(), 2);
        assert_eq!(back.messages[0].role, MessageRole::User);
        assert_eq!(back.messages[1].content, "olá");
    }

    #[test]
    fn relative_time_buckets() {
        let now = Utc::now();
        assert_eq!(relative_time(now, now), "agora");
        assert_eq!(relative_time(now - chrono::Duration::seconds(30), now), "30s");
        assert_eq!(relative_time(now - chrono::Duration::minutes(5), now), "5m");
        assert_eq!(relative_time(now - chrono::Duration::hours(3), now), "3h");
        assert_eq!(relative_time(now - chrono::Duration::days(2), now), "2d");
    }
}
