import { Activity, Cpu, Shield } from "lucide-react"

import { EcosystemMap } from "@/components/services/ecosystem-map"
import { MetricCard } from "@/components/shared/metric-card"
import { PageHeader } from "@/components/shared/page-header"
import { getServicesSnapshot } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

export default async function ServicesPage() {
  const services = await getServicesSnapshot()
  const upCount = services.filter((service) => service.status === "up").length
  const degradedCount = services.filter((service) => service.status === "degraded").length
  const stubCount = services.filter((service) => service.status === "stub").length

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Services"
        title="Ecosystem map"
        description="This page aggregates live health probes from the control plane and configured ecosystem services. Stub is a first-class state for services not bound yet."
      />

      <div className="grid gap-4 md:grid-cols-3">
        <MetricCard
          label="Live services"
          value={upCount}
          detail="Services currently reporting healthy or reachable."
          icon={<Activity className="size-5" />}
          tone="approve"
        />
        <MetricCard
          label="Degraded"
          value={degradedCount}
          detail="Surfaces that answer, but not in a fully healthy state."
          icon={<Cpu className="size-5" />}
          tone="warning"
        />
        <MetricCard
          label="Stub services"
          value={stubCount}
          detail="Known surfaces intentionally not wired into the environment yet."
          icon={<Shield className="size-5" />}
          tone="accent"
        />
      </div>

      <EcosystemMap services={services} />
    </div>
  )
}
