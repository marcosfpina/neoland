import { serviceGroupLabel } from "@/lib/neoland/presentation"
import type { ServiceNode } from "@/lib/neoland/types"
import { cn } from "@/lib/utils"

const layout = {
  ui: { x: 50, y: 14 },
  "control-plane": { x: 50, y: 38 },
  pipeline: { x: 22, y: 70 },
  phantom: { x: 50, y: 86 },
  neutron: { x: 78, y: 66 },
  cerebro: { x: 78, y: 88 },
} as const

const connectors = [
  ["ui", "control-plane"],
  ["control-plane", "pipeline"],
  ["control-plane", "phantom"],
  ["control-plane", "neutron"],
  ["neutron", "cerebro"],
] as const

function dotClass(status: ServiceNode["status"]) {
  switch (status) {
    case "up":
      return "bg-emerald-400"
    case "degraded":
      return "bg-amber-400"
    case "down":
      return "bg-red-400"
    case "stub":
      return "bg-violet-400"
  }
}

export function EcosystemMap({ services }: { services: ServiceNode[] }) {
  const nodes = new Map(services.map((service) => [service.id, service]))

  return (
    <div className="space-y-5">
      <div className="relative hidden h-[470px] overflow-hidden rounded-[1.6rem] border border-border/70 bg-card/80 p-6 metal-panel lg:block">
        <svg className="absolute inset-0 size-full" aria-hidden="true">
          {connectors.map(([from, to]) => {
            const fromNode = layout[from]
            const toNode = layout[to]

            return (
              <line
                key={`${from}-${to}`}
                x1={`${fromNode.x}%`}
                y1={`${fromNode.y}%`}
                x2={`${toNode.x}%`}
                y2={`${toNode.y}%`}
                stroke="rgba(94,104,117,0.48)"
                strokeWidth="1.2"
                strokeDasharray="4 6"
              />
            )
          })}
        </svg>

        {Object.entries(layout).map(([id, point]) => {
          const service = nodes.get(id)

          if (!service) {
            return null
          }

          return (
            <div
              key={service.id}
              className="absolute -translate-x-1/2 -translate-y-1/2"
              style={{ left: `${point.x}%`, top: `${point.y}%` }}
            >
              <div className="w-48 rounded-[1.2rem] border border-border/70 bg-background/80 p-4 shadow-[0_18px_40px_rgba(0,0,0,0.22)] backdrop-blur-sm">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                      {serviceGroupLabel(service.group)}
                    </div>
                    <div className="truncate text-sm font-medium text-foreground">{service.name}</div>
                    <div className="truncate pt-1 font-mono text-[11px] text-muted-foreground">
                      {service.endpoint}
                    </div>
                  </div>
                  <span className={cn("mt-1 inline-flex size-2.5 rounded-full", dotClass(service.status))} />
                </div>
              </div>
            </div>
          )
        })}
      </div>

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        {services.map((service) => (
          <div
            key={service.id}
            className="rounded-[1.2rem] border border-border/70 bg-card/75 p-4 metal-panel"
          >
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                  {serviceGroupLabel(service.group)}
                </div>
                <div className="text-lg font-semibold tracking-tight text-foreground">{service.name}</div>
              </div>
              <span className={cn("mt-2 inline-flex size-2.5 rounded-full", dotClass(service.status))} />
            </div>
            <div className="mt-3 space-y-2 text-sm">
              <div className="font-mono text-xs text-muted-foreground">{service.endpoint}</div>
              <div className="leading-6 text-muted-foreground">{service.detail}</div>
              {service.latencyMs ? (
                <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                  {service.latencyMs} ms
                </div>
              ) : null}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
