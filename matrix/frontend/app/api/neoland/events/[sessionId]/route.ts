import { NextResponse } from "next/server"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

const CONTROL_PLANE_URL = (
  process.env.NEOLAND_CONTROL_PLANE_URL ||
  process.env.NEXT_PUBLIC_BACKEND_URL ||
  "http://127.0.0.1:3001"
).replace(/\/$/, "")

function authHeaders() {
  const apiKey =
    process.env.NEOLAND_USER_API_KEY ||
    process.env.NEOLAND_API_KEY ||
    process.env.NEOLAND_UI_API_KEY

  return {
    Accept: "text/event-stream",
    "Cache-Control": "no-cache",
    "X-API-Key": apiKey || "neoland_unauthorized",
  }
}

export async function GET(
  _request: Request,
  context: { params: Promise<{ sessionId: string }> },
) {
  const { sessionId } = await context.params

  const upstream = await fetch(`${CONTROL_PLANE_URL}/v1/agents/events/${sessionId}`, {
    headers: authHeaders(),
    cache: "no-store",
  })

  if (!upstream.ok || !upstream.body) {
    const body = await upstream.text().catch(() => "Failed to open event stream.")
    return NextResponse.json({ error: body || "Failed to open event stream." }, { status: 502 })
  }

  return new Response(upstream.body, {
    status: 200,
    headers: {
      "Cache-Control": "no-cache, no-transform",
      Connection: "keep-alive",
      "Content-Type": "text/event-stream; charset=utf-8",
    },
  })
}
