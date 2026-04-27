import type { LucideIcon } from "lucide-react"
import {
  Activity,
  BookCopy,
  Cpu,
  GitBranchPlus,
  Radar,
  Shield,
  SlidersHorizontal,
  Waypoints,
} from "lucide-react"

export interface NeolandNavigationItem {
  href: string
  label: string
  shortLabel: string
  description: string
  icon: LucideIcon
}

export const neolandNavigation: NeolandNavigationItem[] = [
  {
    href: "/",
    label: "Overview",
    shortLabel: "Overview",
    description: "Operational overview of the control plane.",
    icon: Activity,
  },
  {
    href: "/pipeline",
    label: "Pipeline",
    shortLabel: "Pipeline",
    description: "Run a task and inspect the multi-agent execution.",
    icon: Waypoints,
  },
  {
    href: "/sessions",
    label: "Sessions",
    shortLabel: "Sessions",
    description: "Inspect session state and continue prior work.",
    icon: GitBranchPlus,
  },
  {
    href: "/adr",
    label: "ADR Vault",
    shortLabel: "ADR",
    description: "Browse real checkpoints written by the pipeline.",
    icon: BookCopy,
  },
  {
    href: "/services",
    label: "Services",
    shortLabel: "Services",
    description: "Monitor the ecosystem map and service health.",
    icon: Radar,
  },
  {
    href: "/agents",
    label: "Agents",
    shortLabel: "Agents",
    description: "Stage metrics derived from real ADR history.",
    icon: Cpu,
  },
  {
    href: "/phantom",
    label: "Phantom",
    shortLabel: "Phantom",
    description: "Inspect the security-service seam for future scans.",
    icon: Shield,
  },
  {
    href: "/settings",
    label: "Settings",
    shortLabel: "Settings",
    description: "Review current control-plane bindings and secrets posture.",
    icon: SlidersHorizontal,
  },
]
