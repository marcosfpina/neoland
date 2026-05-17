import { PipelineRunner } from "@/components/pipeline/pipeline-runner"
import { getServicesSnapshot } from "@/lib/neoland/server"

export const dynamic = "force-dynamic"

export default async function PipelinePage() {
  const services = await getServicesSnapshot()
  const pipeline = services.find((s) => s.id === "pipeline")

  return <PipelineRunner pipelineStatus={pipeline?.status} />
}
