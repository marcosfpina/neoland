import { NextResponse } from "next/server"

import { getServicesSnapshot } from "@/lib/neoland/server"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

export async function GET() {
  return NextResponse.json(await getServicesSnapshot())
}
