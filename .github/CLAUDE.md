# Neoland — Claude Instructions

## Projeto

Neoland é uma plataforma de agentes AI construída em Rust com pipeline multi-agent DSPy (Python).
Localização: `~/master/neoland`

## Stack

| Camada | Tecnologia |
|--------|-----------|
| Control plane | Rust (tokio, axum, tonic, sqlx) |
| Agent pipeline | Python 3.13 + DSPy + FastAPI |
| Storage | PostgreSQL + pgvector |
| Infra | NixOS modules declarativos |
| Secrets | HashiCorp Vault + sops-nix |
| Inference bridge | ML-Ops API + SecureLLM Bridge |

## Comandos Essenciais

`neoland <cmd>` é a interface canônica. `just` é conveniência para dev.

```bash
# SEMPRE dentro do dev shell — fora o linker quebra
nix develop

# Binário principal
neoland server                             # gRPC :50051 + REST :3001
neoland client                             # TUI
neoland doctor --json                      # diagnóstico completo (JSON)
neoland test --json                        # health check
neoland restart                            # kill + restart

# Aliases de dev shell (atalhos para o binário)
nsrv                                       # == neoland server
ncli                                       # == neoland client

# Rust (dev loop)
cargo check --lib                          # validação rápida
cargo test --lib                           # testes unitários (sem mocks)
cargo test agent_ -- --test-threads=1     # testes do control plane
cargo build --release

# Python pipeline
agents-start                               # uvicorn :8001 (--reload) — alias do dev shell
cd agents && poetry run pytest tests/ -m contract -v      # testes sem LLM
cd agents && poetry run pytest tests/ -m integration -v  # requer LLM_API_KEY

# DB migrations
sqlx migrate run --database-url "$DATABASE_URL"
```

## Arquitetura do Pipeline Multi-Agent

```
neoland client (TUI)
    │  gRPC :50051 / REST :3001
    ▼
src/server/mod.rs  (axum REST :3001 / gRPC :50051)
    │
    ├──► src/agents/orchestrator.rs  ──HTTP──►  agents/neoland_agents/app.py (:8001)
    │        │                                      Junior → Senior → Architect? → TechLeader
    │        │                                      checkpoint → ADR JSON + PostgreSQL
    │        ├──► src/mcp/server.rs (NativeMcpServer — breakpoints + tool routing)
    │        ├──► src/matrix/client.rs (MatrixClient — pipeline metrics)
    │        └──► steering_channels (live human-in-the-loop via POST /steer)
    │
    ├──► src/llm/unified_client.rs  (não tocar — LLM provider encapsulado)
    ├──► src/ml_offload/client.rs   (MLOffloadClient — inference bridge local)
    ├──► src/storage/vector_store.rs (RAG context)
    └──► PostgreSQL (agent_sessions, agent_session_metadata)
```

### Perfis dos Agentes

| Agente | Perfil cognitivo | Sempre executa? |
|--------|-----------------|-----------------|
| Junior | Criativo, sem filtro, `unknowns` como auto-consciência | Sim |
| Senior | Cético, refina, avalia risco real | Sim |
| Architect | Soundness estrutural, composabilidade | Só se `senior.escalate_to_architect = true` |
| Tech-Leader | Decisão final + checkpoint ADR automático | Sim |

## Estrutura de Diretórios

```
neoland/
├── src/
│   ├── agents/              # Control plane Rust
│   │   ├── client.rs        # HTTP client + tipos espelho dos schemas Python
│   │   ├── orchestrator.rs  # Lógica central de orquestração + live steering + breakpoints
│   │   ├── session.rs       # Estado de sessão via PostgreSQL
│   │   ├── escalation.rs    # Política de escalada entre agentes
│   │   └── checkpoint_store.rs
│   ├── mcp/                 # Native MCP Server (Ciclo 3)
│   │   ├── server.rs        # NativeMcpServer + BreakpointRequest/Resolution
│   │   ├── client.rs        # McpClient (stdio/HTTP)
│   │   ├── registry.rs      # McpRegistry (tool discovery)
│   │   └── types.rs         # Tool, CallToolResult, JsonRpcResponse
│   ├── tools/               # Native tools (Ciclo 3)
│   │   ├── mod.rs           # NativeTool trait
│   │   └── shell.rs         # ShellTool
│   ├── matrix/              # Pipeline metrics reporting (Ciclo 3)
│   │   └── client.rs        # MatrixClient + PipelineMetricsPayload
│   ├── ml_offload/          # Inference bridge client (Ciclo 3)
│   │   ├── client.rs        # MLOffloadClient
│   │   └── models.rs        # tipos
│   ├── tui/                 # Terminal UI (overhaul Ciclo 3)
│   │   ├── app.rs           # TaskStatus::WaitingForBreakpoint + breakpoint UI
│   │   ├── events.rs        # AgentStreamEvent::BreakpointHit
│   │   ├── ui.rs            # Glassmorphism 4K + zen-mode centering
│   │   └── presets.rs       # QueryConfig presets
│   ├── llm/                 # LLM proxy + unified client (NÃO MODIFICAR sem motivo)
│   ├── storage/             # pgvector
│   ├── server/              # axum REST + tonic gRPC
│   ├── openapi.rs           # OpenAPI schema gerado via utoipa
│   ├── auth.rs / audit.rs / validation.rs  # Segurança (Phase 1)
│   └── config.rs            # AgentsConfig + ServerConfig + McpConfig + MatrixConfig
│
├── agents/                  # DSPy pipeline (Python)
│   └── neoland_agents/
│       ├── signatures/      # Contratos DSPy (4 agentes)
│       ├── modules/         # Implementações dos agentes
│       ├── pipeline/        # orchestrator.py + checkpoint.py
│       ├── schemas/api.py   # Pydantic ↔ Rust mirror types
│       ├── ipc/flags.py     # AgentFlags mmap reader/writer
│       └── rag/retriever.py
│
├── modules/applications/    # NixOS modules externos (Ciclo 3)
│   ├── ml-ops-api.nix       # services.ml-ops-api — inference bridge
│   ├── securellm-bridge-api.nix  # services.securellm-bridge-api — gateway
│   └── neoland-llm-suite.nix    # suite completo LLM local
│
├── migrations/
│   ├── 001_create_vector_store.sql
│   └── 002_agent_sessions.sql   # agent_sessions + agent_session_metadata
│
├── tests/                   # Testes de integração Rust (sem mocks)
│   ├── agent_contract_test.rs
│   ├── agent_integration_test.rs
│   └── agent_session_test.rs
│
└── /etc/nixos/modules/ai/neoland/  # NixOS modules declarativos
    ├── control-plane.nix
    ├── dspy-pipeline.nix
    ├── agent-config.nix
    └── checkpoint-storage.nix
```

## Contratos entre Rust e Python

Os tipos em `src/agents/client.rs` **espelham** os schemas em `agents/neoland_agents/schemas/api.py`.
**Qualquer mudança num lado deve ser refletida no outro.**

### API REST (Rust → Python pipeline)

| Endpoint | Descrição |
|----------|-----------|
| `POST /v1/pipeline/run` | Executa pipeline completo |
| `GET /v1/pipeline/session/{id}` | Histórico de checkpoints da sessão |
| `GET /health` | Health do pipeline Python |

### API REST (Cliente → Control Plane)

| Endpoint | Auth | Descrição |
|----------|------|-----------|
| `POST /v1/agents/task` | X-API-Key (User+) | Envia task ao orchestrator |
| `POST /v1/agents/session/:id/steer` | X-API-Key (User+) | Live steering — intervenção humana mid-pipeline |
| `POST /v1/agents/session/:id/breakpoint/resolve` | X-API-Key (User+) | Aprova/rejeita/redireciona breakpoint |
| `GET /v1/agents/session/:id` | ReadOnly+ | Estado da sessão |
| `GET /v1/agents/sessions` | ReadOnly+ | Lista sessões recentes |
| `GET /v1/agents/events` | ReadOnly+ | SSE global — todos os eventos do pipeline |
| `GET /v1/agents/events/:session` | ReadOnly+ | SSE filtrado por sessão |
| `GET /v1/agents/tools` | ReadOnly+ | Lista ferramentas disponíveis no MCP nativo |
| `POST /v1/agents/tools/call` | X-API-Key (User+) | Executa tool via MCP nativo (com breakpoint) |
| `GET /v1/agents/health` | Público | Health do pipeline Python |

## Formato do Checkpoint ADR

Cada decisão do Tech-Leader gera um arquivo JSON em `/var/lib/neoland/checkpoints/adr/`:

```json
{
  "adr_id": "ADR-<session_id>-<timestamp>",
  "title": "<adr_title>",
  "status": "accepted|rejected|deferred|escalated",
  "context": "<task>",
  "decision": "<rationale>",
  "action_items": [...],
  "session_summary": "<resumo para próxima sessão>",
  "full_pipeline": { "junior": {...}, "senior": {...}, "architect": {...}, "tech_leader": {...} }
}
```

## Regras de Trabalho

### Rust
1. **Sempre dentro de `nix develop`** — fora do shell o linker quebra
2. `cargo check --lib` antes de qualquer modificação mais ampla
3. `cargo test --lib` deve passar 100% antes de commitar
4. **Sem mocks** — testes de integração usam LLM real e PostgreSQL real
5. `src/llm/unified_client.rs` é o único ponto de acesso ao LLM — não criar outros
6. Modificações em `src/agents/client.rs` requerem atualização em `schemas/api.py`
7. `NativeTool` trait em `src/tools/mod.rs` — novas ferramentas MCP implementam este trait

### Python
1. Validar schemas com pytest `-m contract` (sem LLM) primeiro
2. RAG context limitado: 5 docs × 500 chars — não aumentar sem medir impacto no context window
4. FastAPI escuta em `127.0.0.1:8001` — nunca `0.0.0.0` sem configuração explícita

### NixOS Modules
1. Todo config dos agentes via `services.neoland-agents.*` — não hardcodar valores
2. Secrets via sops-nix — nunca inline em `.nix`
3. `nix flake check` em `/etc/nixos` após qualquer mudança nos módulos

## Variáveis de Ambiente

| Variável | Padrão | Descrição |
|----------|--------|-----------|
| `DATABASE_URL` | — | PostgreSQL connection string (obrigatório em produção) |
| `LLM_API_KEY` | — | API key do provider LLM |
| `NEOLAND_LLM_PROVIDER` | `openai` | Provider: openai, deepseek, anthropic |
| `NEOLAND_LLM_MODEL` | `gpt-4o-mini` | Modelo do provider |
| `NEOLAND_INFERENCE_PROVIDER` | — | Provider de inferência local (ex: llamacpp) |
| `NEOLAND_CHECKPOINT_DIR` | `/var/lib/neoland/checkpoints/adr` | Diretório dos ADRs |
| `NEOLAND_PIPELINE_PORT` | `8001` | Porta do FastAPI |
| `NEOLAND_DSPY_URL` | `http://127.0.0.1:8001` | URL do pipeline Python (Rust) |
| `NEOLAND_SHM_PATH` | `/run/neoland/agent-flags.shm` | Arquivo mmap IPC (Ciclo 1 Fase A) |
| `NEOLAND_NATS_URL` | — (desabilitado) | URL do NATS — define também `nats.enabled = true` (Ciclo 1 Fase B) |
| `NEOLAND_MCP_BINARY` | `securellm-mcp` | Binário do MCP server stdio (Ciclo 3) |
| `NEOLAND_MCP_ENABLED` | `false` | Habilita spawn do MCP server externo (Ciclo 3) |
| `NEOLAND_ML_API_URL` | `http://127.0.0.1:8080` | URL do ML-Ops API inference bridge (Ciclo 3) |
| `NEOLAND_MATRIX_URL` | — | URL do Matrix metrics backend (Ciclo 3) |
| `NEOLAND_MATRIX_ENABLED` | `false` | Habilita envio de métricas ao Matrix (Ciclo 3) |
| `NEOLAND_FRONTEND_HOST` | `127.0.0.1` | Bind do frontend Next.js |

## Fases do Projeto (Ciclo 0 — fechado 2026-04-07)

| Fase | Status | Descrição |
|------|--------|-----------|
| 1 — Contratos | ✅ | Signatures DSPy, Pydantic schemas, tipos Rust espelho, migration SQL |
| 2 — Python Pipeline | ✅ | 4 módulos DSPy, orchestrator, checkpoint, RAG, FastAPI |
| 3 — Rust Control Plane | ✅ | client, session, escalation, orchestrator, mods em config/lib/audit |
| 3b — Rotas HTTP | ✅ | POST /v1/agents/task, GET /v1/agents/session/:id, GET /v1/agents/health |
| 4 — NixOS Modules | ✅ | control-plane.nix (shmPath + natsUrl), dspy-pipeline.nix (shmPath), ledger-subscriber.nix (secp256k1 + hardening) |
| 5 — Testes Rust | ✅ | agent_contract_test (9), agent_session_test (4), agent_integration_test (4) |

## Ciclo 1 ✅ (fechado 2026-04-07)

| Fase | Status | Descrição |
|------|--------|-----------|
| A — mmap IPC | ✅ | `src/agents/flags.rs` + `mmap.rs` + `ipc/flags.py` — IPC zero-copy intra-host (13 testes) |
| B — NATS | ✅ | `src/agents/nats.rs` — publisher via spectre-events, 5 testes unitários, `NatsConfig` + `NEOLAND_NATS_URL` |
| C — adr-ledger | ✅ | `adr-ledger/crates/ledger-subscriber/` — `Signer` trait + `FileKeySigner` (sops) + `TrezorSigner` (stub preview) + `MerkleStore` (PG) + **JetStream** at-least-once, 15 testes |
| D — Phantom | ✅ | `neoland.pipeline.output.v1` (neoland) + `phantom/nats/neoland_scanner.py` — sentiment + risk keywords → `phantom.pipeline.scan.v1` |
| NixOS Modules | ✅ | `ledger-subscriber.nix` + `control-plane.nix` (shmPath/natsUrl) + `dspy-pipeline.nix` (shmPath) |
| JetStream | ✅ | `jetstream.rs` — stream `NEOLAND_EVENTS`, pull consumer `ledger-sub` (durable), explicit ack, max_deliver=5, 7d retention |

### mmap IPC (Fase A)

Layout binário `SharedFlags` (64 bytes, 1 cache line, `repr(C, align(64))`):

| Offset | Size | Campo |
|--------|------|-------|
| 0 | 1 | `pipeline_active` (AtomicBool) |
| 1 | 1 | `escalate_to_architect` (AtomicBool) |
| 2 | 1 | `abort_requested` (AtomicBool) |
| 4 | 4 | `junior_confidence` (AtomicU32, f32 bits) |
| 8 | 1 | `risk_level` (AtomicU8: 0=low…3=critical) |
| 16 | 36 | `session_id` ([u8; 36], UUID UTF-8) |

Arquivo shm criado pelo control plane em `/run/neoland/agent-flags.shm` na inicialização.
Python lê/escreve via `AgentFlags` em `agents/neoland_agents/ipc/flags.py`.

## Ciclo 2 ✅ (fechado 2026-05-16)

| Fase | Status | Descrição |
|------|--------|-----------|
| 4.1 — Metrics | ✅ | Agent pipeline metrics (Prometheus counters/histograms) wired no orchestrator |
| 4.2 — cargo-audit | ✅ | `cargo-audit` no devShell — supply-chain CVE scanning |
| 4.3 — Coding partner skill | ✅ | `neoland-agents` package metadata + skill coding partner |
| 4.4 — NKey + ACL | ✅ | NKey SOPS-encrypted em `~/master/secrets/`, NATS ACL neoland (publish `neoland.>` only) |
| 4.5 — Cross-stack | ✅ | Neotron: Synapse ↔ Cortex per-agent embeddings + stress tests BASTION/SENTINEL |
| 4.6 — Owasaka tests | ✅ | Event pipeline tests (12) + API server tests (5) |
| 4.7 — EDR rules | ✅ | SIGMA (3 rules) + YARA (6 rules) neoland-specific em `sentinel/sigma/` e `sentinel/yara/` |

### EDR Detection Rules (Fase 4.7)

**SIGMA** (`sentinel/sigma/neoland/agent_pipeline_anomalies.yml`):
- Process anomalies — child processes inesperados do pipeline
- NATS connection spike — >20 conexões/minuto (reconnect storm ou impersonation)
- Checkpoint write outside path — escrita fora de `/var/lib/neoland/checkpoints/adr/`

**YARA** (`sentinel/yara/neoland/agent_pipeline.yar`):
- `ADR_PromptInjection` — payloads de prompt injection em checkpoints
- `ADR_ShellPayload` — comandos shell em action_items/decision
- `Checkpoint_PathTraversal` — path traversal em ficheiros de checkpoint
- `Pipeline_SuspiciousConfidence` — NaN/Infinity/string em confidence
- `DSPy_ModuleTamper` — monkey-patching ou imports suspeitos em módulos DSPy
- `RiskLevel_Escalation` — override forçado de risk_level em mensagens inter-agente

## Ciclo 3 — Agentic Operator Stack (em progresso)

| Fase | Status | Descrição |
|------|--------|-----------|
| 5.1 — Native MCP Server | ✅ | `src/mcp/` — NativeMcpServer + BreakpointRequest/Resolution + tool routing |
| 5.2 — Native Tools | ✅ | `src/tools/` — `NativeTool` trait + `ShellTool` |
| 5.3 — Live Steering | ✅ | `orchestrator.steer_task()` + `POST /v1/agents/session/:id/steer` — human-in-the-loop |
| 5.4 — Breakpoints | ✅ | `TaskStatus::WaitingForBreakpoint` + TUI breakpoint UI + `POST .../breakpoint/resolve` |
| 5.5 — SSE Events | ✅ | `GET /v1/agents/events` + `GET /v1/agents/events/:session` — streaming de eventos |
| 5.6 — Sessions list | ✅ | `GET /v1/agents/sessions` + checkpoint listing por sessão |
| 5.7 — TUI overhaul | ✅ | Glassmorphism 4K, zen-mode centering, LlamaManager TUI |
| 5.8 — Matrix client | ✅ | `src/matrix/client.rs` — envio de métricas pipeline pós-run |
| 5.9 — ML Offload | ✅ | `src/ml_offload/` — MLOffloadClient para inference bridge local |
| 5.10 — Frontend | ✅ | `matrix/frontend/` — Next.js 14: session registry, pipeline history, stats |
| 5.11 — God Mode | ✅ | `neoland-up` — sobe control plane + DSPy pipeline + TUI num só comando |
| 5.12 — NixOS externos | ✅ | `ml-ops-api.nix` + `securellm-bridge-api.nix` + `neoland-llm-suite.nix` |
| 5.13 — Forgejo | ✅ | Forgejo como git provider first-class na flake |
| 5.14 — OpenAPI | ✅ | `src/openapi.rs` — schema utoipa gerado automaticamente |

### Native MCP Server (Fase 5.1–5.2)

Arquitetura interna do servidor MCP nativo (`src/mcp/server.rs`):
- `NativeMcpServer::call_tool()` abre um `oneshot::channel` antes de executar
- Envia `BreakpointRequest` ao TUI via `mpsc::Sender`
- TUI bloqueia em `WaitingForBreakpoint` e aguarda `BreakpointResolution::{Approve, Reject, Steer(String)}`
- Só prossegue com a execução real após resolução humana

### Live Steering (Fase 5.3)

`orchestrator.steer_task(session_id, message)` injeta diretiva no canal `steering_channels` da sessão ativa. O orchestrator replana no próximo tick com `"Received steering directive: {msg}. Replanning..."`.

## Problemas Conhecidos

- `cargo check` fora do `nix develop` quebra o linker (ld-wrapper.sh em store path antigo)
- `sqlx::query!` macros requerem `DATABASE_URL` em compile time — usar `sqlx::query` + `.bind()` no módulo `agents/`
- DSPy 2.x: `ChainOfThought` com Signatures tipadas pode retornar `confidence` como string — usar `float()` no cast
- FastAPI lifespan: inicialização do LLM é síncrona dentro do `asynccontextmanager` — se o provider falhar, o servidor não sobe
- `neoland-up` usa API key hardcoded `neoland_admin_53352f54...` — apenas para dev local; em produção usar `NEOLAND_API_KEY` via sops
