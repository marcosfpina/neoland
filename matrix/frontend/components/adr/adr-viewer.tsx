import { FileJson2, FolderTree } from "lucide-react"

import { AgentCard } from "@/components/pipeline/agent-card"
import { DecisionBadge } from "@/components/shared/decision-badge"
import { PageHeader } from "@/components/shared/page-header"
import { Card, CardContent } from "@/components/ui/card"
import type { AdrDocument } from "@/lib/neoland/types"
import { formatTimestamp } from "@/lib/utils"

export function AdrViewer({ adr }: { adr: AdrDocument }) {
  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow={adr.adr_id}
        title={adr.title}
        description="Structured checkpoint rendering from the real ADR JSON written by the pipeline."
        actions={<DecisionBadge decision={adr.full_pipeline.tech_leader.decision} />}
      />

      <div className="grid gap-6 xl:grid-cols-[1.2fr_0.8fr]">
        <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
          <CardContent className="space-y-6 p-6">
            <section className="space-y-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Context
              </div>
              <p className="text-sm leading-7 text-foreground">{adr.context}</p>
            </section>

            <section className="space-y-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Decision
              </div>
              <p className="text-sm leading-7 text-foreground">{adr.decision}</p>
            </section>

            <section className="space-y-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Session summary
              </div>
              <p className="text-sm leading-7 text-muted-foreground">{adr.session_summary}</p>
            </section>

            <section className="space-y-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                Action items
              </div>
              {adr.action_items.length ? (
                <ul className="space-y-3 text-sm text-foreground">
                  {adr.action_items.map((item) => (
                    <li key={item} className="flex items-start gap-3 rounded-2xl border border-border/70 bg-background/30 px-4 py-3">
                      <span className="mt-2 inline-flex size-1.5 rounded-full bg-primary" />
                      <span>{item}</span>
                    </li>
                  ))}
                </ul>
              ) : (
                <div className="rounded-2xl border border-dashed border-border/70 bg-background/30 p-4 text-sm text-muted-foreground">
                  No action items were captured in this checkpoint.
                </div>
              )}
            </section>
          </CardContent>
        </Card>

        <div className="space-y-6">
          <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
            <CardContent className="space-y-4 p-6">
              <div className="flex items-center gap-3">
                <div className="rounded-2xl border border-primary/20 bg-primary/10 p-3 text-primary">
                  <FileJson2 className="size-5" />
                </div>
                <div>
                  <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                    Metadata
                  </div>
                  <div className="text-lg font-semibold tracking-tight text-foreground">
                    Checkpoint index
                  </div>
                </div>
              </div>
              <div className="space-y-3 text-sm">
                <div className="rounded-2xl border border-border/70 bg-background/30 p-4">
                  <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                    Timestamp
                  </div>
                  <div className="mt-2 text-foreground">{formatTimestamp(adr.timestamp)}</div>
                </div>
                <div className="rounded-2xl border border-border/70 bg-background/30 p-4">
                  <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                    Stored path
                  </div>
                  <div className="mt-2 break-all font-mono text-xs text-muted-foreground">
                    {adr.source_path}
                  </div>
                </div>
                <div className="rounded-2xl border border-border/70 bg-background/30 p-4">
                  <div className="font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                    Session reuse
                  </div>
                  <div className="mt-2 text-muted-foreground">
                    `session_summary` is available to be injected into a later run as RAG context by the backend pipeline.
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="overflow-hidden border-border/70 bg-card/85 metal-panel">
            <CardContent className="space-y-4 p-6">
              <div className="flex items-center gap-3">
                <div className="rounded-2xl border border-primary/20 bg-primary/10 p-3 text-primary">
                  <FolderTree className="size-5" />
                </div>
                <div>
                  <div className="font-mono text-[11px] uppercase tracking-[0.24em] text-primary/80">
                    Full pipeline
                  </div>
                  <div className="text-lg font-semibold tracking-tight text-foreground">
                    Agent snapshot
                  </div>
                </div>
              </div>
              <div className="grid gap-4">
                <AgentCard stage="junior" state="complete" junior={adr.full_pipeline.junior} compact />
                <AgentCard stage="senior" state="complete" senior={adr.full_pipeline.senior} compact />
                {adr.full_pipeline.architect ? (
                  <AgentCard
                    stage="architect"
                    state="complete"
                    architect={adr.full_pipeline.architect}
                    compact
                  />
                ) : (
                  <AgentCard stage="architect" state="skipped" compact />
                )}
                <AgentCard
                  stage="tech_leader"
                  state="complete"
                  techLeader={adr.full_pipeline.tech_leader}
                  compact
                />
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
