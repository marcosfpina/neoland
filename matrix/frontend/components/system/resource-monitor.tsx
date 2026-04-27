"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { Cpu, MemoryStick, Wifi, Thermometer, Activity } from "lucide-react"
import type { SystemResources } from "@/lib/agents/types"
import { cn } from "@/lib/utils"

// Simulated system resources - in production would come from actual system APIs
function useSystemResources(): SystemResources {
  const [resources, setResources] = useState<SystemResources>({
    cpu: { usage: 0, cores: 8, temperature: 0 },
    memory: { used: 0, total: 16384, available: 16384, swapUsed: 0, swapTotal: 16384 },
    gpu: { name: "NVIDIA RTX", usage: 0, memoryUsed: 0, memoryTotal: 8192, temperature: 0 },
    disk: { used: 0, total: 512000, readSpeed: 0, writeSpeed: 0 },
    network: { bytesIn: 0, bytesOut: 0, connections: 0 },
    processes: [],
  })

  useEffect(() => {
    const interval = setInterval(() => {
      setResources((prev) => ({
        cpu: {
          usage: Math.min(100, Math.max(5, prev.cpu.usage + (Math.random() - 0.5) * 10)),
          cores: 8,
          temperature: Math.min(85, Math.max(35, (prev.cpu.temperature || 45) + (Math.random() - 0.5) * 3)),
        },
        memory: {
          used: Math.min(14000, Math.max(4000, prev.memory.used + (Math.random() - 0.3) * 500)),
          total: 16384,
          available: 16384 - prev.memory.used,
          swapUsed: Math.min(8000, Math.max(0, prev.memory.swapUsed + (Math.random() - 0.5) * 200)),
          swapTotal: 16384,
        },
        gpu: {
          name: "NVIDIA GeForce RTX",
          usage: Math.min(100, Math.max(0, (prev.gpu?.usage || 20) + (Math.random() - 0.5) * 15)),
          memoryUsed: Math.min(7500, Math.max(500, (prev.gpu?.memoryUsed || 2000) + (Math.random() - 0.5) * 300)),
          memoryTotal: 8192,
          temperature: Math.min(80, Math.max(30, (prev.gpu?.temperature || 40) + (Math.random() - 0.5) * 2)),
        },
        disk: {
          used: 234500,
          total: 512000,
          readSpeed: Math.max(0, prev.disk.readSpeed + (Math.random() - 0.5) * 50),
          writeSpeed: Math.max(0, prev.disk.writeSpeed + (Math.random() - 0.5) * 30),
        },
        network: {
          bytesIn: prev.network.bytesIn + Math.random() * 100000,
          bytesOut: prev.network.bytesOut + Math.random() * 50000,
          connections: Math.floor(Math.random() * 50) + 10,
        },
        processes: [],
      }))
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  return resources
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

function formatMB(mb: number): string {
  if (mb < 1024) return `${mb.toFixed(0)} MB`
  return `${(mb / 1024).toFixed(1)} GB`
}

function getTempColor(temp: number): string {
  if (temp < 50) return "text-emerald-400"
  if (temp < 70) return "text-amber-400"
  return "text-red-400"
}

function getUsageColor(usage: number): string {
  if (usage < 50) return "bg-emerald-500"
  if (usage < 80) return "bg-amber-500"
  return "bg-red-500"
}

export function ResourceMonitor() {
  const resources = useSystemResources()

  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
      {/* CPU */}
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm flex items-center gap-2 text-slate-300">
            <Cpu className="w-4 h-4 text-blue-400" />
            CPU
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-white mb-2">{resources.cpu.usage.toFixed(1)}%</div>
          <Progress value={resources.cpu.usage} className={cn("h-2 mb-2", getUsageColor(resources.cpu.usage))} />
          <div className="flex justify-between text-xs text-slate-400">
            <span>{resources.cpu.cores} cores</span>
            <span className={getTempColor(resources.cpu.temperature || 0)}>
              <Thermometer className="w-3 h-3 inline mr-1" />
              {resources.cpu.temperature?.toFixed(0)}°C
            </span>
          </div>
        </CardContent>
      </Card>

      {/* Memory */}
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm flex items-center gap-2 text-slate-300">
            <MemoryStick className="w-4 h-4 text-purple-400" />
            Memory
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-white mb-2">{formatMB(resources.memory.used)}</div>
          <Progress value={(resources.memory.used / resources.memory.total) * 100} className="h-2 mb-2" />
          <div className="flex justify-between text-xs text-slate-400">
            <span>of {formatMB(resources.memory.total)}</span>
            <span>Swap: {formatMB(resources.memory.swapUsed)}</span>
          </div>
        </CardContent>
      </Card>

      {/* GPU */}
      {resources.gpu && (
        <Card className="bg-slate-800/50 border-slate-700/50">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center gap-2 text-slate-300">
              <Activity className="w-4 h-4 text-emerald-400" />
              GPU
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-white mb-2">{resources.gpu.usage.toFixed(1)}%</div>
            <Progress value={resources.gpu.usage} className="h-2 mb-2" />
            <div className="flex justify-between text-xs text-slate-400">
              <span>
                {formatMB(resources.gpu.memoryUsed)}/{formatMB(resources.gpu.memoryTotal)}
              </span>
              <span className={getTempColor(resources.gpu.temperature || 0)}>
                {resources.gpu.temperature?.toFixed(0)}°C
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Network */}
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm flex items-center gap-2 text-slate-300">
            <Wifi className="w-4 h-4 text-cyan-400" />
            Network
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-white mb-2">{resources.network.connections}</div>
          <div className="text-xs text-slate-400 mb-1">connections</div>
          <div className="flex justify-between text-xs text-slate-400">
            <span className="text-emerald-400">↓ {formatBytes(resources.network.bytesIn)}</span>
            <span className="text-blue-400">↑ {formatBytes(resources.network.bytesOut)}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
