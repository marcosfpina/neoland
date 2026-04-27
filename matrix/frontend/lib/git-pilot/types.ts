// Git Pilot - Types for git operations and repository management

export interface Repository {
  id: string
  name: string
  path: string
  remote?: string
  branch: string
  status: RepoStatus
  lastActivity: Date
  commits: number
  branches: string[]
  tags: string[]
  tracked: boolean
}

export type RepoStatus = "clean" | "modified" | "staged" | "conflict" | "detached" | "unknown"

export interface GitBranch {
  name: string
  current: boolean
  remote?: string
  ahead: number
  behind: number
  lastCommit: string
  lastCommitDate: Date
}

export interface GitCommit {
  hash: string
  shortHash: string
  message: string
  author: string
  email: string
  date: Date
  files: number
  additions: number
  deletions: number
}

export interface GitDiff {
  file: string
  status: "added" | "modified" | "deleted" | "renamed"
  additions: number
  deletions: number
  hunks: DiffHunk[]
}

export interface DiffHunk {
  header: string
  lines: DiffLine[]
}

export interface DiffLine {
  type: "context" | "addition" | "deletion"
  content: string
  lineNumber: number
}

export interface GitStash {
  id: string
  message: string
  branch: string
  date: Date
  files: number
}

export interface TerminalCommand {
  id: string
  command: string
  output: string
  exitCode: number
  timestamp: Date
  duration: number
  cwd: string
}

export interface TerminalSession {
  id: string
  name: string
  cwd: string
  history: TerminalCommand[]
  isActive: boolean
}
