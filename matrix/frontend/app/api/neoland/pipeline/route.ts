import { NextResponse } from "next/server"

import { normalizeHistoricalRun } from "@/lib/neoland/history"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

const MATRIX_BACKEND_URL =
  (process.env.NEOLAND_MATRIX_URL || "http://localhost:8002").replace(/\/$/, "")

export async function GET(req: Request) {
  const { searchParams } = new URL(req.url)
  const limit = searchParams.get("limit") ?? "50"

  try {
    const resp = await fetch(`${MATRIX_BACKEND_URL}/pipeline/history?limit=${limit}`, {
      cache: "no-store",
    })
    if (!resp.ok) {
      return NextResponse.json({ runs: [], total: 0 }, { status: 200 })
    }
    const data = await resp.json()
    const runs = Array.isArray(data.runs) ? data.runs.map(normalizeHistoricalRun) : []
    return NextResponse.json({ ...data, runs })
  } catch {
    return NextResponse.json({ runs: [], total: 0 }, { status: 200 })
  }
}
