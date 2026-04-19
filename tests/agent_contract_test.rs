//! Agent contract tests — schema roundtrip Rust ↔ JSON.
//!
//! These tests do NOT require:
//!   - PostgreSQL (DATABASE_URL)
//!   - Python DSPy pipeline running
//!   - LLM API key
//!
//! They validate that the Rust types in src/agents/client.rs correctly
//! serialize/deserialize to/from the JSON format expected by the Python API.

use neoland::agents::client::{
    AgentDecision, AgentStage, AgentTaskRequest, JuniorOutput, PipelineResult, RiskLevel,
    TechLeaderOutput,
};
use uuid::Uuid;

// ── AgentStage
// ────────────────────────────────────────────────────────────────

#[test]
fn test_agent_stage_serializes_snake_case() {
    let cases = [
        (AgentStage::Junior, "\"junior\""),
        (AgentStage::Senior, "\"senior\""),
        (AgentStage::Architect, "\"architect\""),
        (AgentStage::TechLeader, "\"tech_leader\""),
    ];
    for (stage, expected) in cases {
        let json = serde_json::to_string(&stage).expect("serialize AgentStage");
        assert_eq!(json, expected, "AgentStage variant mismatch");
    }
}

#[test]
fn test_agent_stage_deserializes_snake_case() {
    let cases = [
        ("\"junior\"", AgentStage::Junior),
        ("\"senior\"", AgentStage::Senior),
        ("\"architect\"", AgentStage::Architect),
        ("\"tech_leader\"", AgentStage::TechLeader),
    ];
    for (input, expected) in cases {
        let stage: AgentStage = serde_json::from_str(input).expect("deserialize AgentStage");
        assert_eq!(stage, expected);
    }
}

// ── RiskLevel
// ─────────────────────────────────────────────────────────────────

#[test]
fn test_risk_level_roundtrip() {
    for level in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High] {
        let json = serde_json::to_string(&level).expect("serialize RiskLevel");
        let back: RiskLevel = serde_json::from_str(&json).expect("deserialize RiskLevel");
        assert_eq!(level, back);
    }
}

// ── AgentDecision
// ─────────────────────────────────────────────────────────────

#[test]
fn test_agent_decision_all_variants_roundtrip() {
    let variants = [
        AgentDecision::Approve,
        AgentDecision::Reject,
        AgentDecision::Defer,
        AgentDecision::Escalate,
    ];
    for decision in variants {
        let json = serde_json::to_string(&decision).expect("serialize AgentDecision");
        let back: AgentDecision = serde_json::from_str(&json).expect("deserialize AgentDecision");
        assert_eq!(decision, back);
    }
}

// ── AgentTaskRequest ─────────────────────────────────────────────────────────

#[test]
fn test_agent_task_request_serializes_all_fields() {
    let task_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let req = AgentTaskRequest {
        task_id,
        session_id,
        task: "Refactor auth middleware to use JWT RS256".to_string(),
        requester_role: "user".to_string(),
        rag_context: "Context about JWT".to_string(),
        start_from: AgentStage::Junior,
    };

    let json = serde_json::to_value(&req).expect("serialize AgentTaskRequest");

    assert_eq!(json["task_id"], task_id.to_string());
    assert_eq!(json["session_id"], session_id.to_string());
    assert_eq!(json["task"], "Refactor auth middleware to use JWT RS256");
    assert_eq!(json["requester_role"], "user");
    assert_eq!(json["rag_context"], "Context about JWT");
    assert_eq!(json["start_from"], "junior");
}

// ── PipelineResult deserialization ───────────────────────────────────────────

#[test]
fn test_pipeline_result_deserializes_from_json() {
    let task_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let json = serde_json::json!({
        "task_id": task_id.to_string(),
        "session_id": session_id.to_string(),
        "timestamp": "2026-04-07T12:00:00Z",
        "junior": {
            "hypothesis": "Use RS256 for better security",
            "confidence": 0.85,
            "risk_level": "medium",
            "unknowns": ["token rotation strategy"],
            "innovation_vectors": ["zero-trust auth"]
        },
        "senior": {
            "valid_parts": ["RS256 is correct"],
            "rejected_parts": [],
            "risk_assessment": "Low risk if keys are managed properly",
            "escalate_to_architect": false,
            "refined_hypothesis": "Use RS256 with vault key rotation"
        },
        "architect": null,
        "tech_leader": {
            "decision": "approve",
            "rationale": "Solid approach, well-understood",
            "action_items": ["implement key rotation"],
            "adr_title": "ADR-0001: JWT RS256 adoption",
            "session_summary": "Auth refactor approved"
        },
        "checkpoint_path": "/var/lib/neoland/checkpoints/adr/2026-04-07-jwt.json"
    });

    let result: PipelineResult = serde_json::from_value(json).expect("deserialize PipelineResult");

    assert_eq!(result.task_id, task_id);
    assert_eq!(result.session_id, session_id);
    assert!((result.junior.confidence - 0.85).abs() < f64::EPSILON);
    assert_eq!(result.junior.risk_level, RiskLevel::Medium);
    assert!(!result.senior.escalate_to_architect);
    assert!(result.architect.is_none());
    assert_eq!(result.tech_leader.decision, AgentDecision::Approve);
}

#[test]
fn test_pipeline_result_with_architect_deserializes() {
    let json = serde_json::json!({
        "task_id": Uuid::new_v4().to_string(),
        "session_id": Uuid::new_v4().to_string(),
        "timestamp": "2026-04-07T12:00:00Z",
        "junior": {
            "hypothesis": "Introduce event sourcing",
            "confidence": 0.72,
            "risk_level": "high",
            "unknowns": ["snapshot strategy", "replay performance"],
            "innovation_vectors": ["CQRS pattern"]
        },
        "senior": {
            "valid_parts": ["event sourcing for audit trail"],
            "rejected_parts": ["full replacement in one shot"],
            "risk_assessment": "High complexity risk",
            "escalate_to_architect": true,
            "refined_hypothesis": "Gradual event sourcing for critical aggregates"
        },
        "architect": {
            "structural_soundness": true,
            "composability_score": 0.78,
            "long_term_concerns": ["event schema evolution"],
            "recommended_structure": "Separate bounded context per domain",
            "blockers": []
        },
        "tech_leader": {
            "decision": "defer",
            "rationale": "Needs more RFC discussion",
            "action_items": ["RFC document", "PoC"],
            "adr_title": "ADR-0002: Event sourcing evaluation",
            "session_summary": "Deferred for RFC"
        },
        "checkpoint_path": "/var/lib/neoland/checkpoints/adr/2026-04-07-es.json"
    });

    let result: PipelineResult =
        serde_json::from_value(json).expect("deserialize PipelineResult with architect");

    assert!(result.architect.is_some());
    let arch = result.architect.unwrap();
    assert!(arch.structural_soundness);
    assert!((arch.composability_score - 0.78).abs() < 0.001);
    assert!(arch.blockers.is_empty());
    assert_eq!(result.tech_leader.decision, AgentDecision::Defer);
}

// ── JuniorOutput confidence bounds ───────────────────────────────────────────

#[test]
fn test_junior_output_confidence_precision() {
    let json = serde_json::json!({
        "hypothesis": "test",
        "confidence": 0.0,
        "risk_level": "low",
        "unknowns": [],
        "innovation_vectors": []
    });
    let out: JuniorOutput = serde_json::from_value(json).unwrap();
    assert_eq!(out.confidence, 0.0);

    let json = serde_json::json!({
        "hypothesis": "test",
        "confidence": 1.0,
        "risk_level": "high",
        "unknowns": [],
        "innovation_vectors": []
    });
    let out: JuniorOutput = serde_json::from_value(json).unwrap();
    assert_eq!(out.confidence, 1.0);
}

// ── TechLeaderOutput serialization (for REST response) ───────────────────────

#[test]
fn test_tech_leader_output_serializes() {
    let tl = TechLeaderOutput {
        decision: AgentDecision::Approve,
        rationale: "Well reasoned".to_string(),
        action_items: vec!["ship it".to_string()],
        adr_title: "ADR-0001: Test decision".to_string(),
        session_summary: "Approved on first pass".to_string(),
    };

    let json = serde_json::to_value(&tl).expect("serialize TechLeaderOutput");
    assert_eq!(json["decision"], "approve");
    assert_eq!(json["rationale"], "Well reasoned");
    assert_eq!(json["action_items"][0], "ship it");
}
