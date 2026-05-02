import re

def clean_events():
    with open("src/tui/events.rs", "r") as f: content = f.read()
    content = content.replace("    ToggleLlamaManager, // ^l\n", "")
    content = content.replace("    LlamaManagerRun,    // r or enter\n", "")
    content = content.replace("    LlamaManagerStop,   // s or ctrl+c\n", "")
    with open("src/tui/events.rs", "w") as f: f.write(content)

def clean_ui():
    with open("src/tui/ui.rs", "r") as f: content = f.read()
    content = content.replace("AppMode, ", "")
    content = content.replace(", AppMode", "")
    content = re.sub(r"\s+if app\.mode == AppMode::LlamaManager \{[\s\S]*?\}\n", "", content)
    content = re.sub(r"fn render_llama_manager\(f: &mut Frame<\x27_>, app: &mut AppState\) \{[\s\S]*?(?=\nfn count_visual_lines)", "", content)
    with open("src/tui/ui.rs", "w") as f: f.write(content)

def clean_app():
    with open("src/tui/app.rs", "r") as f: content = f.read()
    content = content.replace("pub enum AppMode {\n    Workstation,\n    LlamaManager,\n}\n\n", "")
    content = content.replace("pub mode: AppMode,\n", "")
    content = content.replace("pub llama_state: LlamaManagerState,\n", "")
    content = content.replace("use super::{llama_manager::LlamaManagerState, presets::QueryConfig};\n", "use super::presets::QueryConfig;\n")
    content = content.replace("            mode: AppMode::Workstation,\n", "")
    content = content.replace("            llama_state: LlamaManagerState::new(),\n", "")
    with open("src/tui/app.rs", "w") as f: f.write(content)

def clean_mod():
    with open("src/tui/mod.rs", "r") as f: content = f.read()
    content = content.replace("pub mod llama_manager;\n", "")
    content = content.replace("pub mod llama_manager_logic;\n", "")
    content = re.sub(r"\s+let models = llama_manager_logic::scan_models\(\)\.await;\n\s+app\.llama_state\.available_models = models;\n", "", content)
    
    # Remove LlamaManager Action Matchers
    content = re.sub(r"\s+Action::ToggleLlamaManager => \{[\s\S]*?Action::CancelTask", "\n                            Action::CancelTask", content)
    
    # Remove LlamaManager Keybindings
    content = re.sub(r"\s+// ── Llama Manager ─────────────────────────────────────────────[\s\S]*?// ── Send message ──────────────────────────────────────────────", "\n        // ── Send message ──────────────────────────────────────────────", content)
    
    # The straggler bug:
    content = re.sub(r"\s+if app\.mode == app::AppMode::LlamaManager \{[\s\S]*?return Action::None;\n\s+\}\n", "", content)
    
    with open("src/tui/mod.rs", "w") as f: f.write(content)

clean_events()
clean_ui()
clean_app()
clean_mod()
