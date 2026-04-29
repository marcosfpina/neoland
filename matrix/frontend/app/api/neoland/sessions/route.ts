import { NextResponse } from "next/server"

import { getSessions } from "@/lib/neoland/server"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const rawLimit = Number(searchParams.get("limit") || "24")
  const limit = Number.isFinite(rawLimit) ? rawLimit : 24

  try {
    return NextResponse.json(await getSessions(limit))
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Failed to load sessions." },
      { status: 502 },
    )
  }
}

