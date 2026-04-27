import { Cpu, GitBranch, Lightbulb, ShieldCheck } from "lucide-react"

import { AgentHistoryChart } from "@/components/agents/agent-history-chart"
import { MetricCard } from "@/components/shared/metric-card"
import { PageHeader } from "@/components/shared/page-header"
import { Card, CardContent } from "@/components/ui/card"
import { getAgentAnalytics } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

const iconMap = {
  junior: Lightbulb,
  senior: GitBranch,
  architect: Cpu,
  tech_leader: ShieldCheck,
} as const

export default async function AgentsPage() {
  const analytics = await getAgentAnalytics()

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Agents"
        title="Stage analytics"
        description="These metrics are derived from real ADR history, not synthetic telemetry. Empty states here simply mean the vault has not accumulated enough pipeline output yet."
      />

      <div className="grid gap-4 xl:grid-cols-4">
        {analytics.cards.map((card) => {
          const Icon = iconMap[card.stage]
          return (
            <MetricCard
              key={card.stage}
              label={card.label}
              value={card.primaryMetricValue}
              detail={`${card.secondaryMetricLabel}: ${card.secondaryMetricValue} · runs: ${card.totalRuns}`}
              icon={<Icon className="size-5" />}
              tone={card.stage === "tech_leader" ? "approve" : card.stage === "architect" ? "accent" : "default"}
            />
          )
        })}
      </div>

      <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
        <CardContent className="space-y-5 p-6">
          <div>
            <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
              Junior confidence
            </div>
            <div className="mt-2 text-2xl font-semibold tracking-tight text-foreground">
              Recent checkpoint history
            </div>
          </div>
          <AgentHistoryChart data={analytics.timeline} />
        </CardContent>
      </Card>
    </div>
  )
}
