import type { LucideIcon } from "lucide-react"
import {
  Activity,
  BookCopy,
  GitBranchPlus,
  Radar,
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
]
