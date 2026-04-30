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
  query: string
  context?: string
  priority?: "low" | "normal" | "high" | "critical"
}

export interface AgentTaskResponse {
  task_id: string
  status: "accepted" | "rejected"
  estimated_duration_ms?: number
}

export interface SessionStateResponse {
  id: string
  status: "active" | "completed" | "failed" | "escalated"
  agent_id: string
  created_at: string
  updated_at: string
  metadata: Record<string, unknown>
  variables: Record<string, unknown>
  history: Array<{
    role: "user" | "agent" | "system"
    content: string
    timestamp: string
  }>
}

export class NeolandClient {
  private static async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${NEOLAND_BASE_URL}${endpoint}`
    
    // Default headers
    const headers = new Headers(options.headers)
    headers.set("Content-Type", "application/json")
    
    // In production/real usage, we would add the API Key from a secure context
    // if (process.env.NEOLAND_API_KEY) {
    //   headers.set("X-API-Key", process.env.NEOLAND_API_KEY)
    // }

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

  static async submitTask(task: AgentTaskRequest): Promise<AgentTaskResponse> {
    return this.request<AgentTaskResponse>("/v1/agents/task", {
      method: "POST",
      body: JSON.stringify(task)
    })
  }

  static async getSession(sessionId: string): Promise<SessionStateResponse> {
    return this.request<SessionStateResponse>(`/v1/agents/session/${sessionId}`)
  }
}
