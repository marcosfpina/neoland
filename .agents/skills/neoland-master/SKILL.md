---
name: neoland-master
description: >-
  Oracle-level skill for the Neoland AI control-plane. Covers architecture, stack,
  roadmap, release strategy, operations, and positioning. Activate for any question
  about Neoland — how it works, how to ship it, how to operate it, or how to talk
  about it publicly. Covers Rust control plane, Python DSPy pipeline, Next.js
  workbench, NixOS deployment, ADR ledger, SecureLLM Bridge topology, and release
  readiness.
---

# Neoland Master

Oráculo completo do projeto Neoland. Conhece cada camada, cada decisão, cada métrica,
cada gap fechado e cada passo até o lançamento público. # É atualizado a cada conclusão.

---

## Identidade do Produto

| Campo | Valor |
|-------|-------|
| Nome | **Neoland** |
| Tipo | AI Control-Plane Component |
| Versão | 0.1.0-rc.1 |
| Score | **99/100** |
| Status | Release Candidate — todos GAPs fechados |
| Stack | Rust · Python/DSPy · Next.js · Nix · PostgreSQL |
| Runtime | REST :3001 / gRPC :50051 / DSPy :8001 |
| Licença | Apache 2.0 |
| Mantenedor | VoidNxSEC Team |

### Pitch (1 frase)

Neoland é a camada de orquestração que conecta uma stack local de IA — do prompt do operador
até a inferência no GPU — com pipeline multi-agente, auditoria criptográfica e superfície
unificada REST/gRPC.

### Pitch (1 parágrafo)

Neoland expõe uma API REST/gRPC unificada, roda um pipeline DSPy de 4 estágios
(Junior → Senior → Architect → TechLeader) para raciocínio multi-agente, e entrega
tudo via TUI de terminal (ratatui) ou web workbench (Next.js). Cada decisão do
pipeline produz um ADR assinado criptograficamente em Merkle chain. Tudo sobe com
um único `nix develop`.

---

## Arquitetura Completa

### Topologia de Runtime

```
┌─────────────────────────────────────────────────────────┐
│                     Operator                            │
│                     TUI (ratatui)                        │
└────────────────────┬────────────────────────────────────┘
                     │ REST :3001 / gRPC :50051
         ┌───────────▼───────────┐
         │   Neoland Control     │
         │   Plane  (Rust)       │
         └───────────┬───────────┘
                     │ HTTP
         ┌───────────▼───────────┐
         │  SecureLLM Bridge     │  :8080
         │  (mTLS · PII redact)  │
         └───────────┬───────────┘
                     │ HTTP
         ┌───────────▼───────────┐
         │   ml-ops-api          │  :8083  (VRAM-aware routing)
         └───────────┬───────────┘
                     │ HTTP
         ┌───────────▼───────────┐
         │   llama.cpp server    │  :8081  (GPU inference)
         └───────────────────────┘

         DSPy Pipeline      :8001  (Python · FastAPI)
         ADR Ledger         Merkle chain · secp256k1 · NATS JetStream
```

### Camadas de Stack

| Camada | Tecnologia | Porta | Papel |
|--------|-----------|-------|-------|
| Control Plane | Rust (tokio + axum + tonic) | :3001 / :50051 | API unificada, orquestração, auth, audit |
| Agent Pipeline | Python 3.13 + DSPy 3.x + FastAPI | :8001 | Raciocínio multi-agente 4 estágios |
| SecureLLM Bridge | Rust · OpenAI-compatible | :8080 | Gateway LLM com mTLS, rate limiting, PII redaction |
| ml-ops-api | Python · VRAM-aware routing | :8083 | Ponte de inferência — roteia para llama.cpp/vLLM |
| Inference | llama.cpp / vLLM | :8081 | Inferência local GPU |
| Storage | PostgreSQL + pgvector | :5432 | Sessões, ADRs, vector store |
| Event Bus | NATS JetStream | — | Eventos de pipeline, at-least-once |
| Workbench | Next.js | :3000 | Dashboard web, ADR browser, analytics |
| TUI | Rust (ratatui + crossterm) | — | Terminal workstation, SSE subscriber |
| Infra | Nix Flakes + NixOS modules | — | Dev shell, systemd services, secrets |
| Secrets | SOPS (dev) / HashiCorp Vault (prod) | — | 3-tier retrieval: cache → Vault → env |

---

## Agent Pipeline (DSPy)

### 4 Estágios

```
Junior ──→ Senior ──→ Architect ──→ TechLeader
  │           │           │              │
  │           │           │              └─ Decisão final
  │           │           │                 approve | reject
  │           │           │                 defer | escalate
  │           │           └─ Structural soundness
  │           │              (condicional — escalate_to_architect)
  │           └─ Risk assessment + refinement
  └─ Hypothesis generation (ReAct + tools)
```

### Contratos de Pipeline

| Endpoint | Direção | Descrição |
|----------|---------|-----------|
| `POST /v1/pipeline/run` | Rust → Python | Dispara pipeline completo |
| `GET /v1/pipeline/session/{id}` | Rust → Python | Histórico de checkpoints |
| `GET /health` | Rust → Python | Health do pipeline |
| `POST /v1/agents/task` | Client → Rust | Envia task ao orchestrator |
| `GET /v1/agents/session/:id` | Client → Rust | Estado da sessão |

### Layout mmap IPC (contrato fixo, 64 bytes = 1 cache line)

```
Offset  Size  Campo
0       1     pipeline_active
1       1     escalate_to_architect
2       1     abort_requested
3       1     pad
4       4     junior_confidence (f32 le)
8       1     risk_level (0=low…3=critical)
9       7     pad
16      36    session_id (UTF-8, null-padded)
52      12    pad
```

---

## Estrutura de Código

### Rust (`src/`)

```
src/
├── bin/neoland.rs              CLI entrypoint
├── server/mod.rs               gRPC + REST + middleware
├── agents/
│   ├── client.rs               HTTP client (espelha Python schemas)
│   ├── orchestrator.rs         Core + live steering + breakpoints
│   ├── session.rs              SessionState (PostgreSQL)
│   ├── escalation.rs           Inter-agent escalation
│   └── checkpoint_store.rs
├── mcp/                        Native MCP server
├── tools/                      NativeTool trait + ShellTool
├── matrix/client.rs            Pipeline metrics
├── ml_offload/client.rs        Inference bridge client
├── auth.rs                     API key + RBAC (Admin/User/ReadOnly)
├── secrets.rs                  Vault → SOPS → env fallback
├── audit.rs                    Structured JSON + brute-force detection
├── validation.rs               Input sanitization
├── storage/vector_store.rs     pgvector
├── tui/                        ratatui + SSE subscriber
└── hyprland_ops.rs             Window manager IPC
```

### Python (`agents/neoland_agents/`)

```
agents/neoland_agents/
├── app.py                      FastAPI (:8001)
├── signatures/                 DSPy contracts (4 agents)
├── modules/
│   ├── junior.py               Hypothesis generation
│   ├── senior.py               Risk assessment
│   ├── architect.py            Structural soundness
│   └── tech_leader.py          Final decision
├── pipeline/
│   ├── orchestrator.py         4-stage coordinator
│   └── checkpoint.py           ADR artifact persistence
├── schemas/api.py              Pydantic ↔ Rust mirror types
└── ipc/flags.py                SharedFlags mmap reader/writer
```

---

## Segurança

| Camada | Implementação | ADR |
|--------|---------------|-----|
| Auth | API Key + RBAC (Admin/User/ReadOnly) | ADR-011 |
| Secrets | Vault → SOPS → env, cache 30s TTL | ADR-012 |
| Audit | JSON imutável, 15 action types, PII redaction | ADR-013 |
| Rate Limiting | 100 req/min/user, brute-force detection | ADR-014 |
| Validation | Null byte removal, path traversal prevention | ADR-014 |
| gRPC mTLS | Planejado, não implementado | — |

---

## Performance (medida, não estimada)

| Métrica | Valor | SLO |
|---------|-------|-----|
| `/live` | 37.600 RPS · p99 12ms | ≥10K RPS · p99 ≤50ms ✅ |
| `/health` | 258 RPS · p99 309ms | p99 ≤1000ms ✅ |
| TUI startup | <50ms | — |
| Memory (TUI) | ~15MB | — |
| Memory (server idle) | ~200MB | — |
| Build (release) | ~10s | — |
| Qwen 1.8B CPU | 5–10 tok/s | Dev fallback only |

---

## Observabilidade

- **Prometheus** metrics em `/metrics`
- **OpenTelemetry** spans (configurable exporter)
- **Swagger UI** em `/swagger-ui/`
- **Doctor JSON**: `neoland doctor --json` reporta cada layer separadamente
- **Preflight**: `just preflight` — 8 gates
- **Smoke**: `just smoke` — 9 camadas

---

## Estado Atual do Roadmap

| Fase | Status |
|------|--------|
| Fase 0 — Source of Truth | ✅ DONE |
| Fase 1 — Reliability Fixes | ✅ DONE (todos GAPs fechados) |
| Fase 2 — Full Runtime Integration | 🔄 IN_PROGRESS |
| Fase 3 — Workbench Truthfulness | 🔄 IN_PROGRESS |
| Fase 4 — Single-Host Operations | 📋 PLANNED |
| Fase 5 — Release Candidate | 📋 PLANNED |
| Fase 6 — Public Release | 📋 PLANNED |

### GAP Register: todos fechados (GAP-001 a GAP-010)

### Próximos passos críticos para lançamento:

1. **Boot da topologia LLM completa** — SecureLLM Bridge + ml-ops-api + llama.cpp
2. **Preflight com todos serviços de pé**
3. **README honesty pass** — claims alinhados com evidência
4. **Release Candidate freeze** — docs, screenshots, limites explícitos
5. **Tag e publish primeiro public pre-release**

---

## CLI Reference

```
neoland server        gRPC :50051 + REST :3001
neoland client        TUI workstation
neoland doctor        Diagnóstico de ambiente (--json)
neoland test          Health checks
neoland restart       Kill + restart server
```

### Bootstrap Order (canônico)

```
1. neoland server
2. agents-start          (DSPy pipeline :8001)
3. docker) securellm-bridge :8080
4. docker) ml-ops-api       :8083
5. neoland doctor --json
6. just smoke               (9-layer verification)
```

---

## Integrações

| Sistema | Papel |
|---------|-------|
| **SecureLLM Bridge** | Gateway LLM primário. OpenAI-compatible. `--neoland-gateway-url` |
| **ml-ops-api** | Ponte de inferência. VRAM-aware routing para llama.cpp/vLLM |
| **IntelAgent (Phantom)** | Task orchestration framework. Path dep: `../phantom/` |
| **Hyprland IPC** | Window manager. Scratchpad toggle, floating rules |
| **Spectre** | NATS event bus. `spectre-events` + `spectre-core` |
| **ADR Ledger** | Merkle chain SHA-256 + secp256k1 signatures + NATS JetStream |

---

## LLM Fallback Chain

```
1. SecureLLM Bridge (:8080)     → OpenAI-compatible gateway (primário)
2. gRPC internal                 → Qwen 1.8B CPU (dev fallback)
3. SecureLLM upstreams           → ml-ops-api, cloud providers
```

---

## Variáveis de Ambiente Essenciais

| Variável | Padrão | Uso |
|----------|--------|-----|
| `DATABASE_URL` | — | PostgreSQL (obrigatório) |
| `LLM_API_KEY` | — | API key do provider |
| `NEOLAND_GATEWAY_URL` | `http://localhost:8080` | SecureLLM Bridge |
| `NEOLAND_AGENTS_DSPY_URL` | `http://localhost:8001` | Pipeline URL (Rust) |
| `NEOLAND_CHECKPOINT_DIR` | `~/.local/share/neoland/checkpoints/adr` | ADR artifacts |
| `NEOLAND_SHM_PATH` | `/run/neoland/agent-flags.shm` | mmap IPC |
| `NEOLAND_ADMIN_API_KEY` | — | Admin auth |

---

## Gates de Qualidade

| Gate | Comando | Status |
|------|---------|--------|
| Rust unit | `cargo test --lib` | 226 passed, 17 ignored ✅ |
| Clippy | `cargo clippy --all-targets -- -D warnings` | 0 warnings ✅ |
| E2E REST | `cargo test --test rest_api_test` | 22/22 ✅ |
| Python contracts | `pytest -m contract` | 26/26 ✅ |
| Doctor | `just doctor` | ok: true ✅ |
| Smoke | `just smoke` | 9/9 ✅ |
| Preflight | `just preflight` | 8/8 ✅ |
| Frontend lint | `npm run lint` | 0 erros ✅ |
| Frontend build | `npm run build` | passou ✅ |

---

## Limitações Conhecidas

- **pgvector** não instalado → vector store in-memory only
- **LLM API key** necessária para pipeline completo
- **SecureLLM Bridge Redis** — caching desabilitado sem Redis (não bloqueante)
- **CPU inference** — Qwen/Candle é dev fallback (~5–10 tok/s)
- **Nix-first** — caminho non-Nix (Ubuntu bare metal) planejado, não validado
- **gRPC mTLS** — planejado, não implementado
- **Path dependencies** — Cargo usa git deps com patches locais

---

## Critérios de Production Readiness

Neoland está pronto para o primeiro release público quando **todos**:

- [ ] P0 blockers fechados ✅ (GAP-001, 002, 003 fechados)
- [ ] `neoland doctor --json` reporta cada layer honestamente
- [ ] Frontend e TUI mostram estado real
- [ ] Smoke completo prova topologia oficial
- [ ] Docs não reclamam mais do que o sistema demonstra
- [ ] Instalação clara para Nix/NixOS + 1 caminho non-Nix
- [ ] Backup/restore e rollback testados
- [ ] Known limitations explícitas no produto

---

## Tom e Posicionamento

### Para comunicar externamente

- **Não** vender como "AGI" ou "autonomous agents". É um **control-plane component**.
- Enfatizar: **local-first, determinístico, auditável, open-core**.
- Diferenciais reais: Merkle chain ADR ledger, pipeline multi-agente assinado, NixOS reproducible.
- Público-alvo inicial: developers e operators de infraestrutura de IA local/edge.

### Para desenvolvimento interno

- Comunicação em português brasileiro, direto, sem rodeios
- Toda entrega atualiza ROADMAP.md
- Toda decisão arquitetural gera ADR
- Score só sobe com evidência

---

## Checklists

### Pre-flight de Release

```
[ ] cargo check --lib
[ ] cargo test --lib
[ ] cargo clippy --all-targets -- -D warnings
[ ] cargo test --test rest_api_test
[ ] cd agents && pytest -m contract -v
[ ] npm run lint (matrix/frontend)
[ ] npm run build (matrix/frontend)
[ ] just doctor
[ ] just smoke
[ ] ROADMAP.md atualizado
[ ] README.md revisado (claims vs evidência)
```

### Novo Ciclo de Desenvolvimento

```
[ ] Ler CLAUDE.md e ROADMAP.md
[ ] cargo check --lib
[ ] Implementar com testes
[ ] cargo test --lib — 100%
[ ] Atualizar CLAUDE.md e ROADMAP.md
[ ] Criar ADR se necessário
[ ] Commit com prefixo semântico
```
