"use client"

import { useEffect, useState } from "react"
import { AlertCircle, ArrowRight, LoaderCircle, Waypoints } from "lucide-react"

import { PipelineLiveView } from "@/components/pipeline/pipeline-live"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import type {
  AgentStreamEvent,
  PipelineResult,
  ServiceStatus,
  StreamStatus,
} from "@/lib/neoland/types"

export function PipelineRunner({
  pipelineStatus,
}: {
  pipelineStatus?: ServiceStatus
}) {
  const [task, setTask] = useState("")
  const [sessionId, setSessionId] = useState("")
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null)
  const [result, setResult] = useState<PipelineResult | null>(null)
  const [liveEvents, setLiveEvents] = useState<AgentStreamEvent[]>([])
  const [error, setError] = useState<string | null>(null)
  const [isRunning, setIsRunning] = useState(false)
  const [streamStatus, setStreamStatus] = useState<StreamStatus>("idle")

  useEffect(() => {
    if (!activeSessionId || !isRunning) {
      return
    }

    setStreamStatus("connecting")
    const source = new EventSource(`/api/neoland/events/${activeSessionId}`)

    source.onopen = () => {
      setStreamStatus("live")
    }

    source.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data) as AgentStreamEvent
        setLiveEvents((current) => {
          const next = [...current, payload]
          return next.length > 80 ? next.slice(-80) : next
        })
      } catch {
        // Keep the stream alive when an event payload is malformed.
      }
    }

    source.onerror = () => {
      setStreamStatus("degraded")
      source.close()
    }

    return () => source.close()
  }, [activeSessionId, isRunning])

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()

    const trimmedTask = task.trim()
    const trimmedSessionId = sessionId.trim()
    const requestSessionId = trimmedSessionId || crypto.randomUUID()

    if (!trimmedTask) {
      setError("Task is required.")
      return
    }

    setError(null)
    setResult(null)
    setSessionId(requestSessionId)
    setActiveSessionId(requestSessionId)
    setLiveEvents([])
    setStreamStatus("connecting")
    setIsRunning(true)

    void (async () => {
      try {
        const response = await fetch("/api/neoland/pipeline/run", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            task: trimmedTask,
            session_id: requestSessionId,
          }),
        })

        const payload = (await response.json()) as PipelineResult | { error?: string }

        if (!response.ok) {
          throw new Error(
            "error" in payload ? payload.error || "Pipeline run failed." : "Pipeline run failed.",
          )
        }

        setResult(payload as PipelineResult)
        setSessionId((payload as PipelineResult).session_id)
        setActiveSessionId((payload as PipelineResult).session_id)
      } catch (submitError) {
        setError(submitError instanceof Error ? submitError.message : "Pipeline run failed.")
      } finally {
        setIsRunning(false)
      }
    })()
  }

  return (
    <div className="space-y-6">
      <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
        <CardHeader className="border-b border-border/70 pb-4">
          <div className="flex items-center gap-3">
            <div className="rounded-2xl border border-primary/20 bg-primary/10 p-3 text-primary">
              <Waypoints className="size-5" />
            </div>
            <div className="space-y-1">
              <div className="font-mono text-[11px] uppercase tracking-[0.28em] text-primary/80">
                Dispatch task
              </div>
              <CardTitle className="text-2xl tracking-tight text-foreground">
                Run the real Neoland pipeline
              </CardTitle>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-6 pt-6">
          <form className="space-y-5" onSubmit={handleSubmit}>
            <div className="grid gap-5 xl:grid-cols-[1.4fr_0.8fr]">
              <div className="space-y-2">
                <label className="font-mono text-[11px] uppercase tracking-[0.24em] text-muted-foreground">
                  Task
                </label>
                <Textarea
                  value={task}
                  onChange={(event) => setTask(event.target.value)}
                  placeholder="Refactor auth middleware to use JWT RS256"
                  className="min-h-32 rounded-[1.2rem] border-border/70 bg-background/40 px-4 py-3 text-sm"
                />
              </div>
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="font-mono text-[11px] uppercase tracking-[0.24em] text-muted-foreground">
                    Session ID
                  </label>
                  <Input
                    value={sessionId}
                    onChange={(event) => setSessionId(event.target.value)}
                    placeholder="Optional existing UUID"
                    className="h-11 rounded-[1.1rem] border-border/70 bg-background/40"
                  />
                  <p className="text-xs leading-5 text-muted-foreground">
                    Reuse a session to keep live events, decision history, and ADR continuity in the
                    same operational thread.
                  </p>
                </div>
                <div className="rounded-[1.2rem] border border-dashed border-border/70 bg-background/30 p-4 text-sm leading-6 text-muted-foreground">
                  The workbench now opens a live SSE relay for the chosen session id before the final
                  pipeline payload lands, so progress, checkpointing, and follow-up inspection stay
                  joined.
                </div>
              </div>
            </div>

            {error ? (
              <div className="rounded-[1.2rem] border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-200">
                <div className="flex items-start gap-3">
                  <AlertCircle className="mt-0.5 size-4 shrink-0" />
                  <span>{error}</span>
                </div>
              </div>
            ) : null}

            <div className="flex flex-wrap items-center gap-3">
              <Button type="submit" size="lg" disabled={isRunning}>
                {isRunning ? (
                  <LoaderCircle className="size-4 animate-spin" />
                ) : (
                  <ArrowRight className="size-4" />
                )}
                {isRunning ? "Running control-plane task" : "Run pipeline"}
              </Button>
              <span className="text-xs uppercase tracking-[0.22em] text-muted-foreground">
                real POST /v1/agents/task
              </span>
            </div>
          </form>
        </CardContent>
      </Card>

      <PipelineLiveView
        mode={isRunning ? "running" : result ? "complete" : "idle"}
        currentTask={task.trim() || undefined}
        activeSessionId={activeSessionId || undefined}
        liveEvents={liveEvents}
        result={result}
        pipelineStatus={pipelineStatus}
        streamStatus={streamStatus}
      />
    </div>
  )
}
