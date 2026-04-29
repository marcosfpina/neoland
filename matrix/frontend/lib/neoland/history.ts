import type { AgentDecision } from "./types"

export type HistoricalDecision = AgentDecision | "unknown"

export interface HistoricalPipelineRun {
  run_id: string
  session_id: string
  task_preview: string
  decision: HistoricalDecision
  source_decision: string
  adr_title?: string
  junior_confidence?: number
  total_latency_ms: number
  score: number
  timestamp: number
}

export function normalizeHistoricalDecision(decision: string): HistoricalDecision {
  switch (decision) {
    case "approve":
    case "reject":
    case "defer":
    case "escalate":
      return decision
    case "accepted":
      return "approve"
    case "rejected":
      return "reject"
    case "deferred":
      return "defer"
    case "escalated":
      return "escalate"
    default:
      return "unknown"
  }
}

export function normalizeHistoricalRun(run: {
  run_id: string
  session_id: string
  task_preview: string
  decision: string
  adr_title?: string
  junior_confidence?: number
  total_latency_ms: number
  score: number
  timestamp: number
}): HistoricalPipelineRun {
  return {
    ...run,
    source_decision: run.decision,
    decision: normalizeHistoricalDecision(run.decision),
  }
}
