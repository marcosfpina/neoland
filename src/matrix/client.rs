use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct AgentRunMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineMetricsPayload {
    pub session_id: Uuid,
    pub task: String,
    pub agents: PipelineAgents,
    pub total_latency_ms: u64,
    pub decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adr_title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineAgents {
    pub junior: AgentRunMetrics,
    pub senior: AgentRunMetrics,
    pub tech_leader: AgentRunMetrics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineRunSummary {
    pub run_id: String,
    pub session_id: String,
    pub score: f64,
    pub decision: String,
}

pub struct MatrixClient {
    http: reqwest::Client,
    base_url: String,
}

impl MatrixClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client");
        Self { http, base_url: base_url.into().trim_end_matches('/').to_owned() }
    }

    pub async fn post_pipeline_metrics(
        &self,
        payload: &PipelineMetricsPayload,
    ) -> Result<PipelineRunSummary> {
        let url = format!("{}/pipeline/metrics", self.base_url);
        let resp = self.http.post(&url).json(payload).send().await?;
        let summary: PipelineRunSummary = resp.error_for_status()?.json().await?;
        Ok(summary)
    }

    pub async fn is_reachable(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        self.http.get(&url).send().await.is_ok()
    }
}
