import type { ReactNode } from "react"

import { cn } from "@/lib/utils"

export function PageHeader({
  eyebrow,
  title,
  description,
  actions,
  className,
}: {
  eyebrow?: string
  title: string
  description: string
  actions?: ReactNode
  className?: string
}) {
  return (
    <div
      className={cn(
        "scanline-overlay panel-ledge relative flex flex-col gap-5 overflow-hidden rounded-[1.6rem] border border-border/70 bg-card/80 p-6 metal-panel sm:p-8",
        className,
      )}
    >
      <div className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent" />
      <div className="pointer-events-none absolute -right-24 top-0 h-48 w-48 rounded-full bg-primary/10 blur-3xl" />
      <div className="pointer-events-none absolute -left-10 bottom-0 h-24 w-24 rounded-full bg-violet-500/10 blur-3xl" />
      <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
        <div className="space-y-3">
          {eyebrow ? (
            <div className="font-mono text-[11px] uppercase tracking-[0.28em] text-primary/80">
              {eyebrow}
            </div>
          ) : null}
          <div className="space-y-2">
            <h1 className="text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">
              {title}
            </h1>
            <p className="max-w-3xl text-sm leading-6 text-muted-foreground sm:text-base">
              {description}
            </p>
          </div>
        </div>
        {actions ? <div className="flex flex-wrap items-center gap-3">{actions}</div> : null}
      </div>
      <div className="grid gap-3 sm:grid-cols-3">
        <div className="signal-rail rounded-2xl border border-border/70 bg-background/35 px-4 py-3">
          <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
            Surface identity
          </div>
          <div className="mt-2 text-sm text-foreground">Neoland Operational Console</div>
        </div>
        <div className="signal-rail rounded-2xl border border-border/70 bg-background/35 px-4 py-3">
          <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
            Contract discipline
          </div>
          <div className="mt-2 text-sm text-foreground">Verified backend fields only, adapters at the boundary</div>
        </div>
        <div className="signal-rail rounded-2xl border border-border/70 bg-background/35 px-4 py-3">
          <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
            Runtime posture
          </div>
          <div className="mt-2 text-sm text-foreground">Empty and stub states stay visible instead of simulated</div>
        </div>
      </div>
    </div>
  )
}
