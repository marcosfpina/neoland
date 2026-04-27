import { Shield } from "lucide-react"

import { PageHeader } from "@/components/shared/page-header"
import { StatusDot } from "@/components/shared/status-dot"
import { Card, CardContent } from "@/components/ui/card"
import { getServicesSnapshot } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

export default async function PhantomPage() {
  const services = await getServicesSnapshot()
  const phantom = services.find((service) => service.id === "phantom")

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Phantom"
        title="Security-service seam"
        description="Phantom is represented here as a real service target in the ecosystem map. Scan result browsing will wait for a verified browser-safe endpoint instead of inventing a contract."
      />

      <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
        <CardContent className="space-y-5 p-6">
          <div className="flex items-center gap-3">
            <div className="rounded-2xl border border-primary/20 bg-primary/10 p-3 text-primary">
              <Shield className="size-5" />
            </div>
            <div>
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Current binding
              </div>
              <div className="text-2xl font-semibold tracking-tight text-foreground">
                {phantom?.name || "Phantom"}
              </div>
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            <StatusDot status={phantom?.status || "stub"} />
            <div className="font-mono text-xs text-muted-foreground">{phantom?.endpoint || "not configured"}</div>
          </div>
          <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4 text-sm leading-7 text-muted-foreground">
            {phantom?.detail || "No Phantom service has been configured yet."}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
