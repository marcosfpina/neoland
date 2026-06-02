"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import {
  Activity,
  ArrowLeft,
  TrendingUp,
  Clock,
  Zap,
  Target,
  AlertTriangle,
  CheckCircle2,
  Brain,
  Cpu,
  MemoryStick,
  HardDrive,
  Wifi,
  Timer,
  Code,
  GitCommit,
  Bug,
  Lightbulb,
  Flame,
  DollarSign,
  Circle,
} from "lucide-react"
import Link from "next/link"
import { cn } from "@/lib/utils"
import type {
  EfficiencyMetrics,
  Bottleneck,
  EfficiencyGoal,
  ProductivityInsight,
  MetricPeriod,
} from "@/lib/efficiency/types"

// Simulated metrics data
function useEfficiencyMetrics(period: MetricPeriod): EfficiencyMetrics {
  const [metrics, setMetrics] = useState<EfficiencyMetrics>({
    timestamp: new Date(),
    period,
    timeMetrics: {
      totalCodingTime: 0,
      productiveTime: 0,
      idleTime: 0,
      meetingTime: 0,
      reviewTime: 0,
    },
    taskMetrics: {
      tasksCompleted: 0,
      tasksInProgress: 0,
      avgCompletionTime: 0,
      blockedTasks: 0,
      estimateAccuracy: 0,
    },
    resourceMetrics: {
      avgCpuUsage: 0,
      avgMemoryUsage: 0,
      avgGpuUsage: 0,
      peakCpuUsage: 0,
      peakMemoryUsage: 0,
      networkBandwidth: 0,
    },
    codeMetrics: {
      linesWritten: 0,
      linesDeleted: 0,
      commits: 0,
      pullRequests: 0,
      codeReviews: 0,
      bugsFiled: 0,
      bugsFixed: 0,
    },
    aiMetrics: {
      promptsExecuted: 0,
      tokensUsed: 0,
      avgResponseTime: 0,
      successRate: 0,
      costEstimate: 0,
    },
  })

  useEffect(() => {
    // Simulate loading metrics based on period
    const multiplier = period === "hour" ? 1 : period === "day" ? 8 : period === "week" ? 40 : 160

    setMetrics({
      timestamp: new Date(),
      period,
      timeMetrics: {
        totalCodingTime: Math.round(45 * multiplier),
        productiveTime: Math.round(38 * multiplier),
        idleTime: Math.round(7 * multiplier),
        meetingTime: Math.round(5 * multiplier),
        reviewTime: Math.round(12 * multiplier),
      },
      taskMetrics: {
        tasksCompleted: Math.round(3 * multiplier),
        tasksInProgress: 4,
        avgCompletionTime: 45,
        blockedTasks: 1,
        estimateAccuracy: 78,
      },
      resourceMetrics: {
        avgCpuUsage: 34,
        avgMemoryUsage: 62,
        avgGpuUsage: 28,
        peakCpuUsage: 89,
        peakMemoryUsage: 84,
        networkBandwidth: Math.round(120 * multiplier),
      },
      codeMetrics: {
        linesWritten: Math.round(247 * multiplier),
        linesDeleted: Math.round(89 * multiplier),
        commits: Math.round(2 * multiplier),
        pullRequests: Math.round(0.5 * multiplier),
        codeReviews: Math.round(1 * multiplier),
        bugsFiled: Math.round(0.3 * multiplier),
        bugsFixed: Math.round(0.8 * multiplier),
      },
      aiMetrics: {
        promptsExecuted: Math.round(15 * multiplier),
        tokensUsed: Math.round(45000 * multiplier),
        avgResponseTime: 1850,
        successRate: 94.2,
        costEstimate: Math.round(0.45 * multiplier * 100) / 100,
      },
    })
  }, [period])

  return metrics
}

// Bottleneck detection
const DETECTED_BOTTLENECKS: Bottleneck[] = [
  {
    id: "bn-1",
    type: "memory-pressure",
    severity: "medium",
    title: "Memory Pressure During LLM Inference",
    description: "Memory usage spikes to 84% when running large model inference, approaching ZRAM threshold.",
    impact: "Potential system slowdown and process termination by earlyoom.",
    recommendation: "Consider limiting concurrent model loads or upgrading to 32GB RAM.",
    detectedAt: new Date(Date.now() - 3600000),
    resolved: false,
    metrics: { peakMemory: 84, avgMemory: 62, threshold: 92 },
  },
  {
    id: "bn-2",
    type: "context-switching",
    severity: "low",
    title: "Frequent Context Switching",
    description: "Average of 12 context switches per hour detected, reducing deep work efficiency.",
    impact: "Estimated 15% productivity loss from task switching overhead.",
    recommendation: "Implement time-blocking and disable non-critical notifications.",
    detectedAt: new Date(Date.now() - 7200000),
    resolved: false,
    metrics: { switchesPerHour: 12, avgFocusTime: 23, targetFocusTime: 45 },
  },
  {
    id: "bn-3",
    type: "inefficient-workflow",
    severity: "high",
    title: "Repeated Manual Operations",
    description: "Same git workflow executed manually 8+ times today. Automation candidate detected.",
    impact: "Approximately 25 minutes wasted on repetitive tasks.",
    recommendation: "Create a git alias or script for this workflow pattern.",
    detectedAt: new Date(Date.now() - 1800000),
    resolved: false,
    metrics: { occurrences: 8, timePerOccurrence: 3.2, totalTimeWasted: 25.6 },
  },
]

// Goals
const EFFICIENCY_GOALS: EfficiencyGoal[] = [
  {
    id: "goal-1",
    name: "Reduce Context Switches",
    metric: "switches_per_hour",
    target: 6,
    current: 12,
    unit: "switches/hour",
    progress: 50,
    status: "at-risk",
  },
  {
    id: "goal-2",
    name: "Increase Productive Time",
    metric: "productive_hours",
    target: 6,
    current: 5.2,
    unit: "hours/day",
    progress: 87,
    status: "on-track",
  },
  {
    id: "goal-3",
    name: "Improve Estimate Accuracy",
    metric: "estimate_accuracy",
    target: 85,
    current: 78,
    unit: "%",
    progress: 92,
    status: "on-track",
  },
  {
    id: "goal-4",
    name: "Reduce Bug Rate",
    metric: "bugs_per_kloc",
    target: 2,
    current: 3.4,
    unit: "bugs/KLOC",
    progress: 59,
    status: "behind",
  },
]

// Insights
const PRODUCTIVITY_INSIGHTS: ProductivityInsight[] = [
  {
    id: "ins-1",
    type: "positive",
    title: "Peak Productivity Window Found",
    description: "Your most productive hours are 9-11 AM with 94% efficiency.",
    metric: "productivity",
    change: 23,
    period: "vs afternoon",
    actionable: true,
    suggestion: "Schedule complex tasks during morning hours.",
  },
  {
    id: "ins-2",
    type: "negative",
    title: "Increasing Review Time",
    description: "Code review time increased 34% this week.",
    metric: "review_time",
    change: -34,
    period: "vs last week",
    actionable: true,
    suggestion: "Consider implementing automated code review tools.",
  },
  {
    id: "ins-3",
    type: "positive",
    title: "AI Assistant ROI",
    description: "AI prompts saved estimated 2.3 hours today.",
    metric: "time_saved",
    change: 15,
    period: "vs manual work",
    actionable: false,
  },
  {
    id: "ins-4",
    type: "neutral",
    title: "Commit Frequency Stable",
    description: "Averaging 4.2 commits per day, consistent with baseline.",
    metric: "commits",
    change: 0,
    period: "vs baseline",
    actionable: false,
  },
]

const _severityColors = {
  low: "border-blue-500/30 bg-blue-500/10 text-blue-400",
  medium: "border-amber-500/30 bg-amber-500/10 text-amber-400",
  high: "border-orange-500/30 bg-orange-500/10 text-orange-400",
  critical: "border-red-500/30 bg-red-500/10 text-red-400",
}

const _statusColors = {
  "on-track": "text-emerald-400",
  "at-risk": "text-amber-400",
  behind: "text-red-400",
  achieved: "text-blue-400",
}

export default function EfficiencyCore() {
  const [period, setPeriod] = useState<MetricPeriod>("day")
  const metrics = useEfficiencyMetrics(period)

  const formatTime = (minutes: number) => {
    if (minutes < 60) return `${minutes}m`
    const hours = Math.floor(minutes / 60)
    const mins = minutes % 60
    return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`
  }

  const productivityScore =
    Math.round((metrics.timeMetrics.productiveTime / metrics.timeMetrics.totalCodingTime) * 100) || 0

  const efficiencyScore =
    Math.round(
      (metrics.taskMetrics.tasksCompleted / (metrics.taskMetrics.tasksCompleted + metrics.taskMetrics.blockedTasks)) *
        (metrics.taskMetrics.estimateAccuracy / 100) *
        100,
    ) || 0

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-secondary/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-4">
              <Link href="/">
                <Button variant="ghost" size="sm" className="text-muted-foreground">
                  <ArrowLeft className="w-4 h-4 mr-2" />
                  Back
                </Button>
              </Link>
              <div className="flex items-center gap-3">
                <div className="p-2 bg-gradient-to-br from-primary to-primary/80 rounded-lg">
                  <Activity className="w-5 h-5 text-primary-foreground" />
                </div>
                <div>
                  <h1 className="text-foreground font-bold">Efficiency Core</h1>
                  <p className="text-xs text-muted-foreground">Performance & Bottleneck Analysis</p>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <Select value={period} onValueChange={(v) => setPeriod(v as MetricPeriod)}>
                <SelectTrigger className="w-32 bg-accent border-border">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="hour">Last Hour</SelectItem>
                  <SelectItem value="day">Today</SelectItem>
                  <SelectItem value="week">This Week</SelectItem>
                  <SelectItem value="month">This Month</SelectItem>
                </SelectContent>
              </Select>
              <Badge variant="outline" className="border-primary/30 text-primary">
                <Activity className="w-3 h-3 mr-1" />
                ATLAS Agent
              </Badge>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6 space-y-6">
        {/* Score Cards */}
        <div className="grid grid-cols-4 gap-4">
          <Card className="bg-gradient-to-br from-primary/10 to-primary/20 border-primary/20">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-muted-foreground text-sm">Productivity Score</span>
                <Zap className="w-5 h-5 text-primary" />
              </div>
              <div className="text-3xl font-bold text-foreground mb-1">{productivityScore}%</div>
              <div className="flex items-center gap-1 text-xs text-primary">
                <TrendingUp className="w-3 h-3" />
                +5% vs yesterday
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-secondary/10 to-secondary/20 border-secondary/20">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-muted-foreground text-sm">Efficiency Score</span>
                <Target className="w-5 h-5 text-secondary" />
              </div>
              <div className="text-3xl font-bold text-foreground mb-1">{efficiencyScore}%</div>
              <div className="flex items-center gap-1 text-xs text-secondary">
                <TrendingUp className="w-3 h-3" />
                +2% vs last week
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-accent/10 to-accent/20 border-accent/20">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-muted-foreground text-sm">Deep Work Time</span>
                <Clock className="w-5 h-5 text-accent" />
              </div>
              <div className="text-3xl font-bold text-foreground mb-1">
                {formatTime(metrics.timeMetrics.productiveTime)}
              </div>
              <div className="flex items-center gap-1 text-xs text-accent">
                <Timer className="w-3 h-3" />
                of {formatTime(metrics.timeMetrics.totalCodingTime)} total
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-destructive/10 to-destructive/20 border-destructive/20">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-muted-foreground text-sm">Tasks Completed</span>
                <CheckCircle2 className="w-5 h-5 text-destructive" />
              </div>
              <div className="text-3xl font-bold text-foreground mb-1">{metrics.taskMetrics.tasksCompleted}</div>
              <div className="flex items-center gap-1 text-xs text-destructive">
                <Flame className="w-3 h-3" />
                {metrics.taskMetrics.avgCompletionTime}m avg time
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Main Content */}
        <div className="grid grid-cols-3 gap-6">
          {/* Metrics Details */}
          <div className="col-span-2 space-y-6">
            <Tabs defaultValue="time" className="w-full">
              <TabsList className="grid w-full grid-cols-5 bg-secondary/50 border border-border">
                <TabsTrigger value="time" className="data-[state=active]:bg-accent">
                  <Clock className="w-4 h-4 mr-2" />
                  Time
                </TabsTrigger>
                <TabsTrigger value="resources" className="data-[state=active]:bg-accent">
                  <Cpu className="w-4 h-4 mr-2" />
                  Resources
                </TabsTrigger>
                <TabsTrigger value="code" className="data-[state=active]:bg-accent">
                  <Code className="w-4 h-4 mr-2" />
                  Code
                </TabsTrigger>
                <TabsTrigger value="ai" className="data-[state=active]:bg-accent">
                  <Brain className="w-4 h-4 mr-2" />
                  AI Usage
                </TabsTrigger>
                <TabsTrigger value="bottlenecks" className="data-[state=active]:bg-accent">
                  <AlertTriangle className="w-4 h-4 mr-2" />
                  Issues
                </TabsTrigger>
              </TabsList>

              <TabsContent value="time" className="mt-4">
                <Card className="bg-secondary/30 border-border">
                  <CardHeader>
                    <CardTitle className="text-foreground text-lg">Time Distribution</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {[
                      {
                        label: "Productive Coding",
                        value: metrics.timeMetrics.productiveTime,
                        color: "bg-primary",
                        total: metrics.timeMetrics.totalCodingTime,
                      },
                      {
                        label: "Code Review",
                        value: metrics.timeMetrics.reviewTime,
                        color: "bg-secondary",
                        total: metrics.timeMetrics.totalCodingTime,
                      },
                      {
                        label: "Meetings",
                        value: metrics.timeMetrics.meetingTime,
                        color: "bg-accent",
                        total: metrics.timeMetrics.totalCodingTime,
                      },
                      {
                        label: "Idle/Breaks",
                        value: metrics.timeMetrics.idleTime,
                        color: "bg-muted",
                        total: metrics.timeMetrics.totalCodingTime,
                      },
                    ].map((item) => (
                      <div key={item.label} className="space-y-2">
                        <div className="flex justify-between text-sm">
                          <span className="text-muted-foreground">{item.label}</span>
                          <span className="text-foreground font-medium">{formatTime(item.value)}</span>
                        </div>
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div
                            className={cn("h-full rounded-full transition-all", item.color)}
                            style={{ width: `${(item.value / item.total) * 100}%` }}
                          />
                        </div>
                      </div>
                    ))}
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="resources" className="mt-4">
                <Card className="bg-secondary/30 border-border">
                  <CardHeader>
                    <CardTitle className="text-foreground text-lg">Resource Utilization</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 mb-3">
                          <Cpu className="w-5 h-5 text-primary" />
                          <span className="text-foreground font-medium">CPU</span>
                        </div>
                        <div className="space-y-2">
                          <div className="flex justify-between text-sm">
                            <span className="text-muted-foreground">Average</span>
                            <span className="text-foreground">{metrics.resourceMetrics.avgCpuUsage}%</span>
                          </div>
                          <Progress value={metrics.resourceMetrics.avgCpuUsage} className="h-2" />
                          <div className="flex justify-between text-xs text-muted-foreground">
                            <span>Peak: {metrics.resourceMetrics.peakCpuUsage}%</span>
                          </div>
                        </div>
                      </div>

                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 mb-3">
                          <MemoryStick className="w-5 h-5 text-secondary" />
                          <span className="text-foreground font-medium">Memory</span>
                        </div>
                        <div className="space-y-2">
                          <div className="flex justify-between text-sm">
                            <span className="text-muted-foreground">Average</span>
                            <span className="text-foreground">{metrics.resourceMetrics.avgMemoryUsage}%</span>
                          </div>
                          <Progress value={metrics.resourceMetrics.avgMemoryUsage} className="h-2" />
                          <div className="flex justify-between text-xs text-muted-foreground">
                            <span>Peak: {metrics.resourceMetrics.peakMemoryUsage}%</span>
                          </div>
                        </div>
                      </div>

                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 mb-3">
                          <HardDrive className="w-5 h-5 text-primary" />
                          <span className="text-foreground font-medium">GPU</span>
                        </div>
                        <div className="space-y-2">
                          <div className="flex justify-between text-sm">
                            <span className="text-muted-foreground">Average</span>
                            <span className="text-foreground">{metrics.resourceMetrics.avgGpuUsage}%</span>
                          </div>
                          <Progress value={metrics.resourceMetrics.avgGpuUsage} className="h-2" />
                          <div className="flex justify-between text-xs text-muted-foreground">
                            <span>Peak: {metrics.resourceMetrics.peakMemoryUsage}%</span>
                          </div>
                        </div>
                      </div>

                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 mb-3">
                          <Wifi className="w-5 h-5 text-destructive" />
                          <span className="text-foreground font-medium">Network</span>
                        </div>
                        <div className="space-y-2">
                          <div className="flex justify-between text-sm">
                            <span className="text-muted-foreground">Bandwidth</span>
                            <span className="text-foreground">{metrics.resourceMetrics.networkBandwidth} MB/s</span>
                          </div>
                          <Progress
                            value={Math.min(metrics.resourceMetrics.networkBandwidth / 1000, 100)}
                            className="h-2"
                          />
                          <div className="flex justify-between text-xs text-muted-foreground">
                            <span>Peak: 500 MB/s</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="code" className="mt-4">
                <Card className="bg-secondary/30 border-border">
                  <CardHeader>
                    <CardTitle className="text-foreground text-lg">Code Metrics</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-3 gap-4">
                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-muted-foreground text-sm">Lines Written</span>
                          <Code className="w-4 h-4 text-primary" />
                        </div>
                        <div className="text-2xl font-bold text-foreground">{metrics.codeMetrics.linesWritten}</div>
                        <div className="text-xs text-muted-foreground mt-1">
                          +{metrics.codeMetrics.linesDeleted} deleted
                        </div>
                      </div>

                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-muted-foreground text-sm">Commits</span>
                          <GitCommit className="w-4 h-4 text-secondary" />
                        </div>
                        <div className="text-2xl font-bold text-foreground">{metrics.codeMetrics.commits}</div>
                        <div className="text-xs text-muted-foreground mt-1">{metrics.codeMetrics.pullRequests} PRs</div>
                      </div>

                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-muted-foreground text-sm">Issues</span>
                          <Bug className="w-4 h-4 text-destructive" />
                        </div>
                        <div className="text-2xl font-bold text-foreground">{metrics.codeMetrics.bugsFixed}</div>
                        <div className="text-xs text-muted-foreground mt-1">{metrics.codeMetrics.bugsFiled} filed</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="ai" className="mt-4">
                <Card className="bg-secondary/30 border-border">
                  <CardHeader>
                    <CardTitle className="text-foreground text-lg">AI Usage & ROI</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-muted-foreground text-sm">Prompts Executed</span>
                          <Brain className="w-4 h-4 text-primary" />
                        </div>
                        <div className="text-2xl font-bold text-foreground">{metrics.aiMetrics.promptsExecuted}</div>
                        <div className="text-xs text-muted-foreground mt-1">
                          Success: {metrics.aiMetrics.successRate}%
                        </div>
                      </div>

                      <div className="p-4 bg-accent/50 rounded-lg">
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-muted-foreground text-sm">Cost Estimate</span>
                          <DollarSign className="w-4 h-4 text-destructive" />
                        </div>
                        <div className="text-2xl font-bold text-foreground">
                          ${metrics.aiMetrics.costEstimate.toFixed(2)}
                        </div>
                        <div className="text-xs text-muted-foreground mt-1">{metrics.aiMetrics.tokensUsed} tokens</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="bottlenecks" className="mt-4">
                <div className="space-y-3">
                  {DETECTED_BOTTLENECKS.map((bn) => (
                    <Card
                      key={bn.id}
                      className={`bg-secondary/30 border-l-4 ${
                        bn.severity === "high"
                          ? "border-l-destructive"
                          : bn.severity === "medium"
                            ? "border-l-accent"
                            : "border-l-primary"
                      }`}
                    >
                      <CardContent className="p-4">
                        <div className="flex items-start justify-between mb-2">
                          <div>
                            <h4 className="text-foreground font-medium">{bn.title}</h4>
                            <p className="text-muted-foreground text-sm mt-1">{bn.description}</p>
                          </div>
                          <Badge variant={bn.severity === "high" ? "destructive" : "secondary"}>{bn.severity}</Badge>
                        </div>
                        <div className="mt-3 p-3 bg-accent/50 rounded text-sm text-muted-foreground">
                          <strong className="text-foreground">Recommendation:</strong> {bn.recommendation}
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </TabsContent>
            </Tabs>
          </div>

          {/* Sidebar - Goals and Insights */}
          <div className="space-y-6">
            <Card className="bg-secondary/30 border-border">
              <CardHeader>
                <CardTitle className="text-foreground text-lg flex items-center gap-2">
                  <Target className="w-5 h-5 text-primary" />
                  Goals
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {EFFICIENCY_GOALS.map((goal) => (
                  <div key={goal.id} className="p-3 bg-accent/30 rounded-lg">
                    <div className="flex justify-between items-start mb-2">
                      <span className="text-foreground font-medium text-sm">{goal.name}</span>
                      <Badge variant={goal.status === "on-track" ? "default" : "secondary"}>
                        {goal.current}/{goal.target}
                        {goal.unit}
                      </Badge>
                    </div>
                    <Progress value={goal.progress} className="h-2" />
                    <div className="text-xs text-muted-foreground mt-1">{goal.progress}% progress</div>
                  </div>
                ))}
              </CardContent>
            </Card>

            <Card className="bg-secondary/30 border-border">
              <CardHeader>
                <CardTitle className="text-foreground text-lg flex items-center gap-2">
                  <Lightbulb className="w-5 h-5 text-accent" />
                  Insights
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {PRODUCTIVITY_INSIGHTS.map((insight) => (
                  <div key={insight.id} className="p-3 bg-accent/30 rounded-lg">
                    <div className="flex items-start gap-2">
                      {insight.type === "positive" ? (
                        <CheckCircle2 className="w-4 h-4 text-primary mt-0.5 flex-shrink-0" />
                      ) : insight.type === "negative" ? (
                        <AlertTriangle className="w-4 h-4 text-destructive mt-0.5 flex-shrink-0" />
                      ) : (
                        <Circle className="w-4 h-4 text-muted-foreground mt-0.5 flex-shrink-0" />
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="text-foreground font-medium text-sm">{insight.title}</div>
                        <div className="text-muted-foreground text-xs mt-1">{insight.description}</div>
                        {insight.actionable && (
                          <div className="text-primary text-xs mt-2 flex items-center gap-1">
                            <Lightbulb className="w-3 h-3" />
                            {insight.suggestion}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </CardContent>
            </Card>
          </div>
        </div>
      </main>
    </div>
  )
}
