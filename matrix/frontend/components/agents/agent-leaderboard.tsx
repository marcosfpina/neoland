"use client"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Trophy, Medal, Award, Zap, Flame, Target } from "lucide-react"
import { getAllAgents } from "@/lib/agents/registry"
import { cn } from "@/lib/utils"

type SortKey = "efficiency" | "tasks" | "streak" | "xp"

interface AgentLeaderboardProps {
  sortBy?: SortKey
}

export function AgentLeaderboard({ sortBy = "efficiency" }: AgentLeaderboardProps) {
  const agents = getAllAgents()

  const sortedAgents = [...agents].sort((a, b) => {
    switch (sortBy) {
      case "efficiency":
        return b.metrics.efficiencyScore - a.metrics.efficiencyScore
      case "tasks":
        return b.metrics.tasksCompleted - a.metrics.tasksCompleted
      case "streak":
        return b.metrics.streak - a.metrics.streak
      case "xp":
        return b.metrics.xp - a.metrics.xp
      default:
        return 0
    }
  })

  const rankIcons = [
    <Trophy key="1" className="w-5 h-5 text-amber-400" />,
    <Medal key="2" className="w-5 h-5 text-slate-300" />,
    <Award key="3" className="w-5 h-5 text-amber-600" />,
  ]

  const getValue = (agent: (typeof agents)[0]) => {
    switch (sortBy) {
      case "efficiency":
        return `${agent.metrics.efficiencyScore}%`
      case "tasks":
        return agent.metrics.tasksCompleted.toLocaleString()
      case "streak":
        return agent.metrics.streak.toString()
      case "xp":
        return agent.metrics.xp.toLocaleString()
    }
  }

  const getIcon = () => {
    switch (sortBy) {
      case "efficiency":
        return <Zap className="w-4 h-4" />
      case "tasks":
        return <Target className="w-4 h-4" />
      case "streak":
        return <Flame className="w-4 h-4" />
      case "xp":
        return <Trophy className="w-4 h-4" />
    }
  }

  return (
    <Card className="bg-slate-800/50 border-slate-700/50">
      <CardHeader className="pb-3">
        <CardTitle className="text-white flex items-center gap-2 text-lg">
          <Trophy className="w-5 h-5 text-amber-400" />
          Agent Leaderboard
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        {sortedAgents.map((agent, index) => (
          <div
            key={agent.id}
            className={cn(
              "flex items-center gap-3 p-3 rounded-lg transition-colors",
              index === 0 && "bg-amber-500/10 border border-amber-500/20",
              index === 1 && "bg-slate-500/10 border border-slate-500/20",
              index === 2 && "bg-amber-700/10 border border-amber-700/20",
              index > 2 && "bg-slate-900/30",
            )}
          >
            <div className="w-8 flex justify-center">
              {index < 3 ? rankIcons[index] : <span className="text-slate-500 font-mono text-sm">#{index + 1}</span>}
            </div>

            <div className="flex-1">
              <div className="flex items-center gap-2">
                <span className="text-white font-medium">{agent.codename}</span>
                <Badge variant="outline" className="text-xs border-slate-600 text-slate-400">
                  Lv.{agent.metrics.level}
                </Badge>
              </div>
              <div className="text-xs text-slate-500">{agent.name}</div>
            </div>

            <div className="flex items-center gap-2 text-right">
              {getIcon()}
              <span
                className={cn(
                  "font-bold",
                  index === 0 && "text-amber-400",
                  index === 1 && "text-slate-300",
                  index === 2 && "text-amber-600",
                  index > 2 && "text-white",
                )}
              >
                {getValue(agent)}
              </span>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
