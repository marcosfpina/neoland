"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  GitBranch,
  GitCommit,
  GitMerge,
  ArrowLeft,
  Brain,
  Clock,
  CheckCircle,
  AlertCircle,
  Plus,
  RefreshCw,
} from "lucide-react"
import Link from "next/link"

export default function GitAssistant() {
  const [commitMessage, setCommitMessage] = useState("")
  const [generatingCommit, setGeneratingCommit] = useState(false)
  const [branchName, setBranchName] = useState("")

  const gitStatus = {
    branch: "feature/user-authentication",
    ahead: 3,
    behind: 1,
    staged: 5,
    unstaged: 2,
    untracked: 1,
  }

  const recentCommits = [
    {
      hash: "a1b2c3d",
      message: "feat: add user authentication system",
      author: "John Doe",
      time: "2 hours ago",
      files: 8,
    },
    {
      hash: "e4f5g6h",
      message: "fix: resolve login validation bug",
      author: "Jane Smith",
      time: "4 hours ago",
      files: 3,
    },
    {
      hash: "i7j8k9l",
      message: "docs: update API documentation",
      author: "Bob Johnson",
      time: "1 day ago",
      files: 2,
    },
  ]

  const branches = [
    { name: "main", type: "main", lastCommit: "2 days ago", ahead: 0, behind: 0 },
    { name: "develop", type: "develop", lastCommit: "1 hour ago", ahead: 5, behind: 2 },
    { name: "feature/user-authentication", type: "feature", lastCommit: "2 hours ago", ahead: 3, behind: 1 },
    { name: "hotfix/security-patch", type: "hotfix", lastCommit: "3 hours ago", ahead: 1, behind: 0 },
  ]

  const generateCommitMessage = async () => {
    setGeneratingCommit(true)
    // Simulate AI generation
    setTimeout(() => {
      const messages = [
        "feat: implement user authentication with JWT tokens\n\n- Add login and registration endpoints\n- Implement JWT token generation and validation\n- Add password hashing with bcrypt\n- Create user session management\n- Add authentication middleware",
        "fix: resolve memory leak in user session handling\n\n- Fix event listener cleanup in auth component\n- Optimize token refresh mechanism\n- Add proper error handling for expired tokens",
        "refactor: improve code organization in auth module\n\n- Extract authentication logic into separate service\n- Simplify component structure\n- Add comprehensive error handling\n- Improve type definitions",
      ]
      setCommitMessage(messages[Math.floor(Math.random() * messages.length)])
      setGeneratingCommit(false)
    }, 1500)
  }

  const generateBranchName = () => {
    const types = ["feature", "bugfix", "hotfix", "refactor"]
    const features = ["user-profile", "payment-system", "notification-service", "data-export", "admin-panel"]
    const type = types[Math.floor(Math.random() * types.length)]
    const feature = features[Math.floor(Math.random() * features.length)]
    setBranchName(`${type}/${feature}`)
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-orange-900 to-slate-900">
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
            <div className="p-2 bg-orange-500 rounded-lg">
              <GitBranch className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-white">Git Flow Assistant</h1>
              <p className="text-gray-300">Intelligent git workflow management and automation</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Git Status */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <GitBranch className="w-5 h-5" />
                Repository Status
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Current Branch */}
              <div className="p-3 bg-slate-700/30 rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-white font-medium">Current Branch</span>
                  <Badge variant="secondary">{gitStatus.branch}</Badge>
                </div>
                <div className="flex items-center gap-4 text-sm">
                  <span className="text-green-400">↑ {gitStatus.ahead} ahead</span>
                  <span className="text-red-400">↓ {gitStatus.behind} behind</span>
                </div>
              </div>

              {/* Changes */}
              <div className="space-y-2">
                <h4 className="text-white font-medium">Changes</h4>
                <div className="space-y-1">
                  <div className="flex items-center justify-between p-2 bg-slate-700/20 rounded">
                    <span className="text-green-400 text-sm">Staged</span>
                    <Badge variant="outline" className="text-xs">
                      {gitStatus.staged}
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between p-2 bg-slate-700/20 rounded">
                    <span className="text-yellow-400 text-sm">Unstaged</span>
                    <Badge variant="outline" className="text-xs">
                      {gitStatus.unstaged}
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between p-2 bg-slate-700/20 rounded">
                    <span className="text-gray-400 text-sm">Untracked</span>
                    <Badge variant="outline" className="text-xs">
                      {gitStatus.untracked}
                    </Badge>
                  </div>
                </div>
              </div>

              {/* Quick Actions */}
              <div className="space-y-2">
                <h4 className="text-white font-medium">Quick Actions</h4>
                <div className="grid grid-cols-2 gap-2">
                  <Button variant="outline" size="sm" className="border-slate-600">
                    <Plus className="w-4 h-4 mr-1" />
                    Stage All
                  </Button>
                  <Button variant="outline" size="sm" className="border-slate-600">
                    <RefreshCw className="w-4 h-4 mr-1" />
                    Pull
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Main Content */}
          <div className="lg:col-span-2">
            <Tabs defaultValue="commit" className="w-full">
              <TabsList className="grid w-full grid-cols-4 bg-slate-800/50">
                <TabsTrigger value="commit" className="data-[state=active]:bg-slate-700">
                  Smart Commit
                </TabsTrigger>
                <TabsTrigger value="branches" className="data-[state=active]:bg-slate-700">
                  Branches
                </TabsTrigger>
                <TabsTrigger value="history" className="data-[state=active]:bg-slate-700">
                  History
                </TabsTrigger>
                <TabsTrigger value="merge" className="data-[state=active]:bg-slate-700">
                  Merge
                </TabsTrigger>
              </TabsList>

              <TabsContent value="commit" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <GitCommit className="w-5 h-5" />
                      AI-Powered Commit
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Generate intelligent commit messages based on your changes
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="flex gap-2">
                      <Button onClick={generateCommitMessage} disabled={generatingCommit} className="flex-1">
                        {generatingCommit ? (
                          <>
                            <Brain className="w-4 h-4 mr-2 animate-pulse" />
                            Analyzing Changes...
                          </>
                        ) : (
                          <>
                            <Brain className="w-4 h-4 mr-2" />
                            Generate Commit Message
                          </>
                        )}
                      </Button>
                    </div>

                    <Textarea
                      value={commitMessage}
                      onChange={(e) => setCommitMessage(e.target.value)}
                      placeholder="Enter commit message or generate one with AI..."
                      className="min-h-32 bg-slate-700 border-slate-600 text-white"
                    />

                    <div className="flex gap-2">
                      <Button disabled={!commitMessage.trim()}>
                        <GitCommit className="w-4 h-4 mr-2" />
                        Commit Changes
                      </Button>
                      <Button variant="outline" className="border-slate-600">
                        Commit & Push
                      </Button>
                    </div>

                    {/* Commit Guidelines */}
                    <div className="p-3 bg-slate-700/30 rounded-lg">
                      <h4 className="text-white font-medium mb-2">Commit Guidelines</h4>
                      <div className="space-y-1 text-sm text-gray-300">
                        <div>• Use conventional commits: feat, fix, docs, style, refactor, test, chore</div>
                        <div>• Keep the first line under 50 characters</div>
                        <div>• Use present tense: "add feature" not "added feature"</div>
                        <div>• Include detailed description for complex changes</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="branches" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <GitBranch className="w-5 h-5" />
                      Branch Management
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Manage and create branches intelligently
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {/* Create Branch */}
                    <div className="p-4 bg-slate-700/30 rounded-lg">
                      <h4 className="text-white font-medium mb-3">Create New Branch</h4>
                      <div className="flex gap-2 mb-2">
                        <Input
                          value={branchName}
                          onChange={(e) => setBranchName(e.target.value)}
                          placeholder="feature/new-feature"
                          className="bg-slate-700 border-slate-600 text-white"
                        />
                        <Button variant="outline" onClick={generateBranchName} className="border-slate-600">
                          <Brain className="w-4 h-4 mr-1" />
                          Generate
                        </Button>
                      </div>
                      <Button disabled={!branchName.trim()}>
                        <Plus className="w-4 h-4 mr-2" />
                        Create Branch
                      </Button>
                    </div>

                    {/* Branch List */}
                    <div className="space-y-2">
                      {branches.map((branch) => (
                        <div
                          key={branch.name}
                          className="flex items-center justify-between p-3 bg-slate-700/20 rounded-lg"
                        >
                          <div className="flex items-center gap-3">
                            <GitBranch className="w-4 h-4 text-gray-400" />
                            <div>
                              <div className="text-white font-medium">{branch.name}</div>
                              <div className="text-gray-400 text-sm">Last commit: {branch.lastCommit}</div>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            {branch.ahead > 0 && (
                              <Badge variant="outline" className="text-xs text-green-400">
                                +{branch.ahead}
                              </Badge>
                            )}
                            {branch.behind > 0 && (
                              <Badge variant="outline" className="text-xs text-red-400">
                                -{branch.behind}
                              </Badge>
                            )}
                            <Badge variant={branch.type === "main" ? "default" : "secondary"} className="text-xs">
                              {branch.type}
                            </Badge>
                          </div>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="history" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Clock className="w-5 h-5" />
                      Commit History
                    </CardTitle>
                    <CardDescription className="text-gray-300">Recent commits and changes</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-3">
                      {recentCommits.map((commit) => (
                        <div key={commit.hash} className="flex items-start gap-3 p-3 bg-slate-700/20 rounded-lg">
                          <div className="w-2 h-2 bg-green-500 rounded-full mt-2"></div>
                          <div className="flex-1">
                            <div className="text-white font-medium">{commit.message}</div>
                            <div className="flex items-center gap-4 mt-1 text-sm text-gray-400">
                              <span>{commit.hash}</span>
                              <span>{commit.author}</span>
                              <span>{commit.time}</span>
                              <span>{commit.files} files</span>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="merge" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <GitMerge className="w-5 h-5" />
                      Merge Assistant
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Intelligent merge conflict resolution and branch merging
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {/* Merge Options */}
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <h4 className="text-white font-medium mb-2">Ready to Merge</h4>
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <span className="text-gray-300 text-sm">feature/user-auth → develop</span>
                            <CheckCircle className="w-4 h-4 text-green-500" />
                          </div>
                          <Button size="sm" className="w-full">
                            <GitMerge className="w-4 h-4 mr-1" />
                            Merge
                          </Button>
                        </div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <h4 className="text-white font-medium mb-2">Conflicts Detected</h4>
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <span className="text-gray-300 text-sm">hotfix/security → main</span>
                            <AlertCircle className="w-4 h-4 text-yellow-500" />
                          </div>
                          <Button size="sm" variant="outline" className="w-full border-slate-600">
                            <Brain className="w-4 h-4 mr-1" />
                            Resolve
                          </Button>
                        </div>
                      </div>
                    </div>

                    {/* Merge Strategy */}
                    <div className="p-4 bg-slate-700/30 rounded-lg">
                      <h4 className="text-white font-medium mb-3">AI Merge Strategy</h4>
                      <div className="space-y-2 text-sm text-gray-300">
                        <div>• Analyzing code changes and potential conflicts</div>
                        <div>• Suggesting optimal merge strategy (merge, rebase, squash)</div>
                        <div>• Auto-resolving simple conflicts</div>
                        <div>• Generating merge commit messages</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>
    </div>
  )
}
