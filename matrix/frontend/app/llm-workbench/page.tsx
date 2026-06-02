"use client"

import { useState, useCallback } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Textarea } from "@/components/ui/textarea"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Checkbox } from "@/components/ui/checkbox"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import {
  Bot,
  ArrowLeft,
  Play,
  Loader2,
  Copy,
  ChevronDown,
  Settings2,
  Sparkles,
  Trash2,
  History,
  CheckCircle2,
  Server,
  Cloud,
} from "lucide-react"
import Link from "next/link"
import { LLM_PROVIDERS, getAllModels, getModelById } from "@/lib/llm-workbench/providers"
import type { LLMModel, ComparisonResponse, GenerationParameters } from "@/lib/llm-workbench/types"
import { cn } from "@/lib/utils"
import { useSearchParams } from "next/navigation"

const DEFAULT_PARAMS: GenerationParameters = {
  temperature: 0.7,
  maxTokens: 2048,
  topP: 1,
  frequencyPenalty: 0,
  presencePenalty: 0,
  stream: true,
}

// Simulated response generation
async function simulateResponse(model: LLMModel, prompt: string, delay: number): Promise<ComparisonResponse> {
  await new Promise((resolve) => setTimeout(resolve, delay))

  const inputTokens = Math.ceil(prompt.length / 4)
  const outputTokens = Math.floor(Math.random() * 500) + 200

  const responses: Record<string, string> = {
    "gpt-4.1": `Here's my analysis of your request:\n\n${prompt.slice(0, 50)}...\n\nBased on the context provided, I recommend:\n\n1. **First Approach**: Consider implementing a modular architecture that separates concerns effectively.\n\n2. **Best Practice**: Use TypeScript for type safety and better developer experience.\n\n3. **Performance**: Implement lazy loading and code splitting for optimal performance.\n\nLet me know if you need more specific guidance on any of these points.`,
    "claude-sonnet-4": `I'll help you with this request.\n\nAnalyzing: "${prompt.slice(0, 50)}..."\n\n**Key Considerations:**\n\n• The approach should prioritize maintainability and scalability\n• Consider using established patterns from the community\n• Ensure proper error handling throughout\n\n**Recommended Solution:**\n\n\`\`\`typescript\n// Example implementation\nconst solution = {\n  approach: 'modular',\n  patterns: ['observer', 'factory'],\n  testing: 'comprehensive'\n}\n\`\`\`\n\nThis provides a solid foundation while remaining flexible for future changes.`,
    "gemini-2.5-pro": `# Response to Your Query\n\nAfter analyzing your input, here are my thoughts:\n\n## Understanding\n"${prompt.slice(0, 40)}..."\n\n## Analysis\nThis appears to be a request that would benefit from a structured approach:\n\n1. **Phase 1**: Initial setup and configuration\n2. **Phase 2**: Core implementation\n3. **Phase 3**: Testing and refinement\n\n## Recommendation\nI suggest starting with a proof-of-concept to validate the approach before full implementation.\n\n*Feel free to ask follow-up questions!*`,
    "llama3.3:70b": `Processing your request...\n\nInput received: "${prompt.slice(0, 50)}..."\n\nHere's what I think:\n\nThe task you've described can be approached in several ways. Based on my understanding, the most effective method would be:\n\n- Start with clear requirements\n- Build incrementally\n- Test frequently\n- Iterate based on feedback\n\nWould you like me to elaborate on any specific aspect of this approach?`,
    "deepseek-chat": `Let me analyze this step by step.\n\n**Query**: ${prompt.slice(0, 40)}...\n\n**Analysis**:\n\n1. Understanding the core requirement\n2. Identifying potential solutions\n3. Evaluating trade-offs\n\n**Recommendation**:\n\nBased on the analysis, I recommend a pragmatic approach that balances complexity with maintainability. Here's a suggested implementation path:\n\n\`\`\`\n// Pseudocode outline\ninitialize()\nprocess(input)\nvalidate(output)\ndeliver(result)\n\`\`\`\n\nThis keeps things simple while achieving the goal.`,
  }

  const defaultResponse = `I've processed your request: "${prompt.slice(0, 50)}..."\n\nHere's my response based on the ${model.displayName} model capabilities:\n\n1. Analysis complete\n2. Recommendations generated\n3. Ready for implementation\n\nLet me know if you need further assistance!`

  return {
    modelId: model.id,
    modelName: model.displayName,
    response: responses[model.id] || defaultResponse,
    tokensUsed: {
      input: inputTokens,
      output: outputTokens,
      total: inputTokens + outputTokens,
    },
    latency: delay,
    cost: (inputTokens * model.inputCost + outputTokens * model.outputCost) / 1000000,
    finishReason: "stop",
    timestamp: new Date(),
  }
}

export default function LLMWorkbench() {
  const searchParams = useSearchParams()
  const initialPrompt = searchParams.get("prompt") || ""

  const [prompt, setPrompt] = useState(initialPrompt)
  const [systemPrompt, setSystemPrompt] = useState("")
  const [selectedModels, setSelectedModels] = useState<string[]>(["gpt-4.1-mini", "claude-haiku-3.5"])
  const [params, setParams] = useState<GenerationParameters>(DEFAULT_PARAMS)
  const [responses, setResponses] = useState<ComparisonResponse[]>([])
  const [isGenerating, setIsGenerating] = useState(false)
  const [showParams, setShowParams] = useState(false)
  const [history, setHistory] = useState<{ prompt: string; timestamp: Date }[]>([])

  const _allModels = getAllModels()

  const toggleModel = (modelId: string) => {
    setSelectedModels((prev) => (prev.includes(modelId) ? prev.filter((id) => id !== modelId) : [...prev, modelId]))
  }

  const runComparison = useCallback(async () => {
    if (!prompt.trim() || selectedModels.length === 0) return

    setIsGenerating(true)
    setResponses([])

    // Add to history
    setHistory((prev) => [{ prompt, timestamp: new Date() }, ...prev.slice(0, 9)])

    // Generate responses in parallel with staggered delays
    const promises = selectedModels.map((modelId, index) => {
      const model = getModelById(modelId)
      if (!model) return null

      const baseDelay = model.speed === "fast" ? 800 : model.speed === "medium" ? 1500 : 2500
      const delay = baseDelay + index * 200 + Math.random() * 500

      return simulateResponse(model, prompt, delay).then((response) => {
        setResponses((prev) => [...prev, response])
        return response
      })
    })

    await Promise.all(promises.filter(Boolean))
    setIsGenerating(false)
  }, [prompt, selectedModels])

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  const clearAll = () => {
    setPrompt("")
    setSystemPrompt("")
    setResponses([])
  }

  return (
    <div className="min-h-screen bg-slate-950">
      {/* Header */}
      <header className="border-b border-border bg-secondary/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-4">
              <Link href="/">
                <Button variant="ghost" size="sm" className="text-muted-foreground">
                  <ArrowLeft className="w-4 h-4 mr-2" />
                  Back
                </Button>
              </Link>
              <div className="flex items-center gap-3">
                <div className="p-2 bg-gradient-to-br from-primary to-primary/80 rounded-lg">
                  <Bot className="w-5 h-5 text-primary-foreground" />
                </div>
                <div>
                  <h1 className="text-foreground font-bold">LLM Workbench</h1>
                  <p className="text-xs text-muted-foreground">Model Arena & Comparison</p>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Badge variant="outline" className="border-primary/30 text-primary">
                <Bot className="w-3 h-3 mr-1" />
                ORACLE Agent
              </Badge>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        <div className="grid grid-cols-12 gap-6">
          {/* Model Selection Sidebar */}
          <div className="col-span-3">
            <Card className="bg-secondary/30 border-border sticky top-24">
              <CardHeader className="pb-3">
                <CardTitle className="text-foreground text-lg flex items-center gap-2">
                  <Server className="w-5 h-5 text-muted-foreground" />
                  Select Models
                </CardTitle>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-[calc(100vh-280px)]">
                  <div className="space-y-4 pr-4">
                    {LLM_PROVIDERS.map((provider) => (
                      <div key={provider.id}>
                        <div className="flex items-center gap-2 mb-2">
                          {provider.baseUrl ? (
                            <Server className="w-4 h-4 text-primary" />
                          ) : (
                            <Cloud className="w-4 h-4 text-secondary" />
                          )}
                          <span className="text-sm font-medium text-foreground">{provider.name}</span>
                          {provider.status === "connected" && <div className="w-2 h-2 bg-primary rounded-full" />}
                        </div>
                        <div className="space-y-1 ml-6">
                          {provider.models.map((model) => (
                            <div
                              key={model.id}
                              className={cn(
                                "flex items-center gap-2 p-2 rounded-lg cursor-pointer transition-colors",
                                selectedModels.includes(model.id)
                                  ? "bg-primary/20 border border-primary/30"
                                  : "hover:bg-accent/30",
                              )}
                              onClick={() => toggleModel(model.id)}
                            >
                              <Checkbox
                                checked={selectedModels.includes(model.id)}
                                onCheckedChange={() => toggleModel(model.id)}
                              />
                              <div className="flex-1 min-w-0">
                                <div className="text-sm text-foreground truncate">{model.displayName}</div>
                                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                  <span
                                    className={cn(
                                      model.speed === "fast" && "text-primary",
                                      model.speed === "medium" && "text-accent",
                                      model.speed === "slow" && "text-destructive",
                                    )}
                                  >
                                    {model.speed}
                                  </span>
                                  <span>•</span>
                                  <span>{model.inputCost === 0 ? "Free" : `$${model.inputCost}/M`}</span>
                                </div>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    ))}
                  </div>
                </ScrollArea>

                <div className="mt-4 pt-4 border-t border-border">
                  <div className="text-xs text-muted-foreground mb-2">
                    {selectedModels.length} model{selectedModels.length !== 1 ? "s" : ""} selected
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full border-border bg-transparent"
                    onClick={() => setSelectedModels([])}
                  >
                    Clear Selection
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Main Content */}
          <div className="col-span-9 space-y-6">
            {/* Input Area */}
            <Card className="bg-secondary/30 border-border">
              <CardContent className="p-4 space-y-4">
                {/* System Prompt */}
                <Collapsible open={showParams} onOpenChange={setShowParams}>
                  <CollapsibleTrigger asChild>
                    <Button variant="ghost" size="sm" className="text-muted-foreground hover:text-foreground">
                      <Settings2 className="w-4 h-4 mr-2" />
                      Advanced Settings
                      <ChevronDown className={cn("w-4 h-4 ml-2 transition-transform", showParams && "rotate-180")} />
                    </Button>
                  </CollapsibleTrigger>
                  <CollapsibleContent className="mt-4 space-y-4">
                    <div>
                      <Label className="text-muted-foreground">System Prompt</Label>
                      <Textarea
                        value={systemPrompt}
                        onChange={(e) => setSystemPrompt(e.target.value)}
                        placeholder="Optional system prompt to set context..."
                        className="mt-1 bg-accent border-border min-h-[80px]"
                      />
                    </div>
                    <div className="grid grid-cols-3 gap-4">
                      <div>
                        <Label className="text-muted-foreground">Temperature: {params.temperature}</Label>
                        <Slider
                          value={[params.temperature]}
                          onValueChange={([v]) => setParams((p) => ({ ...p, temperature: v }))}
                          min={0}
                          max={2}
                          step={0.1}
                          className="mt-2"
                        />
                      </div>
                      <div>
                        <Label className="text-muted-foreground">Max Tokens</Label>
                        <Input
                          type="number"
                          value={params.maxTokens}
                          onChange={(e) =>
                            setParams((p) => ({ ...p, maxTokens: Number.parseInt(e.target.value) || 2048 }))
                          }
                          className="mt-1 bg-accent border-border"
                        />
                      </div>
                      <div>
                        <Label className="text-muted-foreground">Top P: {params.topP}</Label>
                        <Slider
                          value={[params.topP]}
                          onValueChange={([v]) => setParams((p) => ({ ...p, topP: v }))}
                          min={0}
                          max={1}
                          step={0.1}
                          className="mt-2"
                        />
                      </div>
                    </div>
                  </CollapsibleContent>
                </Collapsible>

                {/* Main Prompt */}
                <div>
                  <Label className="text-muted-foreground">Prompt</Label>
                  <Textarea
                    value={prompt}
                    onChange={(e) => setPrompt(e.target.value)}
                    placeholder="Enter your prompt to compare across models..."
                    className="mt-1 bg-accent border-border min-h-[120px]"
                  />
                </div>

                {/* Actions */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm" className="border-border bg-transparent" onClick={clearAll}>
                      <Trash2 className="w-4 h-4 mr-2" />
                      Clear
                    </Button>
                    <Link href="/prompt-foundry">
                      <Button variant="outline" size="sm" className="border-border bg-transparent">
                        <Sparkles className="w-4 h-4 mr-2" />
                        Use Template
                      </Button>
                    </Link>
                  </div>
                  <Button
                    onClick={runComparison}
                    disabled={isGenerating || !prompt.trim() || selectedModels.length === 0}
                    className="bg-gradient-to-r from-primary to-primary/80 hover:from-primary/90 hover:to-primary/70"
                  >
                    {isGenerating ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        Generating...
                      </>
                    ) : (
                      <>
                        <Play className="w-4 h-4 mr-2" />
                        Run Comparison
                      </>
                    )}
                  </Button>
                </div>
              </CardContent>
            </Card>

            {/* Responses */}
            {(responses.length > 0 || isGenerating) && (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h2 className="text-lg font-semibold text-foreground">Responses</h2>
                  {responses.length > 0 && (
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                      <span>
                        {responses.length} of {selectedModels.length} complete
                      </span>
                      <span>Total cost: ${responses.reduce((sum, r) => sum + r.cost, 0).toFixed(4)}</span>
                    </div>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-4">
                  {selectedModels.map((modelId) => {
                    const model = getModelById(modelId)
                    const response = responses.find((r) => r.modelId === modelId)

                    if (!model) return null

                    return (
                      <Card
                        key={modelId}
                        className={cn("bg-secondary/30 border-border overflow-hidden", response && "border-border")}
                      >
                        {/* Model Header */}
                        <div className="flex items-center justify-between p-3 border-b border-border bg-accent/30">
                          <div className="flex items-center gap-2">
                            <Bot className="w-4 h-4 text-primary" />
                            <span className="text-foreground font-medium text-sm">{model.displayName}</span>
                            {response ? (
                              <CheckCircle2 className="w-4 h-4 text-primary" />
                            ) : isGenerating ? (
                              <Loader2 className="w-4 h-4 text-accent animate-spin" />
                            ) : null}
                          </div>
                          {response && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => copyToClipboard(response.response)}
                              className="h-7 text-muted-foreground hover:text-foreground"
                            >
                              <Copy className="w-3 h-3" />
                            </Button>
                          )}
                        </div>

                        {/* Response Content */}
                        <CardContent className="p-4">
                          {response ? (
                            <>
                              <ScrollArea className="h-64 mb-4">
                                <div className="pr-4 text-sm text-muted-foreground leading-relaxed whitespace-pre-wrap">
                                  {response.response}
                                </div>
                              </ScrollArea>
                              <div className="grid grid-cols-2 gap-2 text-xs">
                                <div className="p-2 bg-accent/20 rounded">
                                  <div className="text-muted-foreground">Latency</div>
                                  <div className="text-foreground font-medium">{response.latency}ms</div>
                                </div>
                                <div className="p-2 bg-accent/20 rounded">
                                  <div className="text-muted-foreground">Tokens</div>
                                  <div className="text-foreground font-medium">{response.tokensUsed.total}</div>
                                </div>
                              </div>
                            </>
                          ) : isGenerating ? (
                            <div className="h-64 flex items-center justify-center">
                              <Loader2 className="w-6 h-6 text-accent animate-spin" />
                            </div>
                          ) : (
                            <div className="h-64 flex items-center justify-center">
                              <p className="text-muted-foreground text-sm">Waiting to start...</p>
                            </div>
                          )}
                        </CardContent>
                      </Card>
                    )
                  })}
                </div>
              </div>
            )}

            {/* History */}
            {history.length > 0 && (
              <Card className="bg-secondary/30 border-border">
                <CardHeader>
                  <CardTitle className="text-foreground text-lg flex items-center gap-2">
                    <History className="w-5 h-5 text-muted-foreground" />
                    Prompt History
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    {history.map((item, idx) => (
                      <div
                        key={idx}
                        className="p-2 bg-accent/20 rounded cursor-pointer hover:bg-accent/30 transition-colors"
                        onClick={() => setPrompt(item.prompt)}
                      >
                        <div className="text-sm text-foreground truncate">{item.prompt}</div>
                        <div className="text-xs text-muted-foreground">
                          {new Date(item.timestamp).toLocaleTimeString()}
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            )}
          </div>
        </div>
      </main>
    </div>
  )
}
