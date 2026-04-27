// Prompt Foundry - Types for prompt generation and chaining

export interface PromptTemplate {
  id: string
  name: string
  description: string
  category: PromptCategory
  template: string
  variables: PromptVariable[]
  tags: string[]
  rating: number
  usageCount: number
  createdAt: Date
  updatedAt: Date
  author: string
}

export type PromptCategory =
  | "code-generation"
  | "code-review"
  | "documentation"
  | "debugging"
  | "refactoring"
  | "testing"
  | "explanation"
  | "translation"
  | "creative"
  | "system"
  | "chain"

export interface PromptVariable {
  name: string
  type: "text" | "code" | "select" | "number" | "boolean"
  description: string
  required: boolean
  default?: string
  options?: string[] // for select type
}

export interface PromptChain {
  id: string
  name: string
  description: string
  steps: PromptChainStep[]
  createdAt: Date
  executionCount: number
}

export interface PromptChainStep {
  id: string
  templateId: string
  name: string
  inputMapping: Record<string, string> // maps variable to previous step output or user input
  outputKey: string // key to store result for next steps
  condition?: string // optional condition to execute this step
}

export interface PromptExecution {
  id: string
  templateId?: string
  chainId?: string
  prompt: string
  variables: Record<string, string>
  result?: string
  model: string
  tokensUsed: number
  duration: number
  timestamp: Date
  rating?: number
}

export interface PromptEnhancement {
  type: "clarity" | "specificity" | "context" | "structure" | "constraints"
  original: string
  enhanced: string
  explanation: string
}

export interface SemanticAnalysis {
  intent: string
  entities: string[]
  complexity: "simple" | "moderate" | "complex"
  suggestedModel: string
  estimatedTokens: number
  improvements: string[]
}
