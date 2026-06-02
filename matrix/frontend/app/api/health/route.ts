import { NextResponse } from "next/server"

const OLLAMA_HOST = process.env.OLLAMA_HOST || "http://localhost:11434"
const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:8000"

async function checkService(url: string, _name: string) {
  const start = performance.now()
  try {
    const res = await fetch(url, { signal: AbortSignal.timeout(2000) })
    const latency = Math.round(performance.now() - start)
    return {
      status: res.ok ? "operational" : "degraded",
      latency_ms: latency,
      details: res.ok ? "OK" : `HTTP ${res.status}`
    }
  } catch (error) {
    return {
      status: "down",
      latency_ms: Math.round(performance.now() - start),
      details: error instanceof Error ? error.message : "Connection failed"
    }
  }
}

export async function GET() {
  const [ollama, backend] = await Promise.all([
    checkService(`${OLLAMA_HOST}/api/version`, "ollama"),
    checkService(`${BACKEND_URL}/health`, "backend")
  ])

  // System Density Metrics
  const density = {
    memory: process.memoryUsage(),
    uptime: process.uptime(),
    load: process.cpuUsage(),
    env: process.env.NODE_ENV
  }

  const status = (ollama.status === "operational" && backend.status === "operational")
    ? "healthy"
    : "degraded"

  return NextResponse.json({
    status,
    timestamp: new Date().toISOString(),
    services: {
      ollama,
      backend
    },
    system: {
      version: process.version,
      ...density
    },
    meta: {
      message: "System running at maximum density 🌌",
      version: "2.0.0"
    }
  }, {
    status: status === "healthy" ? 200 : 503
  })
}
