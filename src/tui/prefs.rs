//! User preferences persisted to `$XDG_CONFIG_HOME/neoland/prefs.json`
//! (fallback `$HOME/.config/neoland/`). Currently: theme + streaming mode.
//! No external crate for the config dir — resolved via `std::env`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Prefs {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub stream_mode: Option<String>,
}

/// `$XDG_CONFIG_HOME/neoland` or `$HOME/.config/neoland`.
fn config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("neoland"))
}

fn prefs_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("prefs.json"))
}

/// Load prefs, returning defaults on any error (missing file, bad JSON).
/// Disabled under `cfg!(test)` so unit tests stay isolated from real disk state.
pub fn load() -> Prefs {
    if cfg!(test) {
        return Prefs::default();
    }
    let Some(path) = prefs_path() else {
        return Prefs::default();
    };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Persist prefs best-effort (errors are swallowed — prefs are non-critical).
/// No-op under `cfg!(test)`.
pub fn save(prefs: &Prefs) {
    if cfg!(test) {
        return;
    }
    let Some(dir) = config_dir() else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(prefs) {
        let _ = std::fs::write(dir.join("prefs.json"), json);
    }
}
