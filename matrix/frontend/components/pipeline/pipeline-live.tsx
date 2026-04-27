import Link from "next/link"
import { ArrowUpRight, CircleDashed, FileJson2, FolderTree, ScanSearch } from "lucide-react"

import { AgentCard } from "@/components/pipeline/agent-card"
import { DecisionBadge } from "@/components/shared/decision-badge"
import { StatusDot } from "@/components/shared/status-dot"
import { Card, CardContent } from "@/components/ui/card"
import { stageMeta, stageSequence, stageStateLabel } from "@/lib/neoland/presentation"
import type { PipelineResult, ServiceStatus } from "@/lib/neoland/types"
import { formatRelativeTime, formatTimestamp, slugify, truncateMiddle } from "@/lib/utils"

function checkpointSlug(checkpointPath: string) {
  const basename = checkpointPath.split("/").pop() || checkpointPath
  return slugify(basename.replace(/\.json$/i, ""))
}

export function PipelineLiveView({
  mode,
  currentTask,
  result,
  pipelineStatus,
}: {
  mode: "idle" | "running" | "complete"
  currentTask?: string
  result?: PipelineResult | null
  pipelineStatus?: ServiceStatus
}) {
  const showArchitect = mode === "complete" ? Boolean(result?.architect) : true
  const adrSlug = result ? checkpointSlug(result.checkpoint_path) : null
  const stageStates = {
    junior: mode === "complete" ? "complete" : mode === "running" ? "queued" : "idle",
    senior: mode === "complete" ? "complete" : mode === "running" ? "queued" : "idle",
    architect:
      mode === "complete"
        ? result?.architect
          ? "complete"
          : "skipped"
        : mode === "running"
          ? "queued"
          : "idle",
    tech_leader: mode === "complete" ? "complete" : mode === "running" ? "queued" : "idle",
  } as const

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
                    ? "The task has been submitted to the Neoland control plane. Stage-by-stage streaming is not exposed yet, so this view waits for the final pipeline payload and then renders the true outputs."
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

          {result ? (
            <div className="flex flex-wrap items-center gap-3">
              <Link
                href={`/pipeline/${result.task_id}?sessionId=${encodeURIComponent(result.session_id)}${adrSlug ? `&adr=${encodeURIComponent(adrSlug)}` : ""}`}
                className="inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/10 px-4 py-2 text-sm text-primary transition-colors hover:bg-primary/14"
              >
                <FolderTree className="size-4" />
                Open task surface
              </Link>
              <Link
                href={`/sessions/${result.session_id}`}
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
        />
        <AgentCard
          stage="senior"
          state={stageStates.senior}
          senior={result?.senior}
        />
        {showArchitect ? (
          <AgentCard
            stage="architect"
            state={stageStates.architect}
            architect={result?.architect}
          />
        ) : null}
        <AgentCard
          stage="tech_leader"
          state={stageStates.tech_leader}
          techLeader={result?.tech_leader}
        />
      </div>

      {mode !== "complete" ? (
        <div className="rounded-[1.4rem] border border-dashed border-border/70 bg-card/60 p-5 text-sm leading-6 text-muted-foreground">
          <div className="flex items-start gap-3">
            <CircleDashed className="mt-1 size-4 text-primary" />
            <p>
              The live surface is wired to real control-plane calls only. Until Neoland exposes
              native stage streaming, this page will show queued state during execution and then the
              final payload once the pipeline returns.
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
