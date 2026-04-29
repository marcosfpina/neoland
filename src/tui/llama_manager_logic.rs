use std::path::Path;

use tokio::fs;

pub async fn scan_models() -> Vec<String> {
    let mut models = Vec::new();
    let models_dir = Path::new("/var/lib/ml-models/llamacpp/models");

    if !models_dir.exists() || !models_dir.is_dir() {
        return vec!["Error: Directory not found or inaccessible".to_string()];
    }

    if let Ok(mut entries) = fs::read_dir(models_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "gguf" || ext == "bin" {
                        if let Some(name) = path.file_name() {
                            if let Some(name_str) = name.to_str() {
                                models.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    models.sort();
    if models.is_empty() {
        models.push("No models found in directory".to_string());
    }
    models
}

use std::{process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex,
};

pub async fn run_llama_server(
    model_name: String,
    flags: super::llama_manager::LlamaFlags,
    logs: Arc<Mutex<Vec<String>>>,
    child_handle: Arc<Mutex<Option<tokio::process::Child>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Stop system service
    {
        let mut logs_lock = logs.lock().await;
        logs_lock.push("=> Stopping llamacpp-swap.service...".to_string());
    }

    let stop_status = Command::new("sudo")
        .args(["systemctl", "stop", "llamacpp-swap.service"])
        .status()
        .await?;

    if !stop_status.success() {
        let mut logs_lock = logs.lock().await;
        logs_lock.push(format!("=> Warning: Failed to stop service (exit code {})", stop_status));
    }

    let model_path = format!("/var/lib/ml-models/llamacpp/models/{}", model_name);

    // 2. Build the command
    let mut cmd = Command::new("llama-server");
    cmd.arg("-m").arg(&model_path);
    cmd.arg("-c").arg(&flags.ctx_size);
    cmd.arg("-ngl").arg(&flags.gpu_layers);
    cmd.arg("-t").arg(&flags.threads);
    cmd.arg("--port").arg(&flags.port);

    if flags.flash_attn {
        cmd.arg("-fa");
    }
    if flags.no_kv_offload {
        cmd.arg("-nkvo");
    }
    if flags.cont_batching {
        cmd.arg("-cb");
    }
    if flags.embeddings {
        cmd.arg("--embedding");
    }
    if flags.metrics {
        cmd.arg("--metrics");
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    {
        let mut logs_lock = logs.lock().await;
        logs_lock.push(format!("=> Starting: {:?}", cmd));
    }

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    *child_handle.lock().await = Some(child);

    let logs_stdout = logs.clone();
    let logs_stderr = logs.clone();

    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut logs_lock = logs_stdout.lock().await;
            logs_lock.push(format!("[stdout] {}", line));
            if logs_lock.len() > 200 {
                logs_lock.remove(0);
            }
        }
    });

    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut logs_lock = logs_stderr.lock().await;
            logs_lock.push(format!("[stderr] {}", line));
            if logs_lock.len() > 200 {
                logs_lock.remove(0);
            }
        }
    });

    Ok(())
}

pub async fn stop_llama_server(
    child_handle: Arc<Mutex<Option<tokio::process::Child>>>,
    logs: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut child_opt = child_handle.lock().await;

    if let Some(mut child) = child_opt.take() {
        {
            let mut logs_lock = logs.lock().await;
            logs_lock.push("=> Killing llama-server process...".to_string());
        }
        let _ = child.kill().await;
    }

    {
        let mut logs_lock = logs.lock().await;
        logs_lock.push("=> Restarting llamacpp-swap.service...".to_string());
    }

    let start_status = Command::new("sudo")
        .args(["systemctl", "start", "llamacpp-swap.service"])
        .status()
        .await?;

    if !start_status.success() {
        let mut logs_lock = logs.lock().await;
        logs_lock
            .push(format!("=> Warning: Failed to restart service (exit code {})", start_status));
    }

    Ok(())
}
