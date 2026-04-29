import Link from "next/link"
import { Activity, FolderSearch, Orbit, Radar } from "lucide-react"

import { DecisionBadge } from "@/components/shared/decision-badge"
import { MetricCard } from "@/components/shared/metric-card"
import { SessionLookupForm } from "@/components/sessions/session-lookup-form"
import { PageHeader } from "@/components/shared/page-header"
import { Card, CardContent } from "@/components/ui/card"
import { getSessions } from "@/lib/neoland/server"
import { formatRelativeTime, formatTimestamp, truncateMiddle } from "@/lib/utils"

export const dynamic = "force-dynamic"

export default async function SessionsPage() {
  const sessions = await getSessions(24)
  const activeCount = sessions.filter((session) => session.active).length
  const withDecisionCount = sessions.filter((session) => session.last_decision?.decision).length

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Sessions"
        title="Inspect and revisit the live session graph"
        description="This surface now combines direct lookup with a real recent-session index from the control plane, so active runs, follow-up forensics, and ADR handoff stay connected."
      />

      <div className="grid gap-4 md:grid-cols-3">
        <MetricCard
          label="Recent sessions"
          value={sessions.length}
          detail="Latest control-plane sessions ordered by activity."
          icon={<Orbit className="size-5" />}
          tone="accent"
        />
        <MetricCard
          label="Active now"
          value={activeCount}
          detail="Sessions still marked active in the orchestration layer."
          icon={<Activity className="size-5" />}
          tone="approve"
        />
        <MetricCard
          label="With decision"
          value={withDecisionCount}
          detail="Runs already carrying a last-decision snapshot for ADR follow-through."
          icon={<Radar className="size-5" />}
          tone="warning"
        />
      </div>

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

      {sessions.length ? (
        <div className="grid gap-4 xl:grid-cols-2">
          {sessions.map((session) => (
            <Link
              key={session.session_id}
              href={`/sessions/${session.session_id}`}
              className="group rounded-[1.4rem] border border-border/70 bg-card/80 p-5 transition-colors hover:border-primary/20 hover:bg-card/90"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="space-y-2">
                  <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-primary/80">
                    Session
                  </div>
                  <div className="text-lg font-semibold tracking-tight text-foreground">
                    {truncateMiddle(session.session_id, 14, 10)}
                  </div>
                  <div className="text-sm leading-6 text-muted-foreground">
                    Last activity {formatRelativeTime(session.last_activity)}
                  </div>
                </div>
                {session.last_decision?.decision ? (
                  <DecisionBadge decision={session.last_decision.decision} className="shrink-0" />
                ) : (
                  <div className="rounded-full border border-border/70 px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                    No decision yet
                  </div>
                )}
              </div>

              <div className="mt-5 grid gap-3 sm:grid-cols-3">
                <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                  <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                    Active
                  </div>
                  <div className="mt-2 text-sm text-foreground">{session.active ? "Yes" : "No"}</div>
                </div>
                <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                  <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                    Tasks
                  </div>
                  <div className="mt-2 text-sm text-foreground">{session.task_count}</div>
                </div>
                <div className="rounded-[1.2rem] border border-border/70 bg-background/35 p-4">
                  <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                    Updated
                  </div>
                  <div className="mt-2 text-sm text-foreground">{formatTimestamp(session.last_activity)}</div>
                </div>
              </div>

              <div className="mt-5 text-sm leading-6 text-muted-foreground">
                {session.last_decision?.session_summary ||
                  "Open this session to inspect its latest decision payload and continue the operational trail."}
              </div>
            </Link>
          ))}
        </div>
      ) : (
        <div className="rounded-[1.6rem] border border-dashed border-border/70 bg-card/60 p-6 text-sm leading-7 text-muted-foreground">
          No recent sessions came back from the control plane yet. Run a real pipeline task to seed
          the graph, or paste a known UUID above to jump straight into a specific session.
        </div>
      )}
    </div>
  )
}
