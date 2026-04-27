import Link from "next/link"
import { Search } from "lucide-react"

import { AdrCard } from "@/components/adr/adr-card"
import { PageHeader } from "@/components/shared/page-header"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { getAdrDocuments } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

export default async function AdrPage({
  searchParams,
}: {
  searchParams: Promise<{ q?: string }>
}) {
  const query = await searchParams
  const adrs = await getAdrDocuments()
  const term = query.q?.trim().toLowerCase() || ""
  const filtered = term
    ? adrs.filter((adr) =>
        [adr.adr_id, adr.title, adr.context, adr.decision, adr.session_summary]
          .join(" ")
          .toLowerCase()
          .includes(term),
      )
    : adrs

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="ADR Vault"
        title="Checkpoint repository"
        description="Every item here is rendered from a real ADR checkpoint file. Search stays local to the current JSON corpus and does not fabricate data from outside the vault."
      />

      <form className="flex flex-col gap-3 sm:flex-row" method="GET">
        <Input
          name="q"
          defaultValue={query.q || ""}
          placeholder="Search ADR title, decision, context, or summary"
          className="h-11 rounded-[1.1rem] border-border/70 bg-card/75"
        />
        <Button type="submit" size="lg">
          <Search className="size-4" />
          Search vault
        </Button>
      </form>

      {filtered.length ? (
        <div className="grid gap-4 xl:grid-cols-2">
          {filtered.map((adr) => (
            <AdrCard key={adr.slug} adr={adr} />
          ))}
        </div>
      ) : (
        <div className="rounded-[1.6rem] border border-dashed border-border/70 bg-card/60 p-8 text-sm leading-7 text-muted-foreground">
          {term
            ? `No ADR matched "${query.q}".`
            : "No ADR checkpoint files were found in the configured directory."}{" "}
          <Link href="/pipeline" className="text-primary underline-offset-4 hover:underline">
            Run a real task
          </Link>{" "}
          to generate the next checkpoint.
        </div>
      )}
    </div>
  )
}
