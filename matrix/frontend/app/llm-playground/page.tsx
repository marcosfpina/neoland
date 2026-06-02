"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Slider } from "@/components/ui/slider"
import {
  Bot,
  Send,
  ArrowLeft,
  Zap,
  BarChart3,
  Clock,
  DollarSign,
  Brain,
  Copy,
  Download,
  Settings,
  Pause,
  RotateCcw,
} from "lucide-react"
import Link from "next/link"

export default function LLMPlayground() {
  const [prompt, setPrompt] = useState("")
  const [selectedProviders, setSelectedProviders] = useState(["openai", "anthropic", "google"])
  const [temperature, setTemperature] = useState([0.7])
  const [maxTokens, setMaxTokens] = useState([1000])
  const [responses, setResponses] = useState({})
  const [loading, setLoading] = useState(false)
  const [comparison, setComparison] = useState(null)

  const providers = [
    {
      id: "openai",
      name: "OpenAI",
      models: ["gpt-4o", "gpt-4o-mini", "gpt-3.5-turbo"],
      defaultModel: "gpt-4o",
      color: "bg-green-500",
      pricing: "$0.03/1K tokens",
      speed: "Fast",
      strengths: ["Reasoning", "Code", "General"],
    },
    {
      id: "anthropic",
      name: "Anthropic",
      models: ["claude-3-5-sonnet-20241022", "claude-3-haiku-20240307"],
      defaultModel: "claude-3-5-sonnet-20241022",
      color: "bg-orange-500",
      pricing: "$0.015/1K tokens",
      speed: "Medium",
      strengths: ["Analysis", "Writing", "Safety"],
    },
    {
      id: "google",
      name: "Google",
      models: ["gemini-1.5-pro", "gemini-1.5-flash"],
      defaultModel: "gemini-1.5-pro",
      color: "bg-blue-500",
      pricing: "$0.0075/1K tokens",
      speed: "Fast",
      strengths: ["Multimodal", "Code", "Math"],
    },
    {
      id: "xai",
      name: "xAI",
      models: ["grok-3", "grok-2"],
      defaultModel: "grok-3",
      color: "bg-purple-500",
      pricing: "$0.02/1K tokens",
      speed: "Very Fast",
      strengths: ["Real-time", "Humor", "Current Events"],
    },
    {
      id: "groq",
      name: "Groq",
      models: ["llama-3.1-70b-versatile", "llama-3.1-8b-instant"],
      defaultModel: "llama-3.1-70b-versatile",
      color: "bg-red-500",
      pricing: "$0.0008/1K tokens",
      speed: "Ultra Fast",
      strengths: ["Speed", "Efficiency", "Cost"],
    },
    {
      id: "cohere",
      name: "Cohere",
      models: ["command-r-plus", "command-r"],
      defaultModel: "command-r-plus",
      color: "bg-indigo-500",
      pricing: "$0.015/1K tokens",
      speed: "Medium",
      strengths: ["Enterprise", "RAG", "Embeddings"],
    },
    {
      id: "mistral",
      name: "Mistral AI",
      models: ["mistral-large-latest", "mistral-medium-latest"],
      defaultModel: "mistral-large-latest",
      color: "bg-yellow-500",
      pricing: "$0.012/1K tokens",
      speed: "Fast",
      strengths: ["European", "Multilingual", "Code"],
    },
    {
      id: "deepinfra",
      name: "DeepInfra",
      models: ["meta-llama/Meta-Llama-3.1-70B-Instruct", "microsoft/WizardLM-2-8x22B"],
      defaultModel: "meta-llama/Meta-Llama-3.1-70B-Instruct",
      color: "bg-cyan-500",
      pricing: "$0.0007/1K tokens",
      speed: "Fast",
      strengths: ["Open Source", "Cost", "Variety"],
    },
  ]

  const samplePrompts = [
    {
      category: "Code Review",
      prompt:
        "Review this React component for best practices and potential improvements:\n\n```jsx\nfunction UserProfile({ user }) {\n  const [loading, setLoading] = useState(false);\n  \n  useEffect(() => {\n    fetchUserData();\n  }, []);\n  \n  const fetchUserData = async () => {\n    setLoading(true);\n    const response = await fetch(`/api/users/${user.id}`);\n    const data = await response.json();\n    setUser(data);\n    setLoading(false);\n  };\n  \n  return (\n    <div>\n      {loading ? <div>Loading...</div> : <div>{user.name}</div>}\n    </div>\n  );\n}\n```",
    },
    {
      category: "API Design",
      prompt:
        "Design a RESTful API for a task management system. Include endpoints for users, projects, and tasks with proper HTTP methods, status codes, and request/response examples.",
    },
    {
      category: "Database Query",
      prompt:
        "Optimize this SQL query for better performance:\n\n```sql\nSELECT u.name, p.title, COUNT(t.id) as task_count\nFROM users u\nJOIN projects p ON u.id = p.user_id\nJOIN tasks t ON p.id = t.project_id\nWHERE u.created_at > '2024-01-01'\nGROUP BY u.id, p.id\nORDER BY task_count DESC;\n```",
    },
    {
      category: "Architecture",
      prompt:
        "Explain the trade-offs between microservices and monolithic architecture for a medium-sized e-commerce application. Consider scalability, complexity, and team structure.",
    },
  ]

  const handleSendPrompt = async () => {
    if (!prompt.trim() || selectedProviders.length === 0) return

    setLoading(true)
    setResponses({})

    // Simulate API calls to different providers
    const mockResponses = {}

    for (const providerId of selectedProviders) {
      const provider = providers.find((p) => p.id === providerId)

      // Simulate different response times and characteristics
      const delay = providerId === "groq" ? 500 : providerId === "xai" ? 800 : 1500

      setTimeout(() => {
        mockResponses[providerId] = {
          text: generateMockResponse(provider, prompt),
          tokens: Math.floor(Math.random() * 500) + 200,
          time: delay,
          cost: calculateCost(provider.pricing, Math.floor(Math.random() * 500) + 200),
        }

        setResponses((prev) => ({ ...prev, ...mockResponses }))
      }, delay)
    }

    // Complete loading after all responses
    setTimeout(() => {
      setLoading(false)
      generateComparison(mockResponses)
    }, 2000)
  }

  const generateMockResponse = (provider, _prompt) => {
    const responses = {
      openai:
        "Based on the code review, I've identified several areas for improvement:\n\n1. **Missing Dependencies**: The useEffect hook should include `user.id` in its dependency array\n2. **Error Handling**: Add try-catch blocks for the API call\n3. **State Management**: Consider using a custom hook for data fetching\n4. **Performance**: Implement proper loading states and error boundaries\n\nHere's an improved version with these fixes implemented...",
      anthropic:
        "I'll analyze this React component systematically:\n\n**Strengths:**\n- Clean component structure\n- Proper use of hooks\n\n**Issues to Address:**\n1. Missing dependency in useEffect\n2. No error handling for API failures\n3. Potential memory leaks if component unmounts during fetch\n\n**Recommendations:**\nImplement proper cleanup, add error boundaries, and consider using React Query for data fetching...",
      google:
        "Code Review Analysis:\n\n**Critical Issues:**\n• useEffect dependency array is incomplete\n• No error handling for network requests\n• Missing cleanup for async operations\n\n**Suggestions:**\n• Add AbortController for request cancellation\n• Implement proper error states\n• Consider memoization for performance\n\n**Refactored Code:**\n```jsx\n// Improved version with proper error handling...\n```",
      xai: "Yo, this code has some issues! 😅\n\n**The Good:** You're using hooks properly, nice!\n\n**The Not-So-Good:**\n- That useEffect is missing dependencies (React will be mad)\n- No error handling (what if the API is down?)\n- Potential memory leaks lurking\n\n**The Fix:** Add proper deps, error handling, and cleanup. Want me to show you a bulletproof version?",
      groq: "**React Component Analysis**\n\n**Issues Found:**\n1. Incomplete useEffect dependencies\n2. Missing error handling\n3. No request cancellation\n4. State management concerns\n\n**Optimized Solution:**\n- Add proper dependency array\n- Implement error boundaries\n- Use AbortController\n- Consider custom hooks\n\n**Performance Impact:** Medium priority fixes that will improve reliability and user experience.",
    }

    return (
      responses[provider.id] ||
      "This is a sample response from " +
        provider.name +
        " analyzing your prompt with detailed insights and recommendations."
    )
  }

  const calculateCost = (pricing, tokens) => {
    const rate = Number.parseFloat(pricing.match(/\$([0-9.]+)/)[1])
    return `$${((rate * tokens) / 1000).toFixed(4)}`
  }

  const generateComparison = (responses) => {
    const metrics = Object.entries(responses).map(([providerId, response]) => {
      const provider = providers.find((p) => p.id === providerId)
      return {
        provider: provider.name,
        responseLength: response.text.length,
        tokens: response.tokens,
        time: response.time,
        cost: response.cost,
        readability: Math.floor(Math.random() * 30) + 70, // Mock readability score
        accuracy: Math.floor(Math.random() * 20) + 80, // Mock accuracy score
      }
    })

    setComparison({
      fastest: metrics.reduce((prev, current) => (prev.time < current.time ? prev : current)),
      cheapest: metrics.reduce((prev, current) =>
        Number.parseFloat(prev.cost.slice(1)) < Number.parseFloat(current.cost.slice(1)) ? prev : current,
      ),
      mostDetailed: metrics.reduce((prev, current) => (prev.responseLength > current.responseLength ? prev : current)),
      bestAccuracy: metrics.reduce((prev, current) => (prev.accuracy > current.accuracy ? prev : current)),
      metrics,
    })
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-pink-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <Link href="/">
            <Button variant="outline" size="sm" className="border-slate-600">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Hub
            </Button>
          </Link>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-pink-500 rounded-lg">
              <Bot className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-white">LLM Playground</h1>
              <p className="text-gray-300">Test and compare responses from multiple LLM providers</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-4 gap-8">
          {/* Configuration Sidebar */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Settings className="w-5 h-5" />
                Configuration
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Provider Selection */}
              <div>
                <label className="text-white text-sm font-medium mb-3 block">LLM Providers</label>
                <div className="space-y-2">
                  {providers.map((provider) => (
                    <div key={provider.id} className="flex items-center gap-2">
                      <input
                        type="checkbox"
                        id={provider.id}
                        checked={selectedProviders.includes(provider.id)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setSelectedProviders([...selectedProviders, provider.id])
                          } else {
                            setSelectedProviders(selectedProviders.filter((id) => id !== provider.id))
                          }
                        }}
                        className="rounded border-slate-600"
                      />
                      <label htmlFor={provider.id} className="text-white text-sm flex items-center gap-2">
                        <div className={`w-3 h-3 ${provider.color} rounded`}></div>
                        {provider.name}
                      </label>
                    </div>
                  ))}
                </div>
              </div>

              {/* Parameters */}
              <div>
                <label className="text-white text-sm font-medium mb-2 block">Temperature: {temperature[0]}</label>
                <Slider
                  value={temperature}
                  onValueChange={setTemperature}
                  max={2}
                  min={0}
                  step={0.1}
                  className="w-full"
                />
              </div>

              <div>
                <label className="text-white text-sm font-medium mb-2 block">Max Tokens: {maxTokens[0]}</label>
                <Slider
                  value={maxTokens}
                  onValueChange={setMaxTokens}
                  max={4000}
                  min={100}
                  step={100}
                  className="w-full"
                />
              </div>

              {/* Sample Prompts */}
              <div>
                <label className="text-white text-sm font-medium mb-2 block">Sample Prompts</label>
                <div className="space-y-2">
                  {samplePrompts.map((sample, index) => (
                    <Button
                      key={index}
                      variant="outline"
                      size="sm"
                      onClick={() => setPrompt(sample.prompt)}
                      className="w-full text-left border-slate-600 justify-start"
                    >
                      {sample.category}
                    </Button>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Main Content */}
          <div className="lg:col-span-3">
            <Tabs defaultValue="playground" className="w-full">
              <TabsList className="grid w-full grid-cols-3 bg-slate-800/50">
                <TabsTrigger value="playground" className="data-[state=active]:bg-slate-700">
                  Playground
                </TabsTrigger>
                <TabsTrigger value="comparison" className="data-[state=active]:bg-slate-700">
                  Comparison
                </TabsTrigger>
                <TabsTrigger value="providers" className="data-[state=active]:bg-slate-700">
                  Providers
                </TabsTrigger>
              </TabsList>

              <TabsContent value="playground" className="mt-6">
                <div className="space-y-6">
                  {/* Prompt Input */}
                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <CardTitle className="text-white flex items-center gap-2">
                        <Brain className="w-5 h-5" />
                        Prompt Input
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <Textarea
                        value={prompt}
                        onChange={(e) => setPrompt(e.target.value)}
                        placeholder="Enter your prompt here... Try asking for code review, API design, or any development question."
                        className="min-h-32 bg-slate-700 border-slate-600 text-white"
                      />

                      <div className="flex gap-2">
                        <Button
                          onClick={handleSendPrompt}
                          disabled={loading || !prompt.trim() || selectedProviders.length === 0}
                          className="flex-1"
                        >
                          {loading ? (
                            <>
                              <Pause className="w-4 h-4 mr-2 animate-pulse" />
                              Processing...
                            </>
                          ) : (
                            <>
                              <Send className="w-4 h-4 mr-2" />
                              Send to {selectedProviders.length} Provider{selectedProviders.length !== 1 ? "s" : ""}
                            </>
                          )}
                        </Button>
                        <Button variant="outline" onClick={() => setPrompt("")} className="border-slate-600">
                          <RotateCcw className="w-4 h-4" />
                        </Button>
                      </div>
                    </CardContent>
                  </Card>

                  {/* Responses */}
                  <div className="grid gap-6">
                    {selectedProviders.map((providerId) => {
                      const provider = providers.find((p) => p.id === providerId)
                      const response = responses[providerId]

                      return (
                        <Card key={providerId} className="bg-slate-800/50 border-slate-700">
                          <CardHeader>
                            <div className="flex items-center justify-between">
                              <div className="flex items-center gap-3">
                                <div className={`w-4 h-4 ${provider.color} rounded`}></div>
                                <CardTitle className="text-white">{provider.name}</CardTitle>
                                <Badge variant="outline" className="border-slate-600">
                                  {provider.defaultModel}
                                </Badge>
                              </div>
                              {response && (
                                <div className="flex items-center gap-2">
                                  <Button variant="outline" size="sm" className="border-slate-600">
                                    <Copy className="w-4 h-4" />
                                  </Button>
                                  <Button variant="outline" size="sm" className="border-slate-600">
                                    <Download className="w-4 h-4" />
                                  </Button>
                                </div>
                              )}
                            </div>
                          </CardHeader>
                          <CardContent>
                            {loading && !response ? (
                              <div className="flex items-center gap-2 text-gray-400">
                                <div className="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                Generating response...
                              </div>
                            ) : response ? (
                              <div className="space-y-4">
                                <div className="bg-slate-700/30 rounded-lg p-4">
                                  <pre className="text-white text-sm whitespace-pre-wrap font-sans">
                                    {response.text}
                                  </pre>
                                </div>
                                <div className="flex items-center gap-4 text-sm text-gray-400">
                                  <div className="flex items-center gap-1">
                                    <Clock className="w-4 h-4" />
                                    {response.time}ms
                                  </div>
                                  <div className="flex items-center gap-1">
                                    <Zap className="w-4 h-4" />
                                    {response.tokens} tokens
                                  </div>
                                  <div className="flex items-center gap-1">
                                    <DollarSign className="w-4 h-4" />
                                    {response.cost}
                                  </div>
                                </div>
                              </div>
                            ) : (
                              <div className="text-gray-400 text-center py-8">Send a prompt to see the response</div>
                            )}
                          </CardContent>
                        </Card>
                      )
                    })}
                  </div>
                </div>
              </TabsContent>

              <TabsContent value="comparison" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <BarChart3 className="w-5 h-5" />
                      Response Comparison
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Analyze and compare responses across different metrics
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    {!comparison ? (
                      <div className="text-center py-12">
                        <BarChart3 className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                        <p className="text-gray-400">Send prompts to multiple providers to see comparison</p>
                      </div>
                    ) : (
                      <div className="space-y-6">
                        {/* Winner Cards */}
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                          <div className="p-4 bg-green-500/20 border border-green-500/30 rounded-lg">
                            <div className="text-green-400 text-sm font-medium">Fastest</div>
                            <div className="text-white font-bold">{comparison.fastest.provider}</div>
                            <div className="text-green-300 text-xs">{comparison.fastest.time}ms</div>
                          </div>
                          <div className="p-4 bg-blue-500/20 border border-blue-500/30 rounded-lg">
                            <div className="text-blue-400 text-sm font-medium">Cheapest</div>
                            <div className="text-white font-bold">{comparison.cheapest.provider}</div>
                            <div className="text-blue-300 text-xs">{comparison.cheapest.cost}</div>
                          </div>
                          <div className="p-4 bg-purple-500/20 border border-purple-500/30 rounded-lg">
                            <div className="text-purple-400 text-sm font-medium">Most Detailed</div>
                            <div className="text-white font-bold">{comparison.mostDetailed.provider}</div>
                            <div className="text-purple-300 text-xs">
                              {comparison.mostDetailed.responseLength} chars
                            </div>
                          </div>
                          <div className="p-4 bg-yellow-500/20 border border-yellow-500/30 rounded-lg">
                            <div className="text-yellow-400 text-sm font-medium">Best Accuracy</div>
                            <div className="text-white font-bold">{comparison.bestAccuracy.provider}</div>
                            <div className="text-yellow-300 text-xs">{comparison.bestAccuracy.accuracy}%</div>
                          </div>
                        </div>

                        {/* Detailed Metrics */}
                        <div className="overflow-x-auto">
                          <table className="w-full">
                            <thead>
                              <tr className="border-b border-slate-600">
                                <th className="text-left text-white font-medium p-3">Provider</th>
                                <th className="text-left text-white font-medium p-3">Response Time</th>
                                <th className="text-left text-white font-medium p-3">Tokens</th>
                                <th className="text-left text-white font-medium p-3">Cost</th>
                                <th className="text-left text-white font-medium p-3">Length</th>
                                <th className="text-left text-white font-medium p-3">Accuracy</th>
                              </tr>
                            </thead>
                            <tbody>
                              {comparison.metrics.map((metric, index) => (
                                <tr key={index} className="border-b border-slate-700">
                                  <td className="p-3 text-white">{metric.provider}</td>
                                  <td className="p-3 text-gray-300">{metric.time}ms</td>
                                  <td className="p-3 text-gray-300">{metric.tokens}</td>
                                  <td className="p-3 text-gray-300">{metric.cost}</td>
                                  <td className="p-3 text-gray-300">{metric.responseLength}</td>
                                  <td className="p-3 text-gray-300">{metric.accuracy}%</td>
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        </div>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="providers" className="mt-6">
                <div className="grid md:grid-cols-2 gap-6">
                  {providers.map((provider) => (
                    <Card key={provider.id} className="bg-slate-800/50 border-slate-700">
                      <CardHeader>
                        <div className="flex items-center gap-3">
                          <div className={`w-6 h-6 ${provider.color} rounded-lg`}></div>
                          <div>
                            <CardTitle className="text-white">{provider.name}</CardTitle>
                            <CardDescription className="text-gray-300">{provider.defaultModel}</CardDescription>
                          </div>
                        </div>
                      </CardHeader>
                      <CardContent className="space-y-4">
                        <div className="grid grid-cols-2 gap-4 text-sm">
                          <div>
                            <span className="text-gray-400">Pricing:</span>
                            <div className="text-white">{provider.pricing}</div>
                          </div>
                          <div>
                            <span className="text-gray-400">Speed:</span>
                            <div className="text-white">{provider.speed}</div>
                          </div>
                        </div>

                        <div>
                          <span className="text-gray-400 text-sm">Strengths:</span>
                          <div className="flex flex-wrap gap-1 mt-1">
                            {provider.strengths.map((strength) => (
                              <Badge key={strength} variant="outline" className="text-xs border-slate-600">
                                {strength}
                              </Badge>
                            ))}
                          </div>
                        </div>

                        <div>
                          <span className="text-gray-400 text-sm">Available Models:</span>
                          <div className="mt-1 space-y-1">
                            {provider.models.map((model) => (
                              <div
                                key={model}
                                className="text-white text-xs font-mono bg-slate-700/30 px-2 py-1 rounded"
                              >
                                {model}
                              </div>
                            ))}
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>
    </div>
  )
}
