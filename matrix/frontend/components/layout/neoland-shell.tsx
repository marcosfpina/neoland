"use client"

import type { ReactNode } from "react"

import { NeolandHeader } from "@/components/layout/neoland-header"
import { NeolandSidebar } from "@/components/layout/neoland-sidebar"

export function NeolandShell({ children }: { children: ReactNode }) {
  return (
    <div className="control-shell min-h-screen bg-background text-foreground">
      <NeolandSidebar />
      <div className="lg:pl-72">
        <NeolandHeader />
        <main className="px-4 py-6 sm:px-6 sm:py-8">
          <div className="mx-auto max-w-7xl space-y-8">{children}</div>
        </main>
      </div>
    </div>
  )
}
