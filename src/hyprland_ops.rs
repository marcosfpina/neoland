// Hyprland IPC — implementado diretamente via Unix socket.
// Não depende de crate externo; usa tokio + serde_json já presentes no workspace.
//
// TODO(future): abstrair atrás de um trait `WmBackend` para habilitar mocks em CI headless
// e isolar a superfície compositor-específica do resto do control plane.
// Ver: docs/adr/wm-backend-trait.md (a criar quando priorizarmos portabilidade/testabilidade).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub monitor: String,
    pub windows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub address: String,
    pub title: String,
    pub class: String,
    pub workspace: i32,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum HyprlandEvent {
    WorkspaceChanged { id: i32 },
    WindowOpened { address: String },
    WindowClosed { address: String },
    WindowMoved { address: String, workspace: i32 },
    MonitorAdded { name: String },
    MonitorRemoved { name: String },
}

// ---------------------------------------------------------------------------
// Socket client
// ---------------------------------------------------------------------------

fn resolve_socket_path() -> Result<PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR not set")?;
    let instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE not set — not running under Hyprland?")?;
    let path = PathBuf::from(runtime_dir)
        .join("hypr")
        .join(&instance)
        .join(".socket.sock");
    anyhow::ensure!(path.exists(), "Hyprland socket not found at {:?}", path);
    Ok(path)
}

async fn socket_request(socket_path: &PathBuf, command: &str) -> Result<String> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .context("Failed to connect to Hyprland socket")?;
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;
    Ok(response.trim().to_string())
}

// ---------------------------------------------------------------------------
// Public facade
// ---------------------------------------------------------------------------

pub struct HyprlandIPC {
    socket_path: PathBuf,
}

impl HyprlandIPC {
    pub async fn new() -> Result<Self> {
        Ok(Self { socket_path: resolve_socket_path()? })
    }

    pub async fn dispatch(&self, command: &str) -> Result<String> {
        socket_request(&self.socket_path, command).await
    }

    pub async fn get_active_workspace(&self) -> Result<Workspace> {
        let raw = self.dispatch("j/activeworkspace").await?;
        serde_json::from_str(&raw).context("Failed to parse workspace info")
    }

    pub async fn get_clients(&self) -> Result<Vec<Window>> {
        let raw = self.dispatch("j/clients").await?;
        serde_json::from_str(&raw).context("Failed to parse clients")
    }

    // Window management

    pub async fn set_window_opacity(&self, address: &str, opacity: f32) -> Result<()> {
        self.dispatch(&format!("setprop address:{} alpha {}", address, opacity)).await?;
        Ok(())
    }

    pub async fn set_active_opacity(&self, opacity: f32) -> Result<()> {
        self.dispatch(&format!("setprop active alpha {}", opacity)).await?;
        Ok(())
    }

    pub async fn toggle_float(&self, address: Option<&str>) -> Result<()> {
        let target = address
            .map(|a| format!("address:{}", a))
            .unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch togglefloating {}", target)).await?;
        Ok(())
    }

    pub async fn center_window(&self, address: Option<&str>) -> Result<()> {
        let target = address
            .map(|a| format!("address:{}", a))
            .unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch centerwindow {}", target)).await?;
        Ok(())
    }

    pub async fn move_to_workspace(&self, workspace_id: i32, address: Option<&str>) -> Result<()> {
        let target = address
            .map(|a| format!("address:{}", a))
            .unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch movetoworkspace {},{}", workspace_id, target)).await?;
        Ok(())
    }

    pub async fn move_to_scratchpad(&self, address: Option<&str>) -> Result<()> {
        let target = address
            .map(|a| format!("address:{}", a))
            .unwrap_or_else(|| "active".to_string());
        self.dispatch(&format!("dispatch movetoworkspace special:scratch_neoland,{}", target)).await?;
        Ok(())
    }

    pub async fn toggle_scratchpad(&self) -> Result<()> {
        self.dispatch("dispatch togglespecialworkspace scratch_neoland").await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Event stream
// ---------------------------------------------------------------------------

pub struct HyprlandEventStream {
    reader: BufReader<UnixStream>,
}

impl HyprlandIPC {
    pub async fn subscribe_events(&self) -> Result<HyprlandEventStream> {
        let events_socket = self.socket_path.parent().unwrap().join(".socket2.sock");
        let stream = UnixStream::connect(&events_socket)
            .await
            .context("Failed to connect to Hyprland events socket")?;
        Ok(HyprlandEventStream { reader: BufReader::new(stream) })
    }
}

impl HyprlandEventStream {
    pub async fn next_event(&mut self) -> Result<Option<HyprlandEvent>> {
        let mut line = String::new();
        if self.reader.read_line(&mut line).await? == 0 {
            return Ok(None);
        }
        let parts: Vec<&str> = line.trim().splitn(2, ">>").collect();
        if parts.len() != 2 {
            return Ok(None);
        }
        let event = match parts[0] {
            "workspace" => HyprlandEvent::WorkspaceChanged {
                id: parts[1].parse().unwrap_or(0),
            },
            "openwindow" => HyprlandEvent::WindowOpened { address: parts[1].to_string() },
            "closewindow" => HyprlandEvent::WindowClosed { address: parts[1].to_string() },
            "movewindow" => {
                let data: Vec<&str> = parts[1].split(',').collect();
                HyprlandEvent::WindowMoved {
                    address: data.first().unwrap_or(&"").to_string(),
                    workspace: data.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                }
            }
            "monitoradded" => HyprlandEvent::MonitorAdded { name: parts[1].to_string() },
            "monitorremoved" => HyprlandEvent::MonitorRemoved { name: parts[1].to_string() },
            _ => return Ok(None),
        };
        Ok(Some(event))
    }
}
