import Link from "next/link"
import {
  ArrowUpRight,
  CircleDashed,
  FileJson2,
  FolderTree,
  ScanSearch,
} from "lucide-react"

import { AgentCard } from "@/components/pipeline/agent-card"
import { DecisionBadge } from "@/components/shared/decision-badge"
import { StatusDot } from "@/components/shared/status-dot"
import { Card, CardContent } from "@/components/ui/card"
import {
  stageMeta,
  stageSequence,
  stageStateLabel,
  type StageCardState,
} from "@/lib/neoland/presentation"
import type {
  AgentStage,
  AgentStreamEvent,
  PipelineResult,
  ServiceStatus,
  StreamStatus,
} from "@/lib/neoland/types"
import { formatRelativeTime, formatTimestamp, slugify, truncateMiddle } from "@/lib/utils"

function checkpointSlug(checkpointPath: string) {
  const basename = checkpointPath.split("/").pop() || checkpointPath
  return slugify(basename.replace(/\.json$/i, ""))
}

function buildStageStates(
  mode: "idle" | "running" | "complete",
  result: PipelineResult | null | undefined,
  liveEvents: AgentStreamEvent[],
): Record<AgentStage, StageCardState> {
  if (mode === "complete") {
    return {
      junior: "complete",
      senior: "complete",
      architect: result?.architect ? "complete" : "skipped",
      tech_leader: "complete",
    }
  }

  if (mode !== "running") {
    return {
      junior: "idle",
      senior: "idle",
      architect: "idle",
      tech_leader: "idle",
    }
  }

  const states: Record<AgentStage, StageCardState> = {
    junior: "queued",
    senior: "queued",
    architect: "queued",
    tech_leader: "queued",
  }

  for (const event of liveEvents) {
    switch (event.type) {
      case "stage_started":
        states[event.stage] = "running"
        break
      case "stage_done":
        states[event.stage] = "complete"
        break
      case "stage_skipped":
        states[event.stage] = "skipped"
        break
      default:
        break
    }
  }

  return states
}

function buildStageContent(liveEvents: AgentStreamEvent[]) {
  const content: Partial<Record<AgentStage, string>> = {}

  for (const event of liveEvents) {
    switch (event.type) {
      case "stage_started":
        content[event.stage] = `Stage ${event.stage} started on the control plane.`
        break
      case "stage_output":
        content[event.stage] = event.content
        break
      case "stage_done":
        content[event.stage] = `Completed in ${event.latency_ms}ms.`
        break
      case "stage_skipped":
        content[event.stage] = "Skipped by the active decision flow."
        break
      default:
        break
    }
  }

  return content
}

function describeEvent(event: AgentStreamEvent) {
  switch (event.type) {
    case "pipeline_started":
      return {
        label: "Pipeline started",
        detail: event.task_preview,
      }
    case "stage_started":
      return {
        label: `${stageMeta[event.stage].label} started`,
        detail: "Execution entered this stage.",
      }
    case "stage_output":
      return {
        label: `${stageMeta[event.stage].label} output`,
        detail: event.content,
      }
    case "stage_done":
      return {
        label: `${stageMeta[event.stage].label} done`,
        detail: `${event.latency_ms}ms`,
      }
    case "stage_skipped":
      return {
        label: `${stageMeta[event.stage].label} skipped`,
        detail: "The control plane intentionally bypassed this stage.",
      }
    case "tool_call_started":
      return {
        label: `Tool call: ${event.tool}`,
        detail: event.args_summary,
      }
    case "tool_call_done":
      return {
        label: `Tool done: ${event.tool}`,
        detail: `${event.duration_ms}ms`,
      }
    case "tool_call_failed":
      return {
        label: `Tool failed: ${event.tool}`,
        detail: event.error,
      }
    case "adr_checkpoint":
      return {
        label: `ADR checkpoint: ${event.adr_id}`,
        detail: event.title,
      }
    case "pipeline_done":
      return {
        label: "Pipeline done",
        detail: `${event.latency_ms}ms end-to-end`,
      }
    case "pipeline_error":
      return {
        label: "Pipeline error",
        detail: event.error,
      }
    case "steering_received":
      return {
        label: "Human steering received",
        detail: event.message,
      }
  }
}

export function PipelineLiveView({
  mode,
  currentTask,
  activeSessionId,
  liveEvents = [],
  result,
  pipelineStatus,
  streamStatus = "idle",
}: {
  mode: "idle" | "running" | "complete"
  currentTask?: string
  activeSessionId?: string
  liveEvents?: AgentStreamEvent[]
  result?: PipelineResult | null
  pipelineStatus?: ServiceStatus
  streamStatus?: StreamStatus
}) {
  const liveStageContent = buildStageContent(liveEvents)
  const stageStates = buildStageStates(mode, result, liveEvents)
  const effectiveSessionId = result?.session_id || activeSessionId
  const showArchitect = mode === "complete" ? Boolean(result?.architect) : true
  const adrSlug = result ? checkpointSlug(result.checkpoint_path) : null

  return (
    <div className="space-y-6">
      <Card className="scanline-overlay overflow-hidden border-border/70 bg-card/85 metal-panel panel-ledge">
        <CardContent className="space-y-5 p-6">
          <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
            <div className="space-y-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.28em] text-primary/80">
                Pipeline live view
              </div>
              <div className="space-y-2">
                <h3 className="text-2xl font-semibold tracking-tight text-foreground">
                  {currentTask || "No task running yet"}
                </h3>
                <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
                  {mode === "running"
                    ? streamStatus === "degraded"
                      ? "The task is still running, but the live relay degraded. The final control-plane payload will still land here, and the active session stays usable for follow-up inspection."
                      : "The task is streaming live stage events over the control-plane SSE relay. Sessions, ADR checkpoints, and final decision stay connected through the same run."
                    : mode === "complete"
                      ? "This is the real pipeline result returned by the control plane and written into checkpoint storage."
                      : "Submit a real task to see the Junior, Senior, Architect, and Tech-Leader outputs render here."}
                </p>
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <StatusDot status={pipelineStatus || "stub"} label="Pipeline" />
              {result ? <DecisionBadge decision={result.tech_leader.decision} /> : null}
            </div>
          </div>

          {result ? (
            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Task ID
                </div>
                <div className="mt-2 text-sm text-foreground">{truncateMiddle(result.task_id, 10, 8)}</div>
              </div>
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Session
                </div>
                <div className="mt-2 text-sm text-foreground">{truncateMiddle(result.session_id, 10, 8)}</div>
              </div>
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Timestamp
                </div>
                <div className="mt-2 text-sm text-foreground">{formatTimestamp(result.timestamp)}</div>
                <div className="mt-1 text-xs text-muted-foreground">{formatRelativeTime(result.timestamp)}</div>
              </div>
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Checkpoint
                </div>
                <div className="mt-2 text-sm text-foreground">{result.checkpoint_path.split("/").pop()}</div>
              </div>
            </div>
          ) : null}

          {!result && effectiveSessionId ? (
            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Active session
                </div>
                <div className="mt-2 text-sm text-foreground">
                  {truncateMiddle(effectiveSessionId, 10, 8)}
                </div>
              </div>
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Live relay
                </div>
                <div className="mt-2 text-sm text-foreground">
                  {streamStatus === "live"
                    ? "Connected"
                    : streamStatus === "connecting"
                      ? "Connecting"
                      : streamStatus === "degraded"
                        ? "Degraded"
                        : "Idle"}
                </div>
              </div>
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Live events
                </div>
                <div className="mt-2 text-sm text-foreground">{liveEvents.length}</div>
              </div>
              <div className="rounded-2xl border border-border/70 bg-background/40 p-4">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  Next surface
                </div>
                <div className="mt-2 text-sm text-foreground">Session and ADR chain stay linked.</div>
              </div>
            </div>
          ) : null}

          {effectiveSessionId ? (
            <div className="flex flex-wrap items-center gap-3">
              {result ? (
                <Link
                  href={`/pipeline/${result.task_id}?sessionId=${encodeURIComponent(result.session_id)}${adrSlug ? `&adr=${encodeURIComponent(adrSlug)}` : ""}`}
                  className="inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/10 px-4 py-2 text-sm text-primary transition-colors hover:bg-primary/14"
                >
                  <FolderTree className="size-4" />
                  Open task surface
                </Link>
              ) : null}
              <Link
                href={`/sessions/${effectiveSessionId}`}
                className="inline-flex items-center gap-2 rounded-full border border-border/70 bg-card/80 px-4 py-2 text-sm text-foreground transition-colors hover:border-primary/20 hover:text-primary"
              >
                <ScanSearch className="size-4" />
                Inspect session
              </Link>
              {adrSlug ? (
                <Link
                  href={`/adr/${adrSlug}`}
                  className="inline-flex items-center gap-2 rounded-full border border-border/70 bg-card/80 px-4 py-2 text-sm text-foreground transition-colors hover:border-primary/20 hover:text-primary"
                >
                  <FileJson2 className="size-4" />
                  Open ADR
                </Link>
              ) : null}
            </div>
          ) : null}
        </CardContent>
      </Card>

      {liveEvents.length ? (
        <Card className="overflow-hidden border-border/70 bg-card/80 metal-panel panel-ledge">
          <CardContent className="space-y-5 p-6">
            <div className="flex items-center justify-between gap-4">
              <div>
                <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                  Event relay
                </div>
                <div className="mt-1 text-xl font-semibold tracking-tight text-foreground">
                  Cross-component live trace
                </div>
              </div>
              <div className="text-right text-xs text-muted-foreground">
                Control plane, tools, ADRs, and steering stream through the same session.
              </div>
            </div>

            <div className="grid gap-3 xl:grid-cols-2">
              {liveEvents.slice(-8).reverse().map((event, index) => {
                const summary = describeEvent(event)

                return (
                  <div
                    key={`${event.type}-${index}`}
                    className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4"
                  >
                    <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-primary/80">
                      {summary.label}
                    </div>
                    <div className="mt-2 text-sm leading-6 text-foreground">{summary.detail}</div>
                  </div>
                )
              })}
            </div>
          </CardContent>
        </Card>
      ) : null}

      <Card className="overflow-hidden border-border/70 bg-card/80 metal-panel panel-ledge">
        <CardContent className="space-y-5 p-6">
          <div className="flex items-center justify-between gap-4">
            <div>
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Stage relay
              </div>
              <div className="mt-1 text-xl font-semibold tracking-tight text-foreground">
                Pipeline progression track
              </div>
            </div>
            <div className="text-right text-xs text-muted-foreground">
              The UI reflects the real returned sequence, even when a stage is skipped.
            </div>
          </div>
          <div className="grid gap-4 xl:grid-cols-4">
            {stageSequence
              .filter((stage) => showArchitect || stage !== "architect")
              .map((stage, index, visibleStages) => {
                const state = stageStates[stage]
                const meta = stageMeta[stage]
                const complete = state === "complete"
                const skipped = state === "skipped"

                return (
                  <div key={stage} className="relative">
                    {index < visibleStages.length - 1 ? (
                      <div className="pointer-events-none absolute left-[calc(100%-0.5rem)] top-6 hidden h-px w-[calc(100%+1rem)] xl:block">
                        <div className="h-full bg-gradient-to-r from-primary/40 via-primary/10 to-transparent" />
                      </div>
                    ) : null}
                    <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="space-y-1">
                          <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                            Stage {index + 1}
                          </div>
                          <div className="text-base font-semibold text-foreground">{meta.label}</div>
                        </div>
                        <span
                          className={
                            complete
                              ? "inline-flex size-3 rounded-full bg-primary shadow-[0_0_22px_rgba(34,211,238,0.45)]"
                              : skipped
                                ? "inline-flex size-3 rounded-full border border-border/70 bg-card"
                                : state === "running"
                                  ? "inline-flex size-3 rounded-full bg-primary/90 shadow-[0_0_22px_rgba(34,211,238,0.35)]"
                                  : "inline-flex size-3 rounded-full bg-amber-400/80"
                          }
                        />
                      </div>
                      <div className="mt-4 text-sm leading-6 text-muted-foreground">{meta.description}</div>
                      <div className="mt-4 font-mono text-[11px] uppercase tracking-[0.22em] text-primary/80">
                        {stageStateLabel(state)}
                      </div>
                    </div>
                  </div>
                )
              })}
          </div>
        </CardContent>
      </Card>

      <div className={`grid gap-4 ${showArchitect ? "xl:grid-cols-4" : "xl:grid-cols-3"}`}>
        <AgentCard
          stage="junior"
          state={stageStates.junior}
          junior={result?.junior}
          liveContent={liveStageContent.junior}
        />
        <AgentCard
          stage="senior"
          state={stageStates.senior}
          senior={result?.senior}
          liveContent={liveStageContent.senior}
        />
        {showArchitect ? (
          <AgentCard
            stage="architect"
            state={stageStates.architect}
            architect={result?.architect}
            liveContent={liveStageContent.architect}
          />
        ) : null}
        <AgentCard
          stage="tech_leader"
          state={stageStates.tech_leader}
          techLeader={result?.tech_leader}
          liveContent={liveStageContent.tech_leader}
        />
      </div>

      {mode !== "complete" ? (
        <div className="rounded-[1.4rem] border border-dashed border-border/70 bg-card/60 p-5 text-sm leading-6 text-muted-foreground">
          <div className="flex items-start gap-3">
            <CircleDashed className="mt-1 size-4 text-primary" />
            <p>
              {streamStatus === "degraded"
                ? "The product still preserves progress even when the stream degrades: the run stays anchored to the session id, and the final ADR/result handoff remains intact."
                : "This surface now composes live control-plane events, session continuity, and checkpoint handoff into one operational loop instead of waiting only for the last payload."}
            </p>
          </div>
        </div>
      ) : (
        <div className="flex items-center justify-end">
          <Link
            href={adrSlug ? `/adr/${adrSlug}` : "/adr"}
            className="inline-flex items-center gap-2 text-sm text-primary transition-colors hover:text-primary/80"
          >
            Continue into checkpoint inspection
            <ArrowUpRight className="size-4" />
          </Link>
        </div>
      )}
    </div>
  )
}
