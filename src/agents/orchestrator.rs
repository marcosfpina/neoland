//! Control plane orchestrator: wires together session, RAG, escalation, and
//! pipeline client.

use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    agents::{
        client::{AgentPipelineClient, AgentStage, AgentTaskRequest, PipelineResult},
        escalation::EscalationPolicy,
        nats::{NatsPublisher, PipelineOutputPayload, TaskCompletedPayload, TaskEscalatedPayload},
        session::SessionManager,
    },
    config::AgentsConfig,
    metrics::utils as metrics,
};

pub struct AgentOrchestrator {
    client: AgentPipelineClient,
    sessions: SessionManager,
    escalation: EscalationPolicy,
    nats: Option<NatsPublisher>,
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
            nats: None,
        })
    }

    /// Attach a NATS publisher. Called after async NATS connection is
    /// established.
    pub fn with_nats(mut self, publisher: NatsPublisher) -> Self {
        self.nats = Some(publisher);
        self
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

        // Publish escalation event if architect tier is triggered
        if let (Some(nats), AgentStage::Architect) = (&self.nats, &start_from) {
            nats.publish_task_escalated(&TaskEscalatedPayload {
                session_id,
                task_id,
                reason: "senior_escalation",
            })
            .await;
            metrics::record_escalation("high"); // architect escalation implies
                                                // high risk
        }

        let request = AgentTaskRequest {
            task_id,
            session_id,
            task: task.to_owned(),
            requester_role: requester_role.to_owned(),
            rag_context,
            start_from,
        };

        let t0 = Instant::now();
        let result = self.client.run_pipeline(&request).await;
        let duration_secs = t0.elapsed().as_secs_f64();

        // Record per-agent call outcomes
        metrics::record_agent_call("junior", result.is_ok());
        metrics::record_agent_call("senior", result.is_ok());
        metrics::record_agent_call("tech_leader", result.is_ok());

        let result = result?;

        // Persist session update
        let decision_str = format!("{:?}", result.tech_leader.decision).to_lowercase();
        let decision_json = json!({
            "decision": decision_str,
            "adr_title": result.tech_leader.adr_title,
            "session_summary": result.tech_leader.session_summary,
        });
        self.sessions.update_after_pipeline(session_id, decision_json).await?;

        // Record pipeline completion metrics
        metrics::record_pipeline_completed(&decision_str, duration_secs, result.junior.confidence);
        metrics::record_escalation(&result.senior.risk_assessment);

        // Publish completion + full output events
        if let Some(nats) = &self.nats {
            let risk = result.senior.risk_assessment.clone();
            nats.publish_task_completed(&TaskCompletedPayload {
                session_id,
                task_id,
                decision: &decision_str,
                risk_level: &risk,
                junior_confidence: result.junior.confidence,
                adr_title: &result.tech_leader.adr_title,
            })
            .await;
            metrics::record_nats_publish("neoland.task.completed.v1", true);

            // Fase D — texto completo para scan do Phantom
            nats.publish_pipeline_output(&PipelineOutputPayload {
                session_id,
                task_id,
                decision: &decision_str,
                hypothesis: &result.junior.hypothesis,
                junior_unknowns: &result.junior.unknowns,
                innovation_vectors: &result.junior.innovation_vectors,
                risk_assessment: &result.senior.risk_assessment,
                refined_hypothesis: &result.senior.refined_hypothesis,
                rationale: &result.tech_leader.rationale,
                action_items: &result.tech_leader.action_items,
                adr_title: &result.tech_leader.adr_title,
            })
            .await;
            metrics::record_nats_publish("neoland.pipeline.output.v1", true);
        }

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
