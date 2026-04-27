/**
 * Ollama Management API Route
 * Manage Ollama models and inference
 */

import { type NextRequest, NextResponse } from "next/server"

const OLLAMA_HOST = process.env.OLLAMA_HOST || "http://localhost:11434"

async function ollamaFetch(endpoint: string, options?: RequestInit) {
  const response = await fetch(`${OLLAMA_HOST}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  })

  if (!response.ok) {
    throw new Error(`Ollama API error: ${response.status}`)
  }

  return response.json()
}

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  const action = searchParams.get("action") || "status"

  try {
    switch (action) {
      case "status": {
        // Check if Ollama is running
        const tags = await ollamaFetch("/api/tags")
        const ps = await ollamaFetch("/api/ps")

        return NextResponse.json({
          success: true,
          running: true,
          models: tags.models || [],
          loaded_models: ps.models || [],
          host: OLLAMA_HOST,
        })
      }

      case "models": {
        const tags = await ollamaFetch("/api/tags")
        return NextResponse.json({
          success: true,
          models: tags.models || [],
        })
      }

      case "running": {
        const ps = await ollamaFetch("/api/ps")
        return NextResponse.json({
          success: true,
          loaded_models: ps.models || [],
        })
      }

      default:
        return NextResponse.json({ success: false, error: `Unknown action: ${action}` }, { status: 400 })
    }
  } catch (error) {
    return NextResponse.json({
      success: false,
      running: false,
      error: error instanceof Error ? error.message : "Ollama not available",
    })
  }
}

export async function POST(request: NextRequest) {
  const body = await request.json()
  const { action, model, prompt, options } = body

  try {
    switch (action) {
      case "pull": {
        // Start model pull
        const response = await fetch(`${OLLAMA_HOST}/api/pull`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ name: model }),
        })

        // Return streaming response
        return new NextResponse(response.body, {
          headers: {
            "Content-Type": "application/x-ndjson",
            "Transfer-Encoding": "chunked",
          },
        })
      }

      case "delete": {
        await ollamaFetch("/api/delete", {
          method: "DELETE",
          body: JSON.stringify({ name: model }),
        })

        return NextResponse.json({ success: true, message: `Model ${model} deleted` })
      }

      case "load": {
        // Generate with empty prompt to load model into memory
        await ollamaFetch("/api/generate", {
          method: "POST",
          body: JSON.stringify({
            model,
            prompt: "",
            options: { num_predict: 0 },
          }),
        })

        return NextResponse.json({ success: true, message: `Model ${model} loaded` })
      }

      case "unload": {
        await ollamaFetch("/api/generate", {
          method: "POST",
          body: JSON.stringify({
            model,
            prompt: "",
            keep_alive: 0,
          }),
        })

        return NextResponse.json({ success: true, message: `Model ${model} unloaded` })
      }

      case "generate": {
        const response = await fetch(`${OLLAMA_HOST}/api/generate`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            model,
            prompt,
            stream: true,
            options,
          }),
        })

        return new NextResponse(response.body, {
          headers: {
            "Content-Type": "application/x-ndjson",
            "Transfer-Encoding": "chunked",
          },
        })
      }

      case "chat": {
        const response = await fetch(`${OLLAMA_HOST}/api/chat`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            model,
            messages: body.messages,
            stream: true,
            options,
          }),
        })

        return new NextResponse(response.body, {
          headers: {
            "Content-Type": "application/x-ndjson",
            "Transfer-Encoding": "chunked",
          },
        })
      }

      default:
        return NextResponse.json({ success: false, error: `Unknown action: ${action}` }, { status: 400 })
    }
  } catch (error) {
    return NextResponse.json(
      { success: false, error: error instanceof Error ? error.message : "Unknown error" },
      { status: 500 },
    )
  }
}
