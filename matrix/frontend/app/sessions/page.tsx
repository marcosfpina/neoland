import Link from "next/link"
import { Activity, ArrowRight } from "lucide-react"

import { DecisionBadge } from "@/components/shared/decision-badge"
import { PageHeader } from "@/components/shared/page-header"
import { Card, CardContent } from "@/components/ui/card"
import { getSessions } from "@/lib/neoland/server"
import { formatRelativeTime } from "@/lib/utils"

export const dynamic = "force-dynamic"

export default async function SessionsPage() {
  const sessions = await getSessions(24)

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Forensic sessions"
        title="Session registry"
        description="Recent sessions pulled from GET /v1/agents/sessions. Each entry links to the full decision snapshot and session state."
      />

      {sessions.length === 0 ? (
        <div className="rounded-[1.4rem] border border-dashed border-border/70 bg-card/60 p-12 text-center">
          <div className="mx-auto mb-4 flex size-12 items-center justify-center rounded-full bg-amber-500/10">
            <Activity className="size-5 text-amber-400" />
          </div>
          <h3 className="text-lg font-medium text-foreground">No sessions yet</h3>
          <p className="mx-auto mt-2 max-w-sm text-sm text-muted-foreground">
            Run a pipeline task to create the first session. The control plane registers sessions
            automatically on task submission.
          </p>
          <Link
            href="/pipeline"
            className="mt-6 inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/10 px-5 py-2.5 text-sm text-primary transition-colors hover:bg-primary/14"
          >
            <ArrowRight className="size-4" />
            Go to pipeline
          </Link>
        </div>
      ) : (
        <div className="grid gap-4">
          {sessions.map((session) => (
            <Link key={session.session_id} href={`/sessions/${session.session_id}`}>
              <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel transition-colors hover:border-primary/30">
                <CardContent className="p-5">
                  <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div className="space-y-1 min-w-0">
                      <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-primary/80">
                        Session
                      </div>
                      <div className="truncate font-mono text-sm text-foreground">
                        {session.session_id}
                      </div>
                    </div>

                    <div className="flex flex-wrap items-center gap-6">
                      <div className="text-center">
                        <div className="font-mono text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
                          Tasks
                        </div>
                        <div className="mt-1 text-lg font-semibold text-foreground">
                          {session.task_count}
                        </div>
                      </div>
                      <div className="text-center">
                        <div className="font-mono text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
                          Active
                        </div>
                        <div className="mt-1 text-lg font-semibold text-foreground">
                          {session.active ? "Yes" : "No"}
                        </div>
                      </div>
                      <div className="text-center">
                        <div className="font-mono text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
                          Last activity
                        </div>
                        <div className="mt-1 text-sm text-muted-foreground">
                          {formatRelativeTime(session.last_activity)}
                        </div>
                      </div>
                      {session.last_decision?.decision ? (
                        <DecisionBadge decision={session.last_decision.decision} />
                      ) : (
                        <span className="text-xs text-muted-foreground">no decision</span>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
