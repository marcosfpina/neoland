import { NextResponse } from "next/server"

import { getAdrBySlug } from "@/lib/neoland/server"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

export async function GET(
  _request: Request,
  context: { params: Promise<{ adrId: string }> },
) {
  const { adrId } = await context.params
  const adr = await getAdrBySlug(adrId)

  if (!adr) {
    return NextResponse.json({ error: "ADR not found" }, { status: 404 })
  }

  return NextResponse.json(adr)
}
