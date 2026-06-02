"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import {
  Network,
  Bot,
  Play,
  Pause,
  Square,
  ArrowLeft,
  Brain,
  Zap,
  GitBranch,
  Database,
  Code2,
  FileText,
  Settings,
  Activity,
  Clock,
  CheckCircle,
  AlertCircle,
  Users,
} from "lucide-react"
import Link from "next/link"
import { getBackendClient } from "@/lib/api"
import type { DecisionRequest, RankingResponse } from "@/lib/api"
import { ApprovalDialog } from "@/components/orchestra/approval-dialog"

export default function AgentOrchestra() {
  const [selectedWorkflow, setSelectedWorkflow] = useState("code-review")
  const [workflowStatus, setWorkflowStatus] = useState("idle")
  const [activeAgents, setActiveAgents] = useState([])
  const [workflowProgress, setWorkflowProgress] = useState(0)
  const [customTask, setCustomTask] = useState("")

  // Approval dialog state
  const [showApprovalDialog, setShowApprovalDialog] = useState(false)
  const [pendingRanking, setPendingRanking] = useState<RankingResponse | null>(null)
  const [pendingAgent, setPendingAgent] = useState<unknown>(null)
  const [approvalResolve, setApprovalResolve] = useState<((value: boolean) => void) | null>(null)

  const workflows = [
    {
      id: "code-review",
      name: "Complete Code Review",
      description: "Multi-agent code analysis with security, performance, and quality checks",
      agents: ["CodeAnalyzer", "SecurityScanner", "PerformanceOptimizer", "QualityAssessor"],
      estimatedTime: "3-5 minutes",
      complexity: "Medium",
    },
    {
      id: "api-development",
      name: "API Development Workflow",
      description: "End-to-end API development with testing, documentation, and deployment",
      agents: ["APIDesigner", "CodeGenerator", "TestGenerator", "DocumentationWriter", "DeploymentManager"],
      estimatedTime: "8-12 minutes",
      complexity: "High",
    },
    {
      id: "database-optimization",
      name: "Database Optimization",
      description: "Comprehensive database analysis and optimization workflow",
      agents: ["SchemaAnalyzer", "QueryOptimizer", "IndexAdvisor", "PerformanceMonitor"],
      estimatedTime: "5-8 minutes",
      complexity: "Medium",
    },
    {
      id: "git-workflow",
      name: "Git Workflow Automation",
      description: "Automated git operations with intelligent commit messages and PR management",
      agents: ["GitAnalyzer", "CommitGenerator", "BranchManager", "PRReviewer"],
      estimatedTime: "2-4 minutes",
      complexity: "Low",
    },
    {
      id: "full-stack-review",
      name: "Full-Stack Application Review",
      description: "Comprehensive review of frontend, backend, database, and infrastructure",
      agents: [
        "FrontendAnalyzer",
        "BackendAnalyzer",
        "DatabaseReviewer",
        "InfrastructureAuditor",
        "SecurityExpert",
        "PerformanceTester",
      ],
      estimatedTime: "15-20 minutes",
      complexity: "Very High",
    },
  ]

  const agents = [
    {
      id: "code-analyzer",
      name: "CodeAnalyzer",
      type: "Analysis",
      status: "idle",
      llm: "GPT-4o",
      specialization: "Code quality and structure analysis",
      efficiency: 94,
      tasksCompleted: 127,
    },
    {
      id: "security-scanner",
      name: "SecurityScanner",
      type: "Security",
      status: "idle",
      llm: "Claude-3.5",
      specialization: "Security vulnerability detection",
      efficiency: 91,
      tasksCompleted: 89,
    },
    {
      id: "performance-optimizer",
      name: "PerformanceOptimizer",
      type: "Performance",
      status: "idle",
      llm: "Gemini-Pro",
      specialization: "Performance bottleneck identification",
      efficiency: 88,
      tasksCompleted: 156,
    },
    {
      id: "api-designer",
      name: "APIDesigner",
      type: "Design",
      status: "idle",
      llm: "GPT-4o",
      specialization: "RESTful API design and architecture",
      efficiency: 92,
      tasksCompleted: 73,
    },
    {
      id: "test-generator",
      name: "TestGenerator",
      type: "Testing",
      status: "idle",
      llm: "Groq-Llama",
      specialization: "Automated test case generation",
      efficiency: 89,
      tasksCompleted: 203,
    },
    {
      id: "documentation-writer",
      name: "DocumentationWriter",
      type: "Documentation",
      status: "idle",
      llm: "Claude-3.5",
      specialization: "Technical documentation creation",
      efficiency: 96,
      tasksCompleted: 145,
    },
  ]

  const workflowHistory = [
    {
      id: 1,
      workflow: "Complete Code Review",
      status: "completed",
      duration: "4m 23s",
      agentsUsed: 4,
      timestamp: "2 hours ago",
      results: { issues: 12, suggestions: 8, security: 2 },
    },
    {
      id: 2,
      workflow: "API Development Workflow",
      status: "completed",
      duration: "11m 45s",
      agentsUsed: 5,
      timestamp: "5 hours ago",
      results: { endpoints: 8, tests: 24, docs: 1 },
    },
    {
      id: 3,
      workflow: "Database Optimization",
      status: "failed",
      duration: "2m 15s",
      agentsUsed: 2,
      timestamp: "1 day ago",
      results: { error: "Connection timeout" },
    },
  ]

  const executeWorkflow = async () => {
    setWorkflowStatus("running")
    setWorkflowProgress(0)

    const selectedWorkflowData = workflows.find((w) => w.id === selectedWorkflow)
    const workflowAgents = agents.filter((agent) =>
      selectedWorkflowData.agents.some((agentName) => agent.name === agentName),
    )

    setActiveAgents(workflowAgents.map((agent) => ({ ...agent, status: "active" })))

    const backendClient = getBackendClient()
    const totalSteps = workflowAgents.length

    // Execute workflow with backend ranking for each step
    for (let i = 0; i < workflowAgents.length; i++) {
      const agent = workflowAgents[i]

      try {
        // Create decision request for this workflow step
        const decision: DecisionRequest = {
          agent_id: agent.id,
          decision_type: "workflow_step",
          context: {
            workflow: selectedWorkflowData.name,
            workflow_id: selectedWorkflow,
            agent: agent.name,
            specialization: agent.specialization,
            step: i + 1,
            total_steps: totalSteps,
            llm: agent.llm,
          },
          proposed_action: `Execute ${agent.specialization}`,
          model_version: "v1",
        }

        // Call backend /rank endpoint
        const ranking: RankingResponse = await backendClient.rankDecision(decision)

        // Check if human review is required
        if (ranking.requires_human_review) {
          // Show approval dialog (component to be created)
          const approved = await requestApproval(ranking, agent)

          if (!approved) {
            setWorkflowStatus("paused")
            setActiveAgents(
              activeAgents.map((a) =>
                a.id === agent.id ? { ...a, status: "requires_approval" } : a
              )
            )
            return
          }
        }

        // Update progress
        setWorkflowProgress(((i + 1) / totalSteps) * 100)

        // Mark agent as completed
        setActiveAgents((prev) =>
          prev.map((a) => (a.id === agent.id ? { ...a, status: "completed" } : a))
        )

        // Small delay between steps for UX
        await new Promise((resolve) => setTimeout(resolve, 500))

      } catch (error) {
        console.error(`Error executing step for agent ${agent.name}:`, error)

        // Mark agent as failed
        setActiveAgents((prev) =>
          prev.map((a) => (a.id === agent.id ? { ...a, status: "failed" } : a))
        )

        // Continue with next agent (or stop?)
        // For now, continue
      }
    }

    setWorkflowStatus("completed")
  }

  // Show approval dialog and wait for user decision
  const requestApproval = (ranking: RankingResponse, agent: unknown): Promise<boolean> => {
    return new Promise((resolve) => {
      setPendingRanking(ranking)
      setPendingAgent(agent)
      setApprovalResolve(() => resolve)
      setShowApprovalDialog(true)
    })
  }

  // Handle approval
  const handleApprove = () => {
    setShowApprovalDialog(false)
    if (approvalResolve) {
      approvalResolve(true)
    }
  }

  // Handle rejection
  const handleReject = () => {
    setShowApprovalDialog(false)
    if (approvalResolve) {
      approvalResolve(false)
    }
  }

  const stopWorkflow = () => {
    setWorkflowStatus("stopped")
    setActiveAgents(activeAgents.map((agent) => ({ ...agent, status: "stopped" })))
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-cyan-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <Link href="/">
            <Button variant="outline" size="sm" className="border-slate-600">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Hub
            </Button>
          </Link>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-cyan-500 rounded-lg">
              <Network className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-white">Agent Orchestra</h1>
              <p className="text-gray-300">Coordinate multiple AI agents for complex development workflows</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Workflow Control */}
          <div className="lg:col-span-2">
            <Tabs defaultValue="workflows" className="w-full">
              <TabsList className="grid w-full grid-cols-4 bg-slate-800/50">
                <TabsTrigger value="workflows" className="data-[state=active]:bg-slate-700">
                  Workflows
                </TabsTrigger>
                <TabsTrigger value="execution" className="data-[state=active]:bg-slate-700">
                  Execution
                </TabsTrigger>
                <TabsTrigger value="custom" className="data-[state=active]:bg-slate-700">
                  Custom Task
                </TabsTrigger>
                <TabsTrigger value="history" className="data-[state=active]:bg-slate-700">
                  History
                </TabsTrigger>
              </TabsList>

              <TabsContent value="workflows" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Settings className="w-5 h-5" />
                      Predefined Workflows
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Select a workflow to orchestrate multiple AI agents
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <Select value={selectedWorkflow} onValueChange={setSelectedWorkflow}>
                      <SelectTrigger className="bg-slate-700 border-slate-600">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {workflows.map((workflow) => (
                          <SelectItem key={workflow.id} value={workflow.id}>
                            {workflow.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>

                    {workflows
                      .filter((w) => w.id === selectedWorkflow)
                      .map((workflow) => (
                        <div key={workflow.id} className="p-4 bg-slate-700/30 rounded-lg">
                          <div className="flex items-start justify-between mb-3">
                            <div>
                              <h3 className="text-white font-medium">{workflow.name}</h3>
                              <p className="text-gray-300 text-sm mt-1">{workflow.description}</p>
                            </div>
                            <Badge
                              variant={
                                workflow.complexity === "Low"
                                  ? "secondary"
                                  : workflow.complexity === "Medium"
                                    ? "default"
                                    : workflow.complexity === "High"
                                      ? "destructive"
                                      : "destructive"
                              }
                            >
                              {workflow.complexity}
                            </Badge>
                          </div>

                          <div className="grid grid-cols-2 gap-4 mb-4">
                            <div>
                              <span className="text-gray-400 text-sm">Estimated Time:</span>
                              <div className="text-white">{workflow.estimatedTime}</div>
                            </div>
                            <div>
                              <span className="text-gray-400 text-sm">Agents Required:</span>
                              <div className="text-white">{workflow.agents.length}</div>
                            </div>
                          </div>

                          <div>
                            <span className="text-gray-400 text-sm mb-2 block">Agents in Workflow:</span>
                            <div className="flex flex-wrap gap-2">
                              {workflow.agents.map((agentName) => (
                                <Badge key={agentName} variant="outline" className="border-slate-600">
                                  <Bot className="w-3 h-3 mr-1" />
                                  {agentName}
                                </Badge>
                              ))}
                            </div>
                          </div>
                        </div>
                      ))}

                    <div className="flex gap-2">
                      <Button onClick={executeWorkflow} disabled={workflowStatus === "running"} className="flex-1">
                        {workflowStatus === "running" ? (
                          <>
                            <Pause className="w-4 h-4 mr-2" />
                            Running...
                          </>
                        ) : (
                          <>
                            <Play className="w-4 h-4 mr-2" />
                            Execute Workflow
                          </>
                        )}
                      </Button>
                      {workflowStatus === "running" && (
                        <Button variant="outline" onClick={stopWorkflow} className="border-slate-600">
                          <Square className="w-4 h-4" />
                        </Button>
                      )}
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="execution" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Activity className="w-5 h-5" />
                      Workflow Execution
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Real-time workflow progress and agent coordination
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    {/* Progress Bar */}
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-white font-medium">Overall Progress</span>
                        <span className="text-gray-300">{Math.round(workflowProgress)}%</span>
                      </div>
                      <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-gradient-to-r from-cyan-500 to-blue-500 transition-all duration-500"
                          style={{ width: `${workflowProgress}%` }}
                        ></div>
                      </div>
                    </div>

                    {/* Active Agents */}
                    {activeAgents.length > 0 && (
                      <div>
                        <h4 className="text-white font-medium mb-3">Active Agents</h4>
                        <div className="space-y-2">
                          {activeAgents.map((agent) => (
                            <div key={agent.id} className="flex items-center gap-3 p-3 bg-slate-700/30 rounded-lg">
                              <div className="w-8 h-8 bg-gradient-to-r from-cyan-500 to-blue-500 rounded-lg flex items-center justify-center">
                                <Bot className="w-4 h-4 text-white" />
                              </div>
                              <div className="flex-1">
                                <div className="text-white font-medium">{agent.name}</div>
                                <div className="text-gray-400 text-sm">{agent.specialization}</div>
                              </div>
                              <div className="flex items-center gap-2">
                                <Badge variant="outline" className="text-xs border-slate-600">
                                  {agent.llm}
                                </Badge>
                                <div
                                  className={`w-2 h-2 rounded-full ${agent.status === "active"
                                    ? "bg-green-500 animate-pulse"
                                    : agent.status === "completed"
                                      ? "bg-blue-500"
                                      : agent.status === "stopped"
                                        ? "bg-red-500"
                                        : "bg-gray-500"
                                    }`}
                                ></div>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Workflow Status */}
                    <div className="p-4 bg-slate-700/30 rounded-lg">
                      <div className="flex items-center gap-2 mb-2">
                        {workflowStatus === "running" ? (
                          <Activity className="w-5 h-5 text-green-500 animate-pulse" />
                        ) : workflowStatus === "completed" ? (
                          <CheckCircle className="w-5 h-5 text-blue-500" />
                        ) : workflowStatus === "stopped" ? (
                          <AlertCircle className="w-5 h-5 text-red-500" />
                        ) : (
                          <Clock className="w-5 h-5 text-gray-500" />
                        )}
                        <span className="text-white font-medium">
                          Status: {workflowStatus.charAt(0).toUpperCase() + workflowStatus.slice(1)}
                        </span>
                      </div>
                      <p className="text-gray-300 text-sm">
                        {workflowStatus === "idle" && "Ready to execute workflow"}
                        {workflowStatus === "running" && "Agents are working on your task..."}
                        {workflowStatus === "completed" && "Workflow completed successfully!"}
                        {workflowStatus === "stopped" && "Workflow was stopped by user"}
                      </p>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="custom" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Brain className="w-5 h-5" />
                      Custom Task Orchestration
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Describe a task and let AI orchestrate the appropriate agents
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <Textarea
                      value={customTask}
                      onChange={(e) => setCustomTask(e.target.value)}
                      placeholder="Describe your development task... e.g., 'Review my React application for security issues and performance problems, then generate comprehensive documentation'"
                      className="min-h-32 bg-slate-700 border-slate-600 text-white"
                    />

                    <div className="p-4 bg-slate-700/30 rounded-lg">
                      <h4 className="text-white font-medium mb-2">🤖 AI Task Analysis</h4>
                      {customTask ? (
                        <div className="space-y-2 text-sm">
                          <p className="text-gray-300">
                            Based on your task description, I recommend the following agent orchestration:
                          </p>
                          <div className="flex flex-wrap gap-2">
                            <Badge variant="outline" className="border-slate-600">
                              <Bot className="w-3 h-3 mr-1" />
                              SecurityScanner
                            </Badge>
                            <Badge variant="outline" className="border-slate-600">
                              <Bot className="w-3 h-3 mr-1" />
                              PerformanceOptimizer
                            </Badge>
                            <Badge variant="outline" className="border-slate-600">
                              <Bot className="w-3 h-3 mr-1" />
                              DocumentationWriter
                            </Badge>
                          </div>
                          <p className="text-gray-400 text-xs">Estimated time: 6-9 minutes • Complexity: Medium</p>
                        </div>
                      ) : (
                        <p className="text-gray-400 text-sm">Enter a task description to see AI recommendations</p>
                      )}
                    </div>

                    <Button disabled={!customTask.trim()} className="w-full">
                      <Zap className="w-4 h-4 mr-2" />
                      Execute Custom Task
                    </Button>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="history" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Clock className="w-5 h-5" />
                      Workflow History
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Previous workflow executions and results
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      {workflowHistory.map((execution) => (
                        <div key={execution.id} className="p-4 bg-slate-700/30 rounded-lg">
                          <div className="flex items-center justify-between mb-2">
                            <div className="flex items-center gap-2">
                              <h4 className="text-white font-medium">{execution.workflow}</h4>
                              {execution.status === "completed" ? (
                                <CheckCircle className="w-4 h-4 text-green-500" />
                              ) : (
                                <AlertCircle className="w-4 h-4 text-red-500" />
                              )}
                            </div>
                            <span className="text-gray-400 text-sm">{execution.timestamp}</span>
                          </div>
                          <div className="grid grid-cols-3 gap-4 text-sm">
                            <div>
                              <span className="text-gray-400">Duration:</span>
                              <div className="text-white">{execution.duration}</div>
                            </div>
                            <div>
                              <span className="text-gray-400">Agents:</span>
                              <div className="text-white">{execution.agentsUsed}</div>
                            </div>
                            <div>
                              <span className="text-gray-400">Status:</span>
                              <div className={execution.status === "completed" ? "text-green-400" : "text-red-400"}>
                                {execution.status}
                              </div>
                            </div>
                          </div>
                          {execution.results && (
                            <div className="mt-2 p-2 bg-slate-600/30 rounded text-xs">
                              <span className="text-gray-400">Results: </span>
                              <span className="text-white">
                                {execution.results.error ||
                                  Object.entries(execution.results)
                                    .map(([key, value]) => `${key}: ${value}`)
                                    .join(", ")}
                              </span>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </div>

          {/* Agent Status Sidebar */}
          <div className="space-y-6">
            {/* Available Agents */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Users className="w-5 h-5" />
                  Available Agents
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {agents.map((agent) => (
                    <div key={agent.id} className="p-3 bg-slate-700/30 rounded-lg">
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                          <Bot className="w-4 h-4 text-cyan-400" />
                          <span className="text-white font-medium text-sm">{agent.name}</span>
                        </div>
                        <div className="w-2 h-2 bg-gray-500 rounded-full"></div>
                      </div>
                      <div className="space-y-1 text-xs">
                        <div className="flex justify-between text-gray-400">
                          <span>Type:</span>
                          <span>{agent.type}</span>
                        </div>
                        <div className="flex justify-between text-gray-400">
                          <span>LLM:</span>
                          <span>{agent.llm}</span>
                        </div>
                        <div className="flex justify-between text-gray-400">
                          <span>Efficiency:</span>
                          <span>{agent.efficiency}%</span>
                        </div>
                        <div className="flex justify-between text-gray-400">
                          <span>Tasks:</span>
                          <span>{agent.tasksCompleted}</span>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Orchestra Stats */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Activity className="w-5 h-5" />
                  Orchestra Stats
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="text-center">
                    <div className="text-2xl font-bold text-white">847</div>
                    <div className="text-sm text-gray-400">Total Tasks Completed</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-cyan-400">92%</div>
                    <div className="text-sm text-gray-400">Success Rate</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-green-400">6.2m</div>
                    <div className="text-sm text-gray-400">Avg Completion Time</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-purple-400">3.4</div>
                    <div className="text-sm text-gray-400">Avg Agents per Task</div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Quick Actions */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Zap className="w-5 h-5" />
                  Quick Actions
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <Button variant="outline" size="sm" className="w-full border-slate-600">
                    <Code2 className="w-4 h-4 mr-2" />
                    Quick Code Review
                  </Button>
                  <Button variant="outline" size="sm" className="w-full border-slate-600">
                    <Database className="w-4 h-4 mr-2" />
                    DB Health Check
                  </Button>
                  <Button variant="outline" size="sm" className="w-full border-slate-600">
                    <GitBranch className="w-4 h-4 mr-2" />
                    Git Cleanup
                  </Button>
                  <Button variant="outline" size="sm" className="w-full border-slate-600">
                    <FileText className="w-4 h-4 mr-2" />
                    Generate Docs
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>

      {/* Approval Dialog */}
      {pendingRanking && pendingAgent && (
        <ApprovalDialog
          open={showApprovalDialog}
          ranking={pendingRanking}
          agent={pendingAgent}
          onApprove={handleApprove}
          onReject={handleReject}
        />
      )}
    </div>
  )
}
