import { NextResponse } from "next/server"

import { getAdrDocuments } from "@/lib/neoland/server"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

export async function GET() {
  try {
    const adrs = await getAdrDocuments()
    const total = adrs.length
    const approve_count = adrs.filter(
      (adr) => adr.full_pipeline.tech_leader.decision === "approve",
    ).length

    const avg_score =
      total > 0
        ? Math.round(
            (adrs.reduce((sum, adr) => sum + adr.full_pipeline.junior.confidence, 0) / total) * 100,
          )
        : 0

    return NextResponse.json({
      total,
      approve_count,
      avg_latency_ms: 0,
      avg_score,
      telemetry_source: "adr",
    })
  } catch {
    return NextResponse.json({
      total: 0,
      approve_count: 0,
      avg_latency_ms: 0,
      avg_score: 0,
      telemetry_source: "adr",
    })
  }
}
