import { AlertTriangle, KeyRound, ServerCog } from "lucide-react"

import { MetricCard } from "@/components/shared/metric-card"
import { PageHeader } from "@/components/shared/page-header"
import { Card, CardContent } from "@/components/ui/card"
import { getSettingsSnapshot } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

export default function SettingsPage() {
  const settings = getSettingsSnapshot()

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Settings"
        title="Binding and runtime posture"
        description="This page reflects the current frontend-to-control-plane wiring and highlights whether the workbench is still relying on the development API key fallback."
      />

      <div className="grid gap-4 md:grid-cols-3">
        <MetricCard
          label="Control plane URL"
          value=":3001"
          detail={settings.controlPlaneUrl}
          icon={<ServerCog className="size-5" />}
          tone="accent"
        />
        <MetricCard
          label="DSPy URL"
          value=":8001"
          detail={settings.dspyUrl}
          icon={<ServerCog className="size-5" />}
          tone="default"
        />
        <MetricCard
          label="API key posture"
          value={settings.usingDevelopmentApiKey ? "DEV" : "BOUND"}
          detail={
            settings.usingDevelopmentApiKey
              ? "Using the development fallback key. Bind a real key before any public deployment."
              : "A real UI or Neoland API key is configured."
          }
          icon={settings.usingDevelopmentApiKey ? <AlertTriangle className="size-5" /> : <KeyRound className="size-5" />}
          tone={settings.usingDevelopmentApiKey ? "warning" : "approve"}
        />
      </div>

      <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
        <CardContent className="grid gap-4 p-6 xl:grid-cols-2">
          <div className="space-y-3 rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Checkpoint directory
            </div>
            <div className="break-all font-mono text-sm text-foreground">{settings.checkpointDir}</div>
          </div>
          <div className="space-y-3 rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Optional services
            </div>
            <div className="space-y-1 text-sm text-muted-foreground">
              <div>Phantom: {settings.phantomUrl || "not configured"}</div>
              <div>Neutron: {settings.neutronUrl || "not configured"}</div>
              <div>Cerebro: {settings.cerebroUrl || "not configured"}</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
