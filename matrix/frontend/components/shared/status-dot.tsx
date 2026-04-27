import { serviceMeta } from "@/lib/neoland/presentation"
import type { ServiceStatus } from "@/lib/neoland/types"
import { cn } from "@/lib/utils"

export function StatusDot({
  status,
  label,
  compact = false,
  className,
}: {
  status: ServiceStatus
  label?: string
  compact?: boolean
  className?: string
}) {
  const meta = serviceMeta[status]

  return (
    <div className={cn("inline-flex items-center gap-2", className)}>
      <span className={cn("inline-flex size-2.5 rounded-full", meta.dotClassName)} />
      {compact ? (
        <span className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
          {label || meta.label}
        </span>
      ) : (
        <span
          className={cn(
            "rounded-full border px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em]",
            meta.badgeClassName,
          )}
        >
          {label || meta.label}
        </span>
      )}
    </div>
  )
}
