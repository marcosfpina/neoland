import { FolderSearch } from "lucide-react"

import { SessionLookupForm } from "@/components/sessions/session-lookup-form"
import { PageHeader } from "@/components/shared/page-header"
import { Card, CardContent } from "@/components/ui/card"

export const dynamic = "force-dynamic"

export default function SessionsPage() {
  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Sessions"
        title="Inspect a real Neoland session"
        description="The verified backend exposes `GET /v1/agents/session/:id`, but not a session listing endpoint yet. This page stays lookup-first and avoids fabricating a browser-side session index."
      />

      <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
        <CardContent className="space-y-5 p-6">
          <div className="flex items-center gap-3">
            <div className="rounded-2xl border border-primary/20 bg-primary/10 p-3 text-primary">
              <FolderSearch className="size-5" />
            </div>
            <div>
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Session lookup
              </div>
              <div className="text-2xl font-semibold tracking-tight text-foreground">
                Jump to an existing session
              </div>
            </div>
          </div>
          <SessionLookupForm />
        </CardContent>
      </Card>

      <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
        <CardContent className="grid gap-4 p-6 xl:grid-cols-3">
          <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Verified endpoint
            </div>
            <p className="mt-3 text-sm leading-6 text-foreground">`GET /v1/agents/session/:id`</p>
          </div>
          <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Current limitation
            </div>
            <p className="mt-3 text-sm leading-6 text-foreground">
              No verified list endpoint for browsing all sessions.
            </p>
          </div>
          <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Best entry
            </div>
            <p className="mt-3 text-sm leading-6 text-foreground">
              Capture the returned session id from a pipeline run and continue here.
            </p>
          </div>
        </CardContent>
      </Card>

      <div className="rounded-[1.6rem] border border-dashed border-border/70 bg-card/60 p-6 text-sm leading-7 text-muted-foreground">
        The moment the backend exposes session listing or session-to-ADR chain queries, this page can
        become a richer browser. Until then, the workbench keeps the interface honest and only exposes
        what Neoland can answer today.
      </div>
    </div>
  )
}
