import { NextResponse } from "next/server"

import { normalizeHistoricalDecision } from "@/lib/neoland/history"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

const MATRIX_BACKEND_URL =
  (process.env.NEOLAND_MATRIX_URL || "http://localhost:8002").replace(/\/$/, "")

export async function GET() {
  try {
    const resp = await fetch(`${MATRIX_BACKEND_URL}/pipeline/history?limit=500`, {
      cache: "no-store",
    })
    if (!resp.ok) {
      return NextResponse.json({
        total: 0,
        approve_count: 0,
        avg_latency_ms: 0,
        avg_score: 0,
        telemetry_source: "matrix",
      })
    }
    const data = await resp.json()
    const runs: Array<{ decision: string; total_latency_ms: number; score: number }> =
      data.runs ?? []

    const total = data.total ?? 0
    const approveCount = runs.filter(
      (run) => normalizeHistoricalDecision(run.decision) === "approve",
    ).length
    const avg_latency_ms =
      runs.length > 0
        ? Math.round(runs.reduce((s, r) => s + r.total_latency_ms, 0) / runs.length)
        : 0
    const avg_score =
      runs.length > 0
        ? Math.round((runs.reduce((s, r) => s + r.score, 0) / runs.length) * 100)
        : 0

    return NextResponse.json({
      total,
      approve_count: approveCount,
      avg_latency_ms,
      avg_score,
      telemetry_source: "matrix",
    })
  } catch {
    return NextResponse.json({
      total: 0,
      approve_count: 0,
      avg_latency_ms: 0,
      avg_score: 0,
      telemetry_source: "matrix",
    })
  }
}
