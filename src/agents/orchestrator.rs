//! Control plane orchestrator: wires together session, RAG, escalation, and pipeline client.

use std::time::Duration;

use anyhow::Result;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::agents::client::{AgentPipelineClient, AgentTaskRequest, PipelineResult};
use crate::agents::escalation::EscalationPolicy;
use crate::agents::session::SessionManager;
use crate::config::AgentsConfig;

pub struct AgentOrchestrator {
    client: AgentPipelineClient,
    sessions: SessionManager,
    escalation: EscalationPolicy,
}

impl AgentOrchestrator {
    pub fn new(pool: PgPool, cfg: &AgentsConfig) -> Result<Self> {
        let client = AgentPipelineClient::new(
            &cfg.dspy_url,
            Duration::from_secs(cfg.pipeline_timeout_secs),
        )?;
        Ok(Self {
            client,
            sessions: SessionManager::new(pool),
            escalation: EscalationPolicy {
                junior_confidence_warn_threshold: cfg.junior_confidence_warn_threshold,
                defer_ttl_hours: cfg.tech_leader_defer_ttl_hours,
            },
        })
    }

    pub async fn execute_task(
        &self,
        task: &str,
        session_id: Uuid,
        requester_role: &str,
        rag_context: String,
    ) -> Result<PipelineResult> {
        // Load session state for escalation decision
        let session = self.sessions.get_or_create(session_id).await?;
        let last_decision = session
            .last_decision
            .as_ref()
            .and_then(|v| v["decision"].as_str().map(str::to_owned));

        let start_from = self.escalation.start_from(last_decision.as_deref());
        let task_id = Uuid::new_v4();

        let request = AgentTaskRequest {
            task_id,
            session_id,
            task: task.to_owned(),
            requester_role: requester_role.to_owned(),
            rag_context,
            start_from,
        };

        let result = self.client.run_pipeline(&request).await?;

        // Persist session update
        let decision_json = json!({
            "decision": format!("{:?}", result.tech_leader.decision).to_lowercase(),
            "adr_title": result.tech_leader.adr_title,
            "session_summary": result.tech_leader.session_summary,
        });
        self.sessions.update_after_pipeline(session_id, decision_json).await?;

        Ok(result)
    }

    pub async fn get_session(
        &self,
        session_id: Uuid,
    ) -> Result<crate::agents::session::SessionState> {
        self.sessions.get_or_create(session_id).await
    }

    pub async fn health_check(&self) -> Result<bool> {
        self.client.health_check().await
    }
}
