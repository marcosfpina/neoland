# NEOLAND Production Readiness Progress

**Last Updated**: 2026-05-16
**Overall Progress**: Ciclos 0–4 fechados. Ciclo 5 (Communication Gaps) concluído. Gap Register aberto com 13 itens da varredura proativa.
**Rust tests**: 226 passing, 17 ignored | **Python tests**: 24 contract (sem LLM)

## Ciclo 4 — Agent Workstation (ROADMAP Fases 2–5A) ✅ (fechado 2026-04-27)

| Fase | Commit | Status | Entregável |
|------|--------|--------|-----------|
| 2 — TUI redesign | `a14c057` | ✅ | Agent workstation: task queue, pipeline panel ao vivo, tool calls, SSE consumer, keybindings (`^t ^p ^m ^x`) |
| 3 — MCP stdio | `841ddd6` | ✅ | `src/mcp/` — JSON-RPC 2.0 stdio client, `McpRegistry`, `search_knowledge` pré-pipeline + `save_knowledge` pós-ADR, 7 testes |
| 4 — Matrix | `1ca938a` | ✅ | `src/matrix/` — `MatrixClient`, `POST /pipeline/metrics` + `GET /pipeline/history` no backend Python, dashboard Next.js (`/pipeline-history`), 5 testes |
| 5A — Thinking transparente | `75cc14f` | ✅ | `AgentEvent::StageOutput` via SSE → `PipelineStage.output` → TUI renderiza 2 linhas com `┊` dim por stage; `input_history` + `↑/↓` shell-like; `Shift+Enter` multi-line; 8 novos testes |

### Arquivos-chave Ciclo 4

| Módulo | Arquivo | Descrição |
|--------|---------|-----------|
| MCP | `src/mcp/{mod,client,registry,types}.rs` | Stdio client JSON-RPC 2.0 + tool registry |
| Matrix | `src/matrix/{mod,client}.rs` | HTTP client fire-and-forget para matrix backend |
| TUI app | `src/tui/app.rs` | `PipelineStage.output`, `input_history`, `history_prev/next/commit`, `set_stage_output` |
| TUI render | `src/tui/ui.rs` | Stage output `┊` dim rendering sob cada stage row |
| TUI loop | `src/tui/mod.rs` | `AgentStreamEvent::StageOutput`, `stage_output` SSE parse, history keybindings |
| Orchestrator | `src/agents/orchestrator.rs` | Publica `StageOutput` (junior hypothesis, senior risk, tech-leader rationale) |
| Events | `src/agents/events.rs` | `AgentEvent::StageOutput { session_id, stage, content }` |
| Matrix backend | `matrix/backend/src/ranking/main.py` | `POST /pipeline/metrics`, `GET /pipeline/history?limit=N` |
| Matrix frontend | `matrix/frontend/app/pipeline-history/page.tsx` | Dashboard com summary cards + run table |
| Config | `src/config.rs` | `McpConfig` + `MatrixConfig` com env overrides |

---

## Ciclo 5 — Communication Gaps ✅ (fechado 2026-05-16)

### Commits
| Hash | Mensagem |
|------|---------|
| `ccbad0a` | `chore: consolidate communication baseline` |
| `89ef81e` | `fix: close agent communication gaps` |

### Fixes implementados (7)

| # | Arquivo | Problema | Fix |
|---|---------|---------|-----|
| 1 | `src/agents/orchestrator.rs` | `resolve_breakpoint()` nunca publicava `AgentEvent::BreakpointResolved` — TUI ficava em `WaitingForBreakpoint` para sempre | Adiciona publicação do evento após send do oneshot; `pending_breakpoints` guarda `(tool_name, tx)` |
| 2 | `src/agents/events.rs` | Sem teste para serialização de `BreakpointResolved` | Adicionado `test_breakpoint_resolved_serializes` |
| 3 | `src/server/mod.rs` | `register_breakpoint()` não recebia `tool_name` | Atualizado call site para passar `req.tool_name.clone()` |
| 4 | `src/tui/mod.rs` | `parse_sse_event()` descartava `"breakpoint_resolved"` e `"pipeline_started"` silenciosamente | Adicionados dois arms; `BreakpointResolved` transiciona task `WaitingForBreakpoint → Running` |
| 5 | `src/tui/mod.rs` | Buffer SSE descartado ao fechar a conexão — `pipeline_done` podia ser perdido | Flush do `buf` remanescente após fim do loop |
| 6 | `src/tui/mod.rs` | Falhas de steering/breakpoint iam para `eprintln!` invisível | Erros roteados para `AgentStreamEvent::PipelineError` (visível no painel de output) |
| 7 | `src/health.rs` | `check_llm_health()` retornava sempre `Healthy` sem probar o gateway | Extrai `check_llm_health_with_url()` com HTTP GET + timeout 2 s; `Degraded` em não-2xx/timeout |

### Frontend & Nix (baseline `ccbad0a`)

| Arquivo | Mudança |
|---------|---------|
| `matrix/frontend/app/api/neoland/pipeline/route.ts` | ADR como source of truth (não Matrix backend) |
| `matrix/frontend/app/api/neoland/stats/route.ts` | Vocabulário `"approve"` alinhado com backend Rust |
| `matrix/frontend/app/sessions/page.tsx` | Session browser funcional via `getSessions(24)` |
| `matrix/frontend/lib/neoland/api.ts` | Header `X-API-Key` habilitado, tipo de retorno correto |
| `matrix/frontend/lib/neoland/server.ts` | Dev API key default adicionado |
| `matrix/frontend/app/pipeline/page.tsx` | Stub "Awaiting Backend Contract" substituído por `<PipelineRunner>` real |
| `modules/applications/*.nix` | Porta llama.cpp 5001 → 8081 alinhada em todos os 3 módulos |

---

## Ciclo 1 — IPC + NATS + adr-ledger + Phantom ✅

| Fase | Status | Entregável |
|------|--------|-----------|
| A — mmap IPC | ✅ | `flags.rs` + `mmap.rs` + `ipc/flags.py` — 13 testes, layout 64 bytes 1 cache line |
| B — NATS publisher | ✅ | `src/agents/nats.rs` — publisher via spectre-events, `neoland.task.completed.v1` + `neoland.task.escalated.v1`, 5 testes |
| C — adr-ledger | ✅ | `adr-ledger/crates/ledger-subscriber/` — `Signer` trait + `FileKeySigner` (sops) + `TrezorSigner` (stub) + `MerkleStore` (PG) + JetStream subscriber at-least-once, migration 003, 15 testes |
| D — Phantom | ✅ | `neoland.pipeline.output.v1` → `phantom/nats/neoland_scanner.py` → `phantom.pipeline.scan.v1` (sentiment VADER + risk keywords) |
| NixOS Modules | ✅ | `control-plane.nix` + `dspy-pipeline.nix` + `ledger-subscriber.nix` — systemd hardened, sops-nix secrets |
| JetStream upgrade | ✅ | `jetstream.rs` — stream `NEOLAND_EVENTS`, pull consumer `ledger-sub`, explicit ack, at-least-once, max_deliver=5 |

---

## Ciclo 2 — Operational Stack ✅ (fechado 2026-04-26)

| Fase | Status | Entregável |
|------|--------|-----------|
| 4.1 — Metrics | ✅ | Agent pipeline Prometheus metrics (counters + histograms) wired no orchestrator |
| 4.2 — cargo-audit | ✅ | `cargo-audit` adicionado ao devShell — supply-chain CVE scanning |
| 4.3 — Coding skill | ✅ | `neoland-agents` package metadata + skill coding partner |
| 4.4 — NKey + ACL | ✅ | NKey SOPS-encrypted, NATS ACL (publish `neoland.>` only) |
| 4.5 — Cross-stack | ✅ | Neotron: Synapse ↔ Cortex per-agent embeddings, stress tests BASTION + SENTINEL |
| 4.6 — Owasaka tests | ✅ | Event pipeline tests (12) + API server tests (5) — cobertura cross-stack |
| 4.7 — EDR rules | ✅ | 3 SIGMA + 6 YARA rules neoland-specific (sentinel/sigma + sentinel/yara) |

## Ciclo 3 — Quality & Observability ✅ (2026-04-26)

| Fase | Status | Entregável |
|------|--------|-----------|
| TUI rewrite | ✅ | `src/tui/{mod,ui,app,events}.rs` — async `tokio::select!`, Tokyo Night, cursor UTF-8-safe, word-nav, braille spinner, auto-scroll |
| TUI unit tests | ✅ | 29 testes em `src/tui/app.rs` — cursor (ASCII + multibyte), word-nav, insert/delete, auto_scroll por role, presets |
| Agent tracing | ✅ | `#[instrument]` com `skip()` + `fields()` em `orchestrator.rs`, `client.rs`, `session.rs` — spans OpenTelemetry |
| Python contract tests | ✅ | 24 testes `@pytest.mark.contract` em `agents/tests/test_contracts.py` — AgentFlags IPC + todos schemas Pydantic |

### Test counts across stack (2026-05-16)

| Repo | Tests | Notes |
|------|-------|-------|
| neoland (Rust) | **226** | lib unit tests (243 total com ignored) — +5 vs 2026-04-27 (breakpoint_resolved + llm_health tests) |
| neoland (Python) | 24 | contract tests sem LLM (`pytest -m contract`) |
| adr-ledger | 15 | ledger-subscriber, MerkleStore, JetStream |
| owasaka | 17 | 12 pipeline + 5 API |
| neotron | ~40+ | cortex, synapse, bastion, sentinel |
| sentinel | suite | E2E + chaos + performance |

---

## Ciclo 0 — Core funcionando ✅ (2026-04-07)

Pipeline multi-agent DSPy integrado ao control plane Rust. Ciclo 0 fechado.

| Componente | Status |
|-----------|--------|
| Contratos DSPy (4 signatures) | ✅ |
| Python pipeline (FastAPI :8001) | ✅ |
| Rust control plane (`src/agents/`) | ✅ |
| Rotas HTTP (`/v1/agents/*`) | ✅ |
| Testes de contrato (9 testes) | ✅ |
| Testes de sessão (4 testes, requer PG) | ✅ |
| Testes de integração (4 testes, requer tudo) | ✅ |
| Python 3.13 no devShell | ✅ |
| ADR-019 documentado | ✅ |

---

---

## Phase Completion Status

### ✅ Phase 0: Foundation & Stabilization (Complete)
**Status**: 100% | **Duration**: 2 weeks | **Commits**: 1

- ✅ Stabilized Rust edition (2024 → 2021)
- ✅ Fixed path dependencies
- ✅ Enabled tests in Nix build (`doCheck = true`)
- ✅ Eliminated panic risks (replaced `.expect()` with proper error handling)

**Key Files Modified**:
- Cargo.toml
- flake.nix
- src/nlp.rs
- src/ml_offload/client.rs

---

### ✅ Phase 1: Security Hardening (Complete)
**Status**: 100% | **Duration**: 3 weeks | **Commits**: 5

#### 1.1: Authentication & Authorization ✅
- ✅ REST API authentication (X-API-Key header validation)
- ✅ gRPC mTLS configuration (certificate-based)
- ✅ RBAC implementation (Admin, User, ReadOnly roles)
- ✅ Failed authentication tracking with auto-alerts

**Implementation**:
- `src/auth.rs`: 400+ lines, 11 unit tests
- `src/server/mod.rs`: Auth middleware integration
- ADR-011: Authentication strategy documentation

#### 1.2: Secrets Management ✅
- ✅ HashiCorp Vault integration
- ✅ Environment variable fallback with caching (30s TTL)
- ✅ Audit logging for secret access
- ✅ Cache performance optimization (<1ms cache hits)

**Implementation**:
- `src/secrets.rs`: 350+ lines, 9 unit tests
- Vault client integration
- ADR-012: Secrets management strategy

#### 1.3: Audit Logging ✅
- ✅ Structured JSON audit events
- ✅ Comprehensive action taxonomy (11 actions)
- ✅ Automatic severity classification
- ✅ Real-time security alerting
- ✅ File persistence with rotation

**Implementation**:
- `src/audit.rs`: 500+ lines, 15 unit tests
- Failed authentication tracking
- ADR-013: Audit logging architecture

#### 1.4: Rate Limiting & Input Validation ✅
- ✅ Rate limiting (100 req/min per user/IP)
- ✅ In-memory sliding window implementation
- ✅ Multi-layer input validation (request size, prompt size, sanitization)
- ✅ Security-focused validation middleware

**Implementation**:
- `src/validation.rs`: 440 lines, comprehensive validation
- `src/server/mod.rs`: Rate limiting middleware
- ADR-014: Rate limiting & input validation strategy

**Security Score**: 95/100 ✅

---

### ✅ Phase 2: Testing & Quality Assurance (68% Complete)
**Status**: 68% | **Duration**: 3 weeks | **Commits**: 3

#### 2.1: Unit Testing ✅
- ✅ Test utilities module (`src/test_utils.rs`)
- ✅ 55 unit tests passing (was 30 → 55)
- ✅ 60-65% code coverage (target: 70%+)
- ✅ Comprehensive auth.rs tests (11 tests)
- ✅ Comprehensive audit.rs tests (15 tests)
- ✅ Mock factories for all major components

**Test Coverage**:
- `src/auth.rs`: 11 tests (API key validation, RBAC, revocation)
- `src/audit.rs`: 15 tests (event logging, severity, alerts)
- `src/secrets.rs`: 9 tests (Vault, caching, fallback)
- `src/validation.rs`: Tests for input validation

#### 2.2: Integration Tests ✅
- ✅ REST API integration tests (10 tests in `tests/rest_api_test.rs`)
- ✅ gRPC integration tests (8 tests in `tests/grpc_integration_test.rs`)
- ✅ Server lifecycle management (spawn/cleanup)
- ✅ Authentication flow tests
- ✅ End-to-end request/response tests

**Total Tests**: 73 tests (55 unit + 18 integration)

#### 2.3: E2E & Security Testing (Pending)
- ⏳ TUI automation tests
- ⏳ Security fuzzing
- ⏳ Dependency vulnerability scanning (cargo-audit)
- ⏳ Penetration testing

#### 2.4: Load Testing (Pending)
- ⏳ gRPC load tests (ghz)
- ⏳ Target: 500 RPS sustained, p99 <200ms
- ⏳ Performance benchmarks

**Testing Score**: 68/100 ⏳

---

### ✅ Phase 3: CI/CD Pipeline (Complete)
**Status**: 100% | **Duration**: 1 week | **Commits**: 2

#### 3.1: GitHub Actions Workflows ✅
- ✅ **test.yml**: Automated testing on push/PR
- ✅ **lint.yml**: Code quality checks (fmt, clippy, check)
- ✅ **build.yml**: Release and dev binary builds
- ✅ Cachix integration for Nix caching (~80% faster builds)
- ✅ Parallel workflow execution

#### 3.2: Pre-commit Hooks ✅
- ✅ `.githooks/pre-commit`: Local quality gates
- ✅ Automatic Nix environment detection
- ✅ Format check (cargo fmt)
- ✅ Clippy linting
- ✅ Unit test execution
- ✅ Compilation verification

#### 3.3: Code Quality Configuration ✅
- ✅ `rustfmt.toml`: Formatting rules (max_width: 100, imports_granularity: Crate)
- ✅ `clippy.toml`: Linting thresholds (cognitive-complexity: 30)
- ✅ `scripts/setup-hooks.sh`: Developer onboarding script

#### 3.4: Documentation ✅
- ✅ ADR-015: CI/CD pipeline architecture
- ✅ Comprehensive testing guide (`docs/TESTING.md`)
- ✅ README updates with CI/CD section

**CI/CD Score**: 100/100 ✅

---

### ✅ Phase 4: Operational Readiness (Complete — 95%)
**Status**: 95% | **Duration**: complete

- ✅ Prometheus metrics integration (agent pipeline counters + histograms)
- ✅ cargo-audit supply-chain CVE scanning in devShell
- ✅ NATS NKey auth + ACL (SOPS-encrypted)
- ✅ EDR detection: 3 SIGMA rules + 6 YARA rules (sentinel)
- ✅ Cross-stack test coverage (owasaka pipeline/API, neotron stress tests)
- ✅ OpenTelemetry tracing (`#[instrument]` em orchestrator, client, session)
- ✅ TUI async rewrite (tokio::select!, Tokyo Night, UTF-8-safe cursor)
- ✅ Python contract tests (24, sem LLM, CI-ready)
- ⏳ Centralized logging (JSON via Vector/Loki)
- ⏳ Alerting with operational runbooks

**Current Score**: 95/100

---

### ⏳ Phase 5: Infrastructure & Scalability (Pending)
**Status**: 5A done | **Estimated**: 2 weeks restantes | **Effort**: ~60 hours

**Done (5A)**:
- ✅ TUI transparent thinking (`StageOutput` SSE → `┊` dim por stage)
- ✅ Input history shell-like (`↑/↓`, dedup, commit on submit)
- ✅ Shift+Enter multi-line input

**Pending (5B–5E)**:
- ⏳ Load testing — 500 RPS, p99 <200ms (ghz + wrk)
- ⏳ Test coverage 80%+ (currently ~75%)
- ⏳ Task queue persistida no PostgreSQL (sqlx)
- ⏳ Multi-session management no TUI
- ⏳ Docker containerization (multi-stage builds)
- ⏳ Kubernetes deployment (Helm charts, HA 3 replicas)

**Target Score**: 95/100

---

### ⏳ Phase 6: Compliance & Documentation (Pending)
**Status**: 0% | **Estimated**: 4 weeks | **Effort**: 140 hours

**Planned**:
- SOC 2 Type II documentation
- GDPR compliance (data deletion API)
- ISO 27001 ISMS documentation
- OpenAPI specification
- Complete API documentation
- Security documentation (threat model)
- Legal documentation (license, privacy policy)

**Target Score**: 80/100

---

## Metrics Summary

### Code Quality
- **Total Tests**: 226 Rust + 24 Python contract + 15 adr-ledger = 265+
- **Test Coverage**: ~75% (target: 80%+)
- **Clippy Warnings**: 0 (strict mode enabled)
- **Format Compliance**: 100%

### Security
- **Authentication**: ✅ REST + gRPC
- **Secrets Management**: ✅ Vault integration
- **Audit Logging**: ✅ Comprehensive
- **Rate Limiting**: ✅ 100 req/min
- **Input Validation**: ✅ Multi-layer

### CI/CD
- **Workflows**: 3 (test, lint, build)
- **Pre-commit Hooks**: ✅ Enabled
- **Cachix**: ✅ Configured
- **Build Time**: ~8-12 min (with cache)

### Documentation
- **ADRs**: 15 documents
- **Test Guide**: ✅ Comprehensive
- **README**: ✅ Updated
- **API Docs**: ⏳ Pending

---

## Production Readiness Checklist

### ✅ Completed (68%)
- [x] Stable Rust edition (2021)
- [x] No path dependencies
- [x] Tests enabled in Nix build
- [x] Zero `.expect()` panics in production code
- [x] Authentication on all endpoints
- [x] Secrets in Vault
- [x] Audit logging for security events
- [x] Rate limiting (100 req/min)
- [x] Input validation on all endpoints
- [x] 55+ unit tests
- [x] 18+ integration tests
- [x] CI/CD pipeline (GitHub Actions)
- [x] Pre-commit hooks
- [x] Code formatting enforced
- [x] Clippy strict mode

### ✅ Completed additions (2026-04-26)
- [x] TUI rewrite — async tokio::select!, Tokyo Night palette, UTF-8-safe cursor
- [x] 29 TUI unit tests (cursor, word-nav, insert/delete, auto-scroll, presets)
- [x] OpenTelemetry `#[instrument]` spans on orchestrator, client, session
- [x] 24 Python contract tests (AgentFlags IPC + all Pydantic schemas, no LLM)
- [x] Prometheus metrics wired
- [x] cargo-audit in devShell
- [x] EDR: 3 SIGMA + 6 YARA rules

### ✅ Completed additions (2026-04-27) — Ciclo 4
- [x] TUI agent workstation redesign (task queue, pipeline panel, SSE, keybindings)
- [x] MCP stdio client — `src/mcp/` JSON-RPC 2.0, `McpRegistry`, search + save_knowledge wired
- [x] Matrix integration — `MatrixClient`, matrix backend endpoints, pipeline-history dashboard
- [x] TUI transparent thinking — `AgentEvent::StageOutput`, `┊` dim rendering per stage
- [x] Input history — `↑/↓` shell-like navigation, dedup, `Shift+Enter` multi-line
- [x] 221 Rust tests passing (net +26 vs 2026-04-26)

### ✅ Completed additions (2026-05-16) — Ciclo 5 Communication Gaps
- [x] `BreakpointResolved` event publicado no orchestrator e mapeado no TUI (task transiciona WaitingForBreakpoint → Running via SSE)
- [x] `pipeline_started` event mapeado no TUI (não descartado silenciosamente)
- [x] SSE buffer flush no fim da stream (pipeline_done não é mais perdido)
- [x] Erros de steering/breakpoint roteados para painel de output do TUI (não eprintln!)
- [x] Health check real do LLM gateway — `check_llm_health_with_url()` com HTTP probe + timeout 2 s
- [x] Pipeline page `/pipeline` wired ao `PipelineRunner` real (não stub amber)
- [x] Frontend stats/pipeline routes desacopladas do Matrix backend → ADR como source of truth
- [x] llama.cpp porta 5001→8081 alinhada em todos os módulos Nix
- [x] Pre-commit hook cargo fmt corrigido (path absoluto retornava exit 2)
- [x] 226 Rust tests passing (net +5 vs 2026-04-27)

### ⏳ Pending (Phase 5B–6)
- [ ] 80%+ Rust test coverage (currently ~75%)
- [ ] Load testing (500 RPS target, p99 <200ms)
- [ ] Task queue persistida no PostgreSQL
- [ ] Docker containerization
- [ ] Kubernetes deployment + HA
- [ ] OpenAPI spec for axum endpoints
- [ ] SOC 2 / GDPR documentation

---

## Timeline

| Phase | Start | End | Duration | Status |
|-------|-------|-----|----------|--------|
| Phase 0 | Week 1 | Week 2 | 2 weeks | ✅ Complete |
| Phase 1 | Week 3 | Week 5 | 3 weeks | ✅ Complete |
| Phase 2 | Week 6 | Week 8 | 3 weeks | ⏳ In Progress (68%) |
| Phase 3 | Week 9 | Week 9 | 1 week | ✅ Complete |
| Phase 4 | Week 10 | Week 12 | 3 weeks | ⏳ Pending |
| Phase 5 | Week 13 | Week 15 | 3 weeks | ⏳ Pending |
| Phase 6 | Week 16 | Week 20 | 4 weeks | ⏳ Pending |

**Current**: Week 9 (Phase 3 complete)
**Next**: Complete Phase 2.3-2.4, then proceed to Phase 4

---

## Next Steps

### Crítico — Gaps ativos (varredura 2026-05-16)
1. **Doctor DSPy probe** — `src/commands.rs:265-358`: `neoland doctor` não proba o pipeline Python em `:8001`; adicionar `GET /health` probe + output no TUI
2. **Session serialization silenciosa** — `src/server/mod.rs:1061,1106`: `serde_json::to_value(&session).unwrap_or_default()` retorna `{}` sem log nem HTTP error; substituir por `?`-propagation ou erro 500 explícito
3. **sqlx FromRow frágil** — `src/agents/session.rs:46-64`: destructuring de tupla por índice de coluna; migrar para `#[derive(sqlx::FromRow)]` para ser resiliente a schema changes

### Alto — Estabilidade
4. **Stage name mismatch** — `src/tui/app.rs:214`: orquestrador pode emitir `"tech_leader"` (underscore) mas stages hardcoded como `"tech-leader"` (hyphen); verificar normalização e unificar
5. **Undefined guard analytics** — `matrix/frontend/lib/neoland/server.ts:356`: `document.full_pipeline.tech_leader.decision` sem guard; crash em ADR incompleto

### 5B — Curto prazo
6. **Load testing** — ghz gRPC + wrk REST, target 500 RPS p99 <200ms
7. **Test coverage 80%+** — focar em `src/server/`, `src/mcp/`, `src/matrix/`
8. **Task queue PostgreSQL** — persistir tasks no banco (sqlx), multi-session no TUI

### 5C–5E — Médio prazo
9. **Docker** — multi-stage build Rust + Python pipeline
10. **Kubernetes** — Helm chart, HA 3 replicas, health checks
11. **OpenAPI spec** — `utoipa` ou `aide` para axum REST endpoints

### Fase 6 — Compliance
12. **Centralized logging** — Vector → Loki, JSON structured logs
13. **Alerting** — Prometheus alertrules + runbooks básicos
14. **Compliance docs** — OpenAPI completo, SOC 2 gap analysis

---

## Gap Register (varredura proativa 2026-05-16)

Gaps encontrados durante varredura de anatomia do codebase. Ordenados por severidade.

### Crítico

| ID | Arquivo:Linha | Descrição | Impacto | Fix Sugerido |
|----|--------------|-----------|---------|-------------|
| GAP-001 | `src/commands.rs:265-358` | `neoland doctor` não proba DSPy pipeline em `:8001` | `doctor` reporta tudo OK mesmo com pipeline down — diagnóstico enganoso | Adicionar `GET http://localhost:8001/health` com timeout 3 s; exibir status no output do comando |
| GAP-002 | `src/server/mod.rs:1061,1106` | `serde_json::to_value(&session).unwrap_or_default()` — falha silenciosa, retorna `{}` | Sessão corrompida retorna 200 com corpo vazio; nenhum log, nenhum alerta | Substituir por `serde_json::to_value(&session).map_err(|e| internal_error(e))?` |
| GAP-003 | `src/agents/session.rs:46-64` | sqlx row mapeado por índice de coluna (tupla), não por nome | Qualquer reordenação de colunas na migration silently quebra deserialização de sessões | Adicionar `#[derive(sqlx::FromRow)]` ao struct e usar `query_as!` |

### Alto

| ID | Arquivo:Linha | Descrição | Impacto | Fix Sugerido |
|----|--------------|-----------|---------|-------------|
| GAP-004 | `src/tui/app.rs:214` | Stage names: orquestrador pode emitir `"tech_leader"` (underscore) mas TUI espera `"tech-leader"` (hyphen) | Stage output de tech_leader pode não renderizar no painel | Unificar para underscore em ambos os lados; remover normalização `_→-` do TUI |
| GAP-005 | `matrix/frontend/lib/neoland/server.ts:356` | `document.full_pipeline.tech_leader.decision` sem optional chaining | Dashboard analytics crasha em ADR com pipeline incompleto (ex: sem architect stage) | Usar `document.full_pipeline?.tech_leader?.decision ?? "—"` em todo o bloco analytics |

### Médio

| ID | Arquivo:Linha | Descrição | Impacto | Fix Sugerido |
|----|--------------|-----------|---------|-------------|
| GAP-006 | `src/cli.rs:41-65` | 4 `panic!("expected X command")` em testes CLI | Testes que chegam a este código por um bug de setup travam o runner inteiro | Substituir por `assert!(false, "...")` ou `unreachable!()` com mensagem |
| GAP-007 | `src/server/mod.rs:847` | `api_key.unwrap()` sem guard | Panic se middleware de auth for desabilitado e `api_key` for `None` | Verificar `auth_disabled` flag antes de `unwrap()` |
| GAP-008 | `agents/neoland_agents/app.py:19-24` | `_orchestrator` global inicializado sem logging de falha | Se o DSPy init falhar (ex: `LLM_API_KEY` ausente), FastAPI sobe mas todas as requests falham com 500 sem diagnóstico | Adicionar `logger.error` no bloco `except` do lifespan |
| GAP-009 | `docs/neoland-progress.md` (unificado) | Dois arquivos PROGRESS.md foram consolidados em um só | ✅ RESOLVIDO — `docs/neoland-progress.md` é a fonte única; `docs/archive/neoland-progress-2026-01-31.md` arquivado |

### Baixo

| ID | Arquivo:Linha | Descrição | Impacto | Fix Sugerido |
|----|--------------|-----------|---------|-------------|
| GAP-010 | `src/tui/app.rs:102` | ADR state não limpo em `complete_task()` | ADR de task anterior pode vazar para próxima task se `active_task_id` for reusado | Limpar `self.current_adr = None` em `complete_task()` |
| GAP-011 | `src/metrics.rs` | 26 `.unwrap()` em `lazy_static!` metric initialization | Se `prometheus::register_*` falhar (ex: nome duplicado em tests), process panics | Usar `.expect("metric registration")` com mensagem descritiva; aceitável em lazy_static init |
| GAP-012 | `matrix/frontend/lib/neoland/server.ts:356-369` | `getAgentAnalytics()` sem undefined guard em múltiplos campos | Ver GAP-005 — escopo maior: também afeta `junior.confidence`, `senior.risk_level` | Optional chaining em todo o bloco `full_pipeline` |
| GAP-013 | `src/tui/mod.rs` (geral) | Nenhum timeout no loop SSE — conexão pode ficar aberta indefinidamente | Se o control plane morrer sem fechar a conexão TCP, TUI fica "conectada" eternamente | Adicionar `tokio::time::timeout(Duration::from_secs(300), ...)` ao loop de receive |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation Status |
|------|-------------|--------|-------------------|
| CI/CD workflow failures | Low | Medium | ✅ Mitigated (tested, documented) |
| Test coverage gaps | Medium | High | ⏳ In progress (Phase 2.3-2.4) |
| Performance bottlenecks | Medium | Medium | ⏳ Planned (Phase 2.4 load testing) |
| Monitoring gaps | High | High | ⏳ Planned (Phase 4) |
| Compliance gaps | Low | High | ⏳ Planned (Phase 6) |

---

## Contributors

- **Architecture & Implementation**: Claude Sonnet 4.6 + marcosfpina
- **Code Review**: Production Readiness Team && VoidNxLabs Team
- **Testing**: Automated CI/CD + Manual validation

---

## References

- [Production Readiness Roadmap](neoland-roadmap.md)
- [Architecture Decision Records](ADR/)
- [Testing Guide](neoland-testing.md)
- [README](../README.md)
