// LLM Provider configurations and utilities

export interface LLMProvider {
  id: string
  name: string
  baseUrl: string
  models: string[]
  defaultModel: string
  apiKeyEnv: string
  headers: (apiKey: string) => Record<string, string>
  formatRequest: (prompt: string, options: LLMOptions) => unknown
  parseResponse: (response: unknown) => LLMResponse
  pricing: {
    input: number // per 1K tokens
    output: number // per 1K tokens
  }
  capabilities: string[]
  maxTokens: number
  supportsStreaming: boolean
}

export interface LLMOptions {
  temperature?: number
  maxTokens?: number
  topP?: number
  frequencyPenalty?: number
  presencePenalty?: number
  stop?: string[]
  stream?: boolean
}

export interface LLMResponse {
  text: string
  tokens: {
    input: number
    output: number
    total: number
  }
  model: string
  finishReason: string
  usage?: unknown
}

export const LLM_PROVIDERS: Record<string, LLMProvider> = {
  openai: {
    id: "openai",
    name: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    models: ["gpt-4o", "gpt-4o-mini", "gpt-3.5-turbo"],
    defaultModel: "gpt-4o",
    apiKeyEnv: "OPENAI_API_KEY",
    headers: (apiKey: string) => ({
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "gpt-4o",
      messages: [{ role: "user", content: prompt }],
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 1000,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.choices[0].message.content,
      tokens: {
        input: response.usage.prompt_tokens,
        output: response.usage.completion_tokens,
        total: response.usage.total_tokens,
      },
      model: response.model,
      finishReason: response.choices[0].finish_reason,
      usage: response.usage,
    }),
    pricing: { input: 0.03, output: 0.06 },
    capabilities: ["text", "code", "reasoning", "function-calling"],
    maxTokens: 128000,
    supportsStreaming: true,
  },

  anthropic: {
    id: "anthropic",
    name: "Anthropic",
    baseUrl: "https://api.anthropic.com/v1",
    models: ["claude-3-5-sonnet-20241022", "claude-3-haiku-20240307"],
    defaultModel: "claude-3-5-sonnet-20241022",
    apiKeyEnv: "ANTHROPIC_API_KEY",
    headers: (apiKey: string) => ({
      "x-api-key": apiKey,
      "Content-Type": "application/json",
      "anthropic-version": "2023-06-01",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "claude-3-5-sonnet-20241022",
      messages: [{ role: "user", content: prompt }],
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 1000,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.content[0].text,
      tokens: {
        input: response.usage.input_tokens,
        output: response.usage.output_tokens,
        total: response.usage.input_tokens + response.usage.output_tokens,
      },
      model: response.model,
      finishReason: response.stop_reason,
      usage: response.usage,
    }),
    pricing: { input: 0.015, output: 0.075 },
    capabilities: ["text", "analysis", "reasoning", "safety"],
    maxTokens: 200000,
    supportsStreaming: true,
  },

  google: {
    id: "google",
    name: "Google",
    baseUrl: "https://generativelanguage.googleapis.com/v1beta",
    models: ["gemini-1.5-pro", "gemini-1.5-flash"],
    defaultModel: "gemini-1.5-pro",
    apiKeyEnv: "GOOGLE_API_KEY",
    headers: (_apiKey: string) => ({
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      contents: [{ parts: [{ text: prompt }] }],
      generationConfig: {
        temperature: options.temperature ?? 0.7,
        maxOutputTokens: options.maxTokens ?? 1000,
      },
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.candidates[0].content.parts[0].text,
      tokens: {
        input: response.usageMetadata.promptTokenCount,
        output: response.usageMetadata.candidatesTokenCount,
        total: response.usageMetadata.totalTokenCount,
      },
      model: "gemini-1.5-pro",
      finishReason: response.candidates[0].finishReason,
      usage: response.usageMetadata,
    }),
    pricing: { input: 0.0075, output: 0.03 },
    capabilities: ["text", "multimodal", "code", "math"],
    maxTokens: 2000000,
    supportsStreaming: true,
  },

  xai: {
    id: "xai",
    name: "xAI",
    baseUrl: "https://api.x.ai/v1",
    models: ["grok-3", "grok-2"],
    defaultModel: "grok-3",
    apiKeyEnv: "XAI_API_KEY",
    headers: (apiKey: string) => ({
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "grok-3",
      messages: [{ role: "user", content: prompt }],
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 1000,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.choices[0].message.content,
      tokens: {
        input: response.usage.prompt_tokens,
        output: response.usage.completion_tokens,
        total: response.usage.total_tokens,
      },
      model: response.model,
      finishReason: response.choices[0].finish_reason,
      usage: response.usage,
    }),
    pricing: { input: 0.02, output: 0.04 },
    capabilities: ["text", "real-time", "humor", "current-events"],
    maxTokens: 131072,
    supportsStreaming: true,
  },

  groq: {
    id: "groq",
    name: "Groq",
    baseUrl: "https://api.groq.com/openai/v1",
    models: ["llama-3.1-70b-versatile", "llama-3.1-8b-instant"],
    defaultModel: "llama-3.1-70b-versatile",
    apiKeyEnv: "GROQ_API_KEY",
    headers: (apiKey: string) => ({
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "llama-3.1-70b-versatile",
      messages: [{ role: "user", content: prompt }],
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 1000,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.choices[0].message.content,
      tokens: {
        input: response.usage.prompt_tokens,
        output: response.usage.completion_tokens,
        total: response.usage.total_tokens,
      },
      model: response.model,
      finishReason: response.choices[0].finish_reason,
      usage: response.usage,
    }),
    pricing: { input: 0.0008, output: 0.0008 },
    capabilities: ["text", "speed", "efficiency", "cost"],
    maxTokens: 131072,
    supportsStreaming: true,
  },

  cohere: {
    id: "cohere",
    name: "Cohere",
    baseUrl: "https://api.cohere.ai/v1",
    models: ["command-r-plus", "command-r"],
    defaultModel: "command-r-plus",
    apiKeyEnv: "COHERE_API_KEY",
    headers: (apiKey: string) => ({
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "command-r-plus",
      message: prompt,
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 1000,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.text,
      tokens: {
        input: response.meta.tokens.input_tokens,
        output: response.meta.tokens.output_tokens,
        total: response.meta.tokens.input_tokens + response.meta.tokens.output_tokens,
      },
      model: "command-r-plus",
      finishReason: response.finish_reason,
      usage: response.meta,
    }),
    pricing: { input: 0.015, output: 0.075 },
    capabilities: ["text", "enterprise", "rag", "embeddings"],
    maxTokens: 128000,
    supportsStreaming: true,
  },

  mistral: {
    id: "mistral",
    name: "Mistral AI",
    baseUrl: "https://api.mistral.ai/v1",
    models: ["mistral-large-latest", "mistral-medium-latest"],
    defaultModel: "mistral-large-latest",
    apiKeyEnv: "MISTRAL_API_KEY",
    headers: (apiKey: string) => ({
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "mistral-large-latest",
      messages: [{ role: "user", content: prompt }],
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 1000,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.choices[0].message.content,
      tokens: {
        input: response.usage.prompt_tokens,
        output: response.usage.completion_tokens,
        total: response.usage.total_tokens,
      },
      model: response.model,
      finishReason: response.choices[0].finish_reason,
      usage: response.usage,
    }),
    pricing: { input: 0.012, output: 0.036 },
    capabilities: ["text", "european", "multilingual", "code"],
    maxTokens: 128000,
    supportsStreaming: true,
  },

  deepinfra: {
    id: "deepinfra",
    name: "DeepInfra",
    baseUrl: "https://api.deepinfra.com/v1/openai",
    models: ["meta-llama/Meta-Llama-3.1-70B-Instruct", "microsoft/WizardLM-2-8x22B"],
    defaultModel: "meta-llama/Meta-Llama-3.1-70B-Instruct",
    apiKeyEnv: "DEEPINFRA_API_KEY",
    headers: (apiKey: string) => ({
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "meta-llama/Meta-Llama-3.1-70B-Instruct",
      messages: [{ role: "user", content: prompt }],
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 1000,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.choices[0].message.content,
      tokens: {
        input: response.usage.prompt_tokens,
        output: response.usage.completion_tokens,
        total: response.usage.total_tokens,
      },
      model: response.model,
      finishReason: response.choices[0].finish_reason,
      usage: response.usage,
    }),
    pricing: { input: 0.0007, output: 0.0007 },
    capabilities: ["text", "open-source", "cost", "variety"],
    maxTokens: 131072,
    supportsStreaming: true,
  },

  "llama-swap": {
    id: "llama-swap",
    name: "Llama Swap (Local)",
    baseUrl: "http://localhost:8080/v1",
    models: ["llama-swap-default"],
    defaultModel: "llama-swap-default",
    apiKeyEnv: "LLAMA_SWAP_API_KEY",
    headers: (apiKey: string) => ({
      Authorization: `Bearer ${apiKey || "dummy"}`,
      "Content-Type": "application/json",
    }),
    formatRequest: (prompt: string, options: LLMOptions) => ({
      model: "llama-swap-default",
      messages: [{ role: "user", content: prompt }],
      temperature: options.temperature ?? 0.7,
      max_tokens: options.maxTokens ?? 2048,
      stream: options.stream ?? false,
    }),
    parseResponse: (response: unknown): LLMResponse => ({
      text: response.choices[0].message.content,
      tokens: {
        input: response.usage?.prompt_tokens || 0,
        output: response.usage?.completion_tokens || 0,
        total: response.usage?.total_tokens || 0,
      },
      model: response.model || "llama-swap",
      finishReason: response.choices[0].finish_reason,
      usage: response.usage || {},
    }),
    pricing: { input: 0, output: 0 },
    capabilities: ["text", "local", "private", "uncensored"],
    maxTokens: 32768,
    supportsStreaming: true,
  },
}

export class LLMManager {
  private providers: Map<string, LLMProvider> = new Map()

  constructor() {
    Object.values(LLM_PROVIDERS).forEach((provider) => {
      this.providers.set(provider.id, provider)
    })
  }

  getProvider(id: string): LLMProvider | undefined {
    return this.providers.get(id)
  }

  getAllProviders(): LLMProvider[] {
    return Array.from(this.providers.values())
  }

  async generateResponse(providerId: string, prompt: string, options: LLMOptions = {}): Promise<LLMResponse> {
    const provider = this.getProvider(providerId)
    if (!provider) {
      throw new Error(`Provider ${providerId} not found`)
    }

    const apiKey = process.env[provider.apiKeyEnv]
    if (!apiKey) {
      throw new Error(`API key for ${provider.name} not found`)
    }

    const requestBody = provider.formatRequest(prompt, options)
    const headers = provider.headers(apiKey)

    const response = await fetch(`${provider.baseUrl}/chat/completions`, {
      method: "POST",
      headers,
      body: JSON.stringify(requestBody),
    })

    if (!response.ok) {
      throw new Error(`${provider.name} API error: ${response.statusText}`)
    }

    const data = await response.json()
    return provider.parseResponse(data)
  }

  async generateMultipleResponses(
    providerIds: string[],
    prompt: string,
    options: LLMOptions = {},
  ): Promise<Record<string, LLMResponse | Error>> {
    const promises = providerIds.map(async (providerId) => {
      try {
        const response = await this.generateResponse(providerId, prompt, options)
        return { providerId, response }
      } catch (error) {
        return { providerId, error }
      }
    })

    const results = await Promise.allSettled(promises)
    const responses: Record<string, LLMResponse | Error> = {}

    results.forEach((result, index) => {
      const providerId = providerIds[index]
      if (result.status === "fulfilled") {
        const { response, error } = result.value
        responses[providerId] = error || response
      } else {
        responses[providerId] = new Error(result.reason)
      }
    })

    return responses
  }

  calculateCost(providerId: string, tokens: { input: number; output: number }): number {
    const provider = this.getProvider(providerId)
    if (!provider) return 0

    const inputCost = (tokens.input / 1000) * provider.pricing.input
    const outputCost = (tokens.output / 1000) * provider.pricing.output
    return inputCost + outputCost
  }

  compareProviders(responses: Record<string, LLMResponse>, _criteria: string[] = ["speed", "cost", "quality"]): unknown {
    const comparison = {
      fastest: null,
      cheapest: null,
      mostDetailed: null,
      summary: {},
    }

    // Implementation for provider comparison logic
    // This would analyze response times, costs, quality metrics, etc.

    return comparison
  }
}

export const llmManager = new LLMManager()
