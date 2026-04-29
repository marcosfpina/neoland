import "server-only"

import { readdir, readFile } from "fs/promises"
import path from "path"
import { cache } from "react"

import { slugify } from "@/lib/utils"

import type {
  AdrDocument,
  AgentAnalytics,
  DashboardOverview,
  HealthResponse,
  JuniorTimelinePoint,
  LivenessResponse,
  PipelineHealthResponse,
  PipelineResult,
  PipelineRunInput,
  ReadinessResponse,
  ServiceNode,
  ServiceStatus,
  SettingsSnapshot,
  SessionState,
} from "./types"

const DEFAULT_CONTROL_PLANE_URL = "http://127.0.0.1:3001"
const DEFAULT_DSPY_URL = "http://127.0.0.1:8001"

function controlPlaneUrl() {
  return (
    process.env.NEOLAND_CONTROL_PLANE_URL ||
    process.env.NEXT_PUBLIC_BACKEND_URL ||
    DEFAULT_CONTROL_PLANE_URL
  ).replace(/\/$/, "")
}

function dspyUrl() {
  return (process.env.NEOLAND_DSPY_URL || DEFAULT_DSPY_URL).replace(/\/$/, "")
}

function checkpointDir() {
  return process.env.NEOLAND_CHECKPOINT_DIR || "/var/lib/neoland/checkpoints/adr"
}

function authHeaders() {
  const apiKey =
    process.env.NEOLAND_USER_API_KEY ||
    process.env.NEOLAND_API_KEY ||
    process.env.NEOLAND_UI_API_KEY

  if (!apiKey) {
    console.warn("[Neoland] No API Key provided in environment. Connection to Control Plane may fail.")
  }

  return {
    "Content-Type": "application/json",
    "X-API-Key": apiKey || "neoland_unauthorized",
  }
}


function normalizeHealthUrl(endpoint?: string) {
  if (!endpoint) {
    return null
  }

  if (endpoint.includes("/health") || endpoint.includes("/ready") || endpoint.includes("/live")) {
    return endpoint
  }

  return `${endpoint.replace(/\/$/, "")}/health`
}

async function fetchJson<T>(url: string, init?: RequestInit, timeoutMs = 2500): Promise<T> {
  const response = await fetch(url, {
    ...init,
    cache: "no-store",
    signal: AbortSignal.timeout(timeoutMs),
  })

  if (!response.ok) {
    const text = await response.text().catch(() => "")
    throw new Error(text || `Request failed with ${response.status}`)
  }

  return response.json() as Promise<T>
}

async function checkService(endpoint: string, name: string) {
  const startedAt = Date.now()

  try {
    await fetch(endpoint, { cache: "no-store", signal: AbortSignal.timeout(2500) })
    return {
      name,
      status: "up" as const,
      latencyMs: Date.now() - startedAt,
      detail: "reachable",
    }
  } catch (error) {
    return {
      name,
      status: "down" as const,
      latencyMs: Date.now() - startedAt,
      detail: error instanceof Error ? error.message : "connection failed",
    }
  }
}

function mapHealthStatus(status: HealthResponse["status"]): ServiceStatus {
  if (status === "healthy") {
    return "up"
  }

  if (status === "degraded") {
    return "degraded"
  }

  return "down"
}

export const getAdrDocuments = cache(async (): Promise<AdrDocument[]> => {
  const directory = checkpointDir()

  try {
    const entries = await readdir(directory, { withFileTypes: true })
    const files = entries.filter((entry) => entry.isFile() && entry.name.endsWith(".json"))

    const documents = await Promise.all(
      files.map(async (file) => {
        const filePath = path.join(directory, file.name)
        const raw = await readFile(filePath, "utf8")
        const parsed = JSON.parse(raw) as Omit<AdrDocument, "slug" | "source_path">

        return {
          ...parsed,
          slug: slugify(path.basename(file.name, ".json")) || slugify(parsed.adr_id),
          source_path: filePath,
        } satisfies AdrDocument
      }),
    )

    return documents.sort((a, b) => b.timestamp.localeCompare(a.timestamp))
  } catch {
    return []
  }
})

export async function getAdrBySlug(slug: string) {
  const documents = await getAdrDocuments()
  return documents.find((document) => document.slug === slug || document.adr_id === slug) || null
}

export async function getSessionById(sessionId: string) {
  try {
    return await fetchJson<SessionState>(`${controlPlaneUrl()}/v1/agents/session/${sessionId}`, {
      headers: authHeaders(),
    })
  } catch {
    return null
  }
}

export async function getSessions(limit = 24) {
  const safeLimit = Math.max(1, Math.min(100, Math.trunc(limit) || 24))

  try {
    return await fetchJson<SessionState[]>(
      `${controlPlaneUrl()}/v1/agents/sessions?limit=${safeLimit}`,
      {
        headers: authHeaders(),
      },
    )
  } catch {
    return []
  }
}

export async function runPipelineTask(input: PipelineRunInput) {
  return fetchJson<PipelineResult>(`${controlPlaneUrl()}/v1/agents/task`, {
    method: "POST",
    headers: authHeaders(),
    body: JSON.stringify(input),
  }, 125000)
}

export const getServicesSnapshot = cache(async (): Promise<ServiceNode[]> => {
  const controlPlane = controlPlaneUrl()
  const phantom = normalizeHealthUrl(process.env.NEOLAND_PHANTOM_URL)
  const neutron = normalizeHealthUrl(process.env.NEOLAND_NEUTRON_URL)
  const cerebro = normalizeHealthUrl(process.env.NEOLAND_CEREBRO_URL)

  const [health, ready, live, pipeline, phantomHealth, neutronHealth, cerebroHealth] = await Promise.allSettled([
    fetchJson<HealthResponse>(`${controlPlane}/health`),
    fetchJson<ReadinessResponse>(`${controlPlane}/ready`),
    fetchJson<LivenessResponse>(`${controlPlane}/live`),
    fetchJson<PipelineHealthResponse>(`${controlPlane}/v1/agents/health`),
    phantom ? checkService(phantom, "phantom") : Promise.resolve(null),
    neutron ? checkService(neutron, "neutron") : Promise.resolve(null),
    cerebro ? checkService(cerebro, "cerebro") : Promise.resolve(null),
  ])

  const healthNode: ServiceNode = {
    id: "control-plane",
    name: "Neoland Control Plane",
    endpoint: controlPlane,
    status:
      health.status === "fulfilled"
        ? mapHealthStatus(health.value.status)
        : "down",
    detail:
      health.status === "fulfilled"
        ? `${health.value.components.length} components reported`
        : "health endpoint unreachable",
    group: "control-plane",
  }

  const readyNode: ServiceNode = {
    id: "readiness",
    name: "Readiness Probe",
    endpoint: `${controlPlane}/ready`,
    status:
      ready.status === "fulfilled"
        ? ready.value.ready
          ? "up"
          : "degraded"
        : "down",
    detail:
      ready.status === "fulfilled"
        ? ready.value.ready
          ? "ready to accept traffic"
          : "not ready"
        : "probe unavailable",
    group: "control-plane",
  }

  const liveNode: ServiceNode = {
    id: "liveness",
    name: "Liveness Probe",
    endpoint: `${controlPlane}/live`,
    status:
      live.status === "fulfilled"
        ? live.value.alive
          ? "up"
          : "down"
        : "down",
    detail:
      live.status === "fulfilled"
        ? live.value.alive
          ? "process alive"
          : "process unhealthy"
        : "probe unavailable",
    group: "control-plane",
  }

  const pipelineNode: ServiceNode = {
    id: "pipeline",
    name: "DSPy Pipeline",
    endpoint: dspyUrl(),
    status:
      pipeline.status === "fulfilled"
        ? pipeline.value.pipeline === "up"
          ? "up"
          : pipeline.value.pipeline === "disabled"
            ? "stub"
            : "degraded"
        : "down",
    detail:
      pipeline.status === "fulfilled"
        ? `pipeline ${pipeline.value.pipeline}`
        : "pipeline health unavailable",
    group: "pipeline",
  }

  const optionalNodes: ServiceNode[] = [
    {
      id: "ui",
      name: "Neoland UI",
      endpoint: "matrix/frontend",
      status: "up",
      detail: "current operator surface",
      group: "ui",
    },
    {
      id: "phantom",
      name: "Phantom",
      endpoint: process.env.NEOLAND_PHANTOM_URL || "not configured",
      status:
        phantomHealth.status === "fulfilled"
          ? phantomHealth.value
            ? phantomHealth.value.status
            : "stub"
          : "down",
      detail:
        phantomHealth.status === "fulfilled"
          ? phantomHealth.value
            ? phantomHealth.value.detail
            : "not configured"
          : "phantom health unavailable",
      latencyMs:
        phantomHealth.status === "fulfilled" && phantomHealth.value
          ? phantomHealth.value.latencyMs
          : undefined,
      group: "security",
    },
    {
      id: "neutron",
      name: "Neutron",
      endpoint: process.env.NEOLAND_NEUTRON_URL || "not configured",
      status:
        neutronHealth.status === "fulfilled"
          ? neutronHealth.value
            ? neutronHealth.value.status
            : "stub"
          : "down",
      detail:
        neutronHealth.status === "fulfilled"
          ? neutronHealth.value
            ? neutronHealth.value.detail
            : "not configured"
          : "neutron health unavailable",
      latencyMs:
        neutronHealth.status === "fulfilled" && neutronHealth.value
          ? neutronHealth.value.latencyMs
          : undefined,
      group: "security",
    },
    {
      id: "cerebro",
      name: "Cerebro",
      endpoint: process.env.NEOLAND_CEREBRO_URL || "not configured",
      status:
        cerebroHealth.status === "fulfilled"
          ? cerebroHealth.value
            ? cerebroHealth.value.status
            : "stub"
          : "down",
      detail:
        cerebroHealth.status === "fulfilled"
          ? cerebroHealth.value
            ? cerebroHealth.value.detail
            : "not configured"
          : "cerebro health unavailable",
      latencyMs:
        cerebroHealth.status === "fulfilled" && cerebroHealth.value
          ? cerebroHealth.value.latencyMs
          : undefined,
      group: "knowledge",
    },
  ]

  return [optionalNodes[0], healthNode, readyNode, liveNode, pipelineNode, ...optionalNodes.slice(1)]
})

export async function getDashboardOverview(): Promise<DashboardOverview> {
  const [services, adrs] = await Promise.all([getServicesSnapshot(), getAdrDocuments()])
  const decisions = adrs.map((document) => document.full_pipeline.tech_leader.decision)
  const approves = decisions.filter((decision) => decision === "approve").length

  return {
    controlPlaneStatus: services.find((service) => service.id === "control-plane")?.status || "down",
    adrCount: adrs.length,
    approveRate: decisions.length ? Math.round((approves / decisions.length) * 100) : 0,
    liveServices: services.filter((service) => service.status === "up").length,
    recentAdrs: adrs.slice(0, 4),
    services,
  }
}

export async function getAgentAnalytics(): Promise<{
  cards: AgentAnalytics[]
  timeline: JuniorTimelinePoint[]
}> {
  const adrs = await getAdrDocuments()

  const juniorConfidence = adrs.reduce((sum, adr) => sum + adr.full_pipeline.junior.confidence, 0)
  const seniorEscalations = adrs.filter((adr) => adr.full_pipeline.senior.escalate_to_architect).length
  const architectRuns = adrs.filter((adr) => adr.full_pipeline.architect).length
  const architectScore = adrs.reduce(
    (sum, adr) => sum + (adr.full_pipeline.architect?.composability_score || 0),
    0,
  )
  const approveCount = adrs.filter(
    (adr) => adr.full_pipeline.tech_leader.decision === "approve",
  ).length

  return {
    cards: [
      {
        label: "Junior",
        stage: "junior",
        totalRuns: adrs.length,
        primaryMetricLabel: "Avg confidence",
        primaryMetricValue: adrs.length ? `${(juniorConfidence / adrs.length).toFixed(2)}` : "0.00",
        secondaryMetricLabel: "Unknown clusters",
        secondaryMetricValue: `${adrs.reduce((sum, adr) => sum + adr.full_pipeline.junior.unknowns.length, 0)}`,
      },
      {
        label: "Senior",
        stage: "senior",
        totalRuns: adrs.length,
        primaryMetricLabel: "Escalation rate",
        primaryMetricValue: adrs.length ? `${Math.round((seniorEscalations / adrs.length) * 100)}%` : "0%",
        secondaryMetricLabel: "Refined hypotheses",
        secondaryMetricValue: `${adrs.filter((adr) => adr.full_pipeline.senior.refined_hypothesis).length}`,
      },
      {
        label: "Architect",
        stage: "architect",
        totalRuns: architectRuns,
        primaryMetricLabel: "Avg soundness",
        primaryMetricValue: architectRuns ? `${(architectScore / architectRuns).toFixed(2)}` : "0.00",
        secondaryMetricLabel: "Invocations",
        secondaryMetricValue: `${architectRuns}`,
      },
      {
        label: "Tech-Leader",
        stage: "tech_leader",
        totalRuns: adrs.length,
        primaryMetricLabel: "Approve rate",
        primaryMetricValue: adrs.length ? `${Math.round((approveCount / adrs.length) * 100)}%` : "0%",
        secondaryMetricLabel: "ADRs issued",
        secondaryMetricValue: `${adrs.length}`,
      },
    ],
    timeline: adrs
      .slice(0, 8)
      .reverse()
      .map((adr) => ({
        adr_id: adr.adr_id,
        title: adr.title,
        timestamp: adr.timestamp,
        confidence: adr.full_pipeline.junior.confidence,
      })),
  }
}

export function getSettingsSnapshot(): SettingsSnapshot {
  const activeKey =
    process.env.NEOLAND_UI_API_KEY ||
    process.env.NEOLAND_API_KEY ||
    DEFAULT_DEV_API_KEY

  return {
    controlPlaneUrl: controlPlaneUrl(),
    dspyUrl: dspyUrl(),
    checkpointDir: checkpointDir(),
    phantomUrl: process.env.NEOLAND_PHANTOM_URL,
    neutronUrl: process.env.NEOLAND_NEUTRON_URL,
    cerebroUrl: process.env.NEOLAND_CEREBRO_URL,
    usingDevelopmentApiKey: activeKey === DEFAULT_DEV_API_KEY,
  }
}

export function getKnownSessionPrefixes(adrs: AdrDocument[]) {
  return Array.from(
    new Set(
      adrs
        .map((adr) => adr.adr_id.split("-")[1])
        .filter((prefix): prefix is string => Boolean(prefix)),
    ),
  )
}

export function getRelatedAdrsForSessionPrefix(sessionId: string, adrs: AdrDocument[]) {
  const prefix = sessionId.slice(0, 8)
  return adrs.filter((adr) => adr.adr_id.includes(prefix))
}
