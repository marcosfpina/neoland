"use client"

import { Card, CardContent } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Bot, Zap, Clock, Target, Flame, TrendingUp } from "lucide-react"
import type { AgentIdentity } from "@/lib/agents/types"
import { getXPForNextLevel } from "@/lib/agents/registry"
import { cn } from "@/lib/utils"

interface AgentCardProps {
  agent: AgentIdentity
  compact?: boolean
  onClick?: () => void
}

const statusColors: Record<string, string> = {
  idle: "bg-slate-500",
  working: "bg-emerald-500 animate-pulse",
  paused: "bg-amber-500",
  error: "bg-red-500",
  offline: "bg-slate-700",
}

const roleColors: Record<string, string> = {
  orchestrator: "from-violet-500 to-purple-600",
  "code-intel": "from-cyan-500 to-blue-600",
  embeddings: "from-emerald-500 to-teal-600",
  monitor: "from-amber-500 to-orange-600",
  scribe: "from-pink-500 to-rose-600",
  "git-ops": "from-indigo-500 to-blue-600",
  "prompt-forge": "from-red-500 to-orange-600",
  inference: "from-fuchsia-500 to-purple-600",
}

export function AgentCard({ agent, compact = false, onClick }: AgentCardProps) {
  const xpToNext = getXPForNextLevel(agent.metrics.xp, agent.metrics.level)
  const xpProgress = (agent.metrics.xp % 1000) / 10 // Simplified progress

  if (compact) {
    return (
      <div
        className={cn(
          "flex items-center gap-3 p-3 rounded-lg bg-slate-800/50 border border-slate-700/50",
          "hover:bg-slate-800/70 transition-all cursor-pointer",
          onClick && "cursor-pointer",
        )}
        onClick={onClick}
      >
        <div
          className={cn(
            "w-10 h-10 rounded-lg bg-gradient-to-br flex items-center justify-center",
            roleColors[agent.role],
          )}
        >
          <Bot className="w-5 h-5 text-white" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-white font-medium text-sm">{agent.codename}</span>
            <div className={cn("w-2 h-2 rounded-full", statusColors[agent.status])} />
          </div>
          <div className="text-xs text-slate-400 truncate">{agent.metrics.tasksCompleted} tasks</div>
        </div>
        <div className="text-right">
          <div className="text-sm font-bold text-white">{agent.metrics.efficiencyScore}%</div>
          <div className="text-xs text-slate-500">efficiency</div>
        </div>
      </div>
    )
  }

  return (
    <Card
      className={cn(
        "bg-slate-800/50 border-slate-700/50 overflow-hidden",
        "hover:border-slate-600/50 transition-all",
        onClick && "cursor-pointer",
      )}
      onClick={onClick}
    >
      <div className={cn("h-1 bg-gradient-to-r", roleColors[agent.role])} />

      <CardContent className="p-4">
        {/* Header */}
        <div className="flex items-start justify-between mb-3">
          <div className="flex items-center gap-3">
            <div
              className={cn(
                "w-12 h-12 rounded-xl bg-gradient-to-br flex items-center justify-center shadow-lg",
                roleColors[agent.role],
              )}
            >
              <Bot className="w-6 h-6 text-white" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-white font-bold">{agent.codename}</h3>
                <Badge variant="outline" className="text-xs border-slate-600 text-slate-300">
                  Lv.{agent.metrics.level}
                </Badge>
              </div>
              <p className="text-xs text-slate-400">
                {agent.name} • {agent.role}
              </p>
            </div>
          </div>
          <div className={cn("w-3 h-3 rounded-full", statusColors[agent.status])} />
        </div>

        {/* Personality */}
        <p className="text-xs text-slate-400 mb-4 line-clamp-2">{agent.personality}</p>

        {/* Metrics Grid */}
        <div className="grid grid-cols-3 gap-2 mb-4">
          <div className="text-center p-2 bg-slate-900/50 rounded-lg">
            <div className="flex items-center justify-center gap-1 text-emerald-400 mb-1">
              <Target className="w-3 h-3" />
              <span className="text-sm font-bold">{agent.metrics.successRate.toFixed(1)}%</span>
            </div>
            <div className="text-[10px] text-slate-500">Success</div>
          </div>
          <div className="text-center p-2 bg-slate-900/50 rounded-lg">
            <div className="flex items-center justify-center gap-1 text-blue-400 mb-1">
              <Clock className="w-3 h-3" />
              <span className="text-sm font-bold">{(agent.metrics.avgResponseTime / 1000).toFixed(1)}s</span>
            </div>
            <div className="text-[10px] text-slate-500">Avg Time</div>
          </div>
          <div className="text-center p-2 bg-slate-900/50 rounded-lg">
            <div className="flex items-center justify-center gap-1 text-orange-400 mb-1">
              <Flame className="w-3 h-3" />
              <span className="text-sm font-bold">{agent.metrics.streak}</span>
            </div>
            <div className="text-[10px] text-slate-500">Streak</div>
          </div>
        </div>

        {/* XP Progress */}
        <div className="mb-3">
          <div className="flex justify-between text-xs mb-1">
            <span className="text-slate-400">XP Progress</span>
            <span className="text-slate-300">{agent.metrics.xp.toLocaleString()} XP</span>
          </div>
          <Progress value={xpProgress} className="h-1.5 bg-slate-700" />
          <div className="text-[10px] text-slate-500 mt-1">
            {xpToNext.toLocaleString()} XP to Level {agent.metrics.level + 1}
          </div>
        </div>

        {/* Stats Footer */}
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-1 text-slate-400">
            <TrendingUp className="w-3 h-3" />
            <span>{agent.metrics.tasksCompleted.toLocaleString()} tasks</span>
          </div>
          <div className="flex items-center gap-1">
            <Zap className="w-3 h-3 text-amber-400" />
            <span className="text-white font-bold">{agent.metrics.efficiencyScore}%</span>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
