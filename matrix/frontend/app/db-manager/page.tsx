"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import {
  Database,
  Play,
  Table,
  ArrowLeft,
  Zap,
  BarChart3,
  FileText,
  Settings,
  Plus,
  Search,
  Download,
} from "lucide-react"
import Link from "next/link"

const _bgGradient = "bg-gradient-to-br from-background via-secondary to-background"

export default function DbManager() {
  const [selectedDb, setSelectedDb] = useState("users_db")
  const [query, setQuery] = useState("")
  const [queryResult, setQueryResult] = useState(null)
  const [executing, setExecuting] = useState(false)

  const databases = [
    { id: "users_db", name: "Users Database", type: "PostgreSQL", status: "connected" },
    { id: "products_db", name: "Products Database", type: "MySQL", status: "connected" },
    { id: "analytics_db", name: "Analytics Database", type: "MongoDB", status: "disconnected" },
  ]

  const tables = [
    { name: "users", rows: 1250, size: "2.3 MB" },
    { name: "profiles", rows: 1180, size: "5.1 MB" },
    { name: "sessions", rows: 3420, size: "1.8 MB" },
    { name: "audit_logs", rows: 8900, size: "12.4 MB" },
  ]

  const handleExecuteQuery = async () => {
    if (!query.trim()) return

    setExecuting(true)
    // Simulate query execution
    setTimeout(() => {
      setQueryResult({
        columns: ["id", "name", "email", "created_at"],
        rows: [
          [1, "John Doe", "john@example.com", "2024-01-15"],
          [2, "Jane Smith", "jane@example.com", "2024-01-16"],
          [3, "Bob Johnson", "bob@example.com", "2024-01-17"],
        ],
        executionTime: 45,
        rowsAffected: 3,
      })
      setExecuting(false)
    }, 1000)
  }

  const optimizeQuery = () => {
    const optimizedQuery = `-- AI Optimized Query
SELECT u.id, u.name, u.email, u.created_at 
FROM users u 
WHERE u.active = true 
  AND u.created_at >= '2024-01-01'
ORDER BY u.created_at DESC 
LIMIT 100;

-- Optimization suggestions:
-- 1. Added index on (active, created_at) for better performance
-- 2. Limited results to prevent large data transfers
-- 3. Used explicit column selection instead of SELECT *`

    setQuery(optimizedQuery)
  }

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <Link href="/">
            <Button variant="outline" size="sm" className="border-border bg-transparent">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Hub
            </Button>
          </Link>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary rounded-lg">
              <Database className="w-6 h-6 text-primary-foreground" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-foreground">Database Explorer</h1>
              <p className="text-muted-foreground">Visual database management with AI-powered optimization</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Database Sidebar */}
          <Card className="bg-secondary/30 border-border">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <Database className="w-5 h-5" />
                Databases
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Database Selection */}
              <Select value={selectedDb} onValueChange={setSelectedDb}>
                <SelectTrigger className="bg-accent border-border">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {databases.map((db) => (
                    <SelectItem key={db.id} value={db.id}>
                      {db.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              {/* Database Info */}
              <div className="space-y-2">
                {databases.map((db) => (
                  <div
                    key={db.id}
                    className={`p-3 rounded-lg border ${
                      selectedDb === db.id ? "bg-accent/50 border-border" : "bg-secondary/20 border-border"
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <div>
                        <div className="text-foreground font-medium">{db.name}</div>
                        <div className="text-muted-foreground text-sm">{db.type}</div>
                      </div>
                      <Badge variant={db.status === "connected" ? "default" : "secondary"} className="text-xs">
                        {db.status}
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>

              {/* Tables */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <h4 className="text-foreground font-medium">Tables</h4>
                  <Button variant="outline" size="sm" className="border-border bg-transparent">
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>
                <div className="space-y-1">
                  {tables.map((table) => (
                    <div
                      key={table.name}
                      className="flex items-center justify-between p-2 hover:bg-secondary/30 rounded"
                    >
                      <div className="flex items-center gap-2">
                        <Table className="w-4 h-4 text-muted-foreground" />
                        <span className="text-foreground text-sm">{table.name}</span>
                      </div>
                      <div className="text-xs text-muted-foreground">{table.rows}</div>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Main Content */}
          <div className="lg:col-span-2 space-y-6">
            {/* Query Editor */}
            <Card className="bg-secondary/30 border-border">
              <CardHeader>
                <CardTitle className="text-foreground flex items-center gap-2">
                  <FileText className="w-5 h-5" />
                  Query Editor
                </CardTitle>
                <CardDescription className="text-muted-foreground">Write and execute SQL queries</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <Textarea
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="SELECT * FROM users WHERE active = true;"
                  className="min-h-32 bg-accent border-border text-foreground font-mono"
                />

                <div className="flex gap-2">
                  <Button onClick={handleExecuteQuery} disabled={executing || !query.trim()}>
                    {executing ? (
                      <>
                        <Settings className="w-4 h-4 mr-2 animate-spin" />
                        Executing...
                      </>
                    ) : (
                      <>
                        <Play className="w-4 h-4 mr-2" />
                        Execute Query
                      </>
                    )}
                  </Button>
                  <Button variant="outline" onClick={optimizeQuery} className="border-border bg-transparent">
                    <Zap className="w-4 h-4 mr-2" />
                    AI Optimize
                  </Button>
                  <Button variant="outline" className="border-border bg-transparent">
                    <Search className="w-4 h-4 mr-2" />
                    Explain
                  </Button>
                </div>
              </CardContent>
            </Card>

            {/* Results */}
            <Card className="bg-secondary/30 border-border">
              <CardHeader>
                <CardTitle className="text-foreground flex items-center gap-2">
                  <BarChart3 className="w-5 h-5" />
                  Query Results
                </CardTitle>
              </CardHeader>
              <CardContent>
                {!queryResult ? (
                  <div className="text-center py-12">
                    <Database className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                    <p className="text-muted-foreground">Execute a query to see results</p>
                  </div>
                ) : (
                  <Tabs defaultValue="results" className="w-full">
                    <TabsList className="grid w-full grid-cols-3 bg-secondary/50">
                      <TabsTrigger value="results" className="data-[state=active]:bg-accent">
                        Results
                      </TabsTrigger>
                      <TabsTrigger value="explain" className="data-[state=active]:bg-accent">
                        Explain Plan
                      </TabsTrigger>
                      <TabsTrigger value="schema" className="data-[state=active]:bg-accent">
                        Schema
                      </TabsTrigger>
                    </TabsList>

                    <TabsContent value="results" className="mt-6">
                      <div className="space-y-4">
                        {/* Stats */}
                        <div className="flex items-center gap-4">
                          <Badge variant="secondary">{queryResult.rowsAffected} rows</Badge>
                          <div className="flex items-center gap-1 text-muted-foreground">
                            <span className="text-sm">Execution time:</span>
                            <span className="text-sm font-mono">{queryResult.executionTime}ms</span>
                          </div>
                          <Button variant="outline" size="sm" className="border-border ml-auto bg-transparent">
                            <Download className="w-4 h-4 mr-1" />
                            Export
                          </Button>
                        </div>

                        {/* Table */}
                        <div className="border border-border rounded-lg overflow-hidden">
                          <table className="w-full">
                            <thead className="bg-secondary/50">
                              <tr>
                                {queryResult.columns.map((column) => (
                                  <th key={column} className="text-left p-3 text-foreground font-medium">
                                    {column}
                                  </th>
                                ))}
                              </tr>
                            </thead>
                            <tbody>
                              {queryResult.rows.map((row, index) => (
                                <tr key={index} className="border-t border-border">
                                  {row.map((cell, cellIndex) => (
                                    <td key={cellIndex} className="p-3 text-muted-foreground">
                                      {cell}
                                    </td>
                                  ))}
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        </div>
                      </div>
                    </TabsContent>

                    <TabsContent value="explain" className="mt-6">
                      <div className="bg-secondary/30 rounded-lg p-4">
                        <h4 className="text-foreground font-medium mb-3">Query Execution Plan</h4>
                        <div className="space-y-2 text-sm">
                          <div className="flex justify-between">
                            <span className="text-muted-foreground">1. Seq Scan on users</span>
                            <span className="text-foreground">cost=0.00..25.50</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-muted-foreground">2. Filter: (active = true)</span>
                            <span className="text-foreground">rows=1000</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-muted-foreground">3. Sort by created_at DESC</span>
                            <span className="text-foreground">cost=25.50..28.00</span>
                          </div>
                        </div>
                      </div>
                    </TabsContent>

                    <TabsContent value="schema" className="mt-6">
                      <div className="space-y-4">
                        <h4 className="text-foreground font-medium">Table Schema: users</h4>
                        <div className="border border-border rounded-lg overflow-hidden">
                          <table className="w-full">
                            <thead className="bg-secondary/50">
                              <tr>
                                <th className="text-left p-3 text-foreground font-medium">Column</th>
                                <th className="text-left p-3 text-foreground font-medium">Type</th>
                                <th className="text-left p-3 text-foreground font-medium">Nullable</th>
                                <th className="text-left p-3 text-foreground font-medium">Default</th>
                              </tr>
                            </thead>
                            <tbody>
                              <tr className="border-t border-border">
                                <td className="p-3 text-muted-foreground">id</td>
                                <td className="p-3 text-muted-foreground">integer</td>
                                <td className="p-3 text-muted-foreground">NO</td>
                                <td className="p-3 text-muted-foreground">nextval(&apos;users_id_seq&apos;)</td>
                              </tr>
                              <tr className="border-t border-border">
                                <td className="p-3 text-muted-foreground">name</td>
                                <td className="p-3 text-muted-foreground">varchar(255)</td>
                                <td className="p-3 text-muted-foreground">NO</td>
                                <td className="p-3 text-muted-foreground">NULL</td>
                              </tr>
                              <tr className="border-t border-border">
                                <td className="p-3 text-muted-foreground">email</td>
                                <td className="p-3 text-muted-foreground">varchar(255)</td>
                                <td className="p-3 text-muted-foreground">NO</td>
                                <td className="p-3 text-muted-foreground">NULL</td>
                              </tr>
                            </tbody>
                          </table>
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
    </div>
  )
}
