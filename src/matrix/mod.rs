pub mod client;

pub use client::{AgentRunMetrics, MatrixClient, PipelineAgents, PipelineMetricsPayload};

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::client::*;

    fn sample_payload() -> PipelineMetricsPayload {
        PipelineMetricsPayload {
            session_id: Uuid::nil(),
            task: "refactor the auth module".to_string(),
            agents: PipelineAgents {
                junior: AgentRunMetrics {
                    confidence: Some(0.82),
                    latency_ms: Some(312),
                    escalated: None,
                    decision: None,
                },
                senior: AgentRunMetrics {
                    confidence: None,
                    latency_ms: Some(891),
                    escalated: Some(false),
                    decision: None,
                },
                tech_leader: AgentRunMetrics {
                    confidence: None,
                    latency_ms: Some(445),
                    escalated: None,
                    decision: Some("accepted".to_string()),
                },
            },
            total_latency_ms: 1648,
            decision: "accepted".to_string(),
            adr_title: Some("Refactor auth module".to_string()),
        }
    }

    #[test]
    fn payload_serializes_correctly() {
        let p = sample_payload();
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["decision"], "accepted");
        assert_eq!(json["total_latency_ms"], 1648);
        assert_eq!(json["agents"]["junior"]["confidence"], 0.82);
        // escalated=None should not appear
        assert!(json["agents"]["junior"].get("escalated").is_none());
    }

    #[test]
    fn payload_omits_none_fields() {
        let p = sample_payload();
        let json = serde_json::to_string(&p).unwrap();
        // None fields with skip_serializing_if should be absent
        assert!(!json.contains(r#""escalated":null"#));
    }

    #[test]
    fn adr_title_present_when_some() {
        let p = sample_payload();
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["adr_title"], "Refactor auth module");
    }

    #[test]
    fn adr_title_absent_when_none() {
        let mut p = sample_payload();
        p.adr_title = None;
        let json = serde_json::to_value(&p).unwrap();
        assert!(json.get("adr_title").is_none());
    }

    #[test]
    fn matrix_client_constructs() {
        let c = MatrixClient::new("http://localhost:8002");
        // just verify it doesn't panic
        drop(c);
    }
}
