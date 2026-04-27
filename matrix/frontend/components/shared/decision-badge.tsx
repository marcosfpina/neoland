import { Badge } from "@/components/ui/badge"
import { decisionMeta } from "@/lib/neoland/presentation"
import type { AgentDecision } from "@/lib/neoland/types"
import { cn } from "@/lib/utils"

export function DecisionBadge({
  decision,
  className,
}: {
  decision: AgentDecision
  className?: string
}) {
  const meta = decisionMeta[decision]

  return (
    <Badge
      variant="outline"
      className={cn(
        "rounded-full border px-3 py-1 font-mono text-[11px] uppercase tracking-[0.26em]",
        meta.badgeClassName,
        className,
      )}
    >
      {meta.label}
    </Badge>
  )
}
