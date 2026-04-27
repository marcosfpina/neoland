"use client"

import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  Command,
  Bot,
  GitBranch,
  Database,
  Sparkles,
  Activity,
  FileText,
  Terminal,
  Zap,
  Trophy,
  Clock,
  ArrowRight,
} from "lucide-react"
import Link from "next/link"
import { AgentCard } from "@/components/agents/agent-card"
import { AgentLeaderboard } from "@/components/agents/agent-leaderboard"
import { ResourceMonitor } from "@/components/system/resource-monitor"
import { getAllAgents, getActiveAgents } from "@/lib/agents/registry"

export default function MissionControl() {
  const [selectedTab, setSelectedTab] = useState("overview")
  const agents = getAllAgents()
  const activeAgents = getActiveAgents()

  const modules = [
    {
      id: "pipeline",
      title: "Pipeline Intelligence",
      description: "Live execution tracking and agent progression monitoring",
      icon: Terminal,
      color: "from-emerald-500 to-teal-600",
      href: "/pipeline",
      agent: "ORCHESTRATOR",
      status: "active",
    },
    {
      id: "sessions",
      title: "Forensic Sessions",
      description: "Deep inspection of session state, history, and variables",
      icon: Activity,
      color: "from-blue-500 to-indigo-600",
      href: "/sessions",
      agent: "CLIENT",
      status: "ready",
    },
    {
      id: "adr-vault",
      title: "ADR Vault",
      description: "Architecture Decision Records and signed checkpoints",
      icon: FileText,
      color: "from-violet-500 to-purple-600",
      href: "/adr",
      agent: "LEDGER",
      status: "ready",
    },
    {
      id: "services",
      title: "Service Health",
      description: "Ecosystem status, NATS persistence, and control-plane health",
      icon: Zap,
      color: "from-pink-500 to-rose-600",
      href: "/services",
      agent: "HEALTH",
      status: "ready",
    },
  ]

  // Calculate totals
  const totalTasks = agents.reduce((sum, a) => sum + a.metrics.tasksCompleted, 0)
  const avgEfficiency = Math.round(agents.reduce((sum, a) => sum + a.metrics.efficiencyScore, 0) / agents.length)
  const totalXP = agents.reduce((sum, a) => sum + a.metrics.xp, 0)

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-gradient-to-br from-primary to-purple-600 rounded-lg">
                <Command className="w-5 h-5 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-foreground font-bold">Neoland Orchestrator</h1>
                <p className="text-xs text-muted-foreground">Operational Control Plane</p>
              </div>
            </div>

            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2 px-3 py-1.5 bg-card rounded-full">
                <div className="w-2 h-2 bg-emerald-500 rounded-full animate-pulse" />
                <span className="text-xs text-muted-foreground">{activeAgents.length} agents active</span>
              </div>
              <Button variant="outline" size="sm">
                <Terminal className="w-4 h-4 mr-2" />
                Console
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        {/* System Resources */}
        <section className="mb-8">
          <ResourceMonitor />
        </section>

        {/* Quick Stats */}
        <section className="grid grid-cols-4 gap-4 mb-8">
          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-primary/20 rounded-lg">
                  <Bot className="w-5 h-5 text-primary" />
                </div>
                <div>
                  <div className="text-2xl font-bold text-foreground">{agents.length}</div>
                  <div className="text-xs text-muted-foreground">Active Agents</div>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-accent/20 rounded-lg">
                  <Zap className="w-5 h-5 text-accent" />
                </div>
                <div>
                  <div className="text-2xl font-bold text-foreground">{avgEfficiency}%</div>
                  <div className="text-xs text-muted-foreground">Avg Efficiency</div>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-secondary/20 rounded-lg">
                  <Trophy className="w-5 h-5 text-secondary" />
                </div>
                <div>
                  <div className="text-2xl font-bold text-foreground">{totalXP.toLocaleString()}</div>
                  <div className="text-xs text-muted-foreground">Total XP</div>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-destructive/20 rounded-lg">
                  <Clock className="w-5 h-5 text-destructive" />
                </div>
                <div>
                  <div className="text-2xl font-bold text-foreground">{totalTasks.toLocaleString()}</div>
                  <div className="text-xs text-muted-foreground">Tasks Completed</div>
                </div>
              </div>
            </CardContent>
          </Card>
        </section>

        {/* Main Content */}
        <div className="grid grid-cols-3 gap-6">
          {/* Modules Grid */}
          <div className="col-span-2">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-foreground">Operational Modules</h2>
              <Badge variant="outline">{modules.length} modules</Badge>
            </div>

            <div className="grid grid-cols-2 gap-4">
              {modules.map((module) => {
                const Icon = module.icon
                return (
                  <Link key={module.id} href={module.href}>
                    <Card className="bg-card border-border hover:border-primary transition-all group cursor-pointer h-full">
                      <CardContent className="p-5">
                        <div className="flex items-start justify-between mb-3">
                          <div
                            className={`p-2.5 bg-gradient-to-br ${module.color} rounded-xl shadow-lg group-hover:scale-110 transition-transform`}
                          >
                            <Icon className="w-5 h-5 text-white" />
                          </div>
                          <div className="flex items-center gap-2">
                            <Badge variant="outline">{module.agent}</Badge>
                            {module.status === "active" && (
                              <div className="w-2 h-2 bg-emerald-500 rounded-full animate-pulse" />
                            )}
                          </div>
                        </div>
                        <h3 className="text-foreground font-semibold mb-1">{module.title}</h3>
                        <p className="text-sm text-muted-foreground mb-3">{module.description}</p>
                        <div className="flex items-center text-xs text-muted-foreground group-hover:text-primary transition-colors">
                          Open module
                          <ArrowRight className="w-3 h-3 ml-1 group-hover:translate-x-1 transition-transform" />
                        </div>
                      </CardContent>
                    </Card>
                  </Link>
                )
              })}
            </div>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Agent Leaderboard */}
            <AgentLeaderboard sortBy="efficiency" />

            {/* Quick Agent Status */}
            <Card className="bg-card border-border">
              <CardHeader className="pb-3">
                <CardTitle className="text-foreground flex items-center gap-2 text-lg">
                  <Bot className="w-5 h-5 text-primary" />
                  Agent Squad
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {agents.slice(0, 4).map((agent) => (
                  <AgentCard key={agent.id} agent={agent} compact />
                ))}
                <Link href="/agents">
                  <Button variant="ghost" className="w-full">
                    View All Agents
                    <ArrowRight className="w-4 h-4 ml-2" />
                  </Button>
                </Link>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>
    </div>
  )
}
