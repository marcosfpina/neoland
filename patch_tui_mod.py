import re

content = open("src/tui/mod.rs").read()

content = content.replace("pub mod llama_manager;\n", "")
content = content.replace("pub mod llama_manager_logic;\n", "")

content = content.replace("    let models = llama_manager_logic::scan_models().await;\n", "")
content = content.replace("    app.llama_state.available_models = models;\n", "")

content = re.sub(r"                            Action::ToggleLlamaManager => \{[\s\S]*?app\.mode = match app\.mode \{[\s\S]*?\},", "", content)
content = re.sub(r"                            Action::LlamaManagerRun => \{[\s\S]*?llama_manager_logic::run_llama_server[\s\S]*?\},", "", content)
content = re.sub(r"                            Action::LlamaManagerStop => \{[\s\S]*?llama_manager_logic::stop_llama_server[\s\S]*?\},", "", content)

content = re.sub(r"        \(KeyCode::Char\('l'\), KeyModifiers::ALT\) => return Action::ToggleLlamaManager,\n", "", content)
content = re.sub(r"        \(KeyCode::Esc, _\) => return Action::ToggleLlamaManager,\n", "", content)
content = re.sub(r"        \(KeyCode::Char\('r'\), _\) \| \(KeyCode::Enter, _\) => return Action::LlamaManagerRun,\n", "", content)
content = re.sub(r"        \(KeyCode::Char\('s'\), _\) \| \(KeyCode::Char\('c'\), _\) => return Action::LlamaManagerStop,\n", "", content)
content = re.sub(r"        \(KeyCode::Up, _\) => \{[\s\S]*?return Action::None;\n\s+\},\n", "", content)
content = re.sub(r"        \(KeyCode::Down, _\) => \{[\s\S]*?return Action::None;\n\s+\},\n", "", content)

open("src/tui/mod.rs", "w").write(content)
