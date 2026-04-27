import type {
  AgentDecision,
  AgentStage,
  RiskLevel,
  ServiceStatus,
} from "./types"

export type StageCardState = "idle" | "queued" | "complete" | "skipped"

export const stageSequence: AgentStage[] = [
  "junior",
  "senior",
  "architect",
  "tech_leader",
]

export const stageMeta: Record<
  AgentStage,
  { label: string; eyebrow: string; description: string }
> = {
  junior: {
    label: "Junior",
    eyebrow: "Hypothesis",
    description: "Initial framing, confidence, risk, and open unknowns.",
  },
  senior: {
    label: "Senior",
    eyebrow: "Refinement",
    description: "Validates the path, rejects weak parts, and escalates when needed.",
  },
  architect: {
    label: "Architect",
    eyebrow: "Structure",
    description: "Appears only when the Senior stage escalates for structural review.",
  },
  tech_leader: {
    label: "Tech-Leader",
    eyebrow: "Decision",
    description: "Issues the final decision, ADR title, and operational action items.",
  },
}

export const decisionMeta: Record<
  AgentDecision,
  { label: string; badgeClassName: string; textClassName: string }
> = {
  approve: {
    label: "Approve",
    badgeClassName:
      "border-emerald-500/30 bg-emerald-500/12 text-emerald-300 shadow-[0_0_28px_rgba(34,197,94,0.08)]",
    textClassName: "text-approve",
  },
  reject: {
    label: "Reject",
    badgeClassName:
      "border-red-500/30 bg-red-500/12 text-red-300 shadow-[0_0_28px_rgba(239,68,68,0.08)]",
    textClassName: "text-reject",
  },
  defer: {
    label: "Defer",
    badgeClassName:
      "border-amber-500/30 bg-amber-500/12 text-amber-300 shadow-[0_0_28px_rgba(245,158,11,0.08)]",
    textClassName: "text-defer",
  },
  escalate: {
    label: "Escalate",
    badgeClassName:
      "border-violet-500/30 bg-violet-500/12 text-violet-300 shadow-[0_0_28px_rgba(139,92,246,0.08)]",
    textClassName: "text-escalate",
  },
}

export const riskMeta: Record<
  RiskLevel,
  { label: string; badgeClassName: string; dotClassName: string }
> = {
  low: {
    label: "Low risk",
    badgeClassName: "border-emerald-500/20 bg-emerald-500/10 text-emerald-300",
    dotClassName: "bg-emerald-400",
  },
  medium: {
    label: "Medium risk",
    badgeClassName: "border-amber-500/20 bg-amber-500/10 text-amber-300",
    dotClassName: "bg-amber-400",
  },
  high: {
    label: "High risk",
    badgeClassName: "border-red-500/20 bg-red-500/10 text-red-300",
    dotClassName: "bg-red-400",
  },
}

export const serviceMeta: Record<
  ServiceStatus,
  { label: string; badgeClassName: string; dotClassName: string }
> = {
  up: {
    label: "UP",
    badgeClassName: "border-emerald-500/20 bg-emerald-500/10 text-emerald-300",
    dotClassName: "bg-emerald-400",
  },
  degraded: {
    label: "DEGRADED",
    badgeClassName: "border-amber-500/20 bg-amber-500/10 text-amber-300",
    dotClassName: "bg-amber-400",
  },
  down: {
    label: "DOWN",
    badgeClassName: "border-red-500/20 bg-red-500/10 text-red-300",
    dotClassName: "bg-red-400",
  },
  stub: {
    label: "STUB",
    badgeClassName: "border-violet-500/20 bg-violet-500/10 text-violet-300",
    dotClassName: "bg-violet-400",
  },
}

export function formatConfidence(value: number) {
  return `${value.toFixed(2)}`
}

export function confidencePercent(value: number) {
  return Math.max(0, Math.min(100, Math.round(value * 100)))
}

export function stageStateLabel(state: StageCardState) {
  switch (state) {
    case "queued":
      return "Awaiting result"
    case "complete":
      return "Complete"
    case "skipped":
      return "Skipped"
    default:
      return "Idle"
  }
}

export function serviceGroupLabel(group: string) {
  switch (group) {
    case "ui":
      return "Interface"
    case "control-plane":
      return "Control plane"
    case "pipeline":
      return "Pipeline"
    case "security":
      return "Security"
    case "knowledge":
      return "Knowledge"
    default:
      return "Service"
  }
}
