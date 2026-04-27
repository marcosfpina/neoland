import Link from "next/link"
import { ArrowRight, BookCopy, FolderSearch } from "lucide-react"

import { DecisionBadge } from "@/components/shared/decision-badge"
import { PageHeader } from "@/components/shared/page-header"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { SessionLookupForm } from "@/components/sessions/session-lookup-form"
import { getSessionById } from "@/lib/neoland/server"
import { formatTimestamp } from "@/lib/utils"

export const dynamic = "force-dynamic"

export default async function SessionDetailPage({
  params,
}: {
  params: Promise<{ sessionId: string }>
}) {
  const { sessionId } = await params
  const session = await getSessionById(sessionId)

  if (!session) {
    return (
      <div className="space-y-8">
        <PageHeader
          eyebrow="Session lookup"
          title="Session not available"
          description="The control plane did not return a session for this id. It may be missing, the orchestrator may be disabled, or the backend may be unavailable."
        />
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="space-y-5 p-6">
            <div className="flex items-center gap-3">
              <FolderSearch className="size-5 text-primary" />
              <div className="text-lg font-semibold tracking-tight text-foreground">
                Try another session id
              </div>
            </div>
            <SessionLookupForm initialValue={sessionId} />
          </CardContent>
        </Card>
      </div>
    )
  }

  const decision = session.last_decision?.decision

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Session detail"
        title={session.session_id}
        description="This view reads the session state directly from `GET /v1/agents/session/:id` and stays flexible about the decision payload, because the backend stores it as JSON."
        actions={decision ? <DecisionBadge decision={decision} /> : undefined}
      />

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="p-6">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Active
            </div>
            <div className="mt-3 text-3xl font-semibold tracking-tight text-foreground">
              {session.active ? "Yes" : "No"}
            </div>
          </CardContent>
        </Card>
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="p-6">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Task count
            </div>
            <div className="mt-3 text-3xl font-semibold tracking-tight text-foreground">
              {session.task_count}
            </div>
          </CardContent>
        </Card>
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="p-6">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Last activity
            </div>
            <div className="mt-3 text-sm leading-6 text-foreground">{formatTimestamp(session.last_activity)}</div>
          </CardContent>
        </Card>
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="p-6">
            <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
              Decision payload
            </div>
            <div className="mt-3 text-sm leading-6 text-foreground">
              {session.last_decision ? "Present" : "Not captured yet"}
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 xl:grid-cols-[1fr_0.9fr]">
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="space-y-5 p-6">
            <div>
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Last decision
              </div>
              <div className="mt-2 text-2xl font-semibold tracking-tight text-foreground">
                Session summary snapshot
              </div>
            </div>
            {session.last_decision ? (
              <div className="space-y-4">
                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                    <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                      Decision
                    </div>
                    <div className="mt-3">
                      {decision ? <DecisionBadge decision={decision} /> : <span className="text-muted-foreground">n/a</span>}
                    </div>
                  </div>
                  <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                    <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                      ADR title
                    </div>
                    <div className="mt-3 text-sm leading-6 text-foreground">
                      {session.last_decision.adr_title || "Not present"}
                    </div>
                  </div>
                </div>
                <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                  <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                    Session summary
                  </div>
                  <div className="mt-3 text-sm leading-7 text-foreground">
                    {session.last_decision.session_summary || "Not present"}
                  </div>
                </div>
                <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                  <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                    Raw payload
                  </div>
                  <pre className="mt-3 overflow-x-auto rounded-xl bg-background/60 p-4 text-xs text-muted-foreground">
                    {JSON.stringify(session.last_decision, null, 2)}
                  </pre>
                </div>
              </div>
            ) : (
              <div className="rounded-[1.2rem] border border-dashed border-border/70 bg-background/30 p-4 text-sm text-muted-foreground">
                The session exists, but no decision snapshot has been recorded yet.
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="space-y-5 p-6">
            <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
              Next surface
            </div>
            <div className="text-2xl font-semibold tracking-tight text-foreground">
              Continue from the verified contract
            </div>
            <p className="text-sm leading-7 text-muted-foreground">
              This backend response does not currently expose a session-to-ADR chain query, so the workbench does not invent that relationship. Use the ADR vault for checkpoint browsing and the pipeline page for new runs.
            </p>
            <div className="flex flex-wrap gap-3">
              <Button asChild>
                <Link href="/pipeline">
                  <ArrowRight className="size-4" />
                  Run another task
                </Link>
              </Button>
              <Button asChild variant="outline">
                <Link href="/adr">
                  <BookCopy className="size-4" />
                  Open ADR Vault
                </Link>
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
