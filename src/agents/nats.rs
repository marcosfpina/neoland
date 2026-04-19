//! NATS publisher for neoland agent pipeline events.
//!
//! Publishes structured events to NATS via spectre-events after each pipeline
//! run. Connection is optional — if NATS is unavailable, events are dropped
//! with a warn log.
//!
//! ## Subjects
//!
//! | Subject                          | Trigger                              |
//! |---------------------------------|--------------------------------------|
//! | `neoland.task.completed.v1`     | Every pipeline completion            |
//! | `neoland.task.escalated.v1`     | When `start_from == Architect`       |
//! | `neoland.pipeline.output.v1`    | Full text output — Phantom scan (D)  |

use anyhow::Result;
use serde_json::json;
use spectre_events::{Event, EventBus, EventType, ServiceId};
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::NatsConfig;

const SERVICE_ID: &str = "neoland-control-plane";

/// Payload for `neoland.task.completed.v1`
#[derive(Debug)]
pub struct TaskCompletedPayload<'a> {
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub decision: &'a str,
    pub risk_level: &'a str,
    pub junior_confidence: f64,
    pub adr_title: &'a str,
}

/// Payload for `neoland.pipeline.output.v1` — texto completo para scan do
/// Phantom.
#[derive(Debug)]
pub struct PipelineOutputPayload<'a> {
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub decision: &'a str,
    // Junior
    pub hypothesis: &'a str,
    pub junior_unknowns: &'a [String],
    pub innovation_vectors: &'a [String],
    // Senior
    pub risk_assessment: &'a str,
    pub refined_hypothesis: &'a str,
    // Tech-Leader
    pub rationale: &'a str,
    pub action_items: &'a [String],
    pub adr_title: &'a str,
}

/// Payload for `neoland.task.escalated.v1`
#[derive(Debug)]
pub struct TaskEscalatedPayload<'a> {
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub reason: &'a str,
}

/// NATS publisher for neoland pipeline events.
///
/// Wraps `EventBus` from spectre-events. If NATS is disabled or unreachable,
/// all publish calls are no-ops (warn logged, never panic).
pub struct NatsPublisher {
    bus: EventBus,
}

impl NatsPublisher {
    /// Connect to NATS. Returns `Err` if connection fails — caller decides
    /// whether to abort.
    pub async fn connect(cfg: &NatsConfig) -> Result<Self> {
        let bus = EventBus::connect(&cfg.url)
            .await
            .map_err(|e| anyhow::anyhow!("NATS connect failed ({}): {}", cfg.url, e))?;
        info!(url = %cfg.url, "NATS connected — neoland publisher ready");
        Ok(Self { bus })
    }

    /// Publish `neoland.task.completed.v1`.
    pub async fn publish_task_completed(&self, p: &TaskCompletedPayload<'_>) {
        let event = Event::new(
            EventType::Custom("neoland.task.completed.v1".to_string()),
            ServiceId::new(SERVICE_ID),
            json!({
                "session_id": p.session_id,
                "task_id": p.task_id,
                "decision": p.decision,
                "risk_level": p.risk_level,
                "junior_confidence": p.junior_confidence,
                "adr_title": p.adr_title,
            }),
        );
        if let Err(e) = self.bus.publish(&event).await {
            warn!(error = %e, subject = "neoland.task.completed.v1", "NATS publish failed");
        }
    }

    /// Publish `neoland.pipeline.output.v1` — texto completo para scan do
    /// Phantom.
    pub async fn publish_pipeline_output(&self, p: &PipelineOutputPayload<'_>) {
        let event = Event::new(
            EventType::Custom("neoland.pipeline.output.v1".to_string()),
            ServiceId::new(SERVICE_ID),
            json!({
                "session_id": p.session_id,
                "task_id": p.task_id,
                "decision": p.decision,
                "hypothesis": p.hypothesis,
                "junior_unknowns": p.junior_unknowns,
                "innovation_vectors": p.innovation_vectors,
                "risk_assessment": p.risk_assessment,
                "refined_hypothesis": p.refined_hypothesis,
                "rationale": p.rationale,
                "action_items": p.action_items,
                "adr_title": p.adr_title,
            }),
        );
        if let Err(e) = self.bus.publish(&event).await {
            warn!(error = %e, subject = "neoland.pipeline.output.v1", "NATS publish failed");
        }
    }

    /// Publish `neoland.task.escalated.v1`.
    pub async fn publish_task_escalated(&self, p: &TaskEscalatedPayload<'_>) {
        let event = Event::new(
            EventType::Custom("neoland.task.escalated.v1".to_string()),
            ServiceId::new(SERVICE_ID),
            json!({
                "session_id": p.session_id,
                "task_id": p.task_id,
                "reason": p.reason,
            }),
        );
        if let Err(e) = self.bus.publish(&event).await {
            warn!(error = %e, subject = "neoland.task.escalated.v1", "NATS publish failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn completed_payload() -> TaskCompletedPayload<'static> {
        TaskCompletedPayload {
            session_id: Uuid::nil(),
            task_id: Uuid::nil(),
            decision: "approve",
            risk_level: "low",
            junior_confidence: 0.85_f64,
            adr_title: "Test ADR",
        }
    }

    #[test]
    fn test_task_completed_subject() {
        let et = EventType::Custom("neoland.task.completed.v1".to_string());
        let event = Event::new(et, ServiceId::new(SERVICE_ID), json!({}));
        assert_eq!(event.subject(), "neoland.task.completed.v1");
    }

    #[test]
    fn test_task_escalated_subject() {
        let et = EventType::Custom("neoland.task.escalated.v1".to_string());
        let event = Event::new(et, ServiceId::new(SERVICE_ID), json!({}));
        assert_eq!(event.subject(), "neoland.task.escalated.v1");
    }

    #[test]
    fn test_task_completed_payload_keys() {
        let p = completed_payload();
        let payload = json!({
            "session_id": p.session_id,
            "task_id": p.task_id,
            "decision": p.decision,
            "risk_level": p.risk_level,
            "junior_confidence": p.junior_confidence,
            "adr_title": p.adr_title,
        });
        assert_eq!(payload["decision"], Value::String("approve".to_string()));
        assert_eq!(payload["risk_level"], Value::String("low".to_string()));
        assert!((payload["junior_confidence"].as_f64().unwrap() - 0.85).abs() < 1e-6);
        assert_eq!(payload["adr_title"], Value::String("Test ADR".to_string()));
    }

    #[test]
    fn test_task_escalated_payload_keys() {
        let session_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let payload = json!({
            "session_id": session_id,
            "task_id": task_id,
            "reason": "high_risk",
        });
        assert_eq!(payload["reason"], Value::String("high_risk".to_string()));
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let p = completed_payload();
        let event = Event::new(
            EventType::Custom("neoland.task.completed.v1".to_string()),
            ServiceId::new(SERVICE_ID),
            json!({
                "session_id": p.session_id,
                "task_id": p.task_id,
                "decision": p.decision,
                "risk_level": p.risk_level,
                "junior_confidence": p.junior_confidence,
                "adr_title": p.adr_title,
            }),
        );
        let json_str = event.to_json().unwrap();
        let deserialized = Event::from_json(&json_str).unwrap();
        assert_eq!(deserialized.event_id, event.event_id);
        assert_eq!(deserialized.subject(), "neoland.task.completed.v1");
        assert_eq!(deserialized.payload["decision"], "approve");
    }

    #[test]
    fn test_nats_config_defaults() {
        let cfg = crate::config::NatsConfig::default();
        assert_eq!(cfg.url, "nats://localhost:4222");
        assert!(!cfg.enabled);
    }

    #[tokio::test]
    #[ignore] // Requires NATS server running at nats://localhost:4222
    async fn test_connect_and_publish() {
        let cfg =
            crate::config::NatsConfig { url: "nats://localhost:4222".to_string(), enabled: true };
        let publisher = NatsPublisher::connect(&cfg).await.unwrap();
        publisher
            .publish_task_completed(&TaskCompletedPayload {
                session_id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                decision: "approve",
                risk_level: "low",
                junior_confidence: 0.9,
                adr_title: "Integration test ADR",
            })
            .await;
    }
}
