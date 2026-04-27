"use client"

import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts"

import type { JuniorTimelinePoint } from "@/lib/neoland/types"

export function AgentHistoryChart({ data }: { data: JuniorTimelinePoint[] }) {
  if (!data.length) {
    return (
      <div className="flex h-72 items-center justify-center rounded-[1.4rem] border border-dashed border-border/70 bg-card/60 text-sm text-muted-foreground">
        No ADR history yet. Confidence will start plotting here once real checkpoints exist.
      </div>
    )
  }

  return (
    <div className="h-72 w-full">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 12, right: 12, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="rgba(94,104,117,0.22)" vertical={false} />
          <XAxis
            dataKey="adr_id"
            tick={{ fill: "rgba(148,163,184,0.8)", fontSize: 11 }}
            axisLine={false}
            tickLine={false}
          />
          <YAxis
            domain={[0, 1]}
            tick={{ fill: "rgba(148,163,184,0.8)", fontSize: 11 }}
            axisLine={false}
            tickLine={false}
          />
          <Tooltip
            contentStyle={{
              background: "rgba(10,12,14,0.94)",
              border: "1px solid rgba(94,104,117,0.35)",
              borderRadius: "16px",
              color: "white",
            }}
          />
          <Line
            type="monotone"
            dataKey="confidence"
            stroke="hsl(191, 82%, 56%)"
            strokeWidth={2.5}
            dot={{ fill: "hsl(191, 82%, 56%)", r: 4 }}
            activeDot={{ r: 6 }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
