"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { Command, CornerDownRight } from "lucide-react"

import { neolandNavigation } from "@/components/layout/navigation"
import { cn } from "@/lib/utils"

function isActive(pathname: string, href: string) {
  if (href === "/") {
    return pathname === "/"
  }

  return pathname === href || pathname.startsWith(`${href}/`)
}

export function NeolandSidebar() {
  const pathname = usePathname()

  return (
    <aside className="hidden h-screen w-72 shrink-0 border-r border-border/70 bg-sidebar/95 px-4 py-5 lg:fixed lg:inset-y-0 lg:left-0 lg:flex lg:flex-col">
      <div className="scanline-overlay ring-cyan rounded-[1.4rem] border border-border/70 bg-card/80 p-4 metal-panel">
        <div className="flex items-center gap-3">
          <div className="rounded-2xl border border-primary/20 bg-primary/10 p-3 text-primary shadow-[0_0_30px_rgba(34,211,238,0.1)]">
            <Command className="size-5" />
          </div>
          <div>
            <div className="font-mono text-[11px] uppercase tracking-[0.28em] text-primary/80">
              Neoland
            </div>
            <div className="text-lg font-semibold tracking-tight text-foreground">
              Control Plane
            </div>
          </div>
        </div>
        <p className="mt-4 text-sm leading-6 text-muted-foreground">
          Neoland Control Plane, the specialized workbench for the multi-agent ecosystem.
        </p>
        <div className="mt-4 grid grid-cols-2 gap-3">
          <div className="rounded-xl border border-border/70 bg-background/35 px-3 py-2">
            <div className="font-mono text-[10px] uppercase tracking-[0.22em] text-muted-foreground">
              posture
            </div>
            <div className="mt-1 text-sm text-foreground">bare metal</div>
          </div>
          <div className="rounded-xl border border-border/70 bg-background/35 px-3 py-2">
            <div className="font-mono text-[10px] uppercase tracking-[0.22em] text-muted-foreground">
              focus
            </div>
            <div className="mt-1 text-sm text-foreground">pipeline</div>
          </div>
        </div>
      </div>

      <div className="mt-6 mb-3 font-mono text-[11px] uppercase tracking-[0.28em] text-muted-foreground">
        Operator surfaces
      </div>
      <nav className="flex-1 space-y-2">
        {neolandNavigation.map((item) => {
          const active = isActive(pathname, item.href)
          const Icon = item.icon

          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "group block rounded-[1.2rem] border px-4 py-3 transition-all panel-ledge",
                active
                  ? "border-primary/25 bg-primary/10 shadow-[0_18px_40px_rgba(8,145,178,0.12)]"
                  : "border-transparent bg-transparent hover:border-border/70 hover:bg-card/70",
              )}
            >
              <div className="flex items-start gap-3">
                <div
                  className={cn(
                    "mt-0.5 rounded-xl border p-2 transition-colors",
                    active
                      ? "border-primary/20 bg-primary/10 text-primary"
                      : "border-border/60 bg-card/70 text-muted-foreground group-hover:text-foreground",
                  )}
                >
                  <Icon className="size-4" />
                </div>
                <div className="min-w-0 space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-foreground">{item.label}</span>
                    {active ? (
                      <span className="rounded-full border border-primary/25 bg-primary/10 px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.22em] text-primary">
                        live
                      </span>
                    ) : null}
                  </div>
                  <p className="text-xs leading-5 text-muted-foreground">{item.description}</p>
                </div>
              </div>
            </Link>
          )
        })}
      </nav>

      <div className="scanline-overlay rounded-[1.2rem] border border-border/70 bg-card/70 p-4 metal-panel">
        <div className="font-mono text-[11px] uppercase tracking-[0.28em] text-muted-foreground">
          Operating mode
        </div>
        <div className="mt-2 flex items-center gap-2 text-sm text-foreground">
          <CornerDownRight className="size-4 text-primary" />
          Bare-metal integration
        </div>
        <p className="mt-2 text-xs leading-5 text-muted-foreground">
          No mock pipeline states. Every screen reads from the Neoland control plane, health probes,
          or real checkpoint files.
        </p>
        <div className="mt-4 h-px bg-gradient-to-r from-primary/40 via-primary/0 to-transparent" />
      </div>
    </aside>
  )
}
