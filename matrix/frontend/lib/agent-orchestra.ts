// Agent orchestration system for coordinating multiple AI agents

export interface Agent {
  id: string
  name: string
  type: AgentType
  llmProvider: string
  model: string
  specialization: string
  capabilities: string[]
  status: AgentStatus
  efficiency: number
  tasksCompleted: number
  currentTask?: Task
}

export type AgentType = "analyzer" | "generator" | "reviewer" | "optimizer" | "tester" | "documenter" | "coordinator"

export type AgentStatus = "idle" | "active" | "busy" | "error" | "completed"

export interface Task {
  id: string
  type: TaskType
  description: string
  priority: Priority
  dependencies: string[]
  assignedAgent?: string
  status: TaskStatus
  input: unknown
  output?: unknown
  startTime?: Date
  endTime?: Date
  estimatedDuration: number
}

export type TaskType =
  | "code-analysis"
  | "code-generation"
  | "code-review"
  | "testing"
  | "documentation"
  | "optimization"
  | "security-scan"
  | "performance-analysis"

export type TaskStatus = "pending" | "assigned" | "in-progress" | "completed" | "failed"
export type Priority = "low" | "medium" | "high" | "critical"

export interface Workflow {
  id: string
  name: string
  description: string
  tasks: Task[]
  agents: string[]
  status: WorkflowStatus
  progress: number
  startTime?: Date
  endTime?: Date
  estimatedDuration: number
  results?: unknown
}

export type WorkflowStatus = "idle" | "running" | "paused" | "completed" | "failed"

export class AgentOrchestrator {
  private agents: Map<string, Agent> = new Map()
  private workflows: Map<string, Workflow> = new Map()
  private taskQueue: Task[] = []
  private activeWorkflows: Set<string> = new Set()

  constructor() {
    this.initializeAgents()
  }

  private initializeAgents() {
    const defaultAgents: Agent[] = [
      {
        id: "code-analyzer-1",
        name: "CodeAnalyzer",
        type: "analyzer",
        llmProvider: "openai",
        model: "gpt-4o",
        specialization: "Code quality and structure analysis",
        capabilities: ["static-analysis", "code-review", "best-practices"],
        status: "idle",
        efficiency: 94,
        tasksCompleted: 127,
      },
      {
        id: "security-scanner-1",
        name: "SecurityScanner",
        type: "analyzer",
        llmProvider: "anthropic",
        model: "claude-3-5-sonnet-20241022",
        specialization: "Security vulnerability detection",
        capabilities: ["security-analysis", "vulnerability-scan", "threat-detection"],
        status: "idle",
        efficiency: 91,
        tasksCompleted: 89,
      },
      {
        id: "performance-optimizer-1",
        name: "PerformanceOptimizer",
        type: "optimizer",
        llmProvider: "google",
        model: "gemini-1.5-pro",
        specialization: "Performance bottleneck identification",
        capabilities: ["performance-analysis", "optimization", "profiling"],
        status: "idle",
        efficiency: 88,
        tasksCompleted: 156,
      },
      {
        id: "test-generator-1",
        name: "TestGenerator",
        type: "generator",
        llmProvider: "groq",
        model: "llama-3.1-70b-versatile",
        specialization: "Automated test case generation",
        capabilities: ["test-generation", "unit-tests", "integration-tests"],
        status: "idle",
        efficiency: 89,
        tasksCompleted: 203,
      },
      {
        id: "documentation-writer-1",
        name: "DocumentationWriter",
        type: "documenter",
        llmProvider: "anthropic",
        model: "claude-3-5-sonnet-20241022",
        specialization: "Technical documentation creation",
        capabilities: ["documentation", "api-docs", "user-guides"],
        status: "idle",
        efficiency: 96,
        tasksCompleted: 145,
      },
      {
        id: "workflow-coordinator-1",
        name: "WorkflowCoordinator",
        type: "coordinator",
        llmProvider: "openai",
        model: "gpt-4o",
        specialization: "Multi-agent task coordination",
        capabilities: ["task-planning", "agent-coordination", "workflow-management"],
        status: "idle",
        efficiency: 92,
        tasksCompleted: 78,
      },
    ]

    defaultAgents.forEach((agent) => {
      this.agents.set(agent.id, agent)
    })
  }

  // Agent Management
  getAgent(id: string): Agent | undefined {
    return this.agents.get(id)
  }

  getAllAgents(): Agent[] {
    return Array.from(this.agents.values())
  }

  getAvailableAgents(type?: AgentType): Agent[] {
    return this.getAllAgents().filter((agent) => agent.status === "idle" && (!type || agent.type === type))
  }

  assignTask(agentId: string, task: Task): boolean {
    const agent = this.getAgent(agentId)
    if (!agent || agent.status !== "idle") {
      return false
    }

    agent.status = "active"
    agent.currentTask = task
    task.assignedAgent = agentId
    task.status = "assigned"
    task.startTime = new Date()

    return true
  }

  // Task Management
  createTask(
    type: TaskType,
    description: string,
    input: unknown,
    priority: Priority = "medium",
    dependencies: string[] = [],
  ): Task {
    const task: Task = {
      id: `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type,
      description,
      priority,
      dependencies,
      status: "pending",
      input,
      estimatedDuration: this.estimateTaskDuration(type),
    }

    this.taskQueue.push(task)
    return task
  }

  private estimateTaskDuration(type: TaskType): number {
    const durations = {
      "code-analysis": 120000, // 2 minutes
      "code-generation": 180000, // 3 minutes
      "code-review": 150000, // 2.5 minutes
      testing: 240000, // 4 minutes
      documentation: 300000, // 5 minutes
      optimization: 200000, // 3.5 minutes
      "security-scan": 180000, // 3 minutes
      "performance-analysis": 160000, // 2.5 minutes
    }
    return durations[type] || 120000
  }

  // Workflow Management
  createWorkflow(name: string, description: string, tasks: Omit<Task, "id">[]): Workflow {
    const workflowTasks = tasks.map((task) => ({
      ...task,
      id: `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      status: "pending" as TaskStatus,
    }))

    const workflow: Workflow = {
      id: `workflow-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      name,
      description,
      tasks: workflowTasks,
      agents: [],
      status: "idle",
      progress: 0,
      estimatedDuration: workflowTasks.reduce((total, task) => total + task.estimatedDuration, 0),
    }

    this.workflows.set(workflow.id, workflow)
    return workflow
  }

  async executeWorkflow(workflowId: string): Promise<void> {
    const workflow = this.workflows.get(workflowId)
    if (!workflow) {
      throw new Error(`Workflow ${workflowId} not found`)
    }

    workflow.status = "running"
    workflow.startTime = new Date()
    this.activeWorkflows.add(workflowId)

    try {
      await this.processWorkflowTasks(workflow)
      workflow.status = "completed"
      workflow.progress = 100
    } catch (error) {
      workflow.status = "failed"
      throw error
    } finally {
      workflow.endTime = new Date()
      this.activeWorkflows.delete(workflowId)
    }
  }

  private async processWorkflowTasks(workflow: Workflow): Promise<void> {
    const taskGraph = this.buildTaskDependencyGraph(workflow.tasks)
    const executionOrder = this.topologicalSort(taskGraph)

    for (const taskId of executionOrder) {
      const task = workflow.tasks.find((t) => t.id === taskId)
      if (!task) continue

      await this.executeTask(task)

      // Update workflow progress
      const completedTasks = workflow.tasks.filter((t) => t.status === "completed").length
      workflow.progress = (completedTasks / workflow.tasks.length) * 100
    }
  }

  private buildTaskDependencyGraph(tasks: Task[]): Map<string, string[]> {
    const graph = new Map<string, string[]>()

    tasks.forEach((task) => {
      graph.set(task.id, task.dependencies)
    })

    return graph
  }

  private topologicalSort(graph: Map<string, string[]>): string[] {
    const visited = new Set<string>()
    const result: string[] = []

    const visit = (nodeId: string) => {
      if (visited.has(nodeId)) return
      visited.add(nodeId)

      const dependencies = graph.get(nodeId) || []
      dependencies.forEach((depId) => visit(depId))

      result.push(nodeId)
    }

    graph.forEach((_, nodeId) => visit(nodeId))
    return result
  }

  private async executeTask(task: Task): Promise<void> {
    // Find the best agent for this task
    const suitableAgents = this.findSuitableAgents(task)
    if (suitableAgents.length === 0) {
      throw new Error(`No suitable agents found for task ${task.id}`)
    }

    const bestAgent = this.selectBestAgent(suitableAgents, task)

    if (!this.assignTask(bestAgent.id, task)) {
      throw new Error(`Failed to assign task ${task.id} to agent ${bestAgent.id}`)
    }

    // Simulate task execution
    task.status = "in-progress"

    try {
      // Here you would call the actual LLM API through the agent
      await this.simulateTaskExecution(task, bestAgent)

      task.status = "completed"
      task.endTime = new Date()
      bestAgent.status = "idle"
      bestAgent.currentTask = undefined
      bestAgent.tasksCompleted++
    } catch (error) {
      task.status = "failed"
      bestAgent.status = "error"
      throw error
    }
  }

  private findSuitableAgents(task: Task): Agent[] {
    return this.getAllAgents().filter((agent) => {
      if (agent.status !== "idle") return false

      // Check if agent has required capabilities for this task type
      const requiredCapabilities = this.getRequiredCapabilities(task.type)
      return requiredCapabilities.some((cap) => agent.capabilities.includes(cap))
    })
  }

  private getRequiredCapabilities(taskType: TaskType): string[] {
    const capabilityMap = {
      "code-analysis": ["static-analysis", "code-review"],
      "code-generation": ["code-generation", "programming"],
      "code-review": ["code-review", "best-practices"],
      testing: ["test-generation", "unit-tests"],
      documentation: ["documentation", "technical-writing"],
      optimization: ["optimization", "performance-analysis"],
      "security-scan": ["security-analysis", "vulnerability-scan"],
      "performance-analysis": ["performance-analysis", "profiling"],
    }
    return capabilityMap[taskType] || []
  }

  private selectBestAgent(agents: Agent[], task: Task): Agent {
    // Select agent based on efficiency, specialization match, and current workload
    return agents.reduce((best, current) => {
      const bestScore = this.calculateAgentScore(best, task)
      const currentScore = this.calculateAgentScore(current, task)
      return currentScore > bestScore ? current : best
    })
  }

  private calculateAgentScore(agent: Agent, task: Task): number {
    let score = agent.efficiency

    // Bonus for specialization match
    if (agent.specialization.toLowerCase().includes(task.type.replace("-", " "))) {
      score += 20
    }

    // Bonus for relevant capabilities
    const requiredCapabilities = this.getRequiredCapabilities(task.type)
    const matchingCapabilities = agent.capabilities.filter((cap) => requiredCapabilities.includes(cap)).length
    score += matchingCapabilities * 10

    return score
  }

  private async simulateTaskExecution(task: Task, agent: Agent): Promise<void> {
    // Simulate task execution time
    const executionTime = Math.random() * task.estimatedDuration * 0.5 + task.estimatedDuration * 0.5
    await new Promise((resolve) => setTimeout(resolve, executionTime))

    // Generate mock output based on task type
    task.output = this.generateMockTaskOutput(task, agent)
  }

  private generateMockTaskOutput(task: Task, agent: Agent): unknown {
    const outputs = {
      "code-analysis": {
        issues: Math.floor(Math.random() * 10) + 1,
        suggestions: Math.floor(Math.random() * 8) + 2,
        qualityScore: Math.floor(Math.random() * 30) + 70,
        details: `Code analysis completed by ${agent.name}`,
      },
      "security-scan": {
        vulnerabilities: Math.floor(Math.random() * 5),
        severity: ["low", "medium", "high"][Math.floor(Math.random() * 3)],
        recommendations: Math.floor(Math.random() * 6) + 2,
        details: `Security scan completed by ${agent.name}`,
      },
      "performance-analysis": {
        bottlenecks: Math.floor(Math.random() * 4) + 1,
        optimizations: Math.floor(Math.random() * 6) + 2,
        performanceGain: `${Math.floor(Math.random() * 40) + 10}%`,
        details: `Performance analysis completed by ${agent.name}`,
      },
    }

    return (
      outputs[task.type] || {
        status: "completed",
        details: `Task completed by ${agent.name}`,
        timestamp: new Date().toISOString(),
      }
    )
  }

  // Predefined Workflows
  createCodeReviewWorkflow(codeInput: string): Workflow {
    return this.createWorkflow(
      "Complete Code Review",
      "Multi-agent code analysis with security, performance, and quality checks",
      [
        {
          type: "code-analysis",
          description: "Analyze code structure and quality",
          input: { code: codeInput },
          priority: "high",
          dependencies: [],
          estimatedDuration: 120000,
        },
        {
          type: "security-scan",
          description: "Scan for security vulnerabilities",
          input: { code: codeInput },
          priority: "high",
          dependencies: [],
          estimatedDuration: 180000,
        },
        {
          type: "performance-analysis",
          description: "Analyze performance bottlenecks",
          input: { code: codeInput },
          priority: "medium",
          dependencies: [],
          estimatedDuration: 160000,
        },
        {
          type: "documentation",
          description: "Generate code documentation",
          input: { code: codeInput },
          priority: "low",
          dependencies: [],
          estimatedDuration: 200000,
        },
      ],
    )
  }

  createAPIWorkflow(apiSpec: unknown): Workflow {
    return this.createWorkflow(
      "API Development Workflow",
      "End-to-end API development with testing and documentation",
      [
        {
          type: "code-generation",
          description: "Generate API implementation",
          input: { spec: apiSpec },
          priority: "high",
          dependencies: [],
          estimatedDuration: 300000,
        },
        {
          type: "testing",
          description: "Generate comprehensive test suite",
          input: { spec: apiSpec },
          priority: "high",
          dependencies: [],
          estimatedDuration: 240000,
        },
        {
          type: "documentation",
          description: "Create API documentation",
          input: { spec: apiSpec },
          priority: "medium",
          dependencies: [],
          estimatedDuration: 180000,
        },
        {
          type: "security-scan",
          description: "Security review of API endpoints",
          input: { spec: apiSpec },
          priority: "high",
          dependencies: [],
          estimatedDuration: 200000,
        },
      ],
    )
  }

  // Analytics and Monitoring
  getWorkflowStats(): unknown {
    const allWorkflows = Array.from(this.workflows.values())
    const completedWorkflows = allWorkflows.filter((w) => w.status === "completed")

    return {
      totalWorkflows: allWorkflows.length,
      completedWorkflows: completedWorkflows.length,
      successRate: (completedWorkflows.length / allWorkflows.length) * 100,
      averageExecutionTime:
        completedWorkflows.reduce((sum, w) => {
          if (w.startTime && w.endTime) {
            return sum + (w.endTime.getTime() - w.startTime.getTime())
          }
          return sum
        }, 0) / completedWorkflows.length,
      activeWorkflows: this.activeWorkflows.size,
    }
  }

  getAgentStats(): unknown {
    const agents = this.getAllAgents()

    return {
      totalAgents: agents.length,
      activeAgents: agents.filter((a) => a.status === "active").length,
      averageEfficiency: agents.reduce((sum, a) => sum + a.efficiency, 0) / agents.length,
      totalTasksCompleted: agents.reduce((sum, a) => sum + a.tasksCompleted, 0),
      agentsByType: agents.reduce(
        (acc, agent) => {
          acc[agent.type] = (acc[agent.type] || 0) + 1
          return acc
        },
        {} as Record<string, number>,
      ),
    }
  }
}

export const agentOrchestrator = new AgentOrchestrator()
