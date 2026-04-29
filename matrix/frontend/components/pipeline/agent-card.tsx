import { BadgeCheck, Blocks, GitBranch, Lightbulb } from "lucide-react"

import { ConfidenceBar } from "@/components/shared/confidence-bar"
import { DecisionBadge } from "@/components/shared/decision-badge"
import { RiskBadge } from "@/components/shared/risk-badge"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  stageMeta,
  stageStateLabel,
  type StageCardState,
} from "@/lib/neoland/presentation"
import type {
  AgentStage,
  ArchitectOutput,
  JuniorOutput,
  SeniorOutput,
  TechLeaderOutput,
} from "@/lib/neoland/types"
import { cn } from "@/lib/utils"

function stageIcon(stage: AgentStage) {
  switch (stage) {
    case "junior":
      return Lightbulb
    case "senior":
      return GitBranch
    case "architect":
      return Blocks
    case "tech_leader":
      return BadgeCheck
  }
}

function stateClassName(state: StageCardState) {
  switch (state) {
    case "complete":
      return "border-primary/20 bg-primary/10 text-primary"
    case "running":
      return "border-primary/20 bg-primary/10 text-primary"
    case "queued":
      return "border-amber-500/20 bg-amber-500/10 text-amber-300"
    case "skipped":
      return "border-border/70 bg-card/70 text-muted-foreground"
    default:
      return "border-border/70 bg-card/70 text-muted-foreground"
  }
}

function renderJunior(data: JuniorOutput) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Hypothesis</div>
        <p className="text-sm leading-6 text-foreground">{data.hypothesis}</p>
      </div>
      <ConfidenceBar confidence={data.confidence} />
      <div className="flex flex-wrap items-center gap-2">
        <RiskBadge risk={data.risk_level} />
        <Badge variant="outline" className="rounded-full border-border/70 px-3 py-1 text-xs">
          {data.unknowns.length} unknowns
        </Badge>
        <Badge variant="outline" className="rounded-full border-border/70 px-3 py-1 text-xs">
          {data.innovation_vectors.length} vectors
        </Badge>
      </div>
      {data.unknowns.length ? (
        <div className="space-y-2">
          <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
            Open unknowns
          </div>
          <ul className="space-y-2 text-sm text-muted-foreground">
            {data.unknowns.slice(0, 3).map((item) => (
              <li key={item} className="flex items-start gap-2">
                <span className="mt-2 inline-flex size-1.5 rounded-full bg-amber-400" />
                <span>{item}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  )
}

function renderSenior(data: SeniorOutput) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
          Refined hypothesis
        </div>
        <p className="text-sm leading-6 text-foreground">{data.refined_hypothesis}</p>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        <div className="rounded-2xl border border-border/70 bg-background/40 p-3">
          <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Valid parts</div>
          <div className="mt-2 text-2xl font-semibold text-foreground">{data.valid_parts.length}</div>
        </div>
        <div className="rounded-2xl border border-border/70 bg-background/40 p-3">
          <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
            Rejected parts
          </div>
          <div className="mt-2 text-2xl font-semibold text-foreground">{data.rejected_parts.length}</div>
        </div>
      </div>
      <div className="rounded-2xl border border-border/70 bg-background/40 p-3 text-sm leading-6 text-muted-foreground">
        {data.risk_assessment}
      </div>
      <Badge
        variant="outline"
        className={cn(
          "rounded-full px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em]",
          data.escalate_to_architect
            ? "border-violet-500/20 bg-violet-500/10 text-violet-300"
            : "border-emerald-500/20 bg-emerald-500/10 text-emerald-300",
        )}
      >
        {data.escalate_to_architect ? "Escalates to architect" : "Keeps flow local"}
      </Badge>
    </div>
  )
}

function renderArchitect(data: ArchitectOutput) {
  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-2">
        <div className="rounded-2xl border border-border/70 bg-background/40 p-3">
          <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Soundness</div>
          <div className="mt-2 text-lg font-semibold text-foreground">
            {data.structural_soundness ? "Sound" : "At risk"}
          </div>
        </div>
        <div className="rounded-2xl border border-border/70 bg-background/40 p-3">
          <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
            Composability
          </div>
          <div className="mt-2 text-lg font-semibold text-foreground">
            {data.composability_score.toFixed(2)}
          </div>
        </div>
      </div>
      <div className="space-y-2">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
          Recommended structure
        </div>
        <p className="text-sm leading-6 text-foreground">{data.recommended_structure}</p>
      </div>
      {data.long_term_concerns.length ? (
        <div className="space-y-2">
          <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
            Long-term concerns
          </div>
          <ul className="space-y-2 text-sm text-muted-foreground">
            {data.long_term_concerns.slice(0, 3).map((item) => (
              <li key={item} className="flex items-start gap-2">
                <span className="mt-2 inline-flex size-1.5 rounded-full bg-violet-400" />
                <span>{item}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  )
}

function renderTechLeader(data: TechLeaderOutput) {
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <DecisionBadge decision={data.decision} className="text-xs" />
        <Badge variant="outline" className="rounded-full border-border/70 px-3 py-1 text-xs">
          {data.action_items.length} action items
        </Badge>
      </div>
      <div className="space-y-2">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Rationale</div>
        <p className="text-sm leading-6 text-foreground">{data.rationale}</p>
      </div>
      <div className="rounded-2xl border border-border/70 bg-background/40 p-3">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">ADR title</div>
        <p className="mt-2 text-sm leading-6 text-foreground">{data.adr_title}</p>
      </div>
    </div>
  )
}

export function AgentCard({
  stage,
  state,
  junior,
  senior,
  architect,
  techLeader,
  liveContent,
  compact = false,
}: {
  stage: AgentStage
  state: StageCardState
  junior?: JuniorOutput
  senior?: SeniorOutput
  architect?: ArchitectOutput | null
  techLeader?: TechLeaderOutput
  liveContent?: string
  compact?: boolean
}) {
  const Icon = stageIcon(stage)
  const meta = stageMeta[stage]

  let body = (
    <div className="space-y-2 rounded-2xl border border-dashed border-border/70 bg-background/30 p-4 text-sm leading-6 text-muted-foreground">
      {state === "running"
        ? liveContent || "This stage is running and streaming live control-plane updates."
        : state === "queued"
          ? "This stage is queued behind the active pipeline relay."
        : state === "skipped"
          ? "This stage was intentionally skipped by the backend decision flow."
          : "No execution data is loaded yet."}
    </div>
  )

  if (state === "complete") {
    if (stage === "junior" && junior) {
      body = renderJunior(junior)
    } else if (stage === "senior" && senior) {
      body = renderSenior(senior)
    } else if (stage === "architect" && architect) {
      body = renderArchitect(architect)
    } else if (stage === "tech_leader" && techLeader) {
      body = renderTechLeader(techLeader)
    }
  }

  return (
    <Card className="h-full overflow-hidden border-border/70 bg-card/85 metal-panel">
      <CardHeader className="border-b border-border/70 pb-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="rounded-2xl border border-border/70 bg-background/50 p-3">
              <Icon className="size-5 text-primary" />
            </div>
            <div className="space-y-1">
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                {meta.eyebrow}
              </div>
              <CardTitle className="text-xl tracking-tight text-foreground">{meta.label}</CardTitle>
            </div>
          </div>
          <Badge
            variant="outline"
            className={cn(
              "rounded-full px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em]",
              stateClassName(state),
            )}
          >
            {stageStateLabel(state)}
          </Badge>
        </div>
        {!compact ? <p className="text-sm leading-6 text-muted-foreground">{meta.description}</p> : null}
      </CardHeader>
      <CardContent className="space-y-4 pt-6">{body}</CardContent>
    </Card>
  )
}
