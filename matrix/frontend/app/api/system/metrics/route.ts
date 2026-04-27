/**
 * System Metrics API Route
 * Returns real-time system metrics from the daemon via Unix Domain Socket
 */

import { NextResponse } from "next/server"
import type { SystemMetrics, MetricsHistory } from "@/lib/system/types"
import http from "http"

const UDS_PATH = "/tmp/mission-control.sock"

// Helper to fetch from UDS
function fetchFromUds(path: string): Promise<any> {
  return new Promise((resolve, reject) => {
    const options = {
      socketPath: UDS_PATH,
      path: path,
      method: "GET",
    }

    const req = http.request(options, (res) => {
      let data = ""
      res.on("data", (chunk) => (data += chunk))
      res.on("end", () => {
        if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
          try {
            resolve(JSON.parse(data))
          } catch (e) {
            reject(new Error("Invalid JSON response"))
          }
        } else {
          reject(new Error(`Status code: ${res.statusCode}`))
        }
      })
    })

    req.on("error", (e) => reject(e))
    req.end()
  })
}

// Fallback to simulated data if daemon not running
function generateSimulatedMetrics(): SystemMetrics {
  const now = new Date()
  return {
    timestamp: now.toISOString(),
    hostname: process.env.HOSTNAME || "nixos",
    uptime_seconds: Math.floor(Math.random() * 86400 * 7),
    cpu: {
      cores: 8,
      model: "Intel Core i7-12700H",
      frequency_mhz: 2300 + Math.random() * 1000,
      usage_percent: 15 + Math.random() * 40,
      per_core_usage: Array(8)
        .fill(0)
        .map(() => Math.random() * 60),
      load_average: [0.5 + Math.random(), 0.7 + Math.random(), 0.6 + Math.random()],
      governor: "performance",
    },
    memory: {
      total_bytes: 16 * 1024 * 1024 * 1024,
      available_bytes: 8 * 1024 * 1024 * 1024 + Math.random() * 4 * 1024 * 1024 * 1024,
      used_bytes: 6 * 1024 * 1024 * 1024 + Math.random() * 2 * 1024 * 1024 * 1024,
      free_bytes: 2 * 1024 * 1024 * 1024,
      cached_bytes: 4 * 1024 * 1024 * 1024,
      buffers_bytes: 512 * 1024 * 1024,
      swap_total_bytes: 16 * 1024 * 1024 * 1024,
      swap_free_bytes: 15 * 1024 * 1024 * 1024,
      swap_used_bytes: 1 * 1024 * 1024 * 1024,
      usage_percent: 45 + Math.random() * 20,
      swap_usage_percent: 5 + Math.random() * 10,
      zram_used_bytes: 2 * 1024 * 1024 * 1024,
    },
    gpu: [
      {
        index: 0,
        name: "NVIDIA GeForce RTX 4060",
        temperature_c: 45 + Math.random() * 20,
        memory_used_mb: 2000 + Math.random() * 2000,
        memory_total_mb: 8192,
        utilization_percent: 10 + Math.random() * 50,
        power_draw_w: 30 + Math.random() * 60,
        power_limit_w: 140,
      },
    ],
    disks: [
      {
        device: "/dev/nvme0n1p2",
        mountpoint: "/",
        fstype: "ext4",
        total_bytes: 500 * 1024 * 1024 * 1024,
        used_bytes: 150 * 1024 * 1024 * 1024,
        free_bytes: 350 * 1024 * 1024 * 1024,
        usage_percent: 30,
      },
      {
        device: "/dev/nvme0n1p1",
        mountpoint: "/boot",
        fstype: "vfat",
        total_bytes: 512 * 1024 * 1024,
        used_bytes: 128 * 1024 * 1024,
        free_bytes: 384 * 1024 * 1024,
        usage_percent: 25,
      },
    ],
    network: {
      interfaces: {
        wlp0s20f3: {
          rx_bytes: Math.floor(Math.random() * 1024 * 1024 * 1024),
          rx_packets: Math.floor(Math.random() * 1000000),
          rx_errors: 0,
          tx_bytes: Math.floor(Math.random() * 512 * 1024 * 1024),
          tx_packets: Math.floor(Math.random() * 500000),
          tx_errors: 0,
        },
        lo: {
          rx_bytes: Math.floor(Math.random() * 100 * 1024 * 1024),
          rx_packets: Math.floor(Math.random() * 100000),
          rx_errors: 0,
          tx_bytes: Math.floor(Math.random() * 100 * 1024 * 1024),
          tx_packets: Math.floor(Math.random() * 100000),
          tx_errors: 0,
        },
      },
      total_rx_bytes: Math.floor(Math.random() * 1024 * 1024 * 1024),
      total_tx_bytes: Math.floor(Math.random() * 512 * 1024 * 1024),
    },
    processes: [
      { pid: 1234, name: "ollama", cpu_percent: 15.2, memory_percent: 8.5, status: "running", user: "ollama" },
      { pid: 2345, name: "node", cpu_percent: 8.1, memory_percent: 4.2, status: "running", user: "kernelcore" },
      { pid: 3456, name: "firefox", cpu_percent: 5.3, memory_percent: 6.8, status: "running", user: "kernelcore" },
      { pid: 4567, name: "nvim", cpu_percent: 2.1, memory_percent: 1.5, status: "running", user: "kernelcore" },
      { pid: 5678, name: "gnome-shell", cpu_percent: 3.4, memory_percent: 3.2, status: "running", user: "kernelcore" },
    ],
    services: [
      { name: "ollama", active_state: "active", sub_state: "running", pid: 1234, memory_bytes: 4 * 1024 * 1024 * 1024 },
      {
        name: "mission-control",
        active_state: "active",
        sub_state: "running",
        pid: 2345,
        memory_bytes: 256 * 1024 * 1024,
      },
      { name: "nginx", active_state: "inactive", sub_state: "dead", pid: 0, memory_bytes: 0 },
      { name: "postgresql", active_state: "inactive", sub_state: "dead", pid: 0, memory_bytes: 0 },
      {
        name: "NetworkManager",
        active_state: "active",
        sub_state: "running",
        pid: 567,
        memory_bytes: 32 * 1024 * 1024,
      },
    ],
    ollama: {
      running: true,
      models: [
        { name: "llama3.2:latest", size: 4.7 * 1024 * 1024 * 1024, modified_at: now.toISOString() },
        { name: "codellama:13b", size: 7.4 * 1024 * 1024 * 1024, modified_at: now.toISOString() },
        { name: "mistral:latest", size: 4.1 * 1024 * 1024 * 1024, modified_at: now.toISOString() },
        { name: "deepseek-coder:6.7b", size: 3.8 * 1024 * 1024 * 1024, modified_at: now.toISOString() },
      ],
      loaded_models: [{ name: "llama3.2:latest", size: 4.7 * 1024 * 1024 * 1024, vram_size: 4.2 * 1024 * 1024 * 1024 }],
      queue_size: 0,
    },
    thermal: {
      thermal_zone0: { type: "x86_pkg_temp", temperature_c: 55 + Math.random() * 15 },
      thermal_zone1: { type: "iwlwifi_1", temperature_c: 42 + Math.random() * 10 },
    },
    power: {
      ac_online: true,
      battery_present: true,
      battery_percent: 85,
      battery_status: "Charging",
      power_profile: "performance",
    },
  }
}

export async function GET() {
  try {
    // Try to read metrics from daemon via UDS
    const metrics = await fetchFromUds("/metrics")

    return NextResponse.json({
      success: true,
      source: "daemon",
      metrics,
      // History not yet implemented in new daemon
      history: [],
    })
  } catch (error) {
    // Daemon not running or error, return simulated data
    const metrics = generateSimulatedMetrics()

    return NextResponse.json({
      success: true,
      source: "simulated",
      metrics,
      history: [],
      error: error instanceof Error ? error.message : "Unknown error",
    })
  }
}
