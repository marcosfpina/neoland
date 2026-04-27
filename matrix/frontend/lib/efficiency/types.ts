// Efficiency Core - Real performance metrics and bottleneck analysis

export interface EfficiencyMetrics {
  timestamp: Date
  period: MetricPeriod

  // Time metrics
  timeMetrics: {
    totalCodingTime: number // minutes
    productiveTime: number // minutes
    idleTime: number // minutes
    meetingTime: number // minutes
    reviewTime: number // minutes
  }

  // Task metrics
  taskMetrics: {
    tasksCompleted: number
    tasksInProgress: number
    avgCompletionTime: number // minutes
    blockedTasks: number
    estimateAccuracy: number // percentage
  }

  // Resource metrics
  resourceMetrics: {
    avgCpuUsage: number
    avgMemoryUsage: number
    avgGpuUsage: number
    peakCpuUsage: number
    peakMemoryUsage: number
    networkBandwidth: number // MB
  }

  // Code metrics
  codeMetrics: {
    linesWritten: number
    linesDeleted: number
    commits: number
    pullRequests: number
    codeReviews: number
    bugsFiled: number
    bugsFixed: number
  }

  // AI metrics
  aiMetrics: {
    promptsExecuted: number
    tokensUsed: number
    avgResponseTime: number // ms
    successRate: number // percentage
    costEstimate: number // USD
  }
}

export type MetricPeriod = "hour" | "day" | "week" | "month"

export interface Bottleneck {
  id: string
  type: BottleneckType
  severity: "low" | "medium" | "high" | "critical"
  title: string
  description: string
  impact: string
  recommendation: string
  detectedAt: Date
  resolved: boolean
  metrics: Record<string, number>
}

export type BottleneckType =
  | "memory-pressure"
  | "cpu-saturation"
  | "disk-io"
  | "network-latency"
  | "process-blocking"
  | "resource-contention"
  | "inefficient-workflow"
  | "context-switching"

export interface EfficiencyGoal {
  id: string
  name: string
  metric: string
  target: number
  current: number
  unit: string
  deadline?: Date
  progress: number // percentage
  status: "on-track" | "at-risk" | "behind" | "achieved"
}

export interface ProductivityInsight {
  id: string
  type: "positive" | "negative" | "neutral"
  title: string
  description: string
  metric: string
  change: number // percentage
  period: string
  actionable: boolean
  suggestion?: string
}

export interface WorkSession {
  id: string
  startTime: Date
  endTime?: Date
  duration: number // minutes
  type: "coding" | "review" | "meeting" | "research" | "idle"
  productivity: number // 0-100
  tasks: string[]
  interruptions: number
}
