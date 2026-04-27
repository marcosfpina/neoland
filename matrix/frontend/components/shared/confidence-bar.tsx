import { confidencePercent, formatConfidence } from "@/lib/neoland/presentation"
import { cn } from "@/lib/utils"

export function ConfidenceBar({
  confidence,
  label = "Confidence",
  className,
}: {
  confidence: number
  label?: string
  className?: string
}) {
  const percent = confidencePercent(confidence)

  return (
    <div className={cn("space-y-2", className)}>
      <div className="flex items-center justify-between gap-3">
        <span className="text-xs uppercase tracking-[0.18em] text-muted-foreground">{label}</span>
        <span className="font-mono text-sm text-foreground">{formatConfidence(confidence)}</span>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-primary/10">
        <div
          className="h-full rounded-full bg-gradient-to-r from-primary via-sky-300 to-emerald-300 transition-all"
          style={{ width: `${percent}%` }}
        />
      </div>
    </div>
  )
}
