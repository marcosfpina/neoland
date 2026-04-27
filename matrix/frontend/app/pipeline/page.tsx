import Link from "next/link"
import { BookCopy, Radar } from "lucide-react"

import { PipelineRunner } from "@/components/pipeline/pipeline-runner"
import { PageHeader } from "@/components/shared/page-header"
import { Button } from "@/components/ui/button"
import { getServicesSnapshot } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

export default async function PipelinePage() {
  const services = await getServicesSnapshot()
  const pipelineStatus = services.find((service) => service.id === "pipeline")?.status || "stub"

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Pipeline"
        title="Run the Neoland multi-agent flow"
        description="This surface dispatches real tasks to the control plane and renders the final pipeline payload without inventing extra steps or browser-side mock stages."
        actions={
          <>
            <Button asChild variant="outline">
              <Link href="/services">
                <Radar className="size-4" />
                Check services
              </Link>
            </Button>
            <Button asChild variant="outline">
              <Link href="/adr">
                <BookCopy className="size-4" />
                Open vault
              </Link>
            </Button>
          </>
        }
      />

      <PipelineRunner pipelineStatus={pipelineStatus} />
    </div>
  )
}
