# Neoland — Roadmap

**Última atualização**: 2026-04-26
**Versão atual**: v0.1.0 (beta)
**Direção**: Code agent suite — não um chatbot, software de alto nível para agentes de código

---

## Visão

Neoland é uma **estação de trabalho para agentes de código**. O objetivo não é um chat bonito — é um
terminal especializado onde agentes chamam ferramentas reais, gerenciam tarefas, tomam decisões
arquiteturais e têm seu desempenho rastreado ao longo do tempo.

Referências de nível:
- Claude Code (terminal agent com MCP)
- Aider (agent para código com ferramenta real)
- Cursor (IDE agent com tool calls ao vivo)

Stack alvo:

```
neoland/
├── control plane (Rust)         ← já existe
├── DSPy pipeline (Python)       ← já existe
├── TUI — agent workstation      ← redesign completo
│   ├── SSE consumer             ← real-time events
│   ├── MCP stdio client         ← securellm-mcp como tool provider
│   ├── task manager             ← fila de tasks, status, prioridade
│   ├── pipeline monitor         ← Junior→TechLeader ao vivo
│   └── keybindings              ← power user shortcuts
└── matrix (mover pra cá)
    ├── FastAPI backend          ← /rank, /metrics, /stf/context
    └── Next.js frontend         ← dashboard de performance dos agents
```

---

## Fase 1 — SSE Infrastructure (próxima)

**Objetivo**: o server publica eventos em tempo real; o TUI os consome.

### 1.1 — Tipos de evento

```rust
// src/agents/events.rs (novo)
pub enum AgentEvent {
    PipelineStarted  { session_id: Uuid, task_preview: String },
    StageStarted     { session_id: Uuid, stage: &'static str },
    StageDone        { session_id: Uuid, stage: &'static str,
                       confidence: Option<f32>, risk_level: Option<u8>,
                       latency_ms: u64 },
    StageSkipped     { session_id: Uuid, stage: &'static str },
    ToolCallStarted  { session_id: Uuid, tool: String, args_summary: String },
    ToolCallDone     { session_id: Uuid, tool: String, duration_ms: u64 },
    ToolCallFailed   { session_id: Uuid, tool: String, error: String },
    AdrCheckpoint    { session_id: Uuid, adr_id: String, status: String, title: String },
    PipelineDone     { session_id: Uuid, latency_ms: u64 },
    PipelineError    { session_id: Uuid, error: String },
}
```

### 1.2 — Broadcast channel no AppState (server)

```rust
// Adicionar em src/server/mod.rs — AppState
pub struct AppState {
    // ... campos existentes ...
    pub event_bus: tokio::sync::broadcast::Sender<AgentEvent>,
}
```

### 1.3 — Endpoint SSE

```
GET /v1/agents/events           → stream global (todos os sessions)
GET /v1/agents/events/:session  → stream de um session específico
```

- Auth: `ReadOnly+` (mesmo nível de `/v1/agents/session/:id`)
- Formato: `text/event-stream`, eventos JSON, heartbeat 15s
- O orchestrator publica no `event_bus` durante execução do pipeline

### 1.4 — Orchestrator publica eventos

Modificar `src/agents/orchestrator.rs` para publicar em cada etapa:
- Antes de chamar Junior → `StageStarted { stage: "junior" }`
- Depois de Junior retornar → `StageDone { ..., confidence, risk_level, latency_ms }`
- Idem para Senior, Architect (se escalado), TechLeader
- Na decisão final → `AdrCheckpoint`

**Entregável**: `cargo test --lib` passa + endpoint SSE retorna eventos reais

---

## Fase 2 — TUI Redesign (agent workstation)

**Objetivo**: terminal especializado para trabalho com agents, não chatbot.

### 2.1 — Novo layout

```
╭─ ◆ neoland ─────────────────────────────── ● connected · 42ms ─╮
│                                                                   │
│  TASKS                                                            │
│  ● [abc123] analyze src/auth.rs            running               │
│  ○ [def456] write unit tests               queued                │
│  ○ [ghi789] refactor error handling        queued                │
│                                                                   │
│  ─────────────────────────────────────────────────────────────   │
│                                                                   │
│  ▸ pipeline  [abc123]                                             │
│    ✓  junior       0.82  low        312ms                         │
│    ✓  senior       no-escalate      891ms                         │
│    ⠹  tech-leader  running...                                     │
│    ○  architect    skipped                                        │
│                                                                   │
│  ▸ tools                                                          │
│    ✓  read_file    src/auth.rs      12ms                          │
│    ✓  search       "jwt token"       5ms                          │
│    ⠹  analyze_code running...                                     │
│                                                                   │
│  ▸ output  [abc123]                                               │
│    O módulo de autenticação tem 3 problemas críticos...           │
│                                                                   │
╰───────────────────────────────────────────────────────────────────╯
╭─ [ balanced ] ──── ^t task  ^p pipeline  ^m matrix  ^c quit ──────╮
│  > █                                                               │
╰────────────────────────────────────────────────────────────────────╯
```

### 2.2 — Novos tipos em `app.rs`

```rust
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub status: TaskStatus,   // Queued | Running | Done | Failed
    pub created_at: DateTime<Utc>,
}

pub struct PipelineStage {
    pub name: &'static str,
    pub status: StageStatus,  // Pending | Running | Done{latency_ms} | Skipped | Failed
    pub confidence: Option<f32>,
    pub risk_level: Option<u8>,
}

pub struct ToolCall {
    pub name: String,
    pub args_summary: String,
    pub status: ToolStatus,   // Running | Done{duration_ms} | Failed
}
```

### 2.3 — SSE consumer no TUI

```rust
// mod.rs — novo LlmEvent
enum AgentStreamEvent {
    StageUpdate(PipelineStage),
    ToolUpdate(ToolCall),
    AdrDecision(AdrSummary),
    OutputChunk(String),
    Done { latency_ms: u64 },
    Error(String),
}
```

TUI conecta em `GET /v1/agents/events/:session` via `reqwest::Response::bytes_stream()`.

### 2.4 — Keybindings

| Key | Ação |
|-----|------|
| `^t` | Nova task |
| `^p` | Toggle pipeline panel |
| `^m` | Abrir matrix dashboard |
| `^x` | Cancelar task em execução |
| `^h` | Histórico de tasks |
| `Tab` | Focar próximo painel |
| `↑↓` | Navegar tasks / output |
| `Enter` | Executar task selecionada / enviar |
| `Esc` | Cancelar input / fechar painel |

**Entregável**: TUI mostra pipeline ao vivo via SSE + task queue funcional

---

## Fase 3 — MCP stdio (securellm-mcp como tool provider)

**Objetivo**: agents chamam ferramentas reais via MCP, o TUI mostra cada call.

### 3.1 — Flake input

```nix
# flake.nix
inputs.securellm-mcp.url = "github:marcosfpina/securellm-mcp";
```

O neoland spawna o processo `securellm-mcp` via stdio na inicialização do server.

### 3.2 — MCP client em Rust

```
src/mcp/
├── mod.rs          — spawn processo, gerenciar stdio
├── client.rs       — protocolo MCP (JSON-RPC 2.0 sobre stdio)
├── types.rs        — Tool, CallToolRequest, CallToolResult
└── registry.rs     — lista de tools disponíveis do securellm-mcp
```

### 3.3 — Tools disponíveis (securellm-mcp)

Prioridade para integração:
- `advanced_code_analysis` — análise estática
- `build_and_test` — rodar testes
- `run_tests` — suite específica
- `security_audit` — auditoria de segurança
- `search_knowledge` / `save_knowledge` — base de conhecimento
- `web_search` / `web_crawl` — pesquisa externa
- `ssh_execute` — execução remota

### 3.4 — Wiring

Orchestrator → MCP client → securellm-mcp processo
Tool calls publicadas no `event_bus` → SSE → TUI

**Entregável**: TUI mostra tool calls reais do securellm-mcp em tempo real

---

## Fase 4 — Matrix Integration

**Objetivo**: mover matrix para dentro do neoland e conectar métricas dos agents.

### 4.1 — Estrutura no repo

```
neoland/
└── matrix/
    ├── backend/     ← FastAPI (ranking, STF, metrics)
    ├── frontend/    ← Next.js (dashboard)
    └── flake.nix   ← integrado no flake principal
```

### 4.2 — Métricas dos agents

Matrix recebe dados de performance dos agents via:
- `POST /rank` — ranking de decisões do TechLeader
- `GET /metrics` — pipeline latency, confidence distribution, escalation rate

O control plane Rust faz `POST /rank` ao final de cada pipeline run com:
```json
{
  "session_id": "...",
  "task": "...",
  "agents": {
    "junior":      { "confidence": 0.82, "latency_ms": 312 },
    "senior":      { "escalated": false, "latency_ms": 891 },
    "tech_leader": { "decision": "accepted", "latency_ms": 445 }
  },
  "total_latency_ms": 1648
}
```

### 4.3 — TUI → Matrix dashboard

`^m` no TUI abre o dashboard do matrix no browser:
```rust
open::that("http://localhost:3000")?;  // ou via terminal emulator
```

**Entregável**: pipeline runs rastreados no matrix, dashboard mostra histórico

---

## Fase 5 — Produção e Performance

**Objetivo**: SLOs reais, load testing, HA.

- Load testing: 500 RPS, p99 < 200ms (ghz + wrk)
- Coverage: 80%+ (target atual: ~75%)
- TUI multi-line input + history navigation (↑/↓ no input)
- Task queue persistida no PostgreSQL
- Multi-session management no TUI
- HA: 3+ réplicas via Helm (Phase 5 infra já tem Helm chart)

---

## Dependências entre fases

```
Fase 1 (SSE) ──► Fase 2 (TUI) ──► Fase 3 (MCP) ──► Fase 4 (Matrix)
                                                          │
                                                    Fase 5 (Prod)
```

Fase 1 desbloqueada agora — server existente suporta adição de SSE sem breaking changes.

---

## O que NÃO é prioridade agora

- gRPC mTLS (ADR-011 pendente — não bloqueia nada)
- SOC 2 / GDPR documentation (compliance — Fase 6 original)
- Kubernetes HA multi-region (infra existe, não urgente)
- Neutron NEXUS integration (Q3 2026)

---

## Stack técnico das novas features

| Feature | Tecnologia |
|---------|-----------|
| SSE server | `axum::response::sse::Sse` + `tokio::sync::broadcast` |
| SSE client (TUI) | `reqwest` bytes_stream + `eventsource-stream` |
| MCP stdio | `tokio::process::Command` + JSON-RPC 2.0 |
| Task queue | `Vec<Task>` → PostgreSQL (sqlx) |
| Matrix backend | FastAPI existente (mover + integrar) |
| Matrix frontend | Next.js existente (mover + integrar) |
