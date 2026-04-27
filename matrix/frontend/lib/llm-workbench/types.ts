// LLM Workbench - Types for model arena and comparison

export interface LLMProvider {
  id: string
  name: string
  models: LLMModel[]
  status: "connected" | "disconnected" | "error"
  apiKeyRequired: boolean
  baseUrl?: string
}

export interface LLMModel {
  id: string
  providerId: string
  name: string
  displayName: string
  contextWindow: number
  maxOutput: number
  inputCost: number // per 1M tokens
  outputCost: number // per 1M tokens
  capabilities: ModelCapability[]
  speed: "fast" | "medium" | "slow"
  quality: "standard" | "high" | "premium"
}

export type ModelCapability = "text" | "code" | "vision" | "function-calling" | "json-mode" | "streaming"

export interface ComparisonRequest {
  id: string
  prompt: string
  systemPrompt?: string
  models: string[] // model IDs
  parameters: GenerationParameters
  timestamp: Date
}

export interface GenerationParameters {
  temperature: number
  maxTokens: number
  topP: number
  frequencyPenalty: number
  presencePenalty: number
  stream: boolean
}

export interface ComparisonResponse {
  modelId: string
  modelName: string
  response: string
  tokensUsed: {
    input: number
    output: number
    total: number
  }
  latency: number // ms
  cost: number // USD
  finishReason: "stop" | "length" | "error"
  error?: string
  timestamp: Date
}

export interface ComparisonResult {
  request: ComparisonRequest
  responses: ComparisonResponse[]
  winner?: string // model ID with best response
  ratings: Record<string, ModelRating>
}

export interface ModelRating {
  quality: number // 1-5
  relevance: number // 1-5
  speed: number // 1-5
  value: number // 1-5 (quality/cost ratio)
  overall: number // weighted average
}

export interface PromptHistory {
  id: string
  prompt: string
  systemPrompt?: string
  models: string[]
  timestamp: Date
  favorite: boolean
  tags: string[]
}
