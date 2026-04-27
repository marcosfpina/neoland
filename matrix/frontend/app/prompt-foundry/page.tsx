"use client"

import type React from "react"

import { useState, useMemo } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Label } from "@/components/ui/label"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Sparkles,
  Search,
  Star,
  Zap,
  Copy,
  Play,
  ChevronRight,
  Code,
  FileText,
  Bug,
  RefreshCw,
  TestTube,
  Brain,
  Settings,
  ArrowLeft,
  Flame,
  Clock,
} from "lucide-react"
import Link from "next/link"
import { PROMPT_TEMPLATES, searchTemplates } from "@/lib/prompt-foundry/templates"
import type { PromptTemplate, PromptCategory, PromptVariable } from "@/lib/prompt-foundry/types"
import { cn } from "@/lib/utils"

const categoryIcons: Record<PromptCategory, React.ReactNode> = {
  "code-generation": <Code className="w-4 h-4" />,
  "code-review": <Search className="w-4 h-4" />,
  documentation: <FileText className="w-4 h-4" />,
  debugging: <Bug className="w-4 h-4" />,
  refactoring: <RefreshCw className="w-4 h-4" />,
  testing: <TestTube className="w-4 h-4" />,
  explanation: <Brain className="w-4 h-4" />,
  translation: <Zap className="w-4 h-4" />,
  creative: <Sparkles className="w-4 h-4" />,
  system: <Settings className="w-4 h-4" />,
  chain: <ChevronRight className="w-4 h-4" />,
}

const categoryColors: Record<PromptCategory, string> = {
  "code-generation": "bg-blue-500/10 text-blue-400 border-blue-500/20",
  "code-review": "bg-purple-500/10 text-purple-400 border-purple-500/20",
  documentation: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
  debugging: "bg-red-500/10 text-red-400 border-red-500/20",
  refactoring: "bg-amber-500/10 text-amber-400 border-amber-500/20",
  testing: "bg-cyan-500/10 text-cyan-400 border-cyan-500/20",
  explanation: "bg-pink-500/10 text-pink-400 border-pink-500/20",
  translation: "bg-indigo-500/10 text-indigo-400 border-indigo-500/20",
  creative: "bg-orange-500/10 text-orange-400 border-orange-500/20",
  system: "bg-slate-500/10 text-slate-400 border-slate-500/20",
  chain: "bg-violet-500/10 text-violet-400 border-violet-500/20",
}

export default function PromptFoundry() {
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedCategory, setSelectedCategory] = useState<PromptCategory | "all">("all")
  const [selectedTemplate, setSelectedTemplate] = useState<PromptTemplate | null>(null)
  const [variables, setVariables] = useState<Record<string, string>>({})
  const [generatedPrompt, setGeneratedPrompt] = useState("")
  const [isGenerating, setIsGenerating] = useState(false)

  // Filter templates
  const filteredTemplates = useMemo(() => {
    let templates = PROMPT_TEMPLATES

    if (searchQuery) {
      templates = searchTemplates(searchQuery)
    }

    if (selectedCategory !== "all") {
      templates = templates.filter((t) => t.category === selectedCategory)
    }

    return templates.sort((a, b) => b.rating - a.rating)
  }, [searchQuery, selectedCategory])

  // Get unique categories
  const categories = useMemo(() => {
    const cats = new Set(PROMPT_TEMPLATES.map((t) => t.category))
    return Array.from(cats)
  }, [])

  // Handle template selection
  const handleSelectTemplate = (template: PromptTemplate) => {
    setSelectedTemplate(template)
    // Initialize variables with defaults
    const defaults: Record<string, string> = {}
    template.variables.forEach((v) => {
      if (v.default) defaults[v.name] = v.default
    })
    setVariables(defaults)
    setGeneratedPrompt("")
  }

  // Generate prompt from template
  const generatePrompt = () => {
    if (!selectedTemplate) return

    setIsGenerating(true)

    let prompt = selectedTemplate.template
    selectedTemplate.variables.forEach((v) => {
      const value = variables[v.name] || v.default || `[${v.name}]`
      prompt = prompt.replace(new RegExp(`{{${v.name}}}`, "g"), value)
    })

    setTimeout(() => {
      setGeneratedPrompt(prompt)
      setIsGenerating(false)
    }, 500)
  }

  // Copy to clipboard
  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  // Render variable input
  const renderVariableInput = (variable: PromptVariable) => {
    const value = variables[variable.name] || ""

    switch (variable.type) {
      case "select":
        return (
          <Select value={value} onValueChange={(v) => setVariables((prev) => ({ ...prev, [variable.name]: v }))}>
            <SelectTrigger className="bg-slate-900 border-slate-700">
              <SelectValue placeholder={`Select ${variable.name}`} />
            </SelectTrigger>
            <SelectContent>
              {variable.options?.map((opt) => (
                <SelectItem key={opt} value={opt}>
                  {opt}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )

      case "code":
        return (
          <Textarea
            value={value}
            onChange={(e) => setVariables((prev) => ({ ...prev, [variable.name]: e.target.value }))}
            placeholder={variable.description}
            className="bg-slate-900 border-slate-700 font-mono text-sm min-h-[120px]"
          />
        )

      case "boolean":
        return (
          <Select value={value} onValueChange={(v) => setVariables((prev) => ({ ...prev, [variable.name]: v }))}>
            <SelectTrigger className="bg-slate-900 border-slate-700">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="true">Yes</SelectItem>
              <SelectItem value="false">No</SelectItem>
            </SelectContent>
          </Select>
        )

      default:
        return (
          <Input
            type={variable.type === "number" ? "number" : "text"}
            value={value}
            onChange={(e) => setVariables((prev) => ({ ...prev, [variable.name]: e.target.value }))}
            placeholder={variable.description}
            className="bg-slate-900 border-slate-700"
          />
        )
    }
  }

  return (
    <div className="min-h-screen bg-slate-950">
      {/* Header */}
      <header className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-4">
              <Link href="/">
                <Button variant="ghost" size="sm" className="text-slate-400">
                  <ArrowLeft className="w-4 h-4 mr-2" />
                  Back
                </Button>
              </Link>
              <div className="flex items-center gap-3">
                <div className="p-2 bg-gradient-to-br from-orange-500 to-red-600 rounded-lg">
                  <Sparkles className="w-5 h-5 text-white" />
                </div>
                <div>
                  <h1 className="text-white font-bold">Prompt Foundry</h1>
                  <p className="text-xs text-slate-400">Craft powerful prompts</p>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Badge variant="outline" className="border-orange-500/30 text-orange-400">
                <Flame className="w-3 h-3 mr-1" />
                FORGE Agent
              </Badge>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        <div className="grid grid-cols-12 gap-6">
          {/* Template Browser */}
          <div className="col-span-4">
            <Card className="bg-slate-900/50 border-slate-800 sticky top-24">
              <CardHeader className="pb-3">
                <CardTitle className="text-white text-lg flex items-center gap-2">
                  <Search className="w-5 h-5 text-slate-400" />
                  Template Library
                </CardTitle>

                {/* Search */}
                <div className="relative mt-3">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                  <Input
                    placeholder="Search templates..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-9 bg-slate-800 border-slate-700"
                  />
                </div>

                {/* Category Filter */}
                <div className="flex flex-wrap gap-1 mt-3">
                  <Badge
                    variant="outline"
                    className={cn(
                      "cursor-pointer transition-colors",
                      selectedCategory === "all"
                        ? "bg-violet-500/20 text-violet-400 border-violet-500/30"
                        : "border-slate-700 text-slate-400 hover:border-slate-600",
                    )}
                    onClick={() => setSelectedCategory("all")}
                  >
                    All
                  </Badge>
                  {categories.map((cat) => (
                    <Badge
                      key={cat}
                      variant="outline"
                      className={cn(
                        "cursor-pointer transition-colors",
                        selectedCategory === cat
                          ? categoryColors[cat]
                          : "border-slate-700 text-slate-400 hover:border-slate-600",
                      )}
                      onClick={() => setSelectedCategory(cat)}
                    >
                      {categoryIcons[cat]}
                      <span className="ml-1 capitalize">{cat.replace("-", " ")}</span>
                    </Badge>
                  ))}
                </div>
              </CardHeader>

              <CardContent>
                <ScrollArea className="h-[calc(100vh-320px)]">
                  <div className="space-y-2 pr-4">
                    {filteredTemplates.map((template) => (
                      <div
                        key={template.id}
                        className={cn(
                          "p-3 rounded-lg cursor-pointer transition-all",
                          selectedTemplate?.id === template.id
                            ? "bg-violet-500/20 border border-violet-500/30"
                            : "bg-slate-800/50 border border-transparent hover:bg-slate-800 hover:border-slate-700",
                        )}
                        onClick={() => handleSelectTemplate(template)}
                      >
                        <div className="flex items-start justify-between mb-2">
                          <h3 className="text-white font-medium text-sm">{template.name}</h3>
                          <div className="flex items-center gap-1 text-amber-400">
                            <Star className="w-3 h-3 fill-current" />
                            <span className="text-xs">{template.rating}</span>
                          </div>
                        </div>
                        <p className="text-xs text-slate-400 mb-2 line-clamp-2">{template.description}</p>
                        <div className="flex items-center justify-between">
                          <Badge variant="outline" className={cn("text-xs", categoryColors[template.category])}>
                            {categoryIcons[template.category]}
                            <span className="ml-1">{template.category}</span>
                          </Badge>
                          <span className="text-xs text-slate-500">{template.usageCount} uses</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>

          {/* Template Editor */}
          <div className="col-span-8">
            {selectedTemplate ? (
              <div className="space-y-6">
                {/* Template Info */}
                <Card className="bg-slate-900/50 border-slate-800">
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div>
                        <CardTitle className="text-white text-xl">{selectedTemplate.name}</CardTitle>
                        <p className="text-slate-400 mt-1">{selectedTemplate.description}</p>
                      </div>
                      <div className="flex items-center gap-2">
                        <Badge variant="outline" className="border-slate-700 text-slate-400">
                          <Clock className="w-3 h-3 mr-1" />
                          {new Date(selectedTemplate.updatedAt).toLocaleDateString()}
                        </Badge>
                        <Badge variant="outline" className="border-orange-500/30 text-orange-400">
                          by {selectedTemplate.author}
                        </Badge>
                      </div>
                    </div>

                    {/* Tags */}
                    <div className="flex flex-wrap gap-1 mt-3">
                      {selectedTemplate.tags.map((tag) => (
                        <Badge key={tag} variant="outline" className="text-xs border-slate-700 text-slate-400">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  </CardHeader>
                </Card>

                {/* Variables */}
                <Card className="bg-slate-900/50 border-slate-800">
                  <CardHeader>
                    <CardTitle className="text-white text-lg flex items-center gap-2">
                      <Settings className="w-5 h-5 text-slate-400" />
                      Variables
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-2 gap-4">
                      {selectedTemplate.variables.map((variable) => (
                        <div key={variable.name} className="space-y-2">
                          <Label className="text-slate-300 flex items-center gap-2">
                            {variable.name}
                            {variable.required && <span className="text-red-400">*</span>}
                          </Label>
                          {renderVariableInput(variable)}
                          <p className="text-xs text-slate-500">{variable.description}</p>
                        </div>
                      ))}
                    </div>

                    <div className="flex justify-end mt-6">
                      <Button
                        onClick={generatePrompt}
                        disabled={isGenerating}
                        className="bg-gradient-to-r from-orange-500 to-red-600 hover:from-orange-600 hover:to-red-700"
                      >
                        {isGenerating ? (
                          <>
                            <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                            Generating...
                          </>
                        ) : (
                          <>
                            <Sparkles className="w-4 h-4 mr-2" />
                            Generate Prompt
                          </>
                        )}
                      </Button>
                    </div>
                  </CardContent>
                </Card>

                {/* Generated Prompt */}
                {generatedPrompt && (
                  <Card className="bg-slate-900/50 border-slate-800">
                    <CardHeader>
                      <div className="flex items-center justify-between">
                        <CardTitle className="text-white text-lg flex items-center gap-2">
                          <Zap className="w-5 h-5 text-amber-400" />
                          Generated Prompt
                        </CardTitle>
                        <div className="flex items-center gap-2">
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => copyToClipboard(generatedPrompt)}
                            className="border-slate-700 text-slate-300"
                          >
                            <Copy className="w-4 h-4 mr-2" />
                            Copy
                          </Button>
                          <Link href={`/llm-workbench?prompt=${encodeURIComponent(generatedPrompt)}`}>
                            <Button size="sm" className="bg-violet-600 hover:bg-violet-700">
                              <Play className="w-4 h-4 mr-2" />
                              Run in Workbench
                            </Button>
                          </Link>
                        </div>
                      </div>
                    </CardHeader>
                    <CardContent>
                      <div className="bg-slate-950 rounded-lg p-4 border border-slate-800">
                        <pre className="text-sm text-slate-300 whitespace-pre-wrap font-mono">{generatedPrompt}</pre>
                      </div>

                      {/* Prompt Stats */}
                      <div className="flex items-center gap-4 mt-4 text-sm text-slate-400">
                        <span>{generatedPrompt.length} characters</span>
                        <span>{generatedPrompt.split(/\s+/).length} words</span>
                        <span>~{Math.ceil(generatedPrompt.length / 4)} tokens</span>
                      </div>
                    </CardContent>
                  </Card>
                )}

                {/* Template Preview */}
                <Card className="bg-slate-900/50 border-slate-800">
                  <CardHeader>
                    <CardTitle className="text-white text-lg flex items-center gap-2">
                      <FileText className="w-5 h-5 text-slate-400" />
                      Template Source
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-950 rounded-lg p-4 border border-slate-800">
                      <pre className="text-sm text-slate-400 whitespace-pre-wrap font-mono">
                        {selectedTemplate.template}
                      </pre>
                    </div>
                  </CardContent>
                </Card>
              </div>
            ) : (
              <Card className="bg-slate-900/50 border-slate-800 h-[calc(100vh-200px)] flex items-center justify-center">
                <div className="text-center">
                  <div className="p-4 bg-slate-800 rounded-full inline-block mb-4">
                    <Sparkles className="w-8 h-8 text-slate-500" />
                  </div>
                  <h3 className="text-white font-medium mb-2">Select a Template</h3>
                  <p className="text-slate-400 text-sm">Choose a template from the library to start crafting</p>
                </div>
              </Card>
            )}
          </div>
        </div>
      </main>
    </div>
  )
}
