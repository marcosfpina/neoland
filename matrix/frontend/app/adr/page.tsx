import { FileText } from "lucide-react"

export default function ADRPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex items-center gap-3 mb-8">
        <div className="p-3 bg-violet-500/10 border border-violet-500/20 rounded-xl">
          <FileText className="w-6 h-6 text-violet-500" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-foreground">ADR Vault</h1>
          <p className="text-muted-foreground">Architecture Decision Records and operational checkpoints</p>
        </div>
      </div>

      <div className="rounded-xl border border-border/70 bg-card/50 p-12 text-center border-dashed">
        <div className="mx-auto w-12 h-12 rounded-full bg-amber-500/10 flex items-center justify-center mb-4">
          <div className="w-2 h-2 bg-amber-500 rounded-full animate-ping" />
        </div>
        <h3 className="text-lg font-medium text-foreground mb-2">Awaiting Ledger Contract</h3>
        <p className="text-muted-foreground max-w-sm mx-auto">
          The structured view for Architecture Decision Records is currently degraded. The UI requires the backend to serve structured checkpoints instead of raw JSON.
        </p>
      </div>
    </div>
  )
}
