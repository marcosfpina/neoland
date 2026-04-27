import { Bot } from "lucide-react"

export default function Loading() {
  return (
    <div className="min-h-screen bg-background flex items-center justify-center">
      <div className="text-center">
        <Bot className="w-12 h-12 text-primary animate-pulse mx-auto mb-4" />
        <p className="text-muted-foreground">Connecting to Ollama...</p>
      </div>
    </div>
  )
}
