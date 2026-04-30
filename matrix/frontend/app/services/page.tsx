import { Zap } from "lucide-react"

export default function ServicesPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex items-center gap-3 mb-8">
        <div className="p-3 bg-pink-500/10 border border-pink-500/20 rounded-xl">
          <Zap className="w-6 h-6 text-pink-500" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-foreground">Service Health</h1>
          <p className="text-muted-foreground">Ecosystem status and control-plane health</p>
        </div>
      </div>

      <div className="rounded-xl border border-border/70 bg-card/50 p-12 text-center border-dashed">
        <div className="mx-auto w-12 h-12 rounded-full bg-emerald-500/10 flex items-center justify-center mb-4">
          <div className="w-2 h-2 bg-emerald-500 rounded-full animate-ping" />
        </div>
        <h3 className="text-lg font-medium text-foreground mb-2">Connecting to Control Plane</h3>
        <p className="text-muted-foreground max-w-sm mx-auto">
          Waiting to establish connection with the Neoland Core API on port 3001 to fetch active system topology.
        </p>
      </div>
    </div>
  )
}
