import Link from "next/link"
import { ArrowUpRight, FileStack } from "lucide-react"

import { DecisionBadge } from "@/components/shared/decision-badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { AdrDocument } from "@/lib/neoland/types"
import { formatRelativeTime, formatTimestamp } from "@/lib/utils"

export function AdrCard({ adr }: { adr: AdrDocument }) {
  return (
    <Card className="h-full overflow-hidden border-border/70 bg-card/85 metal-panel">
      <CardHeader className="border-b border-border/70 pb-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-3">
            <div className="rounded-2xl border border-primary/20 bg-primary/10 p-3 text-primary">
              <FileStack className="size-5" />
            </div>
            <div className="space-y-2">
              <div className="font-mono text-[11px] uppercase tracking-[0.26em] text-primary/80">
                {adr.adr_id}
              </div>
              <CardTitle className="text-xl tracking-tight text-foreground">{adr.title}</CardTitle>
            </div>
          </div>
          <DecisionBadge decision={adr.full_pipeline.tech_leader.decision} />
        </div>
      </CardHeader>
      <CardContent className="space-y-4 pt-6">
        <div className="grid gap-3 sm:grid-cols-2">
          <div className="rounded-2xl border border-border/70 bg-background/40 p-3">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Status
            </div>
            <div className="mt-2 text-sm text-foreground">{adr.status}</div>
          </div>
          <div className="rounded-2xl border border-border/70 bg-background/40 p-3">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Action items
            </div>
            <div className="mt-2 text-sm text-foreground">{adr.action_items.length}</div>
          </div>
        </div>
        <p className="line-clamp-4 text-sm leading-6 text-muted-foreground">{adr.decision}</p>
        <div className="flex items-center justify-between gap-3 text-xs text-muted-foreground">
          <div>
            <div>{formatTimestamp(adr.timestamp)}</div>
            <div>{formatRelativeTime(adr.timestamp)}</div>
          </div>
          <Link
            href={`/adr/${adr.slug}`}
            className="inline-flex items-center gap-2 rounded-full border border-border/70 bg-background/50 px-3 py-2 text-foreground transition-colors hover:border-primary/20 hover:text-primary"
          >
            Open ADR
            <ArrowUpRight className="size-4" />
          </Link>
        </div>
      </CardContent>
    </Card>
  )
}
