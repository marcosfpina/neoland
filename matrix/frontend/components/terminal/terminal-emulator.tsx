"use client"

import type React from "react"

import { useState, useRef, useEffect, useCallback } from "react"
import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"
import type { TerminalCommand } from "@/lib/git-pilot/types"

interface TerminalEmulatorProps {
  initialCwd?: string
  onCommand?: (cmd: string) => Promise<{ output: string; exitCode: number }>
  className?: string
}

// Simulated command responses for demo
const SIMULATED_COMMANDS: Record<string, (args: string[]) => string> = {
  help: () => `Available commands:
  git status    - Show working tree status
  git log       - Show commit logs
  git branch    - List branches
  git diff      - Show changes
  git add       - Add files to staging
  git commit    - Record changes
  git push      - Update remote refs
  git pull      - Fetch and merge
  git stash     - Stash changes
  ls            - List directory contents
  cd            - Change directory
  pwd           - Print working directory
  clear         - Clear terminal
  help          - Show this help`,

  "git status": () => `On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
        modified:   src/components/Header.tsx
        modified:   src/lib/utils.ts

Untracked files:
  (use "git add <file>..." to include in what will be committed)
        src/components/NewFeature.tsx

no changes added to commit (use "git add" and/or "git commit -a")`,

  "git log": () => `commit 8f4e2d1a (HEAD -> main, origin/main)
Author: Developer <dev@example.com>
Date:   ${new Date().toLocaleDateString()} 14:32:15

    feat: add new dashboard components

commit 7c3b1a9e
Author: Developer <dev@example.com>
Date:   ${new Date(Date.now() - 86400000).toLocaleDateString()} 10:15:42

    fix: resolve memory leak in useEffect

commit 6a2e8f7b
Author: Developer <dev@example.com>
Date:   ${new Date(Date.now() - 172800000).toLocaleDateString()} 16:48:33

    refactor: optimize database queries`,

  "git branch": () => `* main
  feature/auth-system
  feature/dashboard
  fix/memory-leak
  release/v2.0`,

  "git diff": () => `diff --git a/src/lib/utils.ts b/src/lib/utils.ts
index 1234567..abcdefg 100644
--- a/src/lib/utils.ts
+++ b/src/lib/utils.ts
@@ -15,6 +15,10 @@ export function cn(...inputs: ClassValue[]) {
   return twMerge(clsx(inputs))
 }

+export function formatDate(date: Date): string {
+  return date.toLocaleDateString()
+}
+
 export function debounce<T extends (...args: unknown[]) => unknown>(
   func: T,
   wait: number`,

  ls: () => `app/
components/
lib/
public/
scripts/
package.json
tsconfig.json
README.md
.gitignore`,

  pwd: () => `/home/user/projects/mission-control`,

  whoami: () => `kernelcore`,

  neofetch: () => `        _,met$$$$$gg.          kernelcore@nixos 
     ,g$$$$$$$$$$$$$$$P.       --------------- 
   ,g$$P"     """Y$$.".        OS: NixOS 25.05
  ,$$P'              \`$$$.     Host: Custom Build
',$$P       ,ggs.     \`$$b:    Kernel: 6.1.x
\`d$$'     ,$P"'   .    $$$     Uptime: 2 days, 14 hours
 $$P      d$'     ,    $$P     Packages: 847 (nix-system)
 $$:      $$.   -    ,d$$'     Shell: zsh 5.9
 $$;      Y$b._   _,d$P'       Terminal: Mission Control
 Y$$.    \`.\`"Y$$$$P"'          CPU: Intel i7 @ 4.7GHz
 \`$$b      "-.__               GPU: NVIDIA GeForce RTX
  \`Y$$                         Memory: 8.2GB / 16GB
   \`Y$$.
     \`$$b.
       \`Y$$b.`,

  "git stash list": () => `stash@{0}: WIP on main: 8f4e2d1 feat: add new dashboard
stash@{1}: On feature/auth: 3c2a1b0 work in progress`,

  clear: () => "__CLEAR__",
}

export function TerminalEmulator({ initialCwd = "~", onCommand, className }: TerminalEmulatorProps) {
  const [history, setHistory] = useState<TerminalCommand[]>([])
  const [input, setInput] = useState("")
  const [historyIndex, setHistoryIndex] = useState(-1)
  const [cwd, setCwd] = useState(initialCwd)
  const inputRef = useRef<HTMLInputElement>(null)
  const scrollRef = useRef<HTMLDivElement>(null)

  // Auto scroll to bottom
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [history])

  // Focus input on click
  const focusInput = useCallback(() => {
    inputRef.current?.focus()
  }, [])

  // Execute command
  const executeCommand = async (cmd: string) => {
    const trimmedCmd = cmd.trim()
    if (!trimmedCmd) return

    const startTime = Date.now()
    let output = ""
    let exitCode = 0

    // Handle cd command
    if (trimmedCmd.startsWith("cd ")) {
      const newPath = trimmedCmd.slice(3).trim()
      if (newPath === "..") {
        setCwd((prev) => {
          const parts = prev.split("/")
          parts.pop()
          return parts.join("/") || "~"
        })
      } else if (newPath.startsWith("/")) {
        setCwd(newPath)
      } else {
        setCwd((prev) => `${prev}/${newPath}`)
      }
      output = ""
    } else if (onCommand) {
      // Use provided command handler
      const result = await onCommand(trimmedCmd)
      output = result.output
      exitCode = result.exitCode
    } else {
      // Use simulated commands
      const [baseCmd, ...args] = trimmedCmd.split(" ")
      const fullCmd = trimmedCmd.toLowerCase()

      // Check for exact match first
      if (SIMULATED_COMMANDS[fullCmd]) {
        output = SIMULATED_COMMANDS[fullCmd](args)
      } else if (SIMULATED_COMMANDS[baseCmd]) {
        output = SIMULATED_COMMANDS[baseCmd](args)
      } else {
        output = `command not found: ${baseCmd}`
        exitCode = 127
      }
    }

    // Handle clear command
    if (output === "__CLEAR__") {
      setHistory([])
      return
    }

    const command: TerminalCommand = {
      id: `cmd-${Date.now()}`,
      command: trimmedCmd,
      output,
      exitCode,
      timestamp: new Date(),
      duration: Date.now() - startTime,
      cwd,
    }

    setHistory((prev) => [...prev, command])
    setHistoryIndex(-1)
  }

  // Handle key events
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      executeCommand(input)
      setInput("")
    } else if (e.key === "ArrowUp") {
      e.preventDefault()
      const commands = history.map((h) => h.command)
      if (commands.length > 0) {
        const newIndex = historyIndex < commands.length - 1 ? historyIndex + 1 : historyIndex
        setHistoryIndex(newIndex)
        setInput(commands[commands.length - 1 - newIndex] || "")
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault()
      if (historyIndex > 0) {
        const newIndex = historyIndex - 1
        setHistoryIndex(newIndex)
        const commands = history.map((h) => h.command)
        setInput(commands[commands.length - 1 - newIndex] || "")
      } else {
        setHistoryIndex(-1)
        setInput("")
      }
    } else if (e.key === "Tab") {
      e.preventDefault()
      // Simple tab completion
      const commands = ["git", "ls", "cd", "pwd", "clear", "help", "neofetch", "whoami"]
      const match = commands.find((c) => c.startsWith(input.toLowerCase()))
      if (match) setInput(match)
    } else if (e.ctrlKey && e.key === "l") {
      e.preventDefault()
      setHistory([])
    } else if (e.ctrlKey && e.key === "c") {
      setInput("")
    }
  }

  return (
    <div
      className={cn("bg-slate-950 rounded-lg border border-slate-800 font-mono text-sm overflow-hidden", className)}
      onClick={focusInput}
    >
      {/* Terminal Header */}
      <div className="flex items-center gap-2 px-4 py-2 bg-slate-900 border-b border-slate-800">
        <div className="flex gap-1.5">
          <div className="w-3 h-3 rounded-full bg-red-500" />
          <div className="w-3 h-3 rounded-full bg-amber-500" />
          <div className="w-3 h-3 rounded-full bg-emerald-500" />
        </div>
        <span className="text-slate-400 text-xs ml-2">Terminal — {cwd}</span>
      </div>

      {/* Terminal Content */}
      <ScrollArea className="h-[400px]" ref={scrollRef}>
        <div className="p-4 space-y-2">
          {/* Welcome message */}
          {history.length === 0 && (
            <div className="text-slate-500">Welcome to Git Pilot Terminal. Type 'help' for available commands.</div>
          )}

          {/* Command history */}
          {history.map((cmd) => (
            <div key={cmd.id} className="space-y-1">
              {/* Prompt line */}
              <div className="flex items-center gap-2">
                <span className="text-emerald-400">➜</span>
                <span className="text-cyan-400">{cmd.cwd}</span>
                <span className="text-white">{cmd.command}</span>
              </div>
              {/* Output */}
              {cmd.output && (
                <pre className={cn("whitespace-pre-wrap pl-4", cmd.exitCode === 0 ? "text-slate-300" : "text-red-400")}>
                  {cmd.output}
                </pre>
              )}
            </div>
          ))}

          {/* Current input line */}
          <div className="flex items-center gap-2">
            <span className="text-emerald-400">➜</span>
            <span className="text-cyan-400">{cwd}</span>
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              className="flex-1 bg-transparent text-white outline-none"
              autoFocus
              spellCheck={false}
              autoComplete="off"
            />
          </div>
        </div>
      </ScrollArea>
    </div>
  )
}
