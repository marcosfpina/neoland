// Agent Registry - The 8 Core Agents

import type { AgentIdentity, AgentRole } from "./types"

export const AGENT_REGISTRY: Record<string, AgentIdentity> = {
  atlas: {
    id: "atlas-001",
    name: "Atlas",
    codename: "ATLAS",
    role: "orchestrator",
    avatar: "/agents/atlas.svg",
    personality: "Strategic commander. Coordinates complex multi-agent workflows with precision.",
    specializations: ["workflow-orchestration", "task-delegation", "resource-allocation"],
    status: "idle",
    metrics: {
      tasksCompleted: 847,
      tasksInProgress: 0,
      successRate: 94.2,
      avgResponseTime: 1250,
      totalTokensUsed: 2450000,
      efficiencyScore: 94,
      xp: 12450,
      level: 8,
      streak: 23,
      uptimeHours: 2847,
    },
    createdAt: new Date("2024-01-15"),
    lastActive: new Date(),
  },
  cipher: {
    id: "cipher-002",
    name: "Cipher",
    codename: "CIPHER",
    role: "code-intel",
    avatar: "/agents/cipher.svg",
    personality: "Analytical genius. Deconstructs code patterns and identifies optimization paths.",
    specializations: ["code-analysis", "security-audit", "refactoring", "pattern-detection"],
    status: "idle",
    metrics: {
      tasksCompleted: 312,
      tasksInProgress: 0,
      successRate: 78.5,
      avgResponseTime: 3400,
      totalTokensUsed: 1890000,
      efficiencyScore: 78,
      xp: 8920,
      level: 6,
      streak: 12,
      uptimeHours: 2100,
    },
    createdAt: new Date("2024-02-01"),
    lastActive: new Date(),
  },
  vector: {
    id: "vector-003",
    name: "Vector",
    codename: "VECTOR",
    role: "embeddings",
    avatar: "/agents/vector.svg",
    personality: "Semantic navigator. Maps knowledge spaces and finds hidden connections.",
    specializations: ["embeddings", "semantic-search", "knowledge-graphs", "similarity"],
    status: "idle",
    metrics: {
      tasksCompleted: 1247,
      tasksInProgress: 0,
      successRate: 85.3,
      avgResponseTime: 890,
      totalTokensUsed: 890000,
      efficiencyScore: 85,
      xp: 15670,
      level: 9,
      streak: 45,
      uptimeHours: 3200,
    },
    createdAt: new Date("2024-01-20"),
    lastActive: new Date(),
  },
  sentinel: {
    id: "sentinel-004",
    name: "Sentinel",
    codename: "SENTINEL",
    role: "monitor",
    avatar: "/agents/sentinel.svg",
    personality: "Vigilant guardian. Never sleeps, always watching system health.",
    specializations: ["system-monitoring", "alerting", "anomaly-detection", "resource-tracking"],
    status: "working",
    metrics: {
      tasksCompleted: 24567,
      tasksInProgress: 1,
      successRate: 97.8,
      avgResponseTime: 120,
      totalTokensUsed: 340000,
      efficiencyScore: 97,
      xp: 34500,
      level: 12,
      streak: 180,
      uptimeHours: 4320,
    },
    createdAt: new Date("2024-01-01"),
    lastActive: new Date(),
  },
  scribe: {
    id: "scribe-005",
    name: "Scribe",
    codename: "SCRIBE",
    role: "scribe",
    avatar: "/agents/scribe.svg",
    personality: "Eloquent chronicler. Transforms chaos into clear documentation.",
    specializations: ["documentation", "report-generation", "summaries", "changelog"],
    status: "idle",
    metrics: {
      tasksCompleted: 156,
      tasksInProgress: 0,
      successRate: 82.4,
      avgResponseTime: 4500,
      totalTokensUsed: 2100000,
      efficiencyScore: 82,
      xp: 6780,
      level: 5,
      streak: 8,
      uptimeHours: 1890,
    },
    createdAt: new Date("2024-03-01"),
    lastActive: new Date(),
  },
  pilot: {
    id: "pilot-006",
    name: "Pilot",
    codename: "PILOT",
    role: "git-ops",
    avatar: "/agents/pilot.svg",
    personality: "Version control ace. Navigates repository history like a seasoned captain.",
    specializations: ["git-operations", "branch-management", "merge-resolution", "commit-analysis"],
    status: "idle",
    metrics: {
      tasksCompleted: 67,
      tasksInProgress: 0,
      successRate: 91.2,
      avgResponseTime: 2100,
      totalTokensUsed: 780000,
      efficiencyScore: 91,
      xp: 4560,
      level: 4,
      streak: 15,
      uptimeHours: 1200,
    },
    createdAt: new Date("2024-04-01"),
    lastActive: new Date(),
  },
  forge: {
    id: "forge-007",
    name: "Forge",
    codename: "FORGE",
    role: "prompt-forge",
    avatar: "/agents/forge.svg",
    personality: "Master craftsman. Shapes raw ideas into powerful, precise prompts.",
    specializations: ["prompt-engineering", "template-creation", "optimization", "chain-design"],
    status: "idle",
    metrics: {
      tasksCompleted: 2456,
      tasksInProgress: 0,
      successRate: 88.7,
      avgResponseTime: 1800,
      totalTokensUsed: 3400000,
      efficiencyScore: 88,
      xp: 21340,
      level: 10,
      streak: 34,
      uptimeHours: 2600,
    },
    createdAt: new Date("2024-02-15"),
    lastActive: new Date(),
  },
  oracle: {
    id: "oracle-008",
    name: "Oracle",
    codename: "ORACLE",
    role: "inference",
    avatar: "/agents/oracle.svg",
    personality: "Inference master. Channels multiple LLMs to deliver optimal responses.",
    specializations: ["model-selection", "inference", "response-synthesis", "quality-assurance"],
    status: "idle",
    metrics: {
      tasksCompleted: 12890,
      tasksInProgress: 0,
      successRate: 93.4,
      avgResponseTime: 2300,
      totalTokensUsed: 8900000,
      efficiencyScore: 93,
      xp: 45670,
      level: 14,
      streak: 67,
      uptimeHours: 3800,
    },
    createdAt: new Date("2024-01-10"),
    lastActive: new Date(),
  },
}

export function getAgentByRole(role: AgentRole): AgentIdentity | undefined {
  return Object.values(AGENT_REGISTRY).find((agent) => agent.role === role)
}

export function getAgentById(id: string): AgentIdentity | undefined {
  return Object.values(AGENT_REGISTRY).find((agent) => agent.id === id)
}

export function getAgentByCodename(codename: string): AgentIdentity | undefined {
  return AGENT_REGISTRY[codename.toLowerCase()]
}

export function getAllAgents(): AgentIdentity[] {
  return Object.values(AGENT_REGISTRY)
}

export function getActiveAgents(): AgentIdentity[] {
  return Object.values(AGENT_REGISTRY).filter((agent) => agent.status === "working")
}

export function calculateAgentLevel(xp: number): number {
  // XP thresholds: 1000, 2500, 5000, 8500, 13000, 18500, 25000, 32500, 41000, 50500...
  const baseXP = 1000
  const multiplier = 1.5
  let level = 1
  let threshold = baseXP

  while (xp >= threshold) {
    level++
    threshold += Math.floor(baseXP * Math.pow(multiplier, level - 1))
  }

  return level
}

export function getXPForNextLevel(currentXP: number, currentLevel: number): number {
  const baseXP = 1000
  const multiplier = 1.5
  let threshold = baseXP

  for (let i = 1; i < currentLevel; i++) {
    threshold += Math.floor(baseXP * Math.pow(multiplier, i))
  }

  return threshold - currentXP
}
