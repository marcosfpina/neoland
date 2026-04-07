//! Escalation policy: decides when to escalate and which stage to start from.

use crate::agents::client::AgentStage;

#[derive(Debug, Clone)]
pub struct EscalationPolicy {
    /// Below this confidence, Junior output is flagged but pipeline continues normally.
    pub junior_confidence_warn_threshold: f64,
    /// Hours a 'defer' decision is valid before re-escalating.
    pub defer_ttl_hours: u32,
}

impl Default for EscalationPolicy {
    fn default() -> Self {
        Self { junior_confidence_warn_threshold: 0.4, defer_ttl_hours: 24 }
    }
}

impl EscalationPolicy {
    /// Decide which stage to start from based on session history.
    pub fn start_from(&self, last_decision: Option<&str>) -> AgentStage {
        match last_decision {
            // If last task was deferred, re-enter at Senior (Junior already ran)
            Some(d) if d.contains("defer") => AgentStage::Senior,
            // All other cases start fresh at Junior
            _ => AgentStage::Junior,
        }
    }
}
