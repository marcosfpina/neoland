# neoland-ui — Especificação Completa

**Status**: Planejamento  
**Criado**: 2026-04-06  
**Base**: `/home/kernelcore/dev/_ARCHIVE_2025_12_10/frontend/backend/`  
**Destino**: `~/master/neoland-ui/`  
**Conecta a**: neoland control plane `:3001`, DSPy pipeline `:8001`, ecosystem services

---

## Visão Geral

neoland-ui é o dashboard de controle e observabilidade do ecossistema neoland. Permite rodar tasks no pipeline multi-agent, acompanhar a execução em tempo real, inspecionar checkpoints ADR, monitorar a saúde dos serviços do ecossistema e configurar providers/thresholds.

**Não é** um chat genérico. É um painel de controle operacional para o pipeline DSPy orquestrado pelo control plane Rust.

---

## Stack

| Camada | Tecnologia | Origem |
|--------|-----------|--------|
| Frontend | Next.js 15 (App Router) | herdado do arquivo |
| UI Components | shadcn/ui + Tailwind CSS | herdado |
| Charts | Recharts | herdado |
| Icons | lucide-react | herdado |
| Real-time | socket.io-client | herdado |
| Runtime | Bun | herdado |
| API Gateway | Bun HTTP (substituindo Express/Vercel routes) | adaptado |
| WebSocket server | socket.io + Bun | adaptado |
| Auth | X-API-Key (espelha neoland) | novo |

**Dependências novas** (adicionar ao package.json):
```json
"@tanstack/react-query": "^5.x",   // data fetching + cache
"react-flow-renderer": "^11.x",    // ecosystem map (grafo)
"date-fns": "^3.x"                 // formatação de datas nos ADRs
```

---

## Arquitetura

```
Browser (Next.js SSR/CSR)
        │
        ├── HTTP ──► Bun API Gateway (:4000)
        │               ├── /api/pipeline/* ──► neoland :3001 (X-API-Key)
        │               ├── /api/sessions/*  ──► neoland :3001
        │               ├── /api/adr/*       ──► neoland :3001 / filesystem
        │               ├── /api/services/*  ──► health check todos endpoints
        │               └── /api/settings/*  ──► config local + env
        │
        └── WebSocket ──► Bun WS Gateway (:4000/ws)
                            └── polling neoland SSE → broadcast socket.io
```

O Bun server serve **tanto** a API REST quanto o WebSocket. O Next.js roda na porta padrão `:3000`, com proxy via `next.config.mjs` para `/api/*` → `:4000`.

---

## Estrutura de Pastas

```
neoland-ui/
├── NEOLAND-UI.md                        # este arquivo
├── package.json
├── next.config.mjs                      # proxy /api → :4000
├── tsconfig.json
├── tailwind.config.ts
├── components.json                      # shadcn config
│
├── app/                                 # Next.js App Router
│   ├── globals.css
│   ├── layout.tsx                       # root layout com providers
│   ├── page.tsx                         # /  → Dashboard overview
│   ├── pipeline/
│   │   ├── page.tsx                     # /pipeline → runner + live view
│   │   └── [taskId]/
│   │       └── page.tsx                 # /pipeline/:taskId → detail
│   ├── sessions/
│   │   ├── page.tsx                     # /sessions → lista de sessões
│   │   └── [sessionId]/
│   │       └── page.tsx                 # /sessions/:id → detail + ADR chain
│   ├── adr/
│   │   ├── page.tsx                     # /adr → vault pesquisável
│   │   └── [adrId]/
│   │       └── page.tsx                 # /adr/:id → ADR individual
│   ├── agents/
│   │   └── page.tsx                     # /agents → stats por agente
│   ├── services/
│   │   └── page.tsx                     # /services → ecosystem map + health
│   ├── phantom/
│   │   └── page.tsx                     # /phantom → scan results
│   └── settings/
│       └── page.tsx                     # /settings → providers, thresholds
│
├── components/
│   ├── layout/
│   │   ├── sidebar.tsx                  # navegação lateral (substituir Vercel nav)
│   │   ├── header.tsx                   # header com status do control plane
│   │   └── dashboard-layout.tsx         # wrapper geral
│   │
│   ├── pipeline/
│   │   ├── pipeline-runner.tsx          # input de task + submit (PEÇA DE ENTRADA)
│   │   ├── pipeline-live.tsx            # 4 cards em tempo real (PEÇA CENTRAL)
│   │   ├── agent-card.tsx               # card individual de cada agente
│   │   ├── agent-card-skeleton.tsx      # estado "waiting" animado
│   │   ├── decision-badge.tsx           # approve/reject/defer/escalate
│   │   └── escalation-indicator.tsx    # mostra quando Architect é ativado
│   │
│   ├── sessions/
│   │   ├── session-list.tsx             # tabela de sessões com filtro
│   │   ├── session-row.tsx              # linha de sessão com decision badge
│   │   └── session-timeline.tsx         # visual da cadeia de ADRs da sessão
│   │
│   ├── adr/
│   │   ├── adr-vault.tsx                # grid pesquisável de ADRs
│   │   ├── adr-card.tsx                 # card resumo do ADR
│   │   └── adr-viewer.tsx              # ADR renderizado completo
│   │
│   ├── agents/
│   │   ├── agent-stats-grid.tsx         # grid com métricas por agente
│   │   └── agent-history-chart.tsx      # evolução de confidence por sessão
│   │
│   ├── services/
│   │   ├── ecosystem-map.tsx            # grafo de serviços (react-flow)
│   │   ├── service-node.tsx             # nó do grafo com status dot
│   │   └── services-list.tsx            # fallback tabular do ecosystem map
│   │
│   ├── phantom/
│   │   └── phantom-results.tsx          # resultados de scan (placeholder → Phantom)
│   │
│   └── shared/
│       ├── risk-badge.tsx               # low/medium/high com cor semântica
│       ├── confidence-bar.tsx           # barra de progresso com valor float
│       ├── decision-color.ts            # mapeamento decision → cor
│       └── status-dot.tsx              # indicador de status (UP/DOWN/STUB)
│
├── hooks/
│   ├── use-pipeline.ts                  # react-query: rodar e acompanhar tasks
│   ├── use-session.ts                   # react-query: buscar sessão/ADR chain
│   ├── use-pipeline-live.ts             # socket.io: subscribe a task live
│   ├── use-services-health.ts           # polling: saúde dos serviços
│   └── use-adr-search.ts               # pesquisa no vault de ADRs
│
├── lib/
│   ├── api-client.ts                    # fetch wrapper com X-API-Key header
│   ├── ws-client.ts                     # socket.io client singleton
│   ├── types.ts                         # TypeScript: espelha schemas Rust/Python
│   └── utils.ts                         # cn(), formatDate(), truncate()
│
└── src/                                 # Bun API Gateway
    ├── server.ts                        # entry point: HTTP + WebSocket
    ├── config.ts                        # env vars, endpoints do ecosystem
    ├── middleware/
    │   ├── auth.ts                      # valida X-API-Key da UI
    │   ├── cors.ts                      # CORS config
    │   └── rate-limit.ts               # rate limiting da UI
    ├── routes/
    │   ├── pipeline.ts                  # POST /api/pipeline/run
    │   │                                # GET  /api/pipeline/:taskId
    │   ├── sessions.ts                  # GET  /api/sessions
    │   │                                # GET  /api/sessions/:sessionId
    │   ├── adr.ts                       # GET  /api/adr
    │   │                                # GET  /api/adr/:adrId
    │   ├── services.ts                  # GET  /api/services/health
    │   └── settings.ts                  # GET/PUT /api/settings
    └── ws/
        └── pipeline-relay.ts            # polling neoland → broadcast socket.io
```

---

## Páginas e UX

### `/` — Dashboard Overview

Visão executiva do estado do ecossistema.

**Seções:**
- **Stats bar** (4 cards): Tasks hoje / Sessões ativas / ADRs gerados / Taxa de approve
- **Pipeline recente**: últimas 5 tasks com decision badge e timestamp
- **Services health bar**: status inline de cada serviço (UP/STUB/DOWN)
- **ADR recente**: último ADR gerado com título + status + link

```
┌────────────────────────────────────────────────────────────────────┐
│  neoland                                          ● control plane  │
├──────────┬─────────────────────────────────────────────────────────┤
│          │  Tasks hoje   Sessões ativas   ADRs gerados  Approve %  │
│ Pipeline │  ──────────   ─────────────   ────────────  ─────────  │
│ Sessions │     12             3               8           67%      │
│ Agents   │                                                         │
│ Services │  Pipeline Recente                                       │
│ ADR Vault│  ┌────────────────────────────────────────────────────┐ │
│ Phantom  │  │ refactor auth middleware    [APPROVE] 14:32   →    │ │
│ Settings │  │ add pgvector index          [DEFER]   13:11   →    │ │
│          │  │ integrate cerebro reranker  [APPROVE] 11:05   →    │ │
│          │  └────────────────────────────────────────────────────┘ │
│          │                                                         │
│          │  Services        ADR Recente                           │
│          │  ● DSPy :8001    ADR-2026-04-06 — Migrate auth...     │
│          │  ● Neotron STUB  Status: ACCEPTED                      │
│          │  ○ Phantom DOWN  [Ver ADR →]                           │
└──────────┴─────────────────────────────────────────────────────────┘
```

---

### `/pipeline` — Pipeline Runner + Live View

Coração da aplicação. Roda tasks e acompanha a execução em tempo real.

**Layout:**
- Topo: formulário de input (task, session_id opcional, start_from, provider override)
- Abaixo: 4 agent cards em linha

**Agent Card — Estados:**

| Estado | Visual |
|--------|--------|
| `waiting` | skeleton cinza animado |
| `running` | borda pulsando, spinner no título |
| `done` | borda sólida na cor do risco |
| `skipped` | card transparente com "— não ativado" |

**Formulário de input:**
```
Task: [__________________________________]
      Descreva o que o pipeline deve analisar

Session ID: [auto ▼]   Start from: [Junior ▼]   Provider: [default ▼]
RAG context: [_____________________________] (opcional)

                                      [▶ Executar Pipeline]
```

**Live view — 4 cards:**
```
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│ JUNIOR           │  │ SENIOR           │  │ ARCHITECT        │  │ TECH-LEADER      │
│ ◉ executando...  │  │ ○ aguardando     │  │ — não ativado    │  │ ○ aguardando     │
├──────────────────┤  └──────────────────┘  └──────────────────┘  └──────────────────┘
│ Hypothesis       │
│ Lorem ipsum...   │
│                  │
│ Confidence       │
│ ████████░░ 0.82  │
│                  │
│ Risk             │
│ [■ MEDIUM]       │
│                  │
│ Unknowns (2)     │
│ • auth key scope │
│ • rotation freq  │
│                  │
│ Innovation (1)   │
│ • async refresh  │
└──────────────────┘
```

Quando Senior termina:
```
┌──────────────────┐  ┌──────────────────┐
│ JUNIOR ✓         │  │ SENIOR           │
│ (recolhido)      │  │ ◉ executando...  │
│ [Expandir ▼]     │  ├──────────────────┤
└──────────────────┘  │ Valid Parts (2)  │
                      │ • async approach │
                      │ • interface sep. │
                      │                  │
                      │ Rejected (1)     │
                      │ • inline refresh │
                      │                  │
                      │ Escalate? [SIM ▲]│
                      └──────────────────┘
```

Quando `escalate_to_architect = true`, o card do Architect se expande automaticamente.

TechLeader exibe decision como badge grande centralizado abaixo dos 4 cards:
```
┌──────────────────────────────────────────────────────────────┐
│                    ✓  APROVADO                               │
│  "Migrate auth middleware to RS256 with key rotation"        │
│  Rationale: Abordagem sound, riscos cobertos pelo Senior...  │
│                                                              │
│  Action Items:                                               │
│  □ Implement RS256 signing                                   │
│  □ Definir key rotation policy                              │
│  □ Load test 10k RPS                                        │
│                                                              │
│  Session summary salvo para próxima sessão →                │
│  [Ver ADR completo]                    [Nova task na sessão] │
└──────────────────────────────────────────────────────────────┘
```

---

### `/sessions` — Histórico de Sessões

Tabela de sessões com filtros e visualização de cadeia de ADRs.

**Filtros:** data, decision (approve/reject/defer/escalate), risk level, sessão ativa/encerrada

**Colunas:** Session ID (truncado) / Tasks / Última decision / Último ADR / Atividade / Ações

**Detail `/sessions/:id`:**
- Header: Session ID + task count + duração + status
- Timeline vertical de tasks na sessão, cada uma com decision badge
- ADR chain: lista de ADRs gerados ordenados por timestamp

---

### `/adr` — ADR Vault

Repositório pesquisável de todos os checkpoints ADR gerados.

**Pesquisa:** full-text no título, contexto e decision
**Filtros:** status (accepted/rejected/deferred), data, session, risk level

**Grid de ADR cards** (3 colunas):
```
┌─────────────────────────────┐  ┌─────────────────────────────┐
│ ADR-2026-04-06              │  │ ADR-2026-04-05              │
│ Migrate auth to RS256        │  │ Add pgvector HNSW index     │
│ [ACCEPTED] Session: a3f9     │  │ [DEFERRED] Session: b1e2    │
│ 2026-04-06 14:32            │  │ 2026-04-05 10:15            │
│ Jr: 0.82 Sr: 0.61 TL: 0.94 │  │ Jr: 0.71 Sr: 0.55 TL: 0.88 │
│                  [Ver ADR →] │  │                  [Ver ADR →] │
└─────────────────────────────┘  └─────────────────────────────┘
```

**Detail `/adr/:id`:**
- ADR renderizado completo (não JSON cru)
- Seções: Context / Decision / Action Items / Session Summary
- Confidence timeline: gráfico de barras Jr/Sr/TL
- Pipeline completo recolhível (JSON formatado)
- Botão: "Usar session summary como contexto → nova task"

---

### `/agents` — Stats por Agente

Métricas de performance de cada agente ao longo do tempo.

**Grid de 4 cards** (um por agente):
- Nome + papel + descrição do perfil
- Total de execuções / Média de confidence / Taxa de escalation (Senior) / Taxa de skip (Architect)
- Gráfico de linha: confidence ao longo das últimas N sessões

**Insights automáticos:**
- "Junior confidence abaixo de 0.4 nas últimas 3 sessões → possível degradação"
- "Senior escalou para Architect 80% das vezes esta semana → revisar thresholds"

---

### `/services` — Ecosystem Map + Health

Visualização do ecossistema como grafo interativo.

**Grafo (react-flow):**
```
     [neoland-ui]
          │
    [control plane :3001]
          │
    ┌─────┼──────────┐
    │     │          │
[:8001] [:8010]  [:8016]
 DSPy  Neotron  Cerebro
  ●UP   ●STUB    ●STUB
          │
       [:8008]
       Phantom
        ○DOWN
```

Cada nó tem:
- Nome + porta
- Status dot (UP verde / STUB amarelo / DOWN vermelho)
- Última verificação (tempo relativo)
- Clique → painel lateral com detalhes (versão, uptime, config)

**Abaixo do grafo:** tabela de health checks com latência e última resposta.

---

### `/phantom` — Scan Results

Placeholder funcional. Mostra estado do serviço Phantom e resultados quando ativo.

**Quando DOWN/STUB:**
```
┌──────────────────────────────────────────────────────────────┐
│  Phantom — Security Scanner                         ○ STUB   │
│                                                              │
│  Phantom não está ativo. Quando habilitado, exibirá:        │
│  • Scans de outputs do pipeline                             │
│  • Detecções YARA (secrets, PII, credenciais)               │
│  • Histórico de bloqueios                                    │
│                                                              │
│  [Ver configuração →]   [Docs do Phantom →]                 │
└──────────────────────────────────────────────────────────────┘
```

**Quando UP:** tabela de scans com match count, threat level, timestamp e payload mascarado.

---

### `/settings` — Configuração

Controle total dos parâmetros do ecossistema.

**Seções:**

**1. Control Plane**
- URL do neoland (default: `http://localhost:3001`)
- API Key (campo de senha, mascarado)
- Timeout do pipeline (segundos)
- Botão: Testar conexão

**2. Pipeline Thresholds**
- Junior: confidence warn threshold (slider 0.0–1.0, default 0.4)
- TechLeader: defer TTL (horas, default 24)
- RAG top-k (número, default 5)

**3. Providers** (readonly, vindo do control plane)
- Lista de providers disponíveis com status de habilitação
- Link para configurar no control plane

**4. Ecosystem Services**
- Tabela de serviços com URL configurável
- Toggle de habilitação
- Botão: Health check individual

---

## Modelos TypeScript

Espelham os schemas Rust/Python. Definidos em `lib/types.ts`.

```typescript
// Enums
type RiskLevel = "low" | "medium" | "high"
type AgentDecision = "approve" | "reject" | "defer" | "escalate"
type ServiceStatus = "up" | "stub" | "down"
type StartFrom = "junior" | "senior" | "architect" | "tech_leader"

// Pipeline
interface PipelineRunRequest {
  task: string
  session_id?: string          // UUID — se não informado, nova sessão
  start_from?: StartFrom
  rag_context?: string
}

interface JuniorOutput {
  hypothesis: string
  confidence: number           // 0.0–1.0
  risk_level: RiskLevel
  unknowns: string[]
  innovation_vectors: string[]
}

interface SeniorOutput {
  valid_parts: string[]
  rejected_parts: string[]
  risk_assessment: string
  escalate_to_architect: boolean
  refined_hypothesis: string
}

interface ArchitectOutput {
  structural_soundness: boolean
  composability_score: number  // 0.0–1.0
  long_term_concerns: string[]
  recommended_structure: string
  blockers: string[]
}

interface TechLeaderOutput {
  decision: AgentDecision
  rationale: string
  action_items: string[]
  adr_title: string
  session_summary: string
}

interface PipelineResult {
  task_id: string              // UUID
  session_id: string           // UUID
  timestamp: string            // ISO 8601
  junior: JuniorOutput
  senior: SeniorOutput
  architect?: ArchitectOutput  // null se não escalado
  tech_leader: TechLeaderOutput
  checkpoint_path: string
}

// Pipeline live (via WebSocket)
interface PipelineEvent {
  type: "agent_start" | "agent_done" | "pipeline_done" | "pipeline_error"
  task_id: string
  agent?: "junior" | "senior" | "architect" | "tech_leader"
  data?: Partial<PipelineResult>
  error?: string
}

// Sessions
interface SessionState {
  session_id: string
  task_count: number
  last_activity: string
  last_decision?: AgentDecision
  active: boolean
}

interface AgentSession {
  id: string
  session_id: string
  task_id: string
  task: string
  requester_role: string
  junior_output?: JuniorOutput
  senior_output?: SeniorOutput
  architect_output?: ArchitectOutput
  tech_leader_output?: TechLeaderOutput
  final_decision?: AgentDecision
  checkpoint_path?: string
  created_at: string
  completed_at?: string
}

// ADR
interface ADRDocument {
  adr_id: string               // "ADR-<session_id>-<timestamp>"
  title: string
  status: "accepted" | "rejected" | "deferred"
  context: string
  decision: string
  action_items: string[]
  session_summary: string
  full_pipeline: {
    junior: JuniorOutput
    senior: SeniorOutput
    architect?: ArchitectOutput
    tech_leader: TechLeaderOutput
  }
}

// Services
interface ServiceHealth {
  name: string
  url: string
  status: ServiceStatus
  latency_ms?: number
  last_checked: string
  version?: string
}

interface EcosystemHealth {
  services: ServiceHealth[]
  checked_at: string
}
```

---

## API Gateway — Rotas

Todas as rotas da UI passam pelo Bun gateway em `:4000`.

### Pipeline

```
POST /api/pipeline/run
  Body: PipelineRunRequest
  Header: X-API-Key
  → forward para neoland :3001/v1/agents/task
  ← PipelineResult

GET /api/pipeline/:taskId
  → neoland :3001/v1/agents/session/:taskId
  ← AgentSession
```

### Sessions

```
GET /api/sessions
  Query: ?active=true&limit=20&offset=0
  → neoland :3001 (query PostgreSQL)
  ← { sessions: SessionState[], total: number }

GET /api/sessions/:sessionId
  → neoland :3001
  ← { session: SessionState, tasks: AgentSession[] }
```

### ADR

```
GET /api/adr
  Query: ?q=<search>&status=accepted&limit=20&offset=0
  → neoland :3001 / filesystem /var/lib/neoland/checkpoints/adr/
  ← { adrs: ADRDocument[], total: number }

GET /api/adr/:adrId
  → filesystem (read JSON file)
  ← ADRDocument
```

### Services

```
GET /api/services/health
  → health check paralelo de todos os endpoints do ecosystem
  ← EcosystemHealth
```

### Settings

```
GET /api/settings
  → config local (env vars + defaults)
  ← { control_plane_url, pipeline_timeout, thresholds, services }

PUT /api/settings
  Body: Partial<Settings>
  → atualiza config local (sem restart)
  ← { ok: true }
```

---

## WebSocket — Protocolo

O Bun gateway expõe um WebSocket em `ws://localhost:4000/ws`.

### Subscrição

O cliente emite:
```json
{ "type": "subscribe_pipeline", "task_id": "<uuid>" }
```

O gateway faz polling no neoland (SSE ou polling REST a cada 500ms) e emite para o socket:

```json
{ "type": "agent_start",    "task_id": "...", "agent": "junior" }
{ "type": "agent_done",     "task_id": "...", "agent": "junior", "data": { ...JuniorOutput } }
{ "type": "agent_start",    "task_id": "...", "agent": "senior" }
{ "type": "agent_done",     "task_id": "...", "agent": "senior", "data": { ...SeniorOutput } }
{ "type": "architect_skip", "task_id": "..." }
{ "type": "agent_start",    "task_id": "...", "agent": "tech_leader" }
{ "type": "agent_done",     "task_id": "...", "agent": "tech_leader", "data": { ...TechLeaderOutput } }
{ "type": "pipeline_done",  "task_id": "...", "data": { ...PipelineResult } }
```

Em caso de erro:
```json
{ "type": "pipeline_error", "task_id": "...", "error": "timeout after 120s" }
```

---

## Variáveis de Ambiente

```bash
# Bun API Gateway
NEOLAND_CONTROL_PLANE_URL=http://localhost:3001
NEOLAND_API_KEY=<mesma chave do control plane>
NEOLAND_UI_PORT=4000

# DSPy Pipeline
NEOLAND_DSPY_URL=http://localhost:8001

# Ecosystem services
NEOTRON_URL=http://localhost:8010
CEREBRO_URL=http://localhost:8016
PHANTOM_URL=http://localhost:8008
ML_OPS_API_URL=http://localhost:8080

# ADR filesystem (quando acessar direto)
NEOLAND_CHECKPOINT_DIR=/var/lib/neoland/checkpoints/adr

# Next.js
NEXT_PUBLIC_WS_URL=ws://localhost:4000/ws
NEXT_PUBLIC_API_URL=http://localhost:4000

# Auth (UI → gateway)
UI_API_KEY=<chave para proteger a UI)
```

---

## Fases de Implementação

### Fase 1 — Setup e Base [ ]
- [ ] Copiar `_ARCHIVE_2025_12_10/frontend/backend/` → `~/master/neoland-ui/`
- [ ] Remover arquivos Vercel-específicos (vercel routes, vercel client, deployment pages)
- [ ] Atualizar `package.json`: nome, dependências novas (@tanstack/react-query, react-flow-renderer, date-fns)
- [ ] Criar `lib/types.ts` com todos os TypeScript types
- [ ] Criar `lib/api-client.ts` com X-API-Key header
- [ ] Atualizar `next.config.mjs` com proxy `/api → :4000`
- [ ] Criar `.env.local` com variáveis de ambiente

### Fase 2 — Bun API Gateway [ ]
- [ ] `src/server.ts` — Bun HTTP + WebSocket server
- [ ] `src/config.ts` — leitura de env vars
- [ ] `src/middleware/auth.ts` — validação UI_API_KEY
- [ ] `src/routes/pipeline.ts` — forward para control plane
- [ ] `src/routes/sessions.ts` — buscar sessões
- [ ] `src/routes/adr.ts` — ler ADRs do filesystem + API
- [ ] `src/routes/services.ts` — health check paralelo
- [ ] `src/routes/settings.ts` — config local
- [ ] `src/ws/pipeline-relay.ts` — polling → socket.io broadcast

### Fase 3 — Layout e Navegação [ ]
- [ ] `components/layout/sidebar.tsx` — nova navegação neoland
- [ ] `components/layout/header.tsx` — status control plane + nome da sessão ativa
- [ ] `components/layout/dashboard-layout.tsx` — wrapper atualizado
- [ ] `app/layout.tsx` — providers (react-query, socket.io, theme)
- [ ] `components/shared/risk-badge.tsx`
- [ ] `components/shared/confidence-bar.tsx`
- [ ] `components/shared/decision-color.ts`
- [ ] `components/shared/status-dot.tsx`

### Fase 4 — Pipeline Runner + Live View (PEÇA CENTRAL) [ ]
- [ ] `hooks/use-pipeline-live.ts` — socket.io subscription
- [ ] `hooks/use-pipeline.ts` — react-query mutation para submit
- [ ] `components/pipeline/pipeline-runner.tsx` — formulário de input
- [ ] `components/pipeline/agent-card.tsx` — card por agente com todos os campos
- [ ] `components/pipeline/agent-card-skeleton.tsx` — estado waiting
- [ ] `components/pipeline/decision-badge.tsx` — badge de decision final
- [ ] `components/pipeline/escalation-indicator.tsx` — indicator quando Architect ativa
- [ ] `components/pipeline/pipeline-live.tsx` — layout 4 cards + decision
- [ ] `app/pipeline/page.tsx` — página completa
- [ ] `app/pipeline/[taskId]/page.tsx` — detail de task histórica

### Fase 5 — Sessions [ ]
- [ ] `hooks/use-session.ts`
- [ ] `components/sessions/session-list.tsx`
- [ ] `components/sessions/session-row.tsx`
- [ ] `components/sessions/session-timeline.tsx`
- [ ] `app/sessions/page.tsx`
- [ ] `app/sessions/[sessionId]/page.tsx`

### Fase 6 — ADR Vault [ ]
- [ ] `hooks/use-adr-search.ts`
- [ ] `components/adr/adr-card.tsx`
- [ ] `components/adr/adr-vault.tsx`
- [ ] `components/adr/adr-viewer.tsx`
- [ ] `app/adr/page.tsx`
- [ ] `app/adr/[adrId]/page.tsx`

### Fase 7 — Services + Agents + Dashboard [ ]
- [ ] `hooks/use-services-health.ts`
- [ ] `components/services/ecosystem-map.tsx` (react-flow)
- [ ] `components/services/service-node.tsx`
- [ ] `app/services/page.tsx`
- [ ] `components/agents/agent-stats-grid.tsx`
- [ ] `components/agents/agent-history-chart.tsx`
- [ ] `app/agents/page.tsx`
- [ ] `app/page.tsx` — dashboard overview final

### Fase 8 — Phantom + Settings + Polish [ ]
- [ ] `components/phantom/phantom-results.tsx`
- [ ] `app/phantom/page.tsx`
- [ ] `app/settings/page.tsx`
- [ ] Temas (dark mode — manter o que o shadcn já tem)
- [ ] Responsividade mobile (sidebar colapsável)
- [ ] Error boundaries em cada página
- [ ] Loading states consistentes

---

## NixOS Module (futuro)

Após implementação e validação local, adicionar módulo NixOS em `/etc/nixos/modules/ai/neoland-ui/default.nix`:

```nix
# services.neoland-ui.enable = true
# services.neoland-ui.port = 4000
# services.neoland-ui.controlPlaneUrl = "http://localhost:3001"
# services.neoland-ui.apiKeyFile = config.sops.secrets.neoland-api-key.path
```

---

## Decisões de Design

| Decisão | Escolha | Razão |
|---------|---------|-------|
| Framework | Next.js 15 (herdado) | Já existia, App Router maduro |
| Runtime | Bun | Herdado, rápido para API gateway |
| Real-time | socket.io (herdado) | Já tinha infra WebSocket funcional |
| State | react-query | Cache automático, sem redux overhead |
| Grafo | react-flow | Melhor lib para grafos interativos em React |
| Auth UI→gateway | UI_API_KEY separado | Não expor a X-API-Key do control plane |
| ADR storage | Filesystem + API | ADRs são JSON files; controle plane já serve |
| Streaming | polling 500ms → socket.io | neoland não tem SSE nativo ainda; simples e funciona |

---

## Pendências / Decisões Abertas

- [ ] **Auth**: A UI vai ter login próprio (next-auth) ou acesso direto por API Key?
- [ ] **ADR search**: Full-text no filesystem (ripgrep) ou indexar no PostgreSQL?
- [ ] **Streaming nativo**: Quando neoland tiver SSE, migrar polling → SSE direto
- [ ] **Mobile**: Prioridade ou só desktop por enquanto?
- [ ] **Phantom page**: Placeholder funcional agora, integração real quando Phantom implementado
- [ ] **Multi-user**: Sessões são por usuário ou compartilhadas no dashboard?

---

*Última atualização: 2026-04-06 — Fase de planejamento completo, implementação não iniciada.*
