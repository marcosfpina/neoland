export type AgentStage = "junior" | "senior" | "architect" | "tech_leader"
export type RiskLevel = "low" | "medium" | "high"
export type AgentDecision = "approve" | "reject" | "defer" | "escalate"
export type HealthStatus = "healthy" | "degraded" | "unhealthy"
export type ServiceStatus = "up" | "degraded" | "down" | "stub"
export type StreamStatus = "idle" | "connecting" | "live" | "degraded"

export interface JuniorOutput {
  hypothesis: string
  confidence: number
  risk_level: RiskLevel
  unknowns: string[]
  innovation_vectors: string[]
}

export interface SeniorOutput {
  valid_parts: string[]
  rejected_parts: string[]
  risk_assessment: string
  escalate_to_architect: boolean
  refined_hypothesis: string
}

export interface ArchitectOutput {
  structural_soundness: boolean
  composability_score: number
  long_term_concerns: string[]
  recommended_structure: string
  blockers: string[]
}

export interface TechLeaderOutput {
  decision: AgentDecision
  rationale: string
  action_items: string[]
  adr_title: string
  session_summary: string
}

export interface PipelineResult {
  task_id: string
  session_id: string
  timestamp: string
  junior: JuniorOutput
  senior: SeniorOutput
  architect: ArchitectOutput | null
  tech_leader: TechLeaderOutput
  checkpoint_path: string
}

export type AgentStreamEvent =
  | {
      type: "pipeline_started"
      session_id: string
      task_preview: string
    }
  | {
      type: "stage_started"
      session_id: string
      stage: AgentStage
    }
  | {
      type: "stage_done"
      session_id: string
      stage: AgentStage
      confidence?: number | null
      risk_level?: number | null
      latency_ms: number
    }
  | {
      type: "stage_skipped"
      session_id: string
      stage: AgentStage
    }
  | {
      type: "tool_call_started"
      session_id: string
      tool: string
      args_summary: string
    }
  | {
      type: "tool_call_done"
      session_id: string
      tool: string
      duration_ms: number
    }
  | {
      type: "tool_call_failed"
      session_id: string
      tool: string
      error: string
    }
  | {
      type: "adr_checkpoint"
      session_id: string
      adr_id: string
      status: string
      title: string
    }
  | {
      type: "pipeline_done"
      session_id: string
      latency_ms: number
    }
  | {
      type: "pipeline_error"
      session_id: string
      error: string
    }
  | {
      type: "stage_output"
      session_id: string
      stage: AgentStage
      content: string
    }
  | {
      type: "steering_received"
      session_id: string
      message: string
    }

export interface SessionDecisionSnapshot {
  decision?: AgentDecision
  adr_title?: string
  session_summary?: string
}

export interface SessionState {
  session_id: string
  task_count: number
  last_activity: string
  last_decision: SessionDecisionSnapshot | null
  active: boolean
}

export interface ComponentHealth {
  name: string
  status: HealthStatus
  message: string
  response_time_ms?: number | null
}

export interface HealthResponse {
  status: HealthStatus
  version: string
  uptime_seconds?: number | null
  components: ComponentHealth[]
}

export interface ReadinessResponse {
  ready: boolean
  components: ComponentHealth[]
}

export interface LivenessResponse {
  alive: boolean
}

export interface PipelineHealthResponse {
  status: "ok" | "degraded" | "error"
  pipeline: "up" | "down" | "disabled"
  error?: string
}

export interface AdrDocument {
  slug: string
  adr_id: string
  title: string
  status: string
  context: string
  decision: string
  action_items: string[]
  session_summary: string
  timestamp: string
  full_pipeline: {
    junior: JuniorOutput
    senior: SeniorOutput
    architect: ArchitectOutput | null
    tech_leader: TechLeaderOutput
  }
  source_path: string
}

export interface PipelineRunInput {
  task: string
  session_id?: string
}

export interface ServiceNode {
  id: string
  name: string
  endpoint: string
  status: ServiceStatus
  detail: string
  latencyMs?: number
  group: "ui" | "control-plane" | "pipeline" | "security" | "knowledge"
}

export interface DashboardOverview {
  controlPlaneStatus: ServiceStatus
  adrCount: number
  approveRate: number
  liveServices: number
  recentAdrs: AdrDocument[]
  services: ServiceNode[]
}

export interface AgentAnalytics {
  label: string
  stage: AgentStage
  totalRuns: number
  primaryMetricLabel: string
  primaryMetricValue: string
  secondaryMetricLabel: string
  secondaryMetricValue: string
}

export interface JuniorTimelinePoint {
  adr_id: string
  title: string
  timestamp: string
  confidence: number
}

export interface SettingsSnapshot {
  controlPlaneUrl: string
  dspyUrl: string
  checkpointDir: string
  phantomUrl?: string
  neutronUrl?: string
  cerebroUrl?: string
  usingDevelopmentApiKey: boolean
}
