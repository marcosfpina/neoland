// src/mistral-facade.ts
import {
  generateId,
  loadApiKey,
  withoutTrailingSlash
} from "@ai-sdk/provider-utils";

// src/mistral-chat-language-model.ts
import {
  UnsupportedFunctionalityError as UnsupportedFunctionalityError2
} from "@ai-sdk/provider";
import {
  createEventSourceResponseHandler,
  createJsonResponseHandler,
  postJsonToApi
} from "@ai-sdk/provider-utils";
import { z as z2 } from "zod";

// src/convert-to-mistral-chat-messages.ts
import {
  UnsupportedFunctionalityError
} from "@ai-sdk/provider";
function convertToMistralChatMessages(prompt) {
  const messages = [];
  for (const { role, content } of prompt) {
    switch (role) {
      case "system": {
        messages.push({ role: "system", content });
        break;
      }
      case "user": {
        messages.push({
          role: "user",
          content: content.map((part) => {
            switch (part.type) {
              case "text": {
                return part.text;
              }
              case "image": {
                throw new UnsupportedFunctionalityError({
                  functionality: "image-part"
                });
              }
            }
          }).join("")
        });
        break;
      }
      case "assistant": {
        let text = "";
        const toolCalls = [];
        for (const part of content) {
          switch (part.type) {
            case "text": {
              text += part.text;
              break;
            }
            case "tool-call": {
              toolCalls.push({
                id: part.toolCallId,
                type: "function",
                function: {
                  name: part.toolName,
                  arguments: JSON.stringify(part.args)
                }
              });
              break;
            }
            default: {
              const _exhaustiveCheck = part;
              throw new Error(`Unsupported part: ${_exhaustiveCheck}`);
            }
          }
        }
        messages.push({
          role: "assistant",
          content: text,
          tool_calls: toolCalls.length > 0 ? toolCalls.map(({ function: { name, arguments: args } }) => ({
            id: "null",
            type: "function",
            function: { name, arguments: args }
          })) : void 0
        });
        break;
      }
      case "tool": {
        for (const toolResponse of content) {
          messages.push({
            role: "tool",
            name: toolResponse.toolName,
            content: JSON.stringify(toolResponse.result)
          });
        }
        break;
      }
      default: {
        const _exhaustiveCheck = role;
        throw new Error(`Unsupported role: ${_exhaustiveCheck}`);
      }
    }
  }
  return messages;
}

// src/map-mistral-finish-reason.ts
function mapMistralFinishReason(finishReason) {
  switch (finishReason) {
    case "stop":
      return "stop";
    case "length":
    case "model_length":
      return "length";
    case "tool_calls":
      return "tool-calls";
    default:
      return "other";
  }
}

// src/mistral-error.ts
import { createJsonErrorResponseHandler } from "@ai-sdk/provider-utils";
import { z } from "zod";
var mistralErrorDataSchema = z.object({
  object: z.literal("error"),
  message: z.string(),
  type: z.string(),
  param: z.string().nullable(),
  code: z.string().nullable()
});
var mistralFailedResponseHandler = createJsonErrorResponseHandler({
  errorSchema: mistralErrorDataSchema,
  errorToMessage: (data) => data.message
});

// src/mistral-chat-language-model.ts
var MistralChatLanguageModel = class {
  constructor(modelId, settings, config) {
    this.specificationVersion = "v1";
    this.defaultObjectGenerationMode = "json";
    this.modelId = modelId;
    this.settings = settings;
    this.config = config;
  }
  get provider() {
    return this.config.provider;
  }
  getArgs({
    mode,
    prompt,
    maxTokens,
    temperature,
    topP,
    frequencyPenalty,
    presencePenalty,
    seed
  }) {
    var _a;
    const type = mode.type;
    const warnings = [];
    if (frequencyPenalty != null) {
      warnings.push({
        type: "unsupported-setting",
        setting: "frequencyPenalty"
      });
    }
    if (presencePenalty != null) {
      warnings.push({
        type: "unsupported-setting",
        setting: "presencePenalty"
      });
    }
    const baseArgs = {
      // model id:
      model: this.modelId,
      // model specific settings:
      safe_prompt: this.settings.safePrompt,
      // standardized settings:
      max_tokens: maxTokens,
      temperature,
      top_p: topP,
      random_seed: seed,
      // messages:
      messages: convertToMistralChatMessages(prompt)
    };
    switch (type) {
      case "regular": {
        const tools = ((_a = mode.tools) == null ? void 0 : _a.length) ? mode.tools : void 0;
        return {
          args: {
            ...baseArgs,
            tools: tools == null ? void 0 : tools.map((tool) => ({
              type: "function",
              function: {
                name: tool.name,
                description: tool.description,
                parameters: tool.parameters
              }
            }))
          },
          warnings
        };
      }
      case "object-json": {
        return {
          args: {
            ...baseArgs,
            response_format: { type: "json_object" }
          },
          warnings
        };
      }
      case "object-tool": {
        return {
          args: {
            ...baseArgs,
            tool_choice: "any",
            tools: [{ type: "function", function: mode.tool }]
          },
          warnings
        };
      }
      case "object-grammar": {
        throw new UnsupportedFunctionalityError2({
          functionality: "object-grammar mode"
        });
      }
      default: {
        const _exhaustiveCheck = type;
        throw new Error(`Unsupported type: ${_exhaustiveCheck}`);
      }
    }
  }
  async doGenerate(options) {
    var _a, _b;
    const { args, warnings } = this.getArgs(options);
    const { responseHeaders, value: response } = await postJsonToApi({
      url: `${this.config.baseURL}/chat/completions`,
      headers: this.config.headers(),
      body: args,
      failedResponseHandler: mistralFailedResponseHandler,
      successfulResponseHandler: createJsonResponseHandler(
        mistralChatResponseSchema
      ),
      abortSignal: options.abortSignal
    });
    const { messages: rawPrompt, ...rawSettings } = args;
    const choice = response.choices[0];
    return {
      text: (_a = choice.message.content) != null ? _a : void 0,
      toolCalls: (_b = choice.message.tool_calls) == null ? void 0 : _b.map((toolCall) => ({
        toolCallType: "function",
        toolCallId: this.config.generateId(),
        toolName: toolCall.function.name,
        args: toolCall.function.arguments
      })),
      finishReason: mapMistralFinishReason(choice.finish_reason),
      usage: {
        promptTokens: response.usage.prompt_tokens,
        completionTokens: response.usage.completion_tokens
      },
      rawCall: { rawPrompt, rawSettings },
      rawResponse: { headers: responseHeaders },
      warnings
    };
  }
  async doStream(options) {
    const { args, warnings } = this.getArgs(options);
    const { responseHeaders, value: response } = await postJsonToApi({
      url: `${this.config.baseURL}/chat/completions`,
      headers: this.config.headers(),
      body: {
        ...args,
        stream: true
      },
      failedResponseHandler: mistralFailedResponseHandler,
      successfulResponseHandler: createEventSourceResponseHandler(
        mistralChatChunkSchema
      ),
      abortSignal: options.abortSignal
    });
    const { messages: rawPrompt, ...rawSettings } = args;
    let finishReason = "other";
    let usage = {
      promptTokens: Number.NaN,
      completionTokens: Number.NaN
    };
    const generateId3 = this.config.generateId;
    return {
      stream: response.pipeThrough(
        new TransformStream({
          transform(chunk, controller) {
            if (!chunk.success) {
              controller.enqueue({ type: "error", error: chunk.error });
              return;
            }
            const value = chunk.value;
            if (value.usage != null) {
              usage = {
                promptTokens: value.usage.prompt_tokens,
                completionTokens: value.usage.completion_tokens
              };
            }
            const choice = value.choices[0];
            if ((choice == null ? void 0 : choice.finish_reason) != null) {
              finishReason = mapMistralFinishReason(choice.finish_reason);
            }
            if ((choice == null ? void 0 : choice.delta) == null) {
              return;
            }
            const delta = choice.delta;
            if (delta.content != null) {
              controller.enqueue({
                type: "text-delta",
                textDelta: delta.content
              });
            }
            if (delta.tool_calls != null) {
              for (const toolCall of delta.tool_calls) {
                const toolCallId = generateId3();
                controller.enqueue({
                  type: "tool-call-delta",
                  toolCallType: "function",
                  toolCallId,
                  toolName: toolCall.function.name,
                  argsTextDelta: toolCall.function.arguments
                });
                controller.enqueue({
                  type: "tool-call",
                  toolCallType: "function",
                  toolCallId,
                  toolName: toolCall.function.name,
                  args: toolCall.function.arguments
                });
              }
            }
          },
          flush(controller) {
            controller.enqueue({ type: "finish", finishReason, usage });
          }
        })
      ),
      rawCall: { rawPrompt, rawSettings },
      rawResponse: { headers: responseHeaders },
      warnings
    };
  }
};
var mistralChatResponseSchema = z2.object({
  choices: z2.array(
    z2.object({
      message: z2.object({
        role: z2.literal("assistant"),
        content: z2.string().nullable(),
        tool_calls: z2.array(
          z2.object({
            function: z2.object({
              name: z2.string(),
              arguments: z2.string()
            })
          })
        ).optional().nullable()
      }),
      index: z2.number(),
      finish_reason: z2.string().optional().nullable()
    })
  ),
  object: z2.literal("chat.completion"),
  usage: z2.object({
    prompt_tokens: z2.number(),
    completion_tokens: z2.number()
  })
});
var mistralChatChunkSchema = z2.object({
  object: z2.literal("chat.completion.chunk"),
  choices: z2.array(
    z2.object({
      delta: z2.object({
        role: z2.enum(["assistant"]).optional(),
        content: z2.string().nullable().optional(),
        tool_calls: z2.array(
          z2.object({
            function: z2.object({ name: z2.string(), arguments: z2.string() })
          })
        ).optional().nullable()
      }),
      finish_reason: z2.string().nullable().optional(),
      index: z2.number()
    })
  ),
  usage: z2.object({
    prompt_tokens: z2.number(),
    completion_tokens: z2.number()
  }).optional().nullable()
});

// src/mistral-facade.ts
var Mistral = class {
  /**
   * Creates a new Mistral provider instance.
   */
  constructor(options = {}) {
    var _a, _b, _c;
    this.baseURL = (_b = withoutTrailingSlash((_a = options.baseURL) != null ? _a : options.baseUrl)) != null ? _b : "https://api.mistral.ai/v1";
    this.apiKey = options.apiKey;
    this.headers = options.headers;
    this.generateId = (_c = options.generateId) != null ? _c : generateId;
  }
  get baseConfig() {
    return {
      baseURL: this.baseURL,
      headers: () => ({
        Authorization: `Bearer ${loadApiKey({
          apiKey: this.apiKey,
          environmentVariableName: "MISTRAL_API_KEY",
          description: "Mistral"
        })}`,
        ...this.headers
      })
    };
  }
  chat(modelId, settings = {}) {
    return new MistralChatLanguageModel(modelId, settings, {
      provider: "mistral.chat",
      ...this.baseConfig,
      generateId: this.generateId
    });
  }
};

// src/mistral-provider.ts
import {
  generateId as generateId2,
  loadApiKey as loadApiKey2,
  withoutTrailingSlash as withoutTrailingSlash2
} from "@ai-sdk/provider-utils";
function createMistral(options = {}) {
  const createModel = (modelId, settings = {}) => {
    var _a, _b, _c;
    return new MistralChatLanguageModel(modelId, settings, {
      provider: "mistral.chat",
      baseURL: (_b = withoutTrailingSlash2((_a = options.baseURL) != null ? _a : options.baseUrl)) != null ? _b : "https://api.mistral.ai/v1",
      headers: () => ({
        Authorization: `Bearer ${loadApiKey2({
          apiKey: options.apiKey,
          environmentVariableName: "MISTRAL_API_KEY",
          description: "Mistral"
        })}`,
        ...options.headers
      }),
      generateId: (_c = options.generateId) != null ? _c : generateId2
    });
  };
  const provider = function(modelId, settings) {
    if (new.target) {
      throw new Error(
        "The Mistral model function cannot be called with the new keyword."
      );
    }
    return createModel(modelId, settings);
  };
  provider.chat = createModel;
  return provider;
}
var mistral = createMistral();
export {
  Mistral,
  createMistral,
  mistral
};
//# sourceMappingURL=index.mjs.map