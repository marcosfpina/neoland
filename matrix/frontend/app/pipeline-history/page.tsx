"use client"

import { useEffect, useState, useCallback } from "react"
import Link from "next/link"
import { Activity, ArrowLeft, CheckCircle, XCircle, Clock, RefreshCw, Bot } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"

interface AgentRunMetrics {
  confidence?: number
  latency_ms?: number
  escalated?: boolean
  decision?: string
}

interface PipelineRun {
  run_id: string
  session_id: string
  task_preview: string
  decision: string
  adr_title?: string
  junior_confidence?: number
  total_latency_ms: number
  score: number
  timestamp: number
}

function DecisionBadge({ decision }: { decision: string }) {
  const colors: Record<string, string> = {
    accepted: "bg-green-500/15 text-green-400 border-green-500/30",
    rejected: "bg-red-500/15 text-red-400 border-red-500/30",
    deferred: "bg-yellow-500/15 text-yellow-400 border-yellow-500/30",
    escalated: "bg-purple-500/15 text-purple-400 border-purple-500/30",
  }
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-medium ${colors[decision] ?? "bg-gray-500/15 text-gray-400 border-gray-500/30"}`}
    >
      {decision === "accepted" && <CheckCircle className="size-3" />}
      {decision === "rejected" && <XCircle className="size-3" />}
      {decision}
    </span>
  )
}

function ScoreBar({ score }: { score: number }) {
  const pct = Math.round(score * 100)
  const color =
    pct >= 80 ? "bg-green-500" : pct >= 50 ? "bg-yellow-500" : "bg-red-500"
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
  const [runs, setRuns] = useState<PipelineRun[]>([])
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
      ? Math.round(runs.reduce((s, r) => s + r.total_latency_ms, 0) / runs.length)
      : 0
  const avgScore =
    runs.length > 0
      ? Math.round((runs.reduce((s, r) => s + r.score, 0) / runs.length) * 100)
      : 0
  const acceptedCount = runs.filter((r) => r.decision === "accepted").length

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button asChild variant="ghost" size="sm">
            <Link href="/">
              <ArrowLeft className="size-4" />
            </Link>
          </Button>
          <div>
            <p className="text-xs uppercase tracking-widest text-muted-foreground">Neoland</p>
            <h1 className="text-2xl font-bold">Pipeline History</h1>
          </div>
        </div>
        <Button variant="outline" size="sm" onClick={fetchRuns}>
          <RefreshCw className="size-4" />
          Refresh
        </Button>
      </div>

      {/* Summary cards */}
      <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Total runs</CardDescription>
            <CardTitle className="text-3xl">{total}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Accepted</CardDescription>
            <CardTitle className="text-3xl text-green-400">{acceptedCount}</CardTitle>
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
            <CardDescription>Avg score</CardDescription>
            <CardTitle className="text-3xl">{avgScore}%</CardTitle>
          </CardHeader>
        </Card>
      </div>

      {/* Run list */}
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
              <div className="flex h-40 items-center justify-center text-muted-foreground text-sm">
                Loading…
              </div>
            ) : runs.length === 0 ? (
              <div className="flex h-40 flex-col items-center justify-center gap-2 text-muted-foreground">
                <Bot className="size-8 opacity-30" />
                <p className="text-sm">No pipeline runs yet.</p>
                <p className="text-xs">
                  Set <code>NEOLAND_MATRIX_URL</code> and run a task in the TUI.
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
                      <Clock className="inline size-3 mr-1" />
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
                      className="border-b border-white/5 hover:bg-white/5 transition-colors"
                    >
                      <td className="px-4 py-3 max-w-[200px] truncate" title={run.task_preview}>
                        <span className="font-mono text-xs text-foreground">{run.task_preview}</span>
                        {run.adr_title && (
                          <p className="text-xs text-muted-foreground truncate">{run.adr_title}</p>
                        )}
                      </td>
                      <td className="px-4 py-3">
                        <DecisionBadge decision={run.decision} />
                      </td>
                      <td className="px-4 py-3 hidden sm:table-cell">
                        <ScoreBar score={run.score} />
                      </td>
                      <td className="px-4 py-3 hidden md:table-cell text-muted-foreground">
                        {run.total_latency_ms.toLocaleString()}ms
                      </td>
                      <td className="px-4 py-3 hidden lg:table-cell text-muted-foreground">
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
