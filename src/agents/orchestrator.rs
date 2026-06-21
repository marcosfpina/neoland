//! Control plane orchestrator: wires together session, RAG, escalation, and
//! pipeline client.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use serde_json::json;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    agents::{
        client::{AgentPipelineClient, AgentStage, AgentTaskRequest, PipelineResult},
        escalation::EscalationPolicy,
        events::AgentEvent,
        nats::{NatsPublisher, PipelineOutputPayload, TaskCompletedPayload, TaskEscalatedPayload},
        session::SessionManager,
    },
    config::AgentsConfig,
    matrix::{AgentRunMetrics, MatrixClient, PipelineAgents, PipelineMetricsPayload},
    mcp::{
        server::{BreakpointResolution, NativeMcpServer},
        McpRegistry,
    },
    metrics::utils as metrics,
};

pub struct AgentOrchestrator {
    client: AgentPipelineClient,
    sessions: SessionManager,
    escalation: EscalationPolicy,
    // Arc so publish() can clone a handle into spawned tasks without blocking.
    nats: Option<Arc<NatsPublisher>>,
    event_bus: Option<tokio::sync::broadcast::Sender<AgentEvent>>,
    mcp: Option<Arc<McpRegistry>>,
    matrix: Option<Arc<MatrixClient>>,
    steering_channels:
        tokio::sync::Mutex<std::collections::HashMap<Uuid, tokio::sync::mpsc::Sender<String>>>,
    pub native_mcp: Option<Arc<NativeMcpServer>>,
    pending_breakpoints: tokio::sync::Mutex<
        std::collections::HashMap<
            Uuid,
            (String, tokio::sync::oneshot::Sender<BreakpointResolution>),
        >,
    >,
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
            event_bus: None,
            mcp: None,
            matrix: None,
            steering_channels: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            native_mcp: None,
            pending_breakpoints: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    /// Attach a NATS publisher. Called after async NATS connection is
    /// established.
    pub fn with_native_mcp(mut self, mcp: NativeMcpServer) -> Self {
        self.native_mcp = Some(Arc::new(mcp));
        self
    }

    pub fn with_nats(mut self, publisher: NatsPublisher) -> Self {
        self.nats = Some(Arc::new(publisher));
        self
    }

    /// Attach an SSE event bus. Called after the broadcast channel is created.
    pub fn with_event_bus(mut self, tx: tokio::sync::broadcast::Sender<AgentEvent>) -> Self {
        self.event_bus = Some(tx);
        self
    }

    /// Attach an MCP registry for tool-augmented pipeline execution.
    pub fn with_mcp(mut self, registry: Arc<McpRegistry>) -> Self {
        self.mcp = Some(registry);
        self
    }

    /// Attach a Matrix client for pipeline run tracking.
    pub fn with_matrix(mut self, client: Arc<MatrixClient>) -> Self {
        self.matrix = Some(client);
        self
    }

    pub fn publish(&self, event: AgentEvent) {
        // SSE broadcast — primary path for the TUI client.
        if let Some(tx) = &self.event_bus {
            let _ = tx.send(event.clone());
        }
        // NATS — fire-and-forget; never blocks the SSE path.
        // Consumers: Spectre AI reactor, OWASAKA scanner, Phantom, Grafana.
        if let Some(nats) = self.nats.clone() {
            tokio::spawn(async move {
                nats.publish_agent_event(&event).await;
            });
        }
    }

    #[instrument(skip(self, rag_context), fields(session_id = %session_id, requester_role))]
    pub async fn execute_task(
        &self,
        task: &str,
        session_id: Uuid,
        requester_role: &str,
        mut rag_context: String,
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

        // MCP: search knowledge base to enrich RAG context before pipeline
        if let Some(mcp) = &self.mcp {
            if mcp.has_tool("search_knowledge") {
                let preview = task.chars().take(40).collect::<String>();
                self.publish(AgentEvent::ToolCallStarted {
                    session_id,
                    tool: "search_knowledge".to_string(),
                    args_summary: preview,
                });
                let t0 = Instant::now();
                match mcp
                    .call("search_knowledge", serde_json::json!({ "query": task, "limit": 5 }))
                    .await
                {
                    Ok(r) => {
                        self.publish(AgentEvent::ToolCallDone {
                            session_id,
                            tool: "search_knowledge".to_string(),
                            duration_ms: t0.elapsed().as_millis() as u64,
                        });
                        if !r.text.is_empty() {
                            rag_context = format!("{}\n\n{}", r.text, rag_context);
                        }
                    },
                    Err(e) => {
                        self.publish(AgentEvent::ToolCallFailed {
                            session_id,
                            tool: "search_knowledge".to_string(),
                            error: e.to_string(),
                        });
                    },
                }
            }
        }

        let mut current_task = task.to_owned();
        let mut request = AgentTaskRequest {
            task_id,
            session_id,
            task: current_task.clone(),
            requester_role: requester_role.to_owned(),
            rag_context,
            start_from,
        };

        self.publish(AgentEvent::PipelineStarted {
            session_id,
            task_preview: task.chars().take(120).collect(),
        });
        self.publish(AgentEvent::StageStarted { session_id, stage: "junior" });

        let (steer_tx, mut steer_rx) = tokio::sync::mpsc::channel::<String>(10);
        {
            let mut channels = self.steering_channels.lock().await;
            channels.insert(session_id, steer_tx);
        }

        let (result, duration_secs) = loop {
            let t0 = Instant::now();
            let req_clone = request.clone();

            tokio::select! {
                // 1. The Main Work (Pipeline progression)
                res = self.client.run_pipeline(&req_clone) => {
                    break (res, t0.elapsed().as_secs_f64());
                },

                // 2. The Agent's "Ear" (Human Intervention Channel for Live-Steering)
                Some(user_message) = steer_rx.recv() => {
                    self.publish(AgentEvent::SteeringReceived {
                        session_id,
                        message: user_message.clone(),
                    });

                    tracing::info!(session_id = %session_id, "Live steering message received: {}", user_message);

                    // Replan: Update context with the new steering directive.
                    // This interrupts the Python DSPy call and starts over from Junior with the new rule.
                    current_task = format!("{}\n\n[STEERING UPDATE]: {}", current_task, user_message);
                    request.task = current_task.clone();

                    self.publish(AgentEvent::StageOutput {
                        session_id,
                        stage: "junior",
                        content: format!("Received steering directive: {}. Replanning...", user_message),
                    });
                    self.publish(AgentEvent::StageStarted { session_id, stage: "junior" });
                }
            }
        };

        {
            let mut channels = self.steering_channels.lock().await;
            channels.remove(&session_id);
        }

        // Record per-agent call outcomes
        metrics::record_agent_call("junior", result.is_ok());
        metrics::record_agent_call("senior", result.is_ok());
        metrics::record_agent_call("tech_leader", result.is_ok());

        let result = match result {
            Ok(r) => r,
            Err(e) => {
                self.publish(AgentEvent::PipelineError { session_id, error: e.to_string() });
                return Err(e);
            },
        };

        let total_ms = (duration_secs * 1000.0) as u64;
        let junior_ms = result.stage_latencies_ms.get("junior").copied().unwrap_or(total_ms / 4);
        let senior_ms = result.stage_latencies_ms.get("senior").copied().unwrap_or(total_ms / 4);
        let architect_ms = result.stage_latencies_ms.get("architect").copied();
        let leader_ms =
            result.stage_latencies_ms.get("tech_leader").copied().unwrap_or(
                total_ms.saturating_sub(junior_ms + senior_ms + architect_ms.unwrap_or(0)),
            );

        self.publish(AgentEvent::StageDone {
            session_id,
            stage: "junior",
            confidence: Some(result.junior.confidence as f32),
            risk_level: None,
            latency_ms: junior_ms,
        });
        self.publish(AgentEvent::StageOutput {
            session_id,
            stage: "junior",
            content: format!(
                "{}\n\nUnknowns: {}",
                result.junior.hypothesis,
                result.junior.unknowns.join(", ")
            ),
        });
        self.publish(AgentEvent::StageStarted { session_id, stage: "senior" });
        self.publish(AgentEvent::StageDone {
            session_id,
            stage: "senior",
            confidence: None,
            risk_level: None,
            latency_ms: senior_ms,
        });
        self.publish(AgentEvent::StageOutput {
            session_id,
            stage: "senior",
            content: format!(
                "{}\n\nRisk: {}",
                result.senior.refined_hypothesis, result.senior.risk_assessment
            ),
        });
        if let Some(arch) = &result.architect {
            self.publish(AgentEvent::StageStarted { session_id, stage: "architect" });
            self.publish(AgentEvent::StageDone {
                session_id,
                stage: "architect",
                confidence: Some(arch.composability_score as f32),
                risk_level: None,
                latency_ms: architect_ms.unwrap_or(0),
            });
            self.publish(AgentEvent::StageOutput {
                session_id,
                stage: "architect",
                content: format!(
                    "{}\n\nBlockers: {}",
                    arch.recommended_structure,
                    arch.blockers.join(", ")
                ),
            });
        } else {
            self.publish(AgentEvent::StageSkipped { session_id, stage: "architect" });
        }
        self.publish(AgentEvent::StageStarted { session_id, stage: "tech_leader" });
        self.publish(AgentEvent::StageDone {
            session_id,
            stage: "tech_leader",
            confidence: None,
            risk_level: None,
            latency_ms: leader_ms,
        });
        self.publish(AgentEvent::StageOutput {
            session_id,
            stage: "tech_leader",
            content: result.tech_leader.rationale.clone(),
        });
        self.publish(AgentEvent::AdrCheckpoint {
            session_id,
            adr_id: format!("ADR-{}-{}", session_id, task_id),
            status: format!("{:?}", result.tech_leader.decision).to_lowercase(),
            title: result.tech_leader.adr_title.clone(),
        });
        self.publish(AgentEvent::PipelineDone { session_id, latency_ms: total_ms });

        // Persist session update
        let decision_str = format!("{:?}", result.tech_leader.decision).to_lowercase();

        // Matrix: post pipeline run metrics for dashboard tracking (fire-and-forget)
        if let Some(matrix) = &self.matrix {
            let escalated = result.senior.risk_assessment.to_lowercase().contains("high")
                || result.senior.risk_assessment.to_lowercase().contains("critical");
            let payload = PipelineMetricsPayload {
                session_id,
                task: task.to_owned(),
                agents: PipelineAgents {
                    junior: AgentRunMetrics {
                        confidence: Some(result.junior.confidence),
                        latency_ms: Some(junior_ms),
                        escalated: None,
                        decision: None,
                    },
                    senior: AgentRunMetrics {
                        confidence: None,
                        latency_ms: Some(senior_ms),
                        escalated: Some(escalated),
                        decision: None,
                    },
                    tech_leader: AgentRunMetrics {
                        confidence: None,
                        latency_ms: Some(leader_ms),
                        escalated: None,
                        decision: Some(decision_str.clone()),
                    },
                },
                total_latency_ms: total_ms,
                decision: decision_str.clone(),
                adr_title: Some(result.tech_leader.adr_title.clone()),
            };
            let matrix = Arc::clone(matrix);
            tokio::spawn(async move {
                if let Err(e) = matrix.post_pipeline_metrics(&payload).await {
                    tracing::warn!(error = %e, "Matrix metrics post failed");
                }
            });
        }
        let decision_json = json!({
            "decision": decision_str,
            "adr_title": result.tech_leader.adr_title,
            "session_summary": result.tech_leader.session_summary,
        });
        self.sessions.update_after_pipeline(session_id, decision_json).await?;

        // Record pipeline completion metrics
        metrics::record_pipeline_completed(&decision_str, duration_secs, result.junior.confidence);
        metrics::record_escalation(&result.senior.risk_assessment);

        // MCP: persist the ADR decision to the knowledge base
        if let Some(mcp) = &self.mcp {
            if mcp.has_tool("save_knowledge") {
                let entry = format!(
                    "ADR {session_id}-{task_id} — {decision_str} — {}",
                    result.tech_leader.rationale
                );
                self.publish(AgentEvent::ToolCallStarted {
                    session_id,
                    tool: "save_knowledge".to_string(),
                    args_summary: "adr checkpoint".to_string(),
                });
                let t0 = Instant::now();
                match mcp
                    .call(
                        "save_knowledge",
                        serde_json::json!({ "content": entry, "tags": ["adr", "neoland"] }),
                    )
                    .await
                {
                    Ok(_) => {
                        self.publish(AgentEvent::ToolCallDone {
                            session_id,
                            tool: "save_knowledge".to_string(),
                            duration_ms: t0.elapsed().as_millis() as u64,
                        });
                    },
                    Err(e) => {
                        self.publish(AgentEvent::ToolCallFailed {
                            session_id,
                            tool: "save_knowledge".to_string(),
                            error: e.to_string(),
                        });
                    },
                }
            }
        }

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

    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn steer_task(&self, session_id: Uuid, message: String) -> Result<()> {
        let tx = {
            let channels = self.steering_channels.lock().await;
            channels.get(&session_id).cloned()
        };

        if let Some(tx) = tx {
            tx.send(message)
                .await
                .map_err(|_| anyhow::anyhow!("Failed to send steering message"))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No active task found for session {}", session_id))
        }
    }

    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn get_session(
        &self,
        session_id: Uuid,
    ) -> Result<crate::agents::session::SessionState> {
        self.sessions.get_or_create(session_id).await
    }

    #[instrument(skip(self))]
    pub async fn list_sessions(
        &self,
        limit: i64,
    ) -> Result<Vec<crate::agents::session::SessionState>> {
        self.sessions.list_recent(limit).await
    }

    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool> {
        self.client.health_check().await
    }

    pub async fn register_breakpoint(
        &self,
        session_id: Uuid,
        tool_name: String,
        resolve_tx: tokio::sync::oneshot::Sender<BreakpointResolution>,
    ) {
        self.pending_breakpoints
            .lock()
            .await
            .insert(session_id, (tool_name, resolve_tx));
    }

    /// Returns `Ok(true)` if a breakpoint was found and resolved, `Ok(false)`
    /// if none was pending.
    pub async fn resolve_breakpoint(
        &self,
        session_id: Uuid,
        resolution: BreakpointResolution,
    ) -> Result<bool> {
        let mut lock = self.pending_breakpoints.lock().await;
        if let Some((tool_name, tx)) = lock.remove(&session_id) {
            let resolution_label = match &resolution {
                BreakpointResolution::Approve => "approve",
                BreakpointResolution::Reject => "reject",
                BreakpointResolution::Steer(_) => "steer",
            }
            .to_string();
            let _ = tx.send(resolution);
            self.publish(AgentEvent::BreakpointResolved {
                session_id,
                tool: tool_name,
                resolution: resolution_label,
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
