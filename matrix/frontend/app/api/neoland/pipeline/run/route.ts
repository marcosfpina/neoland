import { NextResponse } from "next/server"
import { z } from "zod"

import { runPipelineTask } from "@/lib/neoland/server"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

const bodySchema = z.object({
  task: z.string().trim().min(1, "task is required"),
  session_id: z
    .string()
    .trim()
    .uuid("session_id must be a valid UUID")
    .optional(),
})

export async function POST(request: Request) {
  try {
    const json = await request.json()
    const body = bodySchema.parse(json)
    const result = await runPipelineTask(body)
    return NextResponse.json(result)
  } catch (error) {
    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { error: error.issues[0]?.message || "Invalid request body." },
        { status: 400 },
      )
    }

    return NextResponse.json(
      {
        error: error instanceof Error ? error.message : "Failed to run pipeline task.",
      },
      { status: 502 },
    )
  }
}
