"use client"

import { useCallback, useEffect, useState } from "react"
import Link from "next/link"
import {
  Activity,
  ArrowLeft,
  CheckCircle,
  Clock,
  Play,
  RefreshCw,
  ShieldAlert,
  Bot,
  PauseCircle,
  Workflow,
} from "lucide-react"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import type { HistoricalDecision, HistoricalPipelineRun } from "@/lib/neoland/history"

function DecisionBadge({ decision }: { decision: HistoricalDecision }) {
  const colors: Record<HistoricalDecision, string> = {
    approve: "bg-green-500/15 text-green-400 border-green-500/30",
    reject: "bg-red-500/15 text-red-400 border-red-500/30",
    defer: "bg-yellow-500/15 text-yellow-400 border-yellow-500/30",
    escalate: "bg-purple-500/15 text-purple-400 border-purple-500/30",
    unknown: "bg-slate-500/15 text-slate-300 border-slate-500/30",
  }

  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-medium ${colors[decision]}`}
    >
      {decision === "approve" && <CheckCircle className="size-3" />}
      {decision === "reject" && <ShieldAlert className="size-3" />}
      {decision === "defer" && <PauseCircle className="size-3" />}
      {decision === "escalate" && <Workflow className="size-3" />}
      {decision}
    </span>
  )
}

function ScoreBar({ score }: { score: number }) {
  const pct = Math.round(score * 100)
  const color = pct >= 80 ? "bg-green-500" : pct >= 50 ? "bg-yellow-500" : "bg-red-500"

  return (
    <div className="flex items-center gap-2">
      <div className="h-1.5 w-20 rounded-full bg-white/10">
        <div className={`h-full rounded-full ${color}`} style={{ width: `${pct}%` }} />
      </div>
      <span className="text-xs text-muted-foreground">{pct}%</span>
    </div>
  )
}

export default function PipelineHistoryPage() {
  const [runs, setRuns] = useState<HistoricalPipelineRun[]>([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(true)
  const [lastRefresh, setLastRefresh] = useState(new Date())

  const fetchRuns = useCallback(async () => {
    try {
      const resp = await fetch("/api/neoland/pipeline?limit=50")
      const data = await resp.json()
      setRuns(data.runs ?? [])
      setTotal(data.total ?? 0)
    } catch {
      // backend not running — show empty state
    } finally {
      setLoading(false)
      setLastRefresh(new Date())
    }
  }, [])

  useEffect(() => {
    fetchRuns()
    const id = setInterval(fetchRuns, 15_000)
    return () => clearInterval(id)
  }, [fetchRuns])

  const avgLatency =
    runs.length > 0
      ? Math.round(runs.reduce((sum, run) => sum + run.total_latency_ms, 0) / runs.length)
      : 0
  const avgScore =
    runs.length > 0
      ? Math.round((runs.reduce((sum, run) => sum + run.score, 0) / runs.length) * 100)
      : 0
  const approveCount = runs.filter((run) => run.decision === "approve").length

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button asChild variant="ghost" size="sm">
            <Link href="/">
              <ArrowLeft className="size-4" />
            </Link>
          </Button>
          <div>
            <p className="text-xs uppercase tracking-widest text-muted-foreground">Neoland + Matrix</p>
            <h1 className="text-2xl font-bold">Pipeline History</h1>
            <p className="text-sm text-muted-foreground">
              Matrix keeps the historical telemetry plane; the UI translates that memory into Neoland
              decision semantics.
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button asChild variant="outline" size="sm">
            <Link href="/pipeline">
              <Play className="size-4" />
              Run pipeline
            </Link>
          </Button>
          <Button variant="outline" size="sm" onClick={fetchRuns}>
            <RefreshCw className="size-4" />
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Total runs</CardDescription>
            <CardTitle className="text-3xl">{total}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Approved</CardDescription>
            <CardTitle className="text-3xl text-green-400">{approveCount}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Avg latency</CardDescription>
            <CardTitle className="text-3xl">{avgLatency}ms</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Matrix score</CardDescription>
            <CardTitle className="text-3xl">{avgScore}%</CardTitle>
          </CardHeader>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Activity className="size-4 text-muted-foreground" />
              <CardTitle className="text-base">Recent runs</CardTitle>
            </div>
            <span className="text-xs text-muted-foreground">
              refreshed {lastRefresh.toLocaleTimeString()}
            </span>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          <ScrollArea className="h-[480px]">
            {loading ? (
              <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
                Loading…
              </div>
            ) : runs.length === 0 ? (
              <div className="flex h-40 flex-col items-center justify-center gap-2 text-muted-foreground">
                <Bot className="size-8 opacity-30" />
                <p className="text-sm">No pipeline runs yet.</p>
                <p className="text-xs">
                  Wire <code>NEOLAND_MATRIX_URL</code> and let Neoland publish its historical
                  telemetry here.
                </p>
              </div>
            ) : (
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-white/10 text-xs text-muted-foreground">
                    <th className="px-4 py-2 text-left font-medium">Task</th>
                    <th className="px-4 py-2 text-left font-medium">Decision</th>
                    <th className="px-4 py-2 text-left font-medium hidden sm:table-cell">Score</th>
                    <th className="px-4 py-2 text-left font-medium hidden md:table-cell">
                      <Clock className="mr-1 inline size-3" />
                      Latency
                    </th>
                    <th className="px-4 py-2 text-left font-medium hidden lg:table-cell">
                      Junior conf.
                    </th>
                    <th className="px-4 py-2 text-right font-medium text-muted-foreground">Time</th>
                  </tr>
                </thead>
                <tbody>
                  {runs.map((run) => (
                    <tr
                      key={run.run_id}
                      className="border-b border-white/5 transition-colors hover:bg-white/5"
                    >
                      <td className="max-w-[200px] px-4 py-3 truncate" title={run.task_preview}>
                        <span className="font-mono text-xs text-foreground">{run.task_preview}</span>
                        {run.adr_title ? (
                          <p className="truncate text-xs text-muted-foreground">{run.adr_title}</p>
                        ) : null}
                      </td>
                      <td className="px-4 py-3">
                        <DecisionBadge decision={run.decision} />
                      </td>
                      <td className="hidden px-4 py-3 sm:table-cell">
                        <ScoreBar score={run.score} />
                      </td>
                      <td className="hidden px-4 py-3 text-muted-foreground md:table-cell">
                        {run.total_latency_ms.toLocaleString()}ms
                      </td>
                      <td className="hidden px-4 py-3 text-muted-foreground lg:table-cell">
                        {run.junior_confidence != null
                          ? `${(run.junior_confidence * 100).toFixed(0)}%`
                          : "—"}
                      </td>
                      <td className="px-4 py-3 text-right text-xs text-muted-foreground">
                        {new Date(run.timestamp * 1000).toLocaleTimeString()}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  )
}
