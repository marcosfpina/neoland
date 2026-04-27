"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Zap, Send, Clock, CheckCircle, XCircle, FileText, ArrowLeft, Plus, Trash2, Copy, Download } from "lucide-react"
import Link from "next/link"

export default function ApiTester() {
  const [method, setMethod] = useState("GET")
  const [url, setUrl] = useState("")
  const [headers, setHeaders] = useState([{ key: "", value: "" }])
  const [body, setBody] = useState("")
  const [response, setResponse] = useState(null)
  const [loading, setLoading] = useState(false)
  const [tests, setTests] = useState([])

  const handleAddHeader = () => {
    setHeaders([...headers, { key: "", value: "" }])
  }

  const handleRemoveHeader = (index) => {
    setHeaders(headers.filter((_, i) => i !== index))
  }

  const handleHeaderChange = (index, field, value) => {
    const newHeaders = [...headers]
    newHeaders[index][field] = value
    setHeaders(newHeaders)
  }

  const handleSendRequest = async () => {
    if (!url.trim()) return

    setLoading(true)
    // Simulate API request
    setTimeout(() => {
      const mockResponse = {
        status: 200,
        statusText: "OK",
        headers: {
          "content-type": "application/json",
          "x-response-time": "45ms",
        },
        data: {
          success: true,
          message: "API request successful",
          timestamp: new Date().toISOString(),
          data: [
            { id: 1, name: "John Doe", email: "john@example.com" },
            { id: 2, name: "Jane Smith", email: "jane@example.com" },
          ],
        },
        time: 45,
      }
      setResponse(mockResponse)
      setLoading(false)
    }, 1000)
  }

  const generateTests = () => {
    const generatedTests = [
      {
        name: "Status Code Check",
        assertion: "response.status === 200",
        passed: true,
      },
      {
        name: "Response Time",
        assertion: "response.time < 100",
        passed: true,
      },
      {
        name: "Content Type",
        assertion: "response.headers['content-type'].includes('application/json')",
        passed: true,
      },
      {
        name: "Data Structure",
        assertion: "response.data.success === true",
        passed: true,
      },
    ]
    setTests(generatedTests)
  }

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
              <Zap className="w-6 h-6 text-primary-foreground" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-foreground">Smart API Tester</h1>
              <p className="text-muted-foreground">Test, validate, and document your APIs with AI assistance</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-2 gap-8">
          {/* Request Builder */}
          <Card className="bg-card border-border">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <Send className="w-5 h-5" />
                Request Builder
              </CardTitle>
              <CardDescription className="text-muted-foreground">Configure and send HTTP requests</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-2">
                <Select value={method} onValueChange={setMethod}>
                  <SelectTrigger className="w-32 bg-input border-input">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="GET">GET</SelectItem>
                    <SelectItem value="POST">POST</SelectItem>
                    <SelectItem value="PUT">PUT</SelectItem>
                    <SelectItem value="DELETE">DELETE</SelectItem>
                    <SelectItem value="PATCH">PATCH</SelectItem>
                  </SelectContent>
                </Select>
                <Input
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  placeholder="https://api.example.com/users"
                  className="bg-input border-input text-foreground"
                />
              </div>

              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-foreground text-sm font-medium">Headers</label>
                  <Button variant="outline" size="sm" onClick={handleAddHeader}>
                    <Plus className="w-4 h-4 mr-1" />
                    Add
                  </Button>
                </div>
                <div className="space-y-2">
                  {headers.map((header, index) => (
                    <div key={index} className="flex gap-2">
                      <Input
                        value={header.key}
                        onChange={(e) => handleHeaderChange(index, "key", e.target.value)}
                        placeholder="Header name"
                        className="bg-input border-input text-foreground"
                      />
                      <Input
                        value={header.value}
                        onChange={(e) => handleHeaderChange(index, "value", e.target.value)}
                        placeholder="Header value"
                        className="bg-input border-input text-foreground"
                      />
                      <Button variant="outline" size="sm" onClick={() => handleRemoveHeader(index)}>
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </div>

              {(method === "POST" || method === "PUT" || method === "PATCH") && (
                <div>
                  <label className="text-foreground text-sm font-medium mb-2 block">Request Body</label>
                  <Textarea
                    value={body}
                    onChange={(e) => setBody(e.target.value)}
                    placeholder='{"key": "value"}'
                    className="min-h-32 bg-input border-input text-foreground font-mono"
                  />
                </div>
              )}

              <div className="flex gap-2">
                <Button onClick={handleSendRequest} disabled={loading || !url.trim()} className="flex-1">
                  {loading ? (
                    <>
                      <Clock className="w-4 h-4 mr-2 animate-spin" />
                      Sending...
                    </>
                  ) : (
                    <>
                      <Send className="w-4 h-4 mr-2" />
                      Send Request
                    </>
                  )}
                </Button>
                <Button variant="outline" onClick={generateTests}>
                  Generate Tests
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* Response & Tests */}
          <Card className="bg-card border-border">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <FileText className="w-5 h-5" />
                Response & Tests
              </CardTitle>
              <CardDescription className="text-muted-foreground">View response and run automated tests</CardDescription>
            </CardHeader>
            <CardContent>
              {!response ? (
                <div className="text-center py-12">
                  <Send className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                  <p className="text-muted-foreground">Send a request to see the response</p>
                </div>
              ) : (
                <Tabs defaultValue="response" className="w-full">
                  <TabsList className="grid w-full grid-cols-3 bg-card">
                    <TabsTrigger value="response">Response</TabsTrigger>
                    <TabsTrigger value="tests">Tests</TabsTrigger>
                    <TabsTrigger value="docs">Docs</TabsTrigger>
                  </TabsList>

                  <TabsContent value="response" className="mt-6">
                    <div className="space-y-4">
                      <div className="flex items-center gap-4">
                        <Badge
                          variant={response.status < 400 ? "default" : "destructive"}
                          className="text-lg px-3 py-1"
                        >
                          {response.status} {response.statusText}
                        </Badge>
                        <div className="flex items-center gap-2 text-muted-foreground">
                          <Clock className="w-4 h-4" />
                          <span>{response.time}ms</span>
                        </div>
                      </div>

                      <div>
                        <h4 className="text-foreground font-medium mb-2">Response Headers</h4>
                        <div className="bg-card rounded-lg p-3 space-y-1 border border-border">
                          {Object.entries(response.headers).map(([key, value]) => (
                            <div key={key} className="flex justify-between text-sm">
                              <span className="text-muted-foreground">{key}:</span>
                              <span className="text-foreground">{value}</span>
                            </div>
                          ))}
                        </div>
                      </div>

                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <h4 className="text-foreground font-medium">Response Body</h4>
                          <Button variant="outline" size="sm">
                            <Copy className="w-4 h-4 mr-1" />
                            Copy
                          </Button>
                        </div>
                        <pre className="bg-background border border-border rounded-lg p-4 text-sm text-foreground overflow-auto max-h-64">
                          {JSON.stringify(response.data, null, 2)}
                        </pre>
                      </div>
                    </div>
                  </TabsContent>

                  <TabsContent value="tests" className="mt-6">
                    <div className="space-y-4">
                      {tests.length === 0 ? (
                        <div className="text-center py-8">
                          <CheckCircle className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                          <p className="text-muted-foreground mb-4">No tests generated yet</p>
                          <Button onClick={generateTests} variant="outline">
                            Generate AI Tests
                          </Button>
                        </div>
                      ) : (
                        <>
                          <div className="flex items-center justify-between">
                            <h4 className="text-foreground font-medium">Test Results</h4>
                            <Badge variant="secondary">
                              {tests.filter((t) => t.passed).length}/{tests.length} Passed
                            </Badge>
                          </div>
                          <div className="space-y-2">
                            {tests.map((test, index) => (
                              <div
                                key={index}
                                className="flex items-center gap-3 p-3 bg-card rounded-lg border border-border"
                              >
                                {test.passed ? (
                                  <CheckCircle className="w-5 h-5 text-emerald-500" />
                                ) : (
                                  <XCircle className="w-5 h-5 text-destructive" />
                                )}
                                <div className="flex-1">
                                  <div className="text-foreground font-medium">{test.name}</div>
                                  <div className="text-muted-foreground text-sm font-mono">{test.assertion}</div>
                                </div>
                              </div>
                            ))}
                          </div>
                        </>
                      )}
                    </div>
                  </TabsContent>

                  <TabsContent value="docs" className="mt-6">
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <h4 className="text-foreground font-medium">API Documentation</h4>
                        <Button variant="outline" size="sm">
                          <Download className="w-4 h-4 mr-1" />
                          Export
                        </Button>
                      </div>
                      <div className="bg-card rounded-lg p-4 space-y-3 border border-border">
                        <div>
                          <h5 className="text-foreground font-medium">Endpoint</h5>
                          <code className="text-primary">
                            {method} {url}
                          </code>
                        </div>
                        <div>
                          <h5 className="text-foreground font-medium">Description</h5>
                          <p className="text-muted-foreground">Auto-generated API endpoint documentation</p>
                        </div>
                        <div>
                          <h5 className="text-foreground font-medium">Response Schema</h5>
                          <pre className="text-sm text-muted-foreground mt-1">
                            {JSON.stringify(
                              {
                                success: "boolean",
                                message: "string",
                                data: "array",
                              },
                              null,
                              2,
                            )}
                          </pre>
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
