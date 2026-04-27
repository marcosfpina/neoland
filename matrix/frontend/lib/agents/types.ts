// Agent Identity System - Each agent has personality and metrics

export interface AgentIdentity {
  id: string
  name: string
  codename: string
  provider: 'ollama' | 'openai' | 'anthropic' | 'google' | 'grok' | 'codex'
  role: AgentRole
  avatar: string
  personality: string
  specializations: string[]
  status: AgentStatus
  metrics: AgentMetrics
  aptitude?: AgentAptitude
  createdAt: Date
  lastActive: Date
}

export type AgentRole =
  | "orchestrator"
  | "code-intel"
  | "embeddings"
  | "monitor"
  | "scribe"
  | "git-ops"
  | "prompt-forge"
  | "inference"

export type AgentStatus = "idle" | "working" | "paused" | "error" | "offline"

export interface AgentMetrics {
  tasksCompleted: number
  tasksInProgress: number
  successRate: number
  avgResponseTime: number // ms
  totalTokensUsed: number
  efficiencyScore: number // 0-100
  xp: number
  level: number
  streak: number
  lastError?: string
  uptimeHours: number
}

export interface AgentTask {
  id: string
  agentId: string
  type: TaskType
  status: TaskStatus
  priority: TaskPriority
  input: Record<string, unknown>
  output?: Record<string, unknown>
  startedAt: Date
  completedAt?: Date
  duration?: number
  tokensUsed?: number
  error?: string
}

export type TaskType =
  | "code-analysis"
  | "doc-generation"
  | "report-creation"
  | "git-operation"
  | "prompt-generation"
  | "model-inference"
  | "system-monitor"
  | "workflow-orchestration"

export type TaskStatus = "queued" | "running" | "completed" | "failed" | "cancelled"

export type TaskPriority = "low" | "medium" | "high" | "critical"

// System Resources
export interface SystemResources {
  cpu: {
    usage: number
    cores: number
    temperature?: number
  }
  memory: {
    used: number
    total: number
    available: number
    swapUsed: number
    swapTotal: number
  }
  gpu?: {
    name: string
    usage: number
    memoryUsed: number
    memoryTotal: number
    temperature?: number
  }
  disk: {
    used: number
    total: number
    readSpeed: number
    writeSpeed: number
  }
  network: {
    bytesIn: number
    bytesOut: number
    connections: number
  }
  processes: ProcessInfo[]
}

export interface ProcessInfo {
  pid: number
  name: string
  cpu: number
  memory: number
  status: string
  uptime: number
}

// Report Types
export interface ExecutiveReport {
  id: string
  title: string
  type: ReportType
  generatedAt: Date
  generatedBy: string // agent codename
  summary: string
  sections: ReportSection[]
  metrics: Record<string, number>
  recommendations: string[]
}

export type ReportType =
  | "daily-summary"
  | "performance-report"
  | "efficiency-analysis"
  | "incident-report"
  | "documentation"

export interface ReportSection {
  title: string
  content: string
  charts?: ChartData[]
}

export interface ChartData {
  type: "line" | "bar" | "pie" | "area"
  data: Record<string, number>[]
  labels: string[]
}
