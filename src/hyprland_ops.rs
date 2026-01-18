// Hyprland Operations Module
// Re-implementado usando crate `hyprland-ipc` do ai-agent-os

use anyhow::Result;
use hyprland_ipc::{HyprlandClient, Workspace, Window};

pub struct HyprlandIPC {
    client: HyprlandClient,
}

impl HyprlandIPC {
    pub async fn new() -> Result<Self> {
        let client = HyprlandClient::new()?;
        Ok(Self { client })
    }

    /// Envia comando raw para Hyprland
    pub async fn dispatch(&self, command: &str) -> Result<String> {
        self.client.dispatch(command).await
    }

    /// Obtém workspace ativo
    pub async fn get_active_workspace(&self) -> Result<Workspace> {
        self.client.get_active_workspace().await
    }

    /// Obtém todas janelas
    pub async fn get_clients(&self) -> Result<Vec<Window>> {
        self.client.get_clients().await
    }

    // ===========================================
    // Window Management (Convenience Methods)
    // ===========================================

    /// Define opacidade da janela (0.0 a 1.0)
    pub async fn set_window_opacity(&self, address: &str, opacity: f32) -> Result<()> {
        let cmd = format!("setprop address:{} alpha {}", address, opacity);
        self.dispatch(&cmd).await?;
        Ok(())
    }

    /// Define opacidade da janela ativa
    pub async fn set_active_opacity(&self, opacity: f32) -> Result<()> {
        let cmd = format!("setprop active alpha {}", opacity);
        self.dispatch(&cmd).await?;
        Ok(())
    }

    /// Toggle floating mode
    pub async fn toggle_float(&self, address: Option<&str>) -> Result<()> {
        let target = address.map(|a| format!("address:{}", a)).unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch togglefloating {}", target)).await?;
        Ok(())
    }

    /// Centraliza janela
    pub async fn center_window(&self, address: Option<&str>) -> Result<()> {
        let target = address.map(|a| format!("address:{}", a)).unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch centerwindow {}", target)).await?;
        Ok(())
    }

    /// Move para workspace
    pub async fn move_to_workspace(&self, workspace_id: i32, address: Option<&str>) -> Result<()> {
        let target = address.map(|a| format!("address:{}", a)).unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch movetoworkspace {},{}", workspace_id, target)).await?;
        Ok(())
    }

    /// Move para scratchpad (special workspace)
    pub async fn move_to_scratchpad(&self, address: Option<&str>) -> Result<()> {
        let target = address.map(|a| format!("address:{}", a)).unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch movetoworkspace special:scratch_neoland,{}", target)).await?;
        Ok(())
    }

    /// Toggle scratchpad
    pub async fn toggle_scratchpad(&self) -> Result<()> {
        self.dispatch("dispatch togglespecialworkspace scratch_neoland").await?;
        Ok(())
    }
}