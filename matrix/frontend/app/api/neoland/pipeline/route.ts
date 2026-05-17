import { NextResponse } from "next/server"

import { getAdrDocuments } from "@/lib/neoland/server"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

export async function GET(req: Request) {
  const { searchParams } = new URL(req.url)
  const limit = Math.min(parseInt(searchParams.get("limit") ?? "50", 10) || 50, 200)

  try {
    const adrs = await getAdrDocuments()
    const slice = adrs.slice(0, limit)

    const runs = slice.map((adr) => ({
      run_id: adr.adr_id,
      session_id: adr.adr_id.split("-")[1] ?? "",
      task_preview: adr.context,
      decision: adr.full_pipeline.tech_leader.decision,
      source_decision: adr.full_pipeline.tech_leader.decision,
      adr_title: adr.title,
      junior_confidence: adr.full_pipeline.junior.confidence,
      total_latency_ms: 0,
      score: 0,
      timestamp: new Date(adr.timestamp).getTime(),
    }))

    return NextResponse.json({ runs, total: adrs.length })
  } catch {
    return NextResponse.json({ runs: [], total: 0 })
  }
}
