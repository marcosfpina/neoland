import Link from "next/link"
import { ArrowRight, BookCopy, FolderSearch } from "lucide-react"

import { AdrCard } from "@/components/adr/adr-card"
import { PageHeader } from "@/components/shared/page-header"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { getAdrBySlug, getSessionById } from "@/lib/neoland/server"
import { formatTimestamp } from "@/lib/utils"

export const dynamic = "force-dynamic"

export default async function PipelineTaskPage({
  params,
  searchParams,
}: {
  params: Promise<{ taskId: string }>
  searchParams: Promise<{ sessionId?: string; adr?: string }>
}) {
  const { taskId } = await params
  const query = await searchParams
  const sessionId = query.sessionId
  const adrSlug = query.adr

  const [session, adr] = await Promise.all([
    sessionId ? getSessionById(sessionId) : Promise.resolve(null),
    adrSlug ? getAdrBySlug(adrSlug) : Promise.resolve(null),
  ])

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Task surface"
        title={`Pipeline task ${taskId}`}
        description="The current backend does not expose a task-detail endpoint by `task_id` yet, so this page acts as a handoff surface into the session and checkpoint artifacts already returned by the real run."
      />

      <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
        <CardContent className="space-y-5 p-6">
          <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
            Verified limitation
          </div>
          <p className="text-sm leading-7 text-foreground">
            Neoland currently verifies `POST /v1/agents/task`, `GET /v1/agents/session/:id`, and
            ADR checkpoint output, but not `GET /v1/agents/task/:id`. This page stays honest about
            that gap and links you to the artifacts that already exist.
          </p>
          <div className="flex flex-wrap gap-3">
            {sessionId ? (
              <Button asChild>
                <Link href={`/sessions/${sessionId}`}>
                  <FolderSearch className="size-4" />
                  Open session
                </Link>
              </Button>
            ) : null}
            {adrSlug ? (
              <Button asChild variant="outline">
                <Link href={`/adr/${adrSlug}`}>
                  <BookCopy className="size-4" />
                  Open ADR
                </Link>
              </Button>
            ) : null}
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-6 xl:grid-cols-2">
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="space-y-4 p-6">
            <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
              Session handoff
            </div>
            {session ? (
              <div className="space-y-3">
                <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                  <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                    Session ID
                  </div>
                  <div className="mt-2 font-mono text-sm text-foreground">{session.session_id}</div>
                </div>
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                    <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                      Task count
                    </div>
                    <div className="mt-2 text-2xl font-semibold text-foreground">{session.task_count}</div>
                  </div>
                  <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                    <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                      Last activity
                    </div>
                    <div className="mt-2 text-sm text-foreground">{formatTimestamp(session.last_activity)}</div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="rounded-[1.2rem] border border-dashed border-border/70 bg-background/30 p-4 text-sm text-muted-foreground">
                No session was attached to this task surface. The runner will add one automatically
                once a real pipeline response includes a session id.
              </div>
            )}
          </CardContent>
        </Card>

        <div>
          {adr ? (
            <AdrCard adr={adr} />
          ) : (
            <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
              <CardContent className="space-y-4 p-6">
                <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                  Checkpoint handoff
                </div>
                <p className="text-sm leading-7 text-muted-foreground">
                  No ADR slug was attached to this task surface. When the pipeline writes a real checkpoint,
                  this page will link into the vault.
                </p>
                <Button asChild variant="ghost" className="w-fit">
                  <Link href="/adr">
                    Open vault
                    <ArrowRight className="size-4" />
                  </Link>
                </Button>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
