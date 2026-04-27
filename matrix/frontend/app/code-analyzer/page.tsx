"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import {
  Brain,
  Code2,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Zap,
  Shield,
  TrendingUp,
  FileText,
  ArrowLeft,
} from "lucide-react"
import Link from "next/link"

export default function CodeAnalyzer() {
  const [code, setCode] = useState("")
  const [language, setLanguage] = useState("javascript")
  const [analyzing, setAnalyzing] = useState(false)
  const [results, setResults] = useState(null)

  const handleAnalyze = async () => {
    if (!code.trim()) return

    setAnalyzing(true)
    // Simulate AI analysis
    setTimeout(() => {
      setResults({
        quality: {
          score: 85,
          issues: [
            { type: "warning", message: "Consider using const instead of let for immutable variables", line: 3 },
            { type: "info", message: "Function could be optimized using array methods", line: 8 },
            { type: "error", message: "Potential null reference error", line: 12 },
          ],
        },
        security: {
          score: 92,
          vulnerabilities: [
            { severity: "medium", message: "Potential XSS vulnerability in user input handling", line: 15 },
          ],
        },
        performance: {
          score: 78,
          suggestions: [
            { message: "Consider memoizing expensive calculations", impact: "high" },
            { message: "Use lazy loading for large datasets", impact: "medium" },
          ],
        },
        complexity: {
          cyclomatic: 6,
          cognitive: 8,
          maintainability: 72,
        },
      })
      setAnalyzing(false)
    }, 2000)
  }

  const sampleCode = `function processUserData(users) {
  let result = [];
  for (let i = 0; i < users.length; i++) {
    if (users[i].active) {
      result.push({
        id: users[i].id,
        name: users[i].name,
        email: users[i].email
      });
    }
  }
  return result;
}`

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <Link href="/">
            <Button variant="outline" size="sm">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Hub
            </Button>
          </Link>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary rounded-lg">
              <Brain className="w-6 h-6 text-primary-foreground" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-foreground">AI Code Analyzer</h1>
              <p className="text-muted-foreground">Intelligent code review and optimization suggestions</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-2 gap-8">
          {/* Input Section */}
          <Card className="bg-card border-border">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <Code2 className="w-5 h-5" />
                Code Input
              </CardTitle>
              <CardDescription className="text-muted-foreground">
                Paste your code below for AI-powered analysis
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-4">
                <Select value={language} onValueChange={setLanguage}>
                  <SelectTrigger className="w-48 bg-input border-input">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="javascript">JavaScript</SelectItem>
                    <SelectItem value="typescript">TypeScript</SelectItem>
                    <SelectItem value="python">Python</SelectItem>
                    <SelectItem value="java">Java</SelectItem>
                    <SelectItem value="csharp">C#</SelectItem>
                    <SelectItem value="go">Go</SelectItem>
                  </SelectContent>
                </Select>
                <Button variant="outline" onClick={() => setCode(sampleCode)}>
                  Load Sample
                </Button>
              </div>

              <Textarea
                value={code}
                onChange={(e) => setCode(e.target.value)}
                placeholder="Paste your code here..."
                className="min-h-96 bg-input border-input text-foreground font-mono text-sm"
              />

              <Button onClick={handleAnalyze} disabled={analyzing || !code.trim()} className="w-full">
                {analyzing ? (
                  <>
                    <Brain className="w-4 h-4 mr-2 animate-pulse" />
                    Analyzing Code...
                  </>
                ) : (
                  <>
                    <Zap className="w-4 h-4 mr-2" />
                    Analyze Code
                  </>
                )}
              </Button>
            </CardContent>
          </Card>

          {/* Results Section */}
          <Card className="bg-card border-border">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <FileText className="w-5 h-5" />
                Analysis Results
              </CardTitle>
              <CardDescription className="text-muted-foreground">
                AI-powered insights and recommendations
              </CardDescription>
            </CardHeader>
            <CardContent>
              {!results ? (
                <div className="text-center py-12">
                  <Brain className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                  <p className="text-muted-foreground">Submit code above to see AI analysis results</p>
                </div>
              ) : (
                <Tabs defaultValue="quality" className="w-full">
                  <TabsList className="grid w-full grid-cols-4 bg-card">
                    <TabsTrigger value="quality">Quality</TabsTrigger>
                    <TabsTrigger value="security">Security</TabsTrigger>
                    <TabsTrigger value="performance">Performance</TabsTrigger>
                    <TabsTrigger value="complexity">Complexity</TabsTrigger>
                  </TabsList>

                  <TabsContent value="quality" className="mt-6">
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <h3 className="text-foreground font-medium">Code Quality Score</h3>
                        <Badge variant="secondary" className="text-lg px-3 py-1">
                          {results.quality.score}/100
                        </Badge>
                      </div>
                      <div className="space-y-3">
                        {results.quality.issues.map((issue, index) => (
                          <div
                            key={index}
                            className="flex items-start gap-3 p-3 bg-card rounded-lg border border-border"
                          >
                            {issue.type === "error" ? (
                              <XCircle className="w-5 h-5 text-destructive mt-0.5" />
                            ) : issue.type === "warning" ? (
                              <AlertTriangle className="w-5 h-5 text-amber-500 mt-0.5" />
                            ) : (
                              <CheckCircle className="w-5 h-5 text-emerald-500 mt-0.5" />
                            )}
                            <div className="flex-1">
                              <p className="text-foreground text-sm">{issue.message}</p>
                              <p className="text-muted-foreground text-xs">Line {issue.line}</p>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  </TabsContent>

                  <TabsContent value="security" className="mt-6">
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <h3 className="text-foreground font-medium">Security Score</h3>
                        <Badge variant="secondary" className="text-lg px-3 py-1">
                          {results.security.score}/100
                        </Badge>
                      </div>
                      <div className="space-y-3">
                        {results.security.vulnerabilities.map((vuln, index) => (
                          <div
                            key={index}
                            className="flex items-start gap-3 p-3 bg-card rounded-lg border border-border"
                          >
                            <Shield className="w-5 h-5 text-amber-600 mt-0.5" />
                            <div className="flex-1">
                              <div className="flex items-center gap-2 mb-1">
                                <Badge
                                  variant={vuln.severity === "high" ? "destructive" : "secondary"}
                                  className="text-xs"
                                >
                                  {vuln.severity}
                                </Badge>
                              </div>
                              <p className="text-foreground text-sm">{vuln.message}</p>
                              <p className="text-muted-foreground text-xs">Line {vuln.line}</p>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  </TabsContent>

                  <TabsContent value="performance" className="mt-6">
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <h3 className="text-foreground font-medium">Performance Score</h3>
                        <Badge variant="secondary" className="text-lg px-3 py-1">
                          {results.performance.score}/100
                        </Badge>
                      </div>
                      <div className="space-y-3">
                        {results.performance.suggestions.map((suggestion, index) => (
                          <div
                            key={index}
                            className="flex items-start gap-3 p-3 bg-card rounded-lg border border-border"
                          >
                            <TrendingUp className="w-5 h-5 text-emerald-500 mt-0.5" />
                            <div className="flex-1">
                              <div className="flex items-center gap-2 mb-1">
                                <Badge
                                  variant={suggestion.impact === "high" ? "default" : "secondary"}
                                  className="text-xs"
                                >
                                  {suggestion.impact} impact
                                </Badge>
                              </div>
                              <p className="text-foreground text-sm">{suggestion.message}</p>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  </TabsContent>

                  <TabsContent value="complexity" className="mt-6">
                    <div className="space-y-4">
                      <div className="grid grid-cols-2 gap-4">
                        <div className="p-4 bg-card rounded-lg text-center border border-border">
                          <div className="text-2xl font-bold text-foreground">{results.complexity.cyclomatic}</div>
                          <div className="text-sm text-muted-foreground">Cyclomatic Complexity</div>
                        </div>
                        <div className="p-4 bg-card rounded-lg text-center border border-border">
                          <div className="text-2xl font-bold text-foreground">{results.complexity.cognitive}</div>
                          <div className="text-sm text-muted-foreground">Cognitive Complexity</div>
                        </div>
                      </div>
                      <div className="p-4 bg-card rounded-lg border border-border">
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-foreground">Maintainability Index</span>
                          <span className="text-foreground font-bold">{results.complexity.maintainability}/100</span>
                        </div>
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div
                            className="h-full bg-gradient-to-r from-emerald-500 to-primary"
                            style={{ width: `${results.complexity.maintainability}%` }}
                          ></div>
                        </div>
                      </div>
                    </div>
                  </TabsContent>
                </Tabs>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
