use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    PipelineStarted {
        session_id: Uuid,
        task_preview: String,
    },
    StageStarted {
        session_id: Uuid,
        stage: &'static str,
    },
    StageDone {
        session_id: Uuid,
        stage: &'static str,
        confidence: Option<f32>,
        risk_level: Option<u8>,
        latency_ms: u64,
    },
    StageSkipped {
        session_id: Uuid,
        stage: &'static str,
    },
    ToolCallStarted {
        session_id: Uuid,
        tool: String,
        args_summary: String,
    },
    ToolCallDone {
        session_id: Uuid,
        tool: String,
        duration_ms: u64,
    },
    ToolCallFailed {
        session_id: Uuid,
        tool: String,
        error: String,
    },
    AdrCheckpoint {
        session_id: Uuid,
        adr_id: String,
        status: String,
        title: String,
    },
    PipelineDone {
        session_id: Uuid,
        latency_ms: u64,
    },
    PipelineError {
        session_id: Uuid,
        error: String,
    },
    /// Actual agent output text — reveals the reasoning for each stage.
    StageOutput {
        session_id: Uuid,
        stage: &'static str,
        content: String,
    },
}

impl AgentEvent {
    pub fn session_id(&self) -> Uuid {
        match self {
            Self::PipelineStarted { session_id, .. }
            | Self::StageStarted { session_id, .. }
            | Self::StageDone { session_id, .. }
            | Self::StageSkipped { session_id, .. }
            | Self::ToolCallStarted { session_id, .. }
            | Self::ToolCallDone { session_id, .. }
            | Self::ToolCallFailed { session_id, .. }
            | Self::AdrCheckpoint { session_id, .. }
            | Self::PipelineDone { session_id, .. }
            | Self::PipelineError { session_id, .. }
            | Self::StageOutput { session_id, .. } => *session_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_started_serializes_with_type_tag() {
        let event = AgentEvent::PipelineStarted {
            session_id: Uuid::nil(),
            task_preview: "analyze auth".to_string(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "pipeline_started");
        assert_eq!(json["task_preview"], "analyze auth");
    }

    #[test]
    fn test_stage_done_serializes_optional_fields() {
        let event = AgentEvent::StageDone {
            session_id: Uuid::nil(),
            stage: "junior",
            confidence: Some(0.82),
            risk_level: Some(1),
            latency_ms: 312,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "stage_done");
        assert_eq!(json["stage"], "junior");
        assert_eq!(json["latency_ms"], 312);
    }

    #[test]
    fn test_session_id_accessor_returns_correct_id() {
        let id = Uuid::new_v4();
        let event = AgentEvent::PipelineDone { session_id: id, latency_ms: 1000 };
        assert_eq!(event.session_id(), id);
    }

    #[test]
    fn test_pipeline_error_serializes() {
        let event =
            AgentEvent::PipelineError { session_id: Uuid::nil(), error: "timeout".to_string() };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "pipeline_error");
        assert_eq!(json["error"], "timeout");
    }

    #[test]
    fn test_adr_checkpoint_serializes() {
        let event = AgentEvent::AdrCheckpoint {
            session_id: Uuid::nil(),
            adr_id: "ADR-001".to_string(),
            status: "accepted".to_string(),
            title: "Use SSE for events".to_string(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "adr_checkpoint");
        assert_eq!(json["status"], "accepted");
    }
}
