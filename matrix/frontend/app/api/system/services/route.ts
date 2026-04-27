/**
 * System Services API Route
 * Control systemd services
 */

import { type NextRequest, NextResponse } from "next/server"
import { exec } from "child_process"
import { promisify } from "util"

const execAsync = promisify(exec)

// Whitelist of services that can be controlled
const ALLOWED_SERVICES = [
  "ollama",
  "mission-control",
  "mission-control-metrics",
  "nginx",
  "postgresql",
  "redis",
  "docker",
]

export async function GET() {
  const services = []

  for (const service of ALLOWED_SERVICES) {
    try {
      const { stdout } = await execAsync(
        `systemctl show ${service} --property=ActiveState,SubState,MainPID,MemoryCurrent,Description`,
        { timeout: 5000 },
      )

      const props: Record<string, string> = {}
      for (const line of stdout.trim().split("\n")) {
        const [key, value] = line.split("=")
        if (key && value) props[key] = value
      }

      services.push({
        name: service,
        description: props.Description || service,
        active_state: props.ActiveState || "unknown",
        sub_state: props.SubState || "unknown",
        pid: Number.parseInt(props.MainPID || "0"),
        memory_bytes: props.MemoryCurrent !== "[not set]" ? Number.parseInt(props.MemoryCurrent || "0") : 0,
      })
    } catch {
      services.push({
        name: service,
        description: service,
        active_state: "unknown",
        sub_state: "unknown",
        pid: 0,
        memory_bytes: 0,
      })
    }
  }

  return NextResponse.json({ success: true, services })
}

export async function POST(request: NextRequest) {
  const body = await request.json()
  const { service, action } = body

  // Validate service
  if (!ALLOWED_SERVICES.includes(service)) {
    return NextResponse.json({ success: false, error: `Service '${service}' not in allowed list` }, { status: 403 })
  }

  // Validate action
  if (!["start", "stop", "restart", "status"].includes(action)) {
    return NextResponse.json({ success: false, error: `Invalid action '${action}'` }, { status: 400 })
  }

  try {
    if (action === "status") {
      const { stdout } = await execAsync(`systemctl status ${service}`, { timeout: 5000 })
      return NextResponse.json({ success: true, output: stdout })
    }

    // For start/stop/restart, we need elevated privileges
    // In production, this would use polkit or a privileged helper
    const { stdout, stderr } = await execAsync(`systemctl ${action} ${service}`, { timeout: 30000 })

    return NextResponse.json({
      success: true,
      action,
      service,
      output: stdout,
      error: stderr || undefined,
    })
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error"
    return NextResponse.json({ success: false, error: message }, { status: 500 })
  }
}
