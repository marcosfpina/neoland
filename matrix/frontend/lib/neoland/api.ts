import type { PipelineResult } from "./types"

export const NEOLAND_BASE_URL = process.env.NEOLAND_CONTROL_PLANE_URL || "http://127.0.0.1:3001"

export interface NeolandHealthResponse {
  status: "healthy" | "degraded" | "unhealthy"
  uptime_seconds: number
  version: string
  components: Array<{
    name: string
    status: "healthy" | "degraded" | "unhealthy"
    latency_ms?: number
    error?: string
  }>
}

export interface AgentTaskRequest {
  task: string
  session_id?: string
}

export interface SessionStateResponse {
  session_id: string
  task_count: number
  last_activity: string
  last_decision: unknown | null
  active: boolean
}

export class NeolandClient {
  private static async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${NEOLAND_BASE_URL}${endpoint}`

    const headers = new Headers(options.headers)
    headers.set("Content-Type", "application/json")

    if (process.env.NEOLAND_API_KEY) {
      headers.set("X-API-Key", process.env.NEOLAND_API_KEY)
    }

    const response = await fetch(url, {
      ...options,
      headers,
    })

    if (!response.ok) {
      throw new Error(`Neoland API error: ${response.status} ${response.statusText}`)
    }

    return response.json()
  }

  static async getHealth(): Promise<NeolandHealthResponse> {
    return this.request<NeolandHealthResponse>("/health")
  }

  static async submitTask(task: AgentTaskRequest): Promise<PipelineResult> {
    return this.request<PipelineResult>("/v1/agents/task", {
      method: "POST",
      body: JSON.stringify(task),
    })
  }

  static async getSession(sessionId: string): Promise<SessionStateResponse> {
    return this.request<SessionStateResponse>(`/v1/agents/session/${sessionId}`)
  }

  static async listSessions(limit = 24): Promise<SessionStateResponse[]> {
    return this.request<SessionStateResponse[]>(`/v1/agents/sessions?limit=${limit}`)
  }
}
