import { Activity } from "lucide-react"

export default function SessionsPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex items-center gap-3 mb-8">
        <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-xl">
          <Activity className="w-6 h-6 text-blue-500" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-foreground">Forensic Sessions</h1>
          <p className="text-muted-foreground">Deep inspection of session state and history</p>
        </div>
      </div>

      <div className="rounded-xl border border-border/70 bg-card/50 p-12 text-center border-dashed">
        <div className="mx-auto w-12 h-12 rounded-full bg-amber-500/10 flex items-center justify-center mb-4">
          <div className="w-2 h-2 bg-amber-500 rounded-full animate-ping" />
        </div>
        <h3 className="text-lg font-medium text-foreground mb-2">Awaiting Session Metadata</h3>
        <p className="text-muted-foreground max-w-sm mx-auto">
          The session viewer is waiting for the Neoland Control Plane to finalize the `GET /v1/agents/session/:id` endpoint contract.
        </p>
      </div>
    </div>
  )
}
