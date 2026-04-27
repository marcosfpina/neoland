"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { Search } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"

export function SessionLookupForm({ initialValue = "" }: { initialValue?: string }) {
  const router = useRouter()
  const [value, setValue] = useState(initialValue)

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const trimmed = value.trim()

    if (!trimmed) {
      return
    }

    router.push(`/sessions/${trimmed}`)
  }

  return (
    <form className="flex flex-col gap-3 sm:flex-row" onSubmit={handleSubmit}>
      <Input
        value={value}
        onChange={(event) => setValue(event.target.value)}
        placeholder="Paste a real session UUID"
        className="h-11 rounded-[1.1rem] border-border/70 bg-background/40"
      />
      <Button type="submit" size="lg">
        <Search className="size-4" />
        Inspect session
      </Button>
    </form>
  )
}
