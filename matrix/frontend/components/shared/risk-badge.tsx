import { Badge } from "@/components/ui/badge"
import { riskMeta } from "@/lib/neoland/presentation"
import type { RiskLevel } from "@/lib/neoland/types"
import { cn } from "@/lib/utils"

export function RiskBadge({
  risk,
  className,
}: {
  risk: RiskLevel
  className?: string
}) {
  const meta = riskMeta[risk]

  return (
    <Badge
      variant="outline"
      className={cn(
        "rounded-full border px-3 py-1 text-[11px] uppercase tracking-[0.22em]",
        meta.badgeClassName,
        className,
      )}
    >
      <span className={cn("mr-2 inline-flex size-1.5 rounded-full", meta.dotClassName)} />
      {meta.label}
    </Badge>
  )
}
