"use client"

import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  GitBranch,
  GitCommit,
  GitMerge,
  GitPullRequest,
  Terminal,
  FolderGit2,
  Search,
  Plus,
  RefreshCw,
  ArrowLeft,
  Clock,
  User,
  FileCode,
  AlertCircle,
  CheckCircle2,
  Circle,
} from "lucide-react"
import Link from "next/link"
import { TerminalEmulator } from "@/components/terminal/terminal-emulator"
import type { Repository, GitCommit as Commit } from "@/lib/git-pilot/types"
import { cn } from "@/lib/utils"

// Mock data for repositories
const MOCK_REPOS: Repository[] = [
  {
    id: "repo-1",
    name: "mission-control",
    path: "/home/kernelcore/projects/mission-control",
    remote: "git@github.com:kernelcore/mission-control.git",
    branch: "main",
    status: "modified",
    lastActivity: new Date(),
    commits: 156,
    branches: ["main", "feature/agents", "feature/terminal", "fix/memory"],
    tags: ["v1.0.0", "v1.1.0", "v2.0.0-beta"],
    tracked: true,
  },
  {
    id: "repo-2",
    name: "nixos-config",
    path: "/home/kernelcore/.config/nixos",
    remote: "git@github.com:kernelcore/nixos-config.git",
    branch: "main",
    status: "clean",
    lastActivity: new Date(Date.now() - 3600000),
    commits: 89,
    branches: ["main", "experimental"],
    tags: ["stable-2024"],
    tracked: true,
  },
  {
    id: "repo-3",
    name: "ai-toolkit",
    path: "/home/kernelcore/projects/ai-toolkit",
    remote: "git@github.com:kernelcore/ai-toolkit.git",
    branch: "develop",
    status: "staged",
    lastActivity: new Date(Date.now() - 7200000),
    commits: 234,
    branches: ["main", "develop", "feature/embeddings"],
    tags: ["v0.1.0"],
    tracked: true,
  },
  {
    id: "repo-4",
    name: "dotfiles",
    path: "/home/kernelcore/dotfiles",
    branch: "master",
    status: "clean",
    lastActivity: new Date(Date.now() - 86400000),
    commits: 45,
    branches: ["master"],
    tags: [],
    tracked: true,
  },
]

const MOCK_COMMITS: Commit[] = [
  {
    hash: "8f4e2d1a3b5c7d9e0f1a2b3c4d5e6f7a8b9c0d1e",
    shortHash: "8f4e2d1",
    message: "feat: add agent orchestration system",
    author: "kernelcore",
    email: "kernelcore@nixos",
    date: new Date(),
    files: 12,
    additions: 847,
    deletions: 123,
  },
  {
    hash: "7c3b1a9e2d4f6a8b0c2d4e6f8a0b2c4d6e8f0a2b",
    shortHash: "7c3b1a9",
    message: "fix: resolve memory leak in resource monitor",
    author: "kernelcore",
    email: "kernelcore@nixos",
    date: new Date(Date.now() - 3600000),
    files: 3,
    additions: 45,
    deletions: 67,
  },
  {
    hash: "6a2e8f7b4c6d8e0a2c4e6f8a0b2d4f6a8c0e2d4f",
    shortHash: "6a2e8f7",
    message: "refactor: optimize LLM provider connections",
    author: "kernelcore",
    email: "kernelcore@nixos",
    date: new Date(Date.now() - 7200000),
    files: 8,
    additions: 234,
    deletions: 189,
  },
  {
    hash: "5b1d7e6a3c5f8d0b2e4a6c8e0b2d4f6a8c0e2d4f",
    shortHash: "5b1d7e6",
    message: "docs: update README with new features",
    author: "kernelcore",
    email: "kernelcore@nixos",
    date: new Date(Date.now() - 14400000),
    files: 1,
    additions: 89,
    deletions: 12,
  },
  {
    hash: "4c0e6d5b2a4e7c9d1b3e5a7c9d1f3a5b7c9e1d3f",
    shortHash: "4c0e6d5",
    message: "feat: implement prompt foundry templates",
    author: "kernelcore",
    email: "kernelcore@nixos",
    date: new Date(Date.now() - 28800000),
    files: 6,
    additions: 456,
    deletions: 78,
  },
]

const statusIcons = {
  clean: <CheckCircle2 className="w-4 h-4 text-emerald-400" />,
  modified: <Circle className="w-4 h-4 text-amber-400 fill-amber-400" />,
  staged: <Circle className="w-4 h-4 text-blue-400 fill-blue-400" />,
  conflict: <AlertCircle className="w-4 h-4 text-red-400" />,
  detached: <AlertCircle className="w-4 h-4 text-purple-400" />,
  unknown: <Circle className="w-4 h-4 text-slate-400" />,
}

const statusColors = {
  clean: "text-emerald-400",
  modified: "text-amber-400",
  staged: "text-blue-400",
  conflict: "text-red-400",
  detached: "text-purple-400",
  unknown: "text-slate-400",
}

export default function GitPilot() {
  const [selectedRepo, setSelectedRepo] = useState<Repository | null>(MOCK_REPOS[0])
  const [searchQuery, setSearchQuery] = useState("")
  const [showTerminal, setShowTerminal] = useState(true)

  const filteredRepos = MOCK_REPOS.filter(
    (repo) =>
      repo.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      repo.path.toLowerCase().includes(searchQuery.toLowerCase()),
  )

  const formatTimeAgo = (date: Date) => {
    const seconds = Math.floor((Date.now() - date.getTime()) / 1000)
    if (seconds < 60) return "just now"
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
    return `${Math.floor(seconds / 86400)}d ago`
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
                  <GitBranch className="w-5 h-5 text-primary-foreground" />
                </div>
                <div>
                  <h1 className="text-foreground font-bold">Git Pilot</h1>
                  <p className="text-xs text-muted-foreground">Repository Command Center</p>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                className={cn("border-border", showTerminal ? "bg-accent text-foreground" : "text-muted-foreground")}
                onClick={() => setShowTerminal(!showTerminal)}
              >
                <Terminal className="w-4 h-4 mr-2" />
                Terminal
              </Button>
              <Badge variant="outline" className="border-primary/30 text-primary">
                <GitBranch className="w-3 h-3 mr-1" />
                PILOT Agent
              </Badge>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        <div className="grid grid-cols-12 gap-6">
          {/* Repository List */}
          <div className="col-span-3">
            <Card className="bg-secondary/30 border-border">
              <CardHeader className="pb-3">
                <CardTitle className="text-foreground text-lg flex items-center gap-2">
                  <FolderGit2 className="w-5 h-5 text-muted-foreground" />
                  Repositories
                </CardTitle>

                {/* Search */}
                <div className="relative mt-3">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                  <Input
                    placeholder="Search repos..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-9 bg-accent border-border"
                  />
                </div>
              </CardHeader>

              <CardContent>
                <ScrollArea className="h-[calc(100vh-280px)]">
                  <div className="space-y-2 pr-4">
                    {filteredRepos.map((repo) => (
                      <div
                        key={repo.id}
                        className={cn(
                          "p-3 rounded-lg cursor-pointer transition-all",
                          selectedRepo?.id === repo.id
                            ? "bg-primary/20 border border-primary/30"
                            : "bg-secondary/50 border border-transparent hover:bg-secondary/70 hover:border-border",
                        )}
                        onClick={() => setSelectedRepo(repo)}
                      >
                        <div className="flex items-start justify-between mb-2">
                          <div className="flex items-center gap-2">
                            {statusIcons[repo.status]}
                            <span className="text-foreground font-medium text-sm">{repo.name}</span>
                          </div>
                        </div>
                        <p className="text-xs text-muted-foreground mb-2 truncate">{repo.path}</p>
                        <div className="flex items-center justify-between text-xs">
                          <div className="flex items-center gap-1 text-muted-foreground">
                            <GitBranch className="w-3 h-3" />
                            {repo.branch}
                          </div>
                          <span className="text-muted-foreground">{formatTimeAgo(repo.lastActivity)}</span>
                        </div>
                      </div>
                    ))}

                    {/* Add Repo Button */}
                    <Button
                      variant="outline"
                      className="w-full border-dashed border-border text-muted-foreground hover:text-foreground bg-transparent"
                    >
                      <Plus className="w-4 h-4 mr-2" />
                      Track Repository
                    </Button>
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>

          {/* Main Content */}
          <div className="col-span-9 space-y-6">
            {selectedRepo ? (
              <>
                {/* Repo Header */}
                <Card className="bg-secondary/30 border-border">
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-4">
                        <div className="p-3 bg-accent rounded-lg">
                          <FolderGit2 className="w-6 h-6 text-primary" />
                        </div>
                        <div>
                          <h2 className="text-xl font-bold text-foreground">{selectedRepo.name}</h2>
                          <p className="text-sm text-muted-foreground">{selectedRepo.path}</p>
                        </div>
                      </div>

                      <div className="flex items-center gap-3">
                        <div className="text-right">
                          <div className="flex items-center gap-2">
                            {statusIcons[selectedRepo.status]}
                            <span className={cn("text-sm font-medium capitalize", statusColors[selectedRepo.status])}>
                              {selectedRepo.status}
                            </span>
                          </div>
                          <span className="text-xs text-muted-foreground">
                            Last activity: {formatTimeAgo(selectedRepo.lastActivity)}
                          </span>
                        </div>
                        <Button variant="outline" size="sm" className="border-border bg-transparent">
                          <RefreshCw className="w-4 h-4 mr-2" />
                          Fetch
                        </Button>
                      </div>
                    </div>

                    {/* Quick Stats */}
                    <div className="grid grid-cols-4 gap-4 mt-4">
                      <div className="p-3 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 text-muted-foreground mb-1">
                          <GitCommit className="w-4 h-4" />
                          <span className="text-xs">Commits</span>
                        </div>
                        <span className="text-lg font-bold text-foreground">{selectedRepo.commits}</span>
                      </div>
                      <div className="p-3 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 text-muted-foreground mb-1">
                          <GitBranch className="w-4 h-4" />
                          <span className="text-xs">Branches</span>
                        </div>
                        <span className="text-lg font-bold text-foreground">{selectedRepo.branches.length}</span>
                      </div>
                      <div className="p-3 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 text-muted-foreground mb-1">
                          <GitMerge className="w-4 h-4" />
                          <span className="text-xs">Tags</span>
                        </div>
                        <span className="text-lg font-bold text-foreground">{selectedRepo.tags.length}</span>
                      </div>
                      <div className="p-3 bg-accent/50 rounded-lg">
                        <div className="flex items-center gap-2 text-muted-foreground mb-1">
                          <GitPullRequest className="w-4 h-4" />
                          <span className="text-xs">Current</span>
                        </div>
                        <span className="text-lg font-bold text-foreground">{selectedRepo.branch}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* Tabs */}
                <Tabs defaultValue="commits" className="w-full">
                  <TabsList className="grid w-full grid-cols-4 bg-secondary/50 border border-border">
                    <TabsTrigger value="commits" className="data-[state=active]:bg-accent">
                      <GitCommit className="w-4 h-4 mr-2" />
                      Commits
                    </TabsTrigger>
                    <TabsTrigger value="branches" className="data-[state=active]:bg-accent">
                      <GitBranch className="w-4 h-4 mr-2" />
                      Branches
                    </TabsTrigger>
                    <TabsTrigger value="changes" className="data-[state=active]:bg-accent">
                      <FileCode className="w-4 h-4 mr-2" />
                      Changes
                    </TabsTrigger>
                    <TabsTrigger value="actions" className="data-[state=active]:bg-accent">
                      <GitMerge className="w-4 h-4 mr-2" />
                      Actions
                    </TabsTrigger>
                  </TabsList>

                  <TabsContent value="commits" className="mt-4">
                    <Card className="bg-secondary/30 border-border">
                      <CardHeader className="pb-3">
                        <CardTitle className="text-foreground text-lg">Recent Commits</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-3">
                          {MOCK_COMMITS.map((commit, idx) => (
                            <div
                              key={commit.hash}
                              className="p-3 bg-accent/30 rounded-lg hover:bg-accent/50 transition-colors"
                            >
                              <div className="flex items-start justify-between">
                                <div className="flex-1">
                                  <div className="flex items-center gap-2 mb-1">
                                    <code className="text-xs text-primary bg-primary/10 px-2 py-0.5 rounded">
                                      {commit.shortHash}
                                    </code>
                                    {idx === 0 && (
                                      <Badge variant="outline" className="text-xs border-primary/30 text-primary">
                                        HEAD
                                      </Badge>
                                    )}
                                  </div>
                                  <p className="text-foreground text-sm mb-2">{commit.message}</p>
                                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                                    <span className="flex items-center gap-1">
                                      <User className="w-3 h-3" />
                                      {commit.author}
                                    </span>
                                    <span className="flex items-center gap-1">
                                      <Clock className="w-3 h-3" />
                                      {formatTimeAgo(commit.date)}
                                    </span>
                                    <span className="flex items-center gap-1">
                                      <FileCode className="w-3 h-3" />
                                      {commit.files} files
                                    </span>
                                    <span className="text-primary">+{commit.additions}</span>
                                    <span className="text-destructive">-{commit.deletions}</span>
                                  </div>
                                </div>
                              </div>
                            </div>
                          ))}
                        </div>
                      </CardContent>
                    </Card>
                  </TabsContent>

                  <TabsContent value="branches" className="mt-4">
                    <Card className="bg-secondary/30 border-border">
                      <CardHeader className="pb-3">
                        <CardTitle className="text-foreground text-lg">Branches</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-2">
                          {selectedRepo.branches.map((branch) => (
                            <div
                              key={branch}
                              className={cn(
                                "p-3 rounded-lg flex items-center justify-between",
                                branch === selectedRepo.branch
                                  ? "bg-primary/20 border border-primary/30"
                                  : "bg-accent/30",
                              )}
                            >
                              <div className="flex items-center gap-2">
                                <GitBranch className="w-4 h-4 text-muted-foreground" />
                                <span className="text-foreground">{branch}</span>
                                {branch === selectedRepo.branch && (
                                  <Badge variant="outline" className="text-xs border-primary/30 text-primary">
                                    current
                                  </Badge>
                                )}
                              </div>
                              <Button variant="ghost" size="sm" className="text-muted-foreground hover:text-foreground">
                                Checkout
                              </Button>
                            </div>
                          ))}
                        </div>
                      </CardContent>
                    </Card>
                  </TabsContent>

                  <TabsContent value="changes" className="mt-4">
                    <Card className="bg-secondary/30 border-border">
                      <CardHeader className="pb-3">
                        <CardTitle className="text-foreground text-lg">Working Tree Changes</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-2">
                          <div className="p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg flex items-center justify-between">
                            <div className="flex items-center gap-2">
                              <Circle className="w-3 h-3 text-amber-400 fill-amber-400" />
                              <span className="text-foreground text-sm">src/components/Header.tsx</span>
                            </div>
                            <Badge variant="outline" className="text-xs border-amber-500/30 text-amber-400">
                              modified
                            </Badge>
                          </div>
                          <div className="p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg flex items-center justify-between">
                            <div className="flex items-center gap-2">
                              <Circle className="w-3 h-3 text-amber-400 fill-amber-400" />
                              <span className="text-foreground text-sm">src/lib/utils.ts</span>
                            </div>
                            <Badge variant="outline" className="text-xs border-amber-500/30 text-amber-400">
                              modified
                            </Badge>
                          </div>
                          <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-lg flex items-center justify-between">
                            <div className="flex items-center gap-2">
                              <Plus className="w-3 h-3 text-emerald-400" />
                              <span className="text-foreground text-sm">src/components/NewFeature.tsx</span>
                            </div>
                            <Badge variant="outline" className="text-xs border-emerald-500/30 text-emerald-400">
                              untracked
                            </Badge>
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  </TabsContent>

                  <TabsContent value="actions" className="mt-4">
                    <Card className="bg-secondary/30 border-border">
                      <CardHeader className="pb-3">
                        <CardTitle className="text-foreground text-lg">Quick Actions</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="grid grid-cols-3 gap-3">
                          <Button
                            variant="outline"
                            className="h-auto p-4 flex flex-col items-center gap-2 border-border hover:bg-accent bg-transparent"
                          >
                            <GitCommit className="w-5 h-5 text-primary" />
                            <span>Commit</span>
                          </Button>
                          <Button
                            variant="outline"
                            className="h-auto p-4 flex flex-col items-center gap-2 border-border hover:bg-accent bg-transparent"
                          >
                            <GitBranch className="w-5 h-5 text-emerald-400" />
                            <span>New Branch</span>
                          </Button>
                          <Button
                            variant="outline"
                            className="h-auto p-4 flex flex-col items-center gap-2 border-border hover:bg-accent bg-transparent"
                          >
                            <GitMerge className="w-5 h-5 text-purple-400" />
                            <span>Merge</span>
                          </Button>
                          <Button
                            variant="outline"
                            className="h-auto p-4 flex flex-col items-center gap-2 border-border hover:bg-accent bg-transparent"
                          >
                            <RefreshCw className="w-5 h-5 text-blue-400" />
                            <span>Pull</span>
                          </Button>
                          <Button
                            variant="outline"
                            className="h-auto p-4 flex flex-col items-center gap-2 border-border hover:bg-accent bg-transparent"
                          >
                            <GitPullRequest className="w-5 h-5 text-amber-400" />
                            <span>Push</span>
                          </Button>
                          <Button
                            variant="outline"
                            className="h-auto p-4 flex flex-col items-center gap-2 border-border hover:bg-accent bg-transparent"
                          >
                            <FileCode className="w-5 h-5 text-cyan-400" />
                            <span>Stash</span>
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  </TabsContent>
                </Tabs>

                {/* Terminal */}
                {showTerminal && (
                  <Card className="bg-secondary/30 border-border">
                    <CardHeader className="pb-2">
                      <CardTitle className="text-foreground text-lg flex items-center gap-2">
                        <Terminal className="w-5 h-5 text-primary" />
                        Terminal
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <TerminalEmulator initialCwd={selectedRepo.path} />
                    </CardContent>
                  </Card>
                )}
              </>
            ) : (
              <Card className="bg-secondary/30 border-border">
                <CardContent className="p-12 text-center">
                  <FolderGit2 className="w-16 h-16 text-muted-foreground mx-auto mb-4 opacity-50" />
                  <p className="text-foreground font-medium mb-2">Select a repository</p>
                  <p className="text-muted-foreground text-sm">Choose from the list on the left to get started</p>
                </CardContent>
              </Card>
            )}
          </div>
        </div>
      </main>
    </div>
  )
}
