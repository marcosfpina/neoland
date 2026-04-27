import { notFound } from "next/navigation"

import { AdrViewer } from "@/components/adr/adr-viewer"
import { getAdrBySlug } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

export default async function AdrDetailPage({
  params,
}: {
  params: Promise<{ adrId: string }>
}) {
  const { adrId } = await params
  const adr = await getAdrBySlug(adrId)

  if (!adr) {
    notFound()
  }

  return <AdrViewer adr={adr} />
}
