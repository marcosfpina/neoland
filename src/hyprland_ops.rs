use std::process::Command;
use std::sync::Once;

static HYPRLAND_INIT: Once = Once::new();

pub struct HyprlandOps;

impl HyprlandOps {
    /// Inicializa regras base para a aplicação
    pub fn init_rules(app_id: &str) {
        HYPRLAND_INIT.call_once(|| {
            // Regex: class:^(app.id)$
            // Format string: r"class:\^({})$" -> {} será substituído pelo app_id
            let class_regex = format!(r"class:\^({})$", app_id.replace(".", "\\."));

            // 1. Força flutuante
            let _ = Self::dispatch("exec", &format!("hyprctl keyword windowrulev2 \"float, {}\"", class_regex));
            let _ = Self::dispatch("exec", &format!("hyprctl keyword windowrulev2 \"center, {}\"", class_regex));
            let _ = Self::dispatch("exec", &format!("hyprctl keyword windowrulev2 \"size 1200 800, {}\"", class_regex));

            // 2. Opacidade
            let _ = Self::dispatch("exec", &format!("hyprctl keyword windowrulev2 \"opacity 0.90 0.90, {}\"", class_regex));
            
            // 3. Animação
            let _ = Self::dispatch("exec", &format!("hyprctl keyword windowrulev2 \"animation popin 80%, {}\"", class_regex));

            println!("Hyprland rules injected for {}", app_id);
        });
    }

    #[allow(dead_code)]
    pub fn bind_global_shortcut(mods: &str, key: &str, action: &str) {
        let _ = Self::dispatch("keyword", &format!("bind {}, {}, {}", mods, key, action));
    }

    pub fn move_to_workspace(id: i32) {
        let _ = Self::dispatch("dispatch", &format!("movetoworkspace {}, activewindow", id));
    }

    pub fn toggle_scratchpad() {
        let _ = Self::dispatch("dispatch", "movetoworkspace special:llamachat, activewindow");
        let _ = Self::dispatch("dispatch", "togglespecialworkspace llamachat");
    }

    fn dispatch(cmd: &str, args: &str) -> std::io::Result<()> {
        let mut command = Command::new("hyprctl");
        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        command.arg(cmd);
        if !parts.is_empty() {
             command.arg(args);
        }
        match command.spawn() {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Hyprland error: {}", e);
                Err(e)
            }
        }
    }
}