import { Terminal } from "lucide-react"

export default function PipelinePage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex items-center gap-3 mb-8">
        <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl">
          <Terminal className="w-6 h-6 text-emerald-500" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-foreground">Pipeline Live View</h1>
          <p className="text-muted-foreground">Execution tracking and agent progression</p>
        </div>
      </div>

      <div className="rounded-xl border border-border/70 bg-card/50 p-12 text-center border-dashed">
        <div className="mx-auto w-12 h-12 rounded-full bg-amber-500/10 flex items-center justify-center mb-4">
          <div className="w-2 h-2 bg-amber-500 rounded-full animate-ping" />
        </div>
        <h3 className="text-lg font-medium text-foreground mb-2">Awaiting Backend Contract</h3>
        <p className="text-muted-foreground max-w-sm mx-auto">
          The pipeline execution view is currently in degraded state while the Neoland backend orchestrator completes its API integration.
        </p>
      </div>
    </div>
  )
}
