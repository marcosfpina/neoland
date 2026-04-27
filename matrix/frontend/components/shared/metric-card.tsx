import type { ReactNode } from "react"

import { Card, CardContent } from "@/components/ui/card"
import { cn } from "@/lib/utils"

export function MetricCard({
  label,
  value,
  detail,
  icon,
  tone = "default",
}: {
  label: string
  value: string | number
  detail: string
  icon?: ReactNode
  tone?: "default" | "approve" | "warning" | "danger" | "accent"
}) {
  const accent =
    tone === "approve"
      ? "from-emerald-500/18 to-emerald-500/0 text-emerald-300 border-emerald-500/20"
      : tone === "warning"
        ? "from-amber-500/18 to-amber-500/0 text-amber-300 border-amber-500/20"
        : tone === "danger"
          ? "from-red-500/18 to-red-500/0 text-red-300 border-red-500/20"
          : tone === "accent"
            ? "from-primary/18 to-primary/0 text-primary border-primary/20"
            : "from-white/6 to-white/0 text-foreground border-border/70"

  return (
    <Card className="overflow-hidden border-border/70 bg-card/80 metal-panel">
      <CardContent className="p-0">
        <div className={cn("border-b bg-gradient-to-br px-5 py-4", accent)}>
          <div className="flex items-center justify-between gap-4">
            <div className="space-y-1">
              <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">
                {label}
              </p>
              <div className="text-3xl font-semibold tracking-tight">{value}</div>
            </div>
            {icon ? <div className="rounded-2xl border border-white/10 p-3">{icon}</div> : null}
          </div>
        </div>
        <div className="px-5 py-4 text-sm leading-6 text-muted-foreground">{detail}</div>
      </CardContent>
    </Card>
  )
}
