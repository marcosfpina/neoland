import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function slugify(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
}

export function truncateMiddle(value: string, start = 8, end = 6): string {
  if (value.length <= start + end + 1) {
    return value
  }

  return `${value.slice(0, start)}...${value.slice(-end)}`
}

function toDate(value: string | Date) {
  return value instanceof Date ? value : new Date(value)
}

export function formatTimestamp(value: string | Date): string {
  const date = toDate(value)

  if (Number.isNaN(date.getTime())) {
    return typeof value === "string" ? value : value.toISOString()
  }

  return new Intl.DateTimeFormat("en", {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date)
}

export function formatRelativeTime(value: string | Date): string {
  const date = toDate(value)

  if (Number.isNaN(date.getTime())) {
    return "time unavailable"
  }

  const diffMs = date.getTime() - Date.now()
  const diffSeconds = Math.round(diffMs / 1000)
  const absSeconds = Math.abs(diffSeconds)
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" })

  if (absSeconds < 60) {
    return formatter.format(diffSeconds, "second")
  }

  const diffMinutes = Math.round(diffSeconds / 60)
  if (Math.abs(diffMinutes) < 60) {
    return formatter.format(diffMinutes, "minute")
  }

  const diffHours = Math.round(diffMinutes / 60)
  if (Math.abs(diffHours) < 24) {
    return formatter.format(diffHours, "hour")
  }

  const diffDays = Math.round(diffHours / 24)
  if (Math.abs(diffDays) < 30) {
    return formatter.format(diffDays, "day")
  }

  const diffMonths = Math.round(diffDays / 30)
  if (Math.abs(diffMonths) < 12) {
    return formatter.format(diffMonths, "month")
  }

  return formatter.format(Math.round(diffDays / 365), "year")
}
