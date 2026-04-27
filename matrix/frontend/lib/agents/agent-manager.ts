// Agent Manager - Controls agent lifecycle and task execution

import { AGENT_REGISTRY, getAgentByCodename, getAllAgents } from "./registry"
import type { AgentIdentity, AgentTask, TaskType, TaskPriority, AgentStatus, AgentMetrics } from "./types"

class AgentManager {
  private taskQueue: AgentTask[] = []
  private runningTasks: Map<string, AgentTask> = new Map()
  private taskHistory: AgentTask[] = []
  private listeners: Set<(event: AgentEvent) => void> = new Set()

  // Assign task to best available agent
  assignTask(
    type: TaskType,
    input: Record<string, unknown>,
    priority: TaskPriority = "medium",
    preferredAgent?: string,
  ): AgentTask | null {
    const agent = preferredAgent ? getAgentByCodename(preferredAgent) : this.selectBestAgent(type)

    if (!agent) return null

    const task: AgentTask = {
      id: `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      agentId: agent.id,
      type,
      status: "queued",
      priority,
      input,
      startedAt: new Date(),
    }

    this.taskQueue.push(task)
    this.emit({ type: "task-queued", task, agent })

    // Auto-start if agent is idle
    if (agent.status === "idle") {
      this.startNextTask(agent.id)
    }

    return task
  }

  // Select best agent for task type
  private selectBestAgent(type: TaskType): AgentIdentity | undefined {
    const roleMap: Record<TaskType, string> = {
      "code-analysis": "cipher",
      "doc-generation": "scribe",
      "report-creation": "scribe",
      "git-operation": "pilot",
      "prompt-generation": "forge",
      "model-inference": "oracle",
      "system-monitor": "sentinel",
      "workflow-orchestration": "atlas",
    }

    const codename = roleMap[type]
    const agent = getAgentByCodename(codename)

    if (agent && agent.status !== "offline" && agent.status !== "error") {
      return agent
    }

    // Fallback: find any available agent
    return getAllAgents().find((a) => a.status === "idle" || a.status === "paused")
  }

  // Start next queued task for agent
  private startNextTask(agentId: string): void {
    const task = this.taskQueue.find((t) => t.agentId === agentId && t.status === "queued")
    if (!task) return

    task.status = "running"
    this.runningTasks.set(task.id, task)
    this.updateAgentStatus(agentId, "working")

    this.emit({ type: "task-started", task })
  }

  // Complete a task
  completeTask(taskId: string, output: Record<string, unknown>, tokensUsed?: number): void {
    const task = this.runningTasks.get(taskId)
    if (!task) return

    task.status = "completed"
    task.output = output
    task.completedAt = new Date()
    task.duration = task.completedAt.getTime() - task.startedAt.getTime()
    task.tokensUsed = tokensUsed

    this.runningTasks.delete(taskId)
    this.taskHistory.push(task)

    // Update agent metrics
    this.updateAgentMetrics(task.agentId, true, task.duration, tokensUsed)
    this.updateAgentStatus(task.agentId, "idle")

    this.emit({ type: "task-completed", task })

    // Start next task if available
    this.startNextTask(task.agentId)
  }

  // Fail a task
  failTask(taskId: string, error: string): void {
    const task = this.runningTasks.get(taskId)
    if (!task) return

    task.status = "failed"
    task.error = error
    task.completedAt = new Date()
    task.duration = task.completedAt.getTime() - task.startedAt.getTime()

    this.runningTasks.delete(taskId)
    this.taskHistory.push(task)

    this.updateAgentMetrics(task.agentId, false, task.duration)
    this.updateAgentStatus(task.agentId, "idle")

    this.emit({ type: "task-failed", task })
  }

  // Update agent status
  private updateAgentStatus(agentId: string, status: AgentStatus): void {
    const agent = Object.values(AGENT_REGISTRY).find((a) => a.id === agentId)
    if (agent) {
      agent.status = status
      agent.lastActive = new Date()
      this.emit({ type: "agent-status-changed", agent })
    }
  }

  // Update agent metrics after task
  private updateAgentMetrics(agentId: string, success: boolean, duration?: number, tokensUsed?: number): void {
    const agent = Object.values(AGENT_REGISTRY).find((a) => a.id === agentId)
    if (!agent) return

    const metrics = agent.metrics
    metrics.tasksCompleted++

    if (success) {
      metrics.streak++
      metrics.xp += this.calculateXPReward(duration)
    } else {
      metrics.streak = 0
      metrics.lastError = new Date().toISOString()
    }

    // Recalculate success rate
    const total = metrics.tasksCompleted
    const successful = Math.round(total * (metrics.successRate / 100))
    metrics.successRate = ((successful + (success ? 1 : 0)) / (total + 1)) * 100

    // Update average response time
    if (duration) {
      metrics.avgResponseTime = Math.round((metrics.avgResponseTime * (total - 1) + duration) / total)
    }

    if (tokensUsed) {
      metrics.totalTokensUsed += tokensUsed
    }

    // Recalculate efficiency score
    metrics.efficiencyScore = this.calculateEfficiency(metrics)
  }

  private calculateXPReward(duration?: number): number {
    const baseXP = 100
    // Faster completion = more XP (up to 2x bonus)
    const speedBonus = duration && duration < 2000 ? 2 : duration && duration < 5000 ? 1.5 : 1
    return Math.round(baseXP * speedBonus)
  }

  private calculateEfficiency(metrics: AgentMetrics): number {
    // Weighted formula: 40% success rate, 30% speed, 20% streak, 10% uptime
    const speedScore = Math.max(0, 100 - metrics.avgResponseTime / 100)
    const streakScore = Math.min(100, metrics.streak * 2)
    const uptimeScore = Math.min(100, metrics.uptimeHours / 50)

    return Math.round(metrics.successRate * 0.4 + speedScore * 0.3 + streakScore * 0.2 + uptimeScore * 0.1)
  }

  // Event system
  private emit(event: AgentEvent): void {
    this.listeners.forEach((listener) => listener(event))
  }

  subscribe(listener: (event: AgentEvent) => void): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  // Getters
  getTaskQueue(): AgentTask[] {
    return [...this.taskQueue]
  }

  getRunningTasks(): AgentTask[] {
    return Array.from(this.runningTasks.values())
  }

  getTaskHistory(limit = 50): AgentTask[] {
    return this.taskHistory.slice(-limit)
  }

  getAgentTasks(agentId: string): AgentTask[] {
    return this.taskHistory.filter((t) => t.agentId === agentId)
  }
}

export type AgentEvent =
  | { type: "task-queued"; task: AgentTask; agent: AgentIdentity }
  | { type: "task-started"; task: AgentTask }
  | { type: "task-completed"; task: AgentTask }
  | { type: "task-failed"; task: AgentTask }
  | { type: "agent-status-changed"; agent: AgentIdentity }

export const agentManager = new AgentManager()
