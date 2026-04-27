"use client"

import { useState, useEffect, useCallback } from "react"
import Link from "next/link"
import {
  Activity,
  Cpu,
  HardDrive,
  Network,
  Thermometer,
  Zap,
  Server,
  RefreshCw,
  ChevronLeft,
  MemoryStick,
  Gauge,
  MonitorSpeaker,
  Play,
  Square,
  RotateCcw,
  Eye,
  AlertTriangle,
  CheckCircle,
  XCircle,
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ScrollArea } from "@/components/ui/scroll-area"
import { XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, AreaChart, Area } from "recharts"
import type { SystemMetrics } from "@/lib/system/types"

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B"
  const k = 1024
  const sizes = ["B", "KB", "MB", "GB", "TB"]
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i]
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const mins = Math.floor((seconds % 3600) / 60)

  if (days > 0) return `${days}d ${hours}h ${mins}m`
  if (hours > 0) return `${hours}h ${mins}m`
  return `${mins}m`
}

function getStatusColor(state: string): string {
  switch (state) {
    case "active":
      return "text-chart-2"
    case "inactive":
      return "text-muted-foreground"
    case "failed":
      return "text-destructive"
    case "activating":
      return "text-chart-4"
    default:
      return "text-muted-foreground"
  }
}

function getStatusBadge(state: string) {
  switch (state) {
    case "active":
      return (
        <Badge className="bg-chart-2/20 text-chart-2 border-chart-2/30">
          <CheckCircle className="w-3 h-3 mr-1" />
          Active
        </Badge>
      )
    case "inactive":
      return (
        <Badge variant="secondary">
          <Square className="w-3 h-3 mr-1" />
          Inactive
        </Badge>
      )
    case "failed":
      return (
        <Badge variant="destructive">
          <XCircle className="w-3 h-3 mr-1" />
          Failed
        </Badge>
      )
    default:
      return <Badge variant="outline">{state}</Badge>
  }
}

export default function LiveMonitorPage() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const [history, setHistory] = useState<
    Array<{
      timestamp: string
      cpu_usage: number
      memory_usage: number
      gpu_usage: number
      gpu_memory: number
    }>
  >([])
  const [source, setSource] = useState<"daemon" | "simulated">("simulated")
  const [loading, setLoading] = useState(true)
  const [autoRefresh, setAutoRefresh] = useState(true)
  const [refreshInterval, setRefreshInterval] = useState(2000)

  const fetchMetrics = useCallback(async () => {
    try {
      const response = await fetch("/api/system/metrics")
      const data = await response.json()

      if (data.success) {
        setMetrics(data.metrics)
        setSource(data.source)
        if (data.history?.length > 0) {
          setHistory(data.history)
        } else if (data.metrics) {
          // Build local history when daemon not available
          setHistory((prev) => {
            const newEntry = {
              timestamp: data.metrics.timestamp,
              cpu_usage: data.metrics.cpu.usage_percent,
              memory_usage: data.metrics.memory.usage_percent,
              gpu_usage: data.metrics.gpu?.[0]?.utilization_percent || 0,
              gpu_memory: data.metrics.gpu?.[0]?.memory_used_mb || 0,
            }
            const updated = [...prev, newEntry].slice(-60)
            return updated
          })
        }
      }
    } catch (error) {
      console.error("Failed to fetch metrics:", error)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchMetrics()
  }, [fetchMetrics])

  useEffect(() => {
    if (!autoRefresh) return

    const interval = setInterval(fetchMetrics, refreshInterval)
    return () => clearInterval(interval)
  }, [autoRefresh, refreshInterval, fetchMetrics])

  const handleServiceAction = async (service: string, action: "start" | "stop" | "restart") => {
    try {
      const response = await fetch("/api/system/services", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ service, action }),
      })
      const data = await response.json()
      if (data.success) {
        // Refresh metrics to get updated service status
        setTimeout(fetchMetrics, 1000)
      }
    } catch (error) {
      console.error(`Failed to ${action} service:`, error)
    }
  }

  if (loading || !metrics) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <Activity className="w-12 h-12 text-primary animate-pulse mx-auto mb-4" />
          <p className="text-muted-foreground">Loading system metrics...</p>
        </div>
      </div>
    )
  }

  const chartData = history.map((h, i) => ({
    time: i,
    cpu: h.cpu_usage,
    memory: h.memory_usage,
    gpu: h.gpu_usage,
  }))

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Link href="/">
                <Button variant="ghost" size="sm">
                  <ChevronLeft className="w-4 h-4 mr-1" />
                  Back
                </Button>
              </Link>
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-primary/10">
                  <Activity className="w-6 h-6 text-primary" />
                </div>
                <div>
                  <h1 className="text-xl font-bold text-foreground">Live Monitor</h1>
                  <p className="text-sm text-muted-foreground">
                    {metrics.hostname} - Uptime: {formatUptime(metrics.uptime_seconds)}
                  </p>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <Badge variant={source === "daemon" ? "default" : "secondary"} className="gap-1">
                {source === "daemon" ? (
                  <>
                    <CheckCircle className="w-3 h-3" /> Real Data
                  </>
                ) : (
                  <>
                    <AlertTriangle className="w-3 h-3" /> Simulated
                  </>
                )}
              </Badge>

              <Button
                variant={autoRefresh ? "default" : "outline"}
                size="sm"
                onClick={() => setAutoRefresh(!autoRefresh)}
              >
                {autoRefresh ? <Eye className="w-4 h-4 mr-1" /> : <RefreshCw className="w-4 h-4 mr-1" />}
                {autoRefresh ? "Live" : "Paused"}
              </Button>

              <Button variant="outline" size="sm" onClick={fetchMetrics}>
                <RefreshCw className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        {/* Key Metrics Cards */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
          {/* CPU */}
          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <Cpu className="w-5 h-5 text-primary" />
                  <span className="font-medium text-foreground">CPU</span>
                </div>
                <span className="text-2xl font-bold text-foreground">{metrics.cpu.usage_percent.toFixed(1)}%</span>
              </div>
              <Progress value={metrics.cpu.usage_percent} className="h-2 mb-2" />
              <p className="text-xs text-muted-foreground">
                {metrics.cpu.cores} cores @ {metrics.cpu.frequency_mhz.toFixed(0)} MHz
              </p>
              <p className="text-xs text-muted-foreground">Governor: {metrics.cpu.governor}</p>
            </CardContent>
          </Card>

          {/* Memory */}
          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <MemoryStick className="w-5 h-5 text-chart-2" />
                  <span className="font-medium text-foreground">Memory</span>
                </div>
                <span className="text-2xl font-bold text-foreground">{metrics.memory.usage_percent.toFixed(1)}%</span>
              </div>
              <Progress value={metrics.memory.usage_percent} className="h-2 mb-2" />
              <p className="text-xs text-muted-foreground">
                {formatBytes(metrics.memory.used_bytes)} / {formatBytes(metrics.memory.total_bytes)}
              </p>
              <p className="text-xs text-muted-foreground">ZRAM: {formatBytes(metrics.memory.zram_used_bytes)}</p>
            </CardContent>
          </Card>

          {/* GPU */}
          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <MonitorSpeaker className="w-5 h-5 text-chart-3" />
                  <span className="font-medium text-foreground">GPU</span>
                </div>
                <span className="text-2xl font-bold text-foreground">
                  {metrics.gpu[0]?.utilization_percent.toFixed(0) || 0}%
                </span>
              </div>
              <Progress value={metrics.gpu[0]?.utilization_percent || 0} className="h-2 mb-2" />
              <p className="text-xs text-muted-foreground">{metrics.gpu[0]?.name || "No GPU"}</p>
              <p className="text-xs text-muted-foreground">
                VRAM: {metrics.gpu[0]?.memory_used_mb.toFixed(0) || 0} / {metrics.gpu[0]?.memory_total_mb || 0} MB
              </p>
            </CardContent>
          </Card>

          {/* Swap */}
          <Card className="bg-card border-border">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <HardDrive className="w-5 h-5 text-chart-4" />
                  <span className="font-medium text-foreground">Swap</span>
                </div>
                <span className="text-2xl font-bold text-foreground">
                  {metrics.memory.swap_usage_percent.toFixed(1)}%
                </span>
              </div>
              <Progress value={metrics.memory.swap_usage_percent} className="h-2 mb-2" />
              <p className="text-xs text-muted-foreground">
                {formatBytes(metrics.memory.swap_used_bytes)} / {formatBytes(metrics.memory.swap_total_bytes)}
              </p>
              <p className="text-xs text-muted-foreground">
                Load: {metrics.cpu.load_average.map((l) => l.toFixed(2)).join(" / ")}
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Charts and Details */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
          {/* Resource History Chart */}
          <Card className="lg:col-span-2 bg-card border-border">
            <CardHeader className="pb-2">
              <CardTitle className="text-foreground flex items-center gap-2">
                <Gauge className="w-5 h-5" />
                Resource Usage History
              </CardTitle>
              <CardDescription>Last 60 samples</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={chartData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
                    <XAxis dataKey="time" stroke="hsl(var(--muted-foreground))" fontSize={12} />
                    <YAxis stroke="hsl(var(--muted-foreground))" fontSize={12} domain={[0, 100]} />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: "hsl(var(--card))",
                        border: "1px solid hsl(var(--border))",
                        borderRadius: "8px",
                      }}
                      labelStyle={{ color: "hsl(var(--foreground))" }}
                    />
                    <Area
                      type="monotone"
                      dataKey="cpu"
                      name="CPU"
                      stroke="hsl(var(--primary))"
                      fill="hsl(var(--primary))"
                      fillOpacity={0.3}
                    />
                    <Area
                      type="monotone"
                      dataKey="memory"
                      name="Memory"
                      stroke="hsl(var(--chart-2))"
                      fill="hsl(var(--chart-2))"
                      fillOpacity={0.3}
                    />
                    <Area
                      type="monotone"
                      dataKey="gpu"
                      name="GPU"
                      stroke="hsl(var(--chart-3))"
                      fill="hsl(var(--chart-3))"
                      fillOpacity={0.3}
                    />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>

          {/* Thermal & Power */}
          <Card className="bg-card border-border">
            <CardHeader className="pb-2">
              <CardTitle className="text-foreground flex items-center gap-2">
                <Thermometer className="w-5 h-5" />
                Thermal & Power
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* GPU Temperature */}
              {metrics.gpu[0] && (
                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-muted-foreground">GPU Temp</span>
                    <span
                      className={`font-medium ${metrics.gpu[0].temperature_c > 80 ? "text-destructive" : "text-foreground"}`}
                    >
                      {metrics.gpu[0].temperature_c.toFixed(0)}°C
                    </span>
                  </div>
                  <Progress value={(metrics.gpu[0].temperature_c / 100) * 100} className="h-2" />
                </div>
              )}

              {/* Thermal Zones */}
              {Object.entries(metrics.thermal)
                .slice(0, 3)
                .map(([zone, data]) => (
                  <div key={zone}>
                    <div className="flex justify-between text-sm mb-1">
                      <span className="text-muted-foreground">{data.type}</span>
                      <span
                        className={`font-medium ${data.temperature_c > 80 ? "text-destructive" : "text-foreground"}`}
                      >
                        {data.temperature_c.toFixed(0)}°C
                      </span>
                    </div>
                    <Progress value={(data.temperature_c / 100) * 100} className="h-2" />
                  </div>
                ))}

              {/* Power */}
              <div className="pt-2 border-t border-border">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-muted-foreground">Power Profile</span>
                  <Badge variant="outline">{metrics.power.power_profile}</Badge>
                </div>
                {metrics.power.battery_present && (
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Battery</span>
                    <div className="flex items-center gap-2">
                      <Zap
                        className={`w-4 h-4 ${metrics.power.ac_online ? "text-chart-4" : "text-muted-foreground"}`}
                      />
                      <span className="font-medium text-foreground">{metrics.power.battery_percent}%</span>
                    </div>
                  </div>
                )}
                {metrics.gpu[0] && (
                  <div className="flex items-center justify-between mt-2">
                    <span className="text-sm text-muted-foreground">GPU Power</span>
                    <span className="font-medium text-foreground">
                      {metrics.gpu[0].power_draw_w.toFixed(0)}W / {metrics.gpu[0].power_limit_w.toFixed(0)}W
                    </span>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Tabs for Services, Processes, Disks, Network */}
        <Tabs defaultValue="services" className="space-y-4">
          <TabsList className="bg-secondary">
            <TabsTrigger value="services" className="data-[state=active]:bg-background">
              <Server className="w-4 h-4 mr-2" />
              Services
            </TabsTrigger>
            <TabsTrigger value="processes" className="data-[state=active]:bg-background">
              <Activity className="w-4 h-4 mr-2" />
              Processes
            </TabsTrigger>
            <TabsTrigger value="disks" className="data-[state=active]:bg-background">
              <HardDrive className="w-4 h-4 mr-2" />
              Disks
            </TabsTrigger>
            <TabsTrigger value="network" className="data-[state=active]:bg-background">
              <Network className="w-4 h-4 mr-2" />
              Network
            </TabsTrigger>
          </TabsList>

          {/* Services Tab */}
          <TabsContent value="services">
            <Card className="bg-card border-border">
              <CardHeader>
                <CardTitle className="text-foreground">Systemd Services</CardTitle>
                <CardDescription>Monitor and control system services</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-3">
                  {metrics.services.map((service) => (
                    <div
                      key={service.name}
                      className="flex items-center justify-between p-3 rounded-lg bg-secondary/50 border border-border"
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className={`w-2 h-2 rounded-full ${
                            service.active_state === "active"
                              ? "bg-chart-2"
                              : service.active_state === "failed"
                                ? "bg-destructive"
                                : "bg-muted-foreground"
                          }`}
                        />
                        <div>
                          <p className="font-medium text-foreground">{service.name}</p>
                          <p className="text-xs text-muted-foreground">
                            PID: {service.pid || "N/A"} | Memory: {formatBytes(service.memory_bytes)}
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {getStatusBadge(service.active_state)}
                        <div className="flex gap-1">
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleServiceAction(service.name, "start")}
                            disabled={service.active_state === "active"}
                          >
                            <Play className="w-3 h-3" />
                          </Button>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleServiceAction(service.name, "stop")}
                            disabled={service.active_state !== "active"}
                          >
                            <Square className="w-3 h-3" />
                          </Button>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleServiceAction(service.name, "restart")}
                          >
                            <RotateCcw className="w-3 h-3" />
                          </Button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Processes Tab */}
          <TabsContent value="processes">
            <Card className="bg-card border-border">
              <CardHeader>
                <CardTitle className="text-foreground">Top Processes</CardTitle>
                <CardDescription>Processes by resource usage</CardDescription>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-80">
                  <div className="space-y-2">
                    {metrics.processes.map((proc) => (
                      <div
                        key={proc.pid}
                        className="flex items-center justify-between p-2 rounded bg-secondary/30 border border-border/50"
                      >
                        <div className="flex items-center gap-3">
                          <span className="text-xs text-muted-foreground font-mono w-16">{proc.pid}</span>
                          <span className="font-medium text-foreground">{proc.name}</span>
                          <Badge variant="outline" className="text-xs">
                            {proc.user}
                          </Badge>
                        </div>
                        <div className="flex items-center gap-4">
                          <div className="w-24 text-right">
                            <span className="text-sm text-primary">{proc.cpu_percent.toFixed(1)}%</span>
                            <span className="text-xs text-muted-foreground ml-1">CPU</span>
                          </div>
                          <div className="w-24 text-right">
                            <span className="text-sm text-chart-2">{proc.memory_percent.toFixed(1)}%</span>
                            <span className="text-xs text-muted-foreground ml-1">MEM</span>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Disks Tab */}
          <TabsContent value="disks">
            <Card className="bg-card border-border">
              <CardHeader>
                <CardTitle className="text-foreground">Disk Usage</CardTitle>
                <CardDescription>Storage volumes and usage</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-4">
                  {metrics.disks.map((disk) => (
                    <div key={disk.mountpoint} className="p-4 rounded-lg bg-secondary/50 border border-border">
                      <div className="flex items-center justify-between mb-2">
                        <div>
                          <p className="font-medium text-foreground">{disk.mountpoint}</p>
                          <p className="text-xs text-muted-foreground">
                            {disk.device} ({disk.fstype})
                          </p>
                        </div>
                        <span
                          className={`text-lg font-bold ${
                            disk.usage_percent > 90
                              ? "text-destructive"
                              : disk.usage_percent > 70
                                ? "text-chart-4"
                                : "text-foreground"
                          }`}
                        >
                          {disk.usage_percent.toFixed(1)}%
                        </span>
                      </div>
                      <Progress value={disk.usage_percent} className="h-2 mb-2" />
                      <div className="flex justify-between text-xs text-muted-foreground">
                        <span>Used: {formatBytes(disk.used_bytes)}</span>
                        <span>Free: {formatBytes(disk.free_bytes)}</span>
                        <span>Total: {formatBytes(disk.total_bytes)}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Network Tab */}
          <TabsContent value="network">
            <Card className="bg-card border-border">
              <CardHeader>
                <CardTitle className="text-foreground">Network Interfaces</CardTitle>
                <CardDescription>Traffic statistics per interface</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-4">
                  {Object.entries(metrics.network.interfaces).map(([iface, stats]) => (
                    <div key={iface} className="p-4 rounded-lg bg-secondary/50 border border-border">
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center gap-2">
                          <Network className="w-5 h-5 text-primary" />
                          <span className="font-medium text-foreground">{iface}</span>
                        </div>
                      </div>
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <p className="text-xs text-muted-foreground mb-1">Received</p>
                          <p className="text-lg font-semibold text-chart-2">{formatBytes(stats.rx_bytes)}</p>
                          <p className="text-xs text-muted-foreground">{stats.rx_packets.toLocaleString()} packets</p>
                        </div>
                        <div>
                          <p className="text-xs text-muted-foreground mb-1">Transmitted</p>
                          <p className="text-lg font-semibold text-primary">{formatBytes(stats.tx_bytes)}</p>
                          <p className="text-xs text-muted-foreground">{stats.tx_packets.toLocaleString()} packets</p>
                        </div>
                      </div>
                      {(stats.rx_errors > 0 || stats.tx_errors > 0) && (
                        <div className="mt-2 pt-2 border-t border-border flex gap-4">
                          <span className="text-xs text-destructive">RX Errors: {stats.rx_errors}</span>
                          <span className="text-xs text-destructive">TX Errors: {stats.tx_errors}</span>
                        </div>
                      )}
                    </div>
                  ))}

                  {/* Total Traffic */}
                  <div className="p-4 rounded-lg bg-primary/10 border border-primary/20">
                    <p className="text-sm font-medium text-foreground mb-2">Total Traffic</p>
                    <div className="flex gap-8">
                      <div>
                        <span className="text-xs text-muted-foreground">Total RX:</span>
                        <span className="ml-2 font-semibold text-foreground">
                          {formatBytes(metrics.network.total_rx_bytes)}
                        </span>
                      </div>
                      <div>
                        <span className="text-xs text-muted-foreground">Total TX:</span>
                        <span className="ml-2 font-semibold text-foreground">
                          {formatBytes(metrics.network.total_tx_bytes)}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  )
}
