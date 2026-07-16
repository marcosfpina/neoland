# Neoland Smoke E2E — v0.3.0

**Date**: 2026-07-16 · **Server**: neoland v0.2.0-dev (debug build)

## Stack Status

| Layer | Status | Detail |
|-------|--------|--------|
| Neoland Control Plane (:3003) | ✅ | REST + gRPC-web operacionais |
| Auth (X-API-Key) | ✅ | Admin key validada, audit log gerado |
| gRPC-web (tonic-web) | ✅ | HTTP/1.1 + GrpcWebLayer ativos |
| llama.cpp (:8080) | ✅ | Health 200, inference backend disponível |
| DSPy Pipeline (:8001) | ⚠️ | Não iniciado (requer Poetry + Python) |
| SecureLLM Bridge (:8080) | ⚠️ | Health OK mas rota LLM via gateway |
| ml-ops-api (:8083) | ❌ | Não iniciado |
| PostgreSQL | ❌ | Não configurado (DATABASE_URL) |
| Vault | ❌ | Não configurado |

## Test Results

### REST Chat Completion
```
POST /v1/chat/completions
→ 200 OK, JSON response
→ LocalEngine (Qwen/Candle) URL parsing bug (pre-existing)
→ Gateway route via llama.cpp funcional
```

### Agent Task
```
POST /v1/agents/task
→ 200 OK (error: "database required" — expected without PostgreSQL)
```

### Doctor
```
neoland doctor --json
→ ok: false (layers degradadas esperadas sem stack completa)
→ 2/11 layers OK, 5 warnings, 2 errors
→ llama.cpp + Gateway saudáveis
```

## Known Issues

- **Candle/Qwen URL bug**: `RelativeUrlWithoutBase` — pre-existing, needs fix in `src/engine/`
- **PostgreSQL required**: agent tasks, sessions, and ADR ledger need DATABASE_URL
- **DSPy pipeline**: needs `just agents-start` (Poetry + Python 3.13)

## Evidence

Server log: `/tmp/neoland-server.log`
Smoke log: terminal output above
