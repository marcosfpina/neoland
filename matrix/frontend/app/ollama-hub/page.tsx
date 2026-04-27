"use client"

import { useState, useEffect, useCallback, useRef } from "react"
import Link from "next/link"
import {
  Bot,
  ChevronLeft,
  Download,
  Trash2,
  Play,
  Square,
  RefreshCw,
  HardDrive,
  Cpu,
  Zap,
  MessageSquare,
  Send,
  Loader2,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Copy,
  Settings,
  MemoryStick,
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Slider } from "@/components/ui/slider"
import { Label } from "@/components/ui/label"

interface OllamaModel {
  name: string
  size: number
  modified_at: string
  digest?: string
}

interface LoadedModel {
  name: string
  size: number
  vram_size: number
}

interface Message {
  role: "user" | "assistant" | "system"
  content: string
}

interface PullProgress {
  status: string
  digest?: string
  total?: number
  completed?: number
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B"
  const k = 1024
  const sizes = ["B", "KB", "MB", "GB", "TB"]
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i]
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  })
}

export default function OllamaHubPage() {
  const [ollamaStatus, setOllamaStatus] = useState<{
    running: boolean
    models: OllamaModel[]
    loaded_models: LoadedModel[]
    host: string
  } | null>(null)
  const [loading, setLoading] = useState(true)
  const [selectedModel, setSelectedModel] = useState<string>("")
  const [pullModel, setPullModel] = useState("")
  const [pulling, setPulling] = useState(false)
  const [pullProgress, setPullProgress] = useState<PullProgress | null>(null)

  // Chat state
  const [messages, setMessages] = useState<Message[]>([])
  const [inputMessage, setInputMessage] = useState("")
  const [generating, setGenerating] = useState(false)
  const [systemPrompt, setSystemPrompt] = useState(
    "You are a helpful AI assistant running locally on NixOS via Ollama.",
  )

  // Generation options
  const [temperature, setTemperature] = useState(0.7)
  const [maxTokens, setMaxTokens] = useState(2048)
  const [topP, setTopP] = useState(0.9)

  const chatEndRef = useRef<HTMLDivElement>(null)
  const abortControllerRef = useRef<AbortController | null>(null)

  const fetchStatus = useCallback(async () => {
    try {
      const response = await fetch("/api/system/ollama?action=status")
      const data = await response.json()
      setOllamaStatus(data)

      if (data.running && data.models?.length > 0 && !selectedModel) {
        setSelectedModel(data.models[0].name)
      }
    } catch (error) {
      console.error("Failed to fetch Ollama status:", error)
      setOllamaStatus({ running: false, models: [], loaded_models: [], host: "" })
    } finally {
      setLoading(false)
    }
  }, [selectedModel])

  useEffect(() => {
    fetchStatus()
    const interval = setInterval(fetchStatus, 10000) // Refresh every 10 seconds
    return () => clearInterval(interval)
  }, [fetchStatus])

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" })
  }, [messages])

  const handlePullModel = async () => {
    if (!pullModel.trim()) return

    setPulling(true)
    setPullProgress({ status: "starting" })

    try {
      const response = await fetch("/api/system/ollama", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action: "pull", model: pullModel.trim() }),
      })

      const reader = response.body?.getReader()
      const decoder = new TextDecoder()

      if (reader) {
        while (true) {
          const { done, value } = await reader.read()
          if (done) break

          const chunk = decoder.decode(value)
          const lines = chunk.split("\n").filter(Boolean)

          for (const line of lines) {
            try {
              const progress = JSON.parse(line)
              setPullProgress(progress)
            } catch {
              // Ignore parse errors
            }
          }
        }
      }

      setPullModel("")
      fetchStatus()
    } catch (error) {
      console.error("Failed to pull model:", error)
    } finally {
      setPulling(false)
      setPullProgress(null)
    }
  }

  const handleDeleteModel = async (model: string) => {
    if (!confirm(`Are you sure you want to delete ${model}?`)) return

    try {
      await fetch("/api/system/ollama", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action: "delete", model }),
      })
      fetchStatus()
    } catch (error) {
      console.error("Failed to delete model:", error)
    }
  }

  const handleLoadModel = async (model: string) => {
    try {
      await fetch("/api/system/ollama", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action: "load", model }),
      })
      fetchStatus()
    } catch (error) {
      console.error("Failed to load model:", error)
    }
  }

  const handleUnloadModel = async (model: string) => {
    try {
      await fetch("/api/system/ollama", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action: "unload", model }),
      })
      fetchStatus()
    } catch (error) {
      console.error("Failed to unload model:", error)
    }
  }

  const handleSendMessage = async () => {
    if (!inputMessage.trim() || !selectedModel || generating) return

    const userMessage: Message = { role: "user", content: inputMessage.trim() }
    setMessages((prev) => [...prev, userMessage])
    setInputMessage("")
    setGenerating(true)

    const assistantMessage: Message = { role: "assistant", content: "" }
    setMessages((prev) => [...prev, assistantMessage])

    abortControllerRef.current = new AbortController()

    try {
      const chatMessages = [{ role: "system", content: systemPrompt }, ...messages, userMessage]

      const response = await fetch("/api/system/ollama", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          action: "chat",
          model: selectedModel,
          messages: chatMessages,
          options: {
            temperature,
            num_predict: maxTokens,
            top_p: topP,
          },
        }),
        signal: abortControllerRef.current.signal,
      })

      const reader = response.body?.getReader()
      const decoder = new TextDecoder()
      let fullContent = ""

      if (reader) {
        while (true) {
          const { done, value } = await reader.read()
          if (done) break

          const chunk = decoder.decode(value)
          const lines = chunk.split("\n").filter(Boolean)

          for (const line of lines) {
            try {
              const data = JSON.parse(line)
              if (data.message?.content) {
                fullContent += data.message.content
                setMessages((prev) => {
                  const updated = [...prev]
                  updated[updated.length - 1] = { role: "assistant", content: fullContent }
                  return updated
                })
              }
            } catch {
              // Ignore parse errors
            }
          }
        }
      }
    } catch (error) {
      if ((error as Error).name !== "AbortError") {
        console.error("Failed to generate:", error)
        setMessages((prev) => {
          const updated = [...prev]
          updated[updated.length - 1] = {
            role: "assistant",
            content: "Error: Failed to generate response. Is Ollama running?",
          }
          return updated
        })
      }
    } finally {
      setGenerating(false)
      abortControllerRef.current = null
    }
  }

  const handleStopGeneration = () => {
    abortControllerRef.current?.abort()
    setGenerating(false)
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <Bot className="w-12 h-12 text-primary animate-pulse mx-auto mb-4" />
          <p className="text-muted-foreground">Connecting to Ollama...</p>
        </div>
      </div>
    )
  }

  const totalVram = ollamaStatus?.loaded_models?.reduce((acc, m) => acc + m.vram_size, 0) || 0
  const isModelLoaded = (model: string) => ollamaStatus?.loaded_models?.some((m) => m.name === model) || false

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Link href="/">
                <Button variant="ghost" size="sm">
                  <ChevronLeft className="w-4 h-4 mr-1" />
                  Back
                </Button>
              </Link>
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-primary/10">
                  <Bot className="w-6 h-6 text-primary" />
                </div>
                <div>
                  <h1 className="text-xl font-bold text-foreground">Ollama Hub</h1>
                  <p className="text-sm text-muted-foreground">Local LLM Management & Inference</p>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <Badge variant={ollamaStatus?.running ? "default" : "destructive"} className="gap-1">
                {ollamaStatus?.running ? (
                  <>
                    <CheckCircle className="w-3 h-3" /> Connected
                  </>
                ) : (
                  <>
                    <XCircle className="w-3 h-3" /> Offline
                  </>
                )}
              </Badge>
              <Button variant="outline" size="sm" onClick={fetchStatus}>
                <RefreshCw className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        {!ollamaStatus?.running ? (
          <Card className="bg-card border-border">
            <CardContent className="p-8 text-center">
              <AlertTriangle className="w-16 h-16 text-chart-4 mx-auto mb-4" />
              <h2 className="text-xl font-bold text-foreground mb-2">Ollama Not Running</h2>
              <p className="text-muted-foreground mb-4">Start the Ollama service to manage models and run inference.</p>
              <code className="block bg-secondary p-3 rounded text-sm text-foreground mb-4">
                sudo systemctl start ollama
              </code>
              <Button onClick={fetchStatus}>
                <RefreshCw className="w-4 h-4 mr-2" />
                Check Again
              </Button>
            </CardContent>
          </Card>
        ) : (
          <>
            {/* Status Cards */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
              <Card className="bg-card border-border">
                <CardContent className="p-4">
                  <div className="flex items-center gap-3">
                    <HardDrive className="w-8 h-8 text-primary" />
                    <div>
                      <p className="text-2xl font-bold text-foreground">{ollamaStatus.models?.length || 0}</p>
                      <p className="text-sm text-muted-foreground">Models Available</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-card border-border">
                <CardContent className="p-4">
                  <div className="flex items-center gap-3">
                    <Cpu className="w-8 h-8 text-chart-2" />
                    <div>
                      <p className="text-2xl font-bold text-foreground">{ollamaStatus.loaded_models?.length || 0}</p>
                      <p className="text-sm text-muted-foreground">Models Loaded</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-card border-border">
                <CardContent className="p-4">
                  <div className="flex items-center gap-3">
                    <MemoryStick className="w-8 h-8 text-chart-3" />
                    <div>
                      <p className="text-2xl font-bold text-foreground">{formatBytes(totalVram)}</p>
                      <p className="text-sm text-muted-foreground">VRAM Used</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-card border-border">
                <CardContent className="p-4">
                  <div className="flex items-center gap-3">
                    <Zap className="w-8 h-8 text-chart-4" />
                    <div>
                      <p className="text-sm font-medium text-foreground truncate">
                        {ollamaStatus.host || "localhost:11434"}
                      </p>
                      <p className="text-sm text-muted-foreground">Ollama Host</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            <Tabs defaultValue="chat" className="space-y-4">
              <TabsList className="bg-secondary">
                <TabsTrigger value="chat" className="data-[state=active]:bg-background">
                  <MessageSquare className="w-4 h-4 mr-2" />
                  Chat
                </TabsTrigger>
                <TabsTrigger value="models" className="data-[state=active]:bg-background">
                  <HardDrive className="w-4 h-4 mr-2" />
                  Models
                </TabsTrigger>
                <TabsTrigger value="settings" className="data-[state=active]:bg-background">
                  <Settings className="w-4 h-4 mr-2" />
                  Settings
                </TabsTrigger>
              </TabsList>

              {/* Chat Tab */}
              <TabsContent value="chat">
                <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
                  {/* Chat Area */}
                  <Card className="lg:col-span-3 bg-card border-border">
                    <CardHeader className="pb-2 border-b border-border">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <Select value={selectedModel} onValueChange={setSelectedModel}>
                            <SelectTrigger className="w-64 bg-secondary border-border">
                              <SelectValue placeholder="Select a model" />
                            </SelectTrigger>
                            <SelectContent>
                              {ollamaStatus.models?.map((model) => (
                                <SelectItem key={model.name} value={model.name}>
                                  <div className="flex items-center gap-2">
                                    {isModelLoaded(model.name) && <div className="w-2 h-2 rounded-full bg-chart-2" />}
                                    {model.name}
                                  </div>
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                          {selectedModel && !isModelLoaded(selectedModel) && (
                            <Badge variant="outline" className="text-chart-4">
                              <AlertTriangle className="w-3 h-3 mr-1" />
                              Not Loaded
                            </Badge>
                          )}
                        </div>
                        <Button variant="outline" size="sm" onClick={() => setMessages([])}>
                          Clear Chat
                        </Button>
                      </div>
                    </CardHeader>
                    <CardContent className="p-0">
                      <ScrollArea className="h-96 p-4">
                        {messages.length === 0 ? (
                          <div className="h-full flex items-center justify-center text-muted-foreground">
                            <div className="text-center">
                              <Bot className="w-12 h-12 mx-auto mb-3 opacity-50" />
                              <p>Start a conversation with {selectedModel || "a model"}</p>
                            </div>
                          </div>
                        ) : (
                          <div className="space-y-4">
                            {messages.map((msg, i) => (
                              <div key={i} className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}>
                                <div
                                  className={`max-w-[80%] p-3 rounded-lg ${
                                    msg.role === "user"
                                      ? "bg-primary text-primary-foreground"
                                      : "bg-secondary text-secondary-foreground"
                                  }`}
                                >
                                  <div className="flex items-start justify-between gap-2">
                                    <p className="text-sm whitespace-pre-wrap">{msg.content}</p>
                                    {msg.role === "assistant" && msg.content && (
                                      <Button
                                        variant="ghost"
                                        size="sm"
                                        className="shrink-0 h-6 w-6 p-0"
                                        onClick={() => copyToClipboard(msg.content)}
                                      >
                                        <Copy className="w-3 h-3" />
                                      </Button>
                                    )}
                                  </div>
                                </div>
                              </div>
                            ))}
                            {generating && (
                              <div className="flex items-center gap-2 text-muted-foreground">
                                <Loader2 className="w-4 h-4 animate-spin" />
                                <span className="text-sm">Generating...</span>
                              </div>
                            )}
                            <div ref={chatEndRef} />
                          </div>
                        )}
                      </ScrollArea>

                      {/* Input Area */}
                      <div className="p-4 border-t border-border">
                        <div className="flex gap-2">
                          <Textarea
                            value={inputMessage}
                            onChange={(e) => setInputMessage(e.target.value)}
                            placeholder="Type your message..."
                            className="min-h-[44px] max-h-32 bg-secondary border-border"
                            onKeyDown={(e) => {
                              if (e.key === "Enter" && !e.shiftKey) {
                                e.preventDefault()
                                handleSendMessage()
                              }
                            }}
                          />
                          {generating ? (
                            <Button variant="destructive" onClick={handleStopGeneration} className="shrink-0">
                              <Square className="w-4 h-4" />
                            </Button>
                          ) : (
                            <Button
                              onClick={handleSendMessage}
                              disabled={!inputMessage.trim() || !selectedModel}
                              className="shrink-0"
                            >
                              <Send className="w-4 h-4" />
                            </Button>
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  {/* Generation Settings */}
                  <Card className="bg-card border-border">
                    <CardHeader>
                      <CardTitle className="text-foreground text-base">Generation Settings</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-6">
                      <div className="space-y-2">
                        <Label className="text-sm">System Prompt</Label>
                        <Textarea
                          value={systemPrompt}
                          onChange={(e) => setSystemPrompt(e.target.value)}
                          className="min-h-20 bg-secondary border-border text-sm"
                        />
                      </div>

                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label className="text-sm">Temperature</Label>
                          <span className="text-sm text-muted-foreground">{temperature}</span>
                        </div>
                        <Slider
                          value={[temperature]}
                          onValueChange={([v]) => setTemperature(v)}
                          min={0}
                          max={2}
                          step={0.1}
                        />
                      </div>

                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label className="text-sm">Max Tokens</Label>
                          <span className="text-sm text-muted-foreground">{maxTokens}</span>
                        </div>
                        <Slider
                          value={[maxTokens]}
                          onValueChange={([v]) => setMaxTokens(v)}
                          min={256}
                          max={8192}
                          step={256}
                        />
                      </div>

                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label className="text-sm">Top P</Label>
                          <span className="text-sm text-muted-foreground">{topP}</span>
                        </div>
                        <Slider value={[topP]} onValueChange={([v]) => setTopP(v)} min={0} max={1} step={0.05} />
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </TabsContent>

              {/* Models Tab */}
              <TabsContent value="models">
                <div className="grid gap-6">
                  {/* Pull New Model */}
                  <Card className="bg-card border-border">
                    <CardHeader>
                      <CardTitle className="text-foreground">Pull New Model</CardTitle>
                      <CardDescription>Download a model from the Ollama library</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="flex gap-2">
                        <Input
                          value={pullModel}
                          onChange={(e) => setPullModel(e.target.value)}
                          placeholder="e.g., llama3.2, codellama:13b, mistral"
                          className="bg-secondary border-border"
                          disabled={pulling}
                        />
                        <Button onClick={handlePullModel} disabled={pulling || !pullModel.trim()}>
                          {pulling ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                          <span className="ml-2">Pull</span>
                        </Button>
                      </div>

                      {pullProgress && (
                        <div className="mt-4">
                          <p className="text-sm text-muted-foreground mb-2">{pullProgress.status}</p>
                          {pullProgress.total && (
                            <Progress
                              value={((pullProgress.completed || 0) / pullProgress.total) * 100}
                              className="h-2"
                            />
                          )}
                        </div>
                      )}
                    </CardContent>
                  </Card>

                  {/* Loaded Models */}
                  {ollamaStatus.loaded_models?.length > 0 && (
                    <Card className="bg-card border-border">
                      <CardHeader>
                        <CardTitle className="text-foreground flex items-center gap-2">
                          <Cpu className="w-5 h-5 text-chart-2" />
                          Loaded in Memory
                        </CardTitle>
                        <CardDescription>Models currently loaded in GPU/RAM</CardDescription>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-3">
                          {ollamaStatus.loaded_models.map((model) => (
                            <div
                              key={model.name}
                              className="flex items-center justify-between p-3 rounded-lg bg-chart-2/10 border border-chart-2/20"
                            >
                              <div className="flex items-center gap-3">
                                <div className="w-2 h-2 rounded-full bg-chart-2 animate-pulse" />
                                <div>
                                  <p className="font-medium text-foreground">{model.name}</p>
                                  <p className="text-xs text-muted-foreground">VRAM: {formatBytes(model.vram_size)}</p>
                                </div>
                              </div>
                              <Button variant="outline" size="sm" onClick={() => handleUnloadModel(model.name)}>
                                <Square className="w-4 h-4 mr-1" />
                                Unload
                              </Button>
                            </div>
                          ))}
                        </div>
                      </CardContent>
                    </Card>
                  )}

                  {/* All Models */}
                  <Card className="bg-card border-border">
                    <CardHeader>
                      <CardTitle className="text-foreground">Available Models</CardTitle>
                      <CardDescription>All models stored locally</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3">
                        {ollamaStatus.models?.map((model) => {
                          const loaded = isModelLoaded(model.name)
                          return (
                            <div
                              key={model.name}
                              className={`flex items-center justify-between p-3 rounded-lg border ${
                                loaded ? "bg-chart-2/5 border-chart-2/20" : "bg-secondary/50 border-border"
                              }`}
                            >
                              <div className="flex items-center gap-3">
                                {loaded && <div className="w-2 h-2 rounded-full bg-chart-2" />}
                                <div>
                                  <p className="font-medium text-foreground">{model.name}</p>
                                  <p className="text-xs text-muted-foreground">
                                    Size: {formatBytes(model.size)} | Modified: {formatDate(model.modified_at)}
                                  </p>
                                </div>
                              </div>
                              <div className="flex gap-2">
                                {!loaded ? (
                                  <Button variant="outline" size="sm" onClick={() => handleLoadModel(model.name)}>
                                    <Play className="w-4 h-4 mr-1" />
                                    Load
                                  </Button>
                                ) : (
                                  <Button variant="outline" size="sm" onClick={() => handleUnloadModel(model.name)}>
                                    <Square className="w-4 h-4 mr-1" />
                                    Unload
                                  </Button>
                                )}
                                <Button variant="destructive" size="sm" onClick={() => handleDeleteModel(model.name)}>
                                  <Trash2 className="w-4 h-4" />
                                </Button>
                              </div>
                            </div>
                          )
                        })}

                        {(!ollamaStatus.models || ollamaStatus.models.length === 0) && (
                          <div className="text-center py-8 text-muted-foreground">
                            <HardDrive className="w-12 h-12 mx-auto mb-3 opacity-50" />
                            <p>No models installed yet</p>
                            <p className="text-sm">Pull a model to get started</p>
                          </div>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </TabsContent>

              {/* Settings Tab */}
              <TabsContent value="settings">
                <Card className="bg-card border-border">
                  <CardHeader>
                    <CardTitle className="text-foreground">Ollama Configuration</CardTitle>
                    <CardDescription>Service configuration and environment</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-4 rounded-lg bg-secondary/50 border border-border">
                        <p className="text-sm text-muted-foreground">Host</p>
                        <p className="font-mono text-foreground">{ollamaStatus.host}</p>
                      </div>
                      <div className="p-4 rounded-lg bg-secondary/50 border border-border">
                        <p className="text-sm text-muted-foreground">Status</p>
                        <p className="font-medium text-chart-2">Running</p>
                      </div>
                    </div>

                    <div className="p-4 rounded-lg bg-secondary/50 border border-border">
                      <p className="text-sm text-muted-foreground mb-2">Systemd Service Control</p>
                      <div className="flex gap-2">
                        <Button variant="outline" size="sm">
                          <RefreshCw className="w-4 h-4 mr-1" />
                          Restart
                        </Button>
                        <Button variant="outline" size="sm">
                          <Square className="w-4 h-4 mr-1" />
                          Stop
                        </Button>
                      </div>
                    </div>

                    <div className="p-4 rounded-lg bg-secondary/50 border border-border">
                      <p className="text-sm text-muted-foreground mb-2">NixOS Configuration</p>
                      <pre className="text-xs text-foreground bg-background p-3 rounded overflow-x-auto">
                        {`services.ollama = {
  enable = true;
  host = "0.0.0.0:11434";
  environment = {
    CUDA_VISIBLE_DEVICES = "0";
    OLLAMA_MAX_LOADED_MODELS = "2";
  };
};`}
                      </pre>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </>
        )}
      </main>
    </div>
  )
}
