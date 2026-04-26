//! HTTP client for the DSPy FastAPI pipeline.
//!
//! Types here mirror the Pydantic schemas in
//! agents/neoland_agents/schemas/api.py. Any change to the Python schemas must
//! be reflected here.

use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

// ─── Request / Response types (mirrors Python schemas/api.py) ───────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStage {
    Junior,
    Senior,
    Architect,
    TechLeader,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentDecision {
    Approve,
    Reject,
    Defer,
    Escalate,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentTaskRequest {
    pub task_id: Uuid,
    pub session_id: Uuid,
    pub task: String,
    pub requester_role: String,
    pub rag_context: String,
    pub start_from: AgentStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorOutput {
    pub hypothesis: String,
    pub confidence: f64,
    pub risk_level: RiskLevel,
    pub unknowns: Vec<String>,
    pub innovation_vectors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorOutput {
    pub valid_parts: Vec<String>,
    pub rejected_parts: Vec<String>,
    pub risk_assessment: String,
    pub escalate_to_architect: bool,
    pub refined_hypothesis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectOutput {
    pub structural_soundness: bool,
    pub composability_score: f64,
    pub long_term_concerns: Vec<String>,
    pub recommended_structure: String,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechLeaderOutput {
    pub decision: AgentDecision,
    pub rationale: String,
    pub action_items: Vec<String>,
    pub adr_title: String,
    pub session_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub task_id: Uuid,
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub junior: JuniorOutput,
    pub senior: SeniorOutput,
    pub architect: Option<ArchitectOutput>,
    pub tech_leader: TechLeaderOutput,
    pub checkpoint_path: String,
}

// ─── HTTP Client ─────────────────────────────────────────────────────────────

pub struct AgentPipelineClient {
    base_url: String,
    client: Client,
}

impl AgentPipelineClient {
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { base_url: base_url.into(), client })
    }

    #[instrument(skip(self, request), fields(task_id = %request.task_id, session_id = %request.session_id, start_from = ?request.start_from))]
    pub async fn run_pipeline(&self, request: &AgentTaskRequest) -> Result<PipelineResult> {
        let resp = self
            .client
            .post(format!("{}/v1/pipeline/run", self.base_url))
            .json(request)
            .send()
            .await
            .context("Agent pipeline HTTP request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Pipeline returned {status}: {body}");
        }

        resp.json::<PipelineResult>()
            .await
            .context("Failed to deserialize pipeline result")
    }

    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool> {
        let resp = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .context("Health check request failed")?;
        Ok(resp.status().is_success())
    }
}
