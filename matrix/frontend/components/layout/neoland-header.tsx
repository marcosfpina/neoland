"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { ChevronRight, DatabaseZap, ShieldCheck, Waypoints } from "lucide-react"

import { neolandNavigation } from "@/components/layout/navigation"
import { StatusDot } from "@/components/shared/status-dot"

function resolveTitle(pathname: string) {
  const match = neolandNavigation.find((item) => pathname === item.href || pathname.startsWith(`${item.href}/`))
  return match || neolandNavigation[0]
}

export function NeolandHeader() {
  const pathname = usePathname()
  const current = resolveTitle(pathname)

  return (
    <header className="sticky top-0 z-30 border-b border-border/70 bg-background/80 backdrop-blur-xl">
      <div className="space-y-4 px-4 py-4 sm:px-6">
        <div className="scanline-overlay panel-ledge overflow-hidden rounded-[1.4rem] border border-border/70 bg-card/75 px-4 py-4 sm:px-5">
          <div className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent" />
          <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div className="space-y-2">
            <div className="flex items-center gap-2 font-mono text-[11px] uppercase tracking-[0.26em] text-muted-foreground">
              <Link href="/" className="transition-colors hover:text-foreground">
                Neoland
              </Link>
              <ChevronRight className="size-3" />
              <span className="text-foreground">{current.shortLabel}</span>
            </div>
            <div>
              <h2 className="text-xl font-semibold tracking-tight text-foreground sm:text-2xl">
                {current.label}
              </h2>
              <p className="text-sm leading-6 text-muted-foreground">{current.description}</p>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <div className="rounded-full border border-primary/20 bg-primary/10 px-3 py-1.5 text-xs text-primary">
              <span className="font-mono uppercase tracking-[0.22em]">real backend</span>
            </div>
            <div className="rounded-full border border-border/70 bg-card/80 px-3 py-1.5 text-xs text-muted-foreground">
              <span className="inline-flex items-center gap-2">
                <Waypoints className="size-3.5 text-primary" />
                Pipeline first
              </span>
            </div>
            <div className="rounded-full border border-border/70 bg-card/80 px-3 py-1.5 text-xs text-muted-foreground">
              <span className="inline-flex items-center gap-2">
                <ShieldCheck className="size-3.5 text-emerald-300" />
                No mock states
              </span>
            </div>
            <StatusDot status="up" label="Neoland Core" />
          </div>
        </div>
        </div>

        <div className="flex gap-2 overflow-x-auto pb-1 lg:hidden">
          {neolandNavigation.map((item) => {
            const active = pathname === item.href || pathname.startsWith(`${item.href}/`)
            const Icon = item.icon

            return (
              <Link
                key={item.href}
                href={item.href}
                className={
                  active
                    ? "inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/10 px-3 py-2 text-sm text-primary"
                    : "inline-flex items-center gap-2 rounded-full border border-border/70 bg-card/80 px-3 py-2 text-sm text-muted-foreground"
                }
              >
                <Icon className="size-4" />
                {item.shortLabel}
              </Link>
            )
          })}
        </div>

        <div className="grid gap-3 sm:grid-cols-3">
          <div className="signal-rail rounded-2xl border border-border/70 bg-card/70 px-4 py-3 panel-ledge">
            <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-muted-foreground">
              Surface
            </div>
            <div className="mt-2 flex items-center gap-2 text-sm text-foreground">
              <DatabaseZap className="size-4 text-primary" />
              Neoland Control Plane Interface
            </div>
          </div>
          <div className="signal-rail rounded-2xl border border-border/70 bg-card/70 px-4 py-3 panel-ledge">
            <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-muted-foreground">
              Source of truth
            </div>
            <div className="mt-2 text-sm text-foreground">Neoland control plane at <span className="font-mono text-primary">:3001</span></div>
          </div>
          <div className="signal-rail rounded-2xl border border-border/70 bg-card/70 px-4 py-3 panel-ledge">
            <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-muted-foreground">
              Runtime posture
            </div>
            <div className="mt-2 text-sm text-foreground">Degraded and empty states are explicit, never simulated.</div>
          </div>
        </div>
      </div>
    </header>
  )
}
