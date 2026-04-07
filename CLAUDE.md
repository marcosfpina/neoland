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

## Comandos Essenciais

```bash
# SEMPRE dentro do dev shell — fora o linker quebra
nix develop

# Rust
cargo check --lib                          # validação rápida
cargo test --lib                           # 156+ testes (sem mocks)
cargo test agent_ -- --test-threads=1     # testes do control plane
cargo build --release

# Python pipeline
cd agents
python -m venv .venv && .venv/bin/pip install -e ".[dev]"
uvicorn neoland_agents.app:app --reload --port 8001   # start pipeline
pytest tests/ -m contract -v              # testes sem LLM
pytest tests/ -m integration -v          # testes com LLM real (requer LLM_API_KEY)

# DB migrations
sqlx migrate run --database-url "$DATABASE_URL"
```

## Arquitetura do Pipeline Multi-Agent

```
CLI/TUI (Rust)
    │
    ▼
src/agents/orchestrator.rs  ──HTTP──►  agents/neoland_agents/app.py (:8001)
    │                                      Junior → Senior → Architect? → TechLeader
    │                                      checkpoint → ADR JSON + PostgreSQL
    ├──► src/llm/unified_client.rs  (não tocar — LLM provider encapsulado)
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
│   │   ├── orchestrator.rs  # Lógica central de orquestração
│   │   ├── session.rs       # Estado de sessão via PostgreSQL
│   │   ├── escalation.rs    # Política de escalada entre agentes
│   │   └── checkpoint_store.rs
│   ├── llm/                 # LLM proxy + unified client (NÃO MODIFICAR sem motivo)
│   ├── storage/             # pgvector
│   ├── server/              # axum REST + tonic gRPC
│   ├── auth.rs / audit.rs / validation.rs  # Segurança (Phase 1)
│   └── config.rs            # AgentsConfig + ServerConfig + ...
│
├── agents/                  # DSPy pipeline (Python)
│   └── neoland_agents/
│       ├── signatures/      # Contratos DSPy (4 agentes)
│       ├── modules/         # Implementações dos agentes
│       ├── pipeline/        # orchestrator.py + checkpoint.py
│       ├── schemas/api.py   # Pydantic ↔ Rust mirror types
│       └── rag/retriever.py
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

### API REST (Rust → Python)

| Endpoint | Descrição |
|----------|-----------|
| `POST /v1/pipeline/run` | Executa pipeline completo |
| `GET /v1/pipeline/session/{id}` | Histórico de checkpoints |
| `GET /health` | Health do pipeline Python |

### API REST (Cliente → Control Plane)

| Endpoint | Auth | Descrição |
|----------|------|-----------|
| `POST /v1/agents/task` | X-API-Key (User+) | Envia task ao orchestrator |
| `GET /v1/agents/session/:id` | ReadOnly+ | Estado da sessão |
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
  "full_pipeline": { "junior": {...}, "senior": {...}, "architect": {...} }
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

### Python
1. Validar schemas com pytest `-m contract` (sem LLM) primeiro
2. `dspy.Assert` nos campos `confidence` (float) e `risk_level` (enum) — não remover
3. RAG context limitado: 5 docs × 500 chars — não aumentar sem medir impacto no context window
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
| `NEOLAND_CHECKPOINT_DIR` | `/var/lib/neoland/checkpoints/adr` | Diretório dos ADRs |
| `NEOLAND_PIPELINE_PORT` | `8001` | Porta do FastAPI |
| `NEOLAND_AGENTS_DSPY_URL` | `http://localhost:8001` | URL do pipeline Python (Rust) |
| `NEOLAND_SHM_PATH` | `/run/neoland/agent-flags.shm` | Arquivo mmap IPC (Ciclo 1 Fase A) |
| `NEOLAND_NATS_URL` | — (desabilitado) | URL do NATS — define também `nats.enabled = true` (Ciclo 1 Fase B) |

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

## Ciclo 2 — Operational Stack (em progresso)

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

## Problemas Conhecidos

- `cargo check` fora do `nix develop` quebra o linker (ld-wrapper.sh em store path antigo)
- `sqlx::query!` macros requerem `DATABASE_URL` em compile time — usar `sqlx::query` + `.bind()` no módulo `agents/`
- DSPy 2.x: `ChainOfThought` com Signatures tipadas pode retornar `confidence` como string — usar `float()` no cast
- FastAPI lifespan: inicialização do LLM é síncrona dentro do `asynccontextmanager` — se o provider falhar, o servidor não sobe
