# LLM Providers Configuration

NEOLAND supports multiple LLM providers through the SecureLLM Bridge. This document explains how to configure and use each provider.

## Supported Providers

| Provider | Type | Cost | Speed | API Key Required |
|----------|------|------|-------|------------------|
| **LlamaCPP** | Local | FREE ✅ | Fast | No |
| **DeepSeek** | Cloud | Very Cheap | Medium | Yes |
| **Groq** | Cloud | Very Cheap | Very Fast | Yes |
| **Gemini** | Cloud | Cheap | Fast | Yes |
| **OpenAI** | Cloud | Expensive | Medium | Yes |
| **Anthropic** | Cloud | Expensive | Medium | Yes |

---

## 1. LlamaCPP (Local - Recommended for Privacy)

**Advantages**:
- ✅ 100% FREE (runs on your hardware)
- ✅ 100% Private (no data leaves your machine)
- ✅ No API key needed
- ✅ Works offline
- ✅ Fast inference on good hardware

**Requirements**:
- llama.cpp server running locally
- Model downloaded (e.g., Llama 3, Mistral, Phi-3)

**Configuration**:
```bash
# Start llama.cpp server (default port: 8081)
./llama-server -m models/llama-3-8b.gguf --port 8081

# Environment variable (optional - customize port/model)
export LLAMACPP_CONFIG="8081:llama-3-8b"

# NEOLAND will auto-detect LlamaCPP on port 8081
```

**Usage**:
```rust
// Proxy initialization
let proxy = SecureLLMProxy::new(
    "llamacpp",
    secrets_manager.clone(),
    Some("8081:llama-3-8b".to_string()), // port:model-name
).await?;
```

**Cost**: $0.00 (FREE)

---

## 2. DeepSeek (Cloud - Cheapest API)

**Advantages**:
- 💰 Extremely cheap ($0.14/1M prompt, $0.28/1M completion)
- 🚀 Good quality (competitive with GPT-3.5)
- 🌐 No censorship
- 📊 128K context window

**Configuration**:
```bash
# Option 1: Environment Variable
export DEEPSEEK_API_KEY="sk-your-api-key-here"

# Option 2: Vault (Production)
vault kv put secret/llm/deepseek api_key="sk-your-api-key-here"
```

**Usage**:
```rust
let proxy = SecureLLMProxy::new(
    "deepseek",
    secrets_manager.clone(),
    None, // Will load from env/vault
).await?;
```

**Cost Example**:
- 1M tokens in + 1M tokens out = $0.42
- 100K conversation = $0.042 (~4 cents)

---

## 3. Groq (Cloud - Fastest API)

**Advantages**:
- ⚡ EXTREMELY FAST (500+ tokens/sec)
- 💰 Very cheap ($0.05/1M prompt, $0.10/1M completion)
- 🔥 Hardware acceleration (LPUs)
- 🆓 Free tier available

**Configuration**:
```bash
export GROQ_API_KEY="gsk-your-api-key-here"
```

**Usage**:
```rust
let proxy = SecureLLMProxy::new("groq", secrets_manager.clone(), None).await?;
```

**Cost Example**:
- 1M tokens = $0.15
- Perfect for real-time chat applications

---

## 4. Google Gemini (Cloud - Good Balance)

**Advantages**:
- 🎯 Good quality/price ratio
- 🖼️ Multimodal (vision support)
- 📈 2M context window (Gemini Pro 1.5)
- 🆓 Free tier (15 RPM)

**Models**:
- **Gemini Flash**: $0.075/1M prompt (very cheap, fast)
- **Gemini Pro**: $0.50/1M prompt (higher quality)

**Configuration**:
```bash
export GEMINI_API_KEY="AIza-your-api-key-here"
```

**Usage**:
```rust
let proxy = SecureLLMProxy::new("gemini", secrets_manager.clone(), None).await?;
```

**Cost Example**:
- Gemini Flash: 1M tokens = $0.375
- Gemini Pro: 1M tokens = $2.00

---

## 5. OpenAI (Cloud - Industry Standard)

**Advantages**:
- 🏆 High quality (GPT-4)
- 🛠️ Best tooling ecosystem
- 📚 128K context (GPT-4 Turbo)
- 🔧 Function calling, vision, TTS

**Models**:
- **GPT-4**: $30/1M prompt, $60/1M completion (expensive)
- **GPT-3.5 Turbo**: $0.50/1M prompt, $1.50/1M completion (cheap)

**Configuration**:
```bash
export OPENAI_API_KEY="sk-your-api-key-here"
```

**Cost Example**:
- GPT-4: 1M tokens = $90 (very expensive)
- GPT-3.5: 1M tokens = $2 (affordable)

---

## 6. Anthropic Claude (Cloud - Safety Focused)

**Advantages**:
- 🛡️ Very safe outputs
- 📖 200K context window
- 🧠 Strong reasoning
- 🔒 Constitutional AI

**Models**:
- **Claude 3 Opus**: $15/1M prompt, $75/1M completion (expensive, best)
- **Claude 3 Sonnet**: $3/1M prompt, $15/1M completion (balanced)
- **Claude 3 Haiku**: $0.25/1M prompt, $1.25/1M completion (cheap)

**Configuration**:
```bash
export ANTHROPIC_API_KEY="sk-ant-your-api-key-here"
```

**Cost Example**:
- Claude Haiku: 1M tokens = $1.50
- Claude Opus: 1M tokens = $90

---

## Strategy Recommendations

### 🏠 Privacy First (LocalFirst)
```
Primary: LlamaCPP (local)
Fallback: DeepSeek or Groq (cloud, cheap)
```

### 💰 Cost Optimized
```
1. LlamaCPP (FREE for local)
2. Groq ($0.15/1M tokens, fastest)
3. DeepSeek ($0.42/1M tokens)
4. Gemini Flash ($0.375/1M tokens)
```

### ⚡ Speed Optimized
```
1. Groq (500+ tok/s)
2. LlamaCPP (local, no network)
3. Gemini Flash
```

### 🎯 Quality Optimized
```
1. GPT-4 (best, expensive)
2. Claude Opus (safe, expensive)
3. Gemini Pro (balanced)
4. DeepSeek (good, cheap)
```

---

## Multi-Provider Setup

You can configure multiple providers and switch between them:

```bash
# Configure all providers
export LLAMACPP_CONFIG="8081:llama-3-8b"
export DEEPSEEK_API_KEY="sk-..."
export GROQ_API_KEY="gsk-..."
export GEMINI_API_KEY="AIza-..."
export OPENAI_API_KEY="sk-..."
```

Then use routing strategy in code:

```rust
// Primary: Local LlamaCPP
// Fallback: Cloud DeepSeek
let unified = UnifiedLLMClient::new_local_first(
    "http://localhost:8080".to_string(),
    secrets_manager.clone(),
    Some(("deepseek", None)), // Fallback provider
).await?;
```

---

## Metrics & Cost Tracking

NEOLAND automatically tracks costs via Prometheus metrics:

```prometheus
# Total estimated cost per provider/model
neoland_llm_estimated_cost_usd{provider="deepseek",model="chat"} 0.025
neoland_llm_estimated_cost_usd{provider="llamacpp",model="local"} 0.0

# Tokens used per provider
neoland_llm_tokens_total{provider="deepseek",model="chat",type="prompt"} 45000
neoland_llm_tokens_total{provider="deepseek",model="chat",type="completion"} 12000
```

Access via:
```bash
curl http://localhost:8080/metrics | grep llm_estimated_cost
```

---

## Quick Start Examples

### Example 1: Use LlamaCPP (Local, Free)
```bash
# 1. Start llama.cpp server
./llama-server -m models/llama-3-8b.gguf --port 8081

# 2. Start NEOLAND (auto-detects LlamaCPP)
cargo run --bin neoland -- server

# 3. Chat (no cost!)
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "X-API-Key: your_key" \
  -H "Content-Type: application/json" \
  -d '{"messages": [{"role": "user", "content": "Hello!"}]}'
```

### Example 2: Use DeepSeek (Cloud, Cheap)
```bash
# 1. Set API key
export DEEPSEEK_API_KEY="sk-..."

# 2. Start NEOLAND with DeepSeek
SECURELLM_PROVIDER=deepseek cargo run --bin neoland -- server

# 3. Chat (very cheap ~$0.0004 per request)
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "X-API-Key: your_key" \
  -H "Content-Type: application/json" \
  -d '{"messages": [{"role": "user", "content": "Hello!"}]}'
```

### Example 3: Use Groq (Cloud, Fastest)
```bash
export GROQ_API_KEY="gsk-..."
SECURELLM_PROVIDER=groq cargo run --bin neoland -- server
```

---

## Troubleshooting

### LlamaCPP not connecting
```bash
# Check if server is running
curl http://localhost:8081/health

# Check logs
./llama-server -m model.gguf --port 8081 --verbose
```

### API Key errors
```bash
# Verify key is set
echo $DEEPSEEK_API_KEY

# Test directly
curl -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  https://api.deepseek.com/v1/models
```

### Cost too high?
```bash
# Check current costs
curl http://localhost:8080/metrics | grep llm_estimated_cost

# Switch to cheaper provider
export SECURELLM_PROVIDER=llamacpp  # FREE
export SECURELLM_PROVIDER=groq      # $0.15/1M
export SECURELLM_PROVIDER=deepseek  # $0.42/1M
```

---

## Security Notes

- **API Keys**: Store in Vault (production) or env vars (development)
- **Audit Logging**: All LLM requests are logged to audit trail
- **Rate Limiting**: 100 req/min per user/IP
- **Cost Tracking**: Real-time cost metrics available
- **Local First**: Use LlamaCPP for sensitive data (100% private)

---

**Last Updated**: 2026-01-31
**Version**: 0.3.0
