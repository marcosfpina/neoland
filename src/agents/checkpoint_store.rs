//! Read checkpoint ADR files written by the Python pipeline.

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

/// Read a checkpoint JSON file by path.
pub async fn read_checkpoint(path: &str) -> Result<Value> {
    let content = tokio::fs::read_to_string(Path::new(path))
        .await
        .context("Failed to read checkpoint file")?;
    serde_json::from_str(&content).context("Failed to parse checkpoint JSON")
}
