# Neoland — Roadmap

**Data de referência**: 2026-02-21
**Production Readiness**: 65/100

Este documento é a fonte de verdade para o estado actual e as próximas metas.
Actualiza-o quando completares um item.

---

## ✅ Concluído

### Fase 1 — Segurança (100%)
- [x] 1.1 Autenticação RBAC (`src/auth.rs`) — API keys, 3 roles
- [x] 1.2 HashiCorp Vault integration (`src/secrets.rs`) — com fallback env vars + cache TTL
- [x] 1.3 Audit logging + brute force detection (`src/audit.rs`)
- [x] 1.4 Input validation + sanitization (`src/validation.rs`)

### Fase 2 — Infra & Testes (90%)
- [x] 2.1 Test utilities & mocks (`src/test_utils.rs`)
- [x] 2.2 Circuit breakers com 9 unit tests (`src/llm/unified_client.rs`)
- [x] 2.3 Integration tests gRPC + REST (`tests/`)
- [x] 2.4 Security test suite (`tests/security/`)
- [x] 2.5 Load testing suite (`tests/load/`)

### Fase 3 — Disaster Recovery & Storage (80%)
- [x] 3.1 Backup/restore scripts (`scripts/backup/`)
- [x] 3.2 PersistentVectorStore com pgvector (`src/storage/vector_store.rs`) — código implementado
- [ ] 3.3 AppState usa PersistentVectorStore em produção (usa in-memory ainda)

### Fase 4 — Operações (65%)
- [x] 4.1 Prometheus metrics definidas (`src/metrics.rs`) — 20+ métricas
- [x] 4.1b Métricas gravadas: HTTP, gRPC, LLM, Auth, Rate Limit, Vector Store
- [x] 4.2 Correlation IDs + structured logging (`src/logging.rs`)
- [x] 4.3 Health checks Kubernetes-ready (`src/health.rs`)
- [x] 4.4 Config file (`src/config.rs`) — `neoland.toml` + env overrides + CLI flags
- [x] 4.5a `neoland doctor` — diagnóstica o ambiente
- [ ] 4.5b PostgreSQL health check real (hardcoded como Healthy em `src/health.rs:69`)
- [ ] 4.6 LoadBalanced routing strategy (`unified_client.rs:230` — unimplemented)
- [ ] 4.7 Auth keys carregadas de Vault (`src/auth.rs:71-99` — hardcoded dev keys)
- [ ] 4.8 Non-streaming REST response (só SSE/stream funciona actualmente)

---

## 🔄 Em Curso — Fase 4 Restante (Target: 2026-03-15)

| # | Item | Ficheiro | Esforço |
|---|------|----------|---------|
| 4.5b | PostgreSQL health check real | `src/health.rs:69` | ~2h |
| 4.6 | LoadBalanced strategy (round-robin por latência) | `src/llm/unified_client.rs:230` | ~3h |
| 4.7 | Auth keys via Vault (remover hardcoded dev keys) | `src/auth.rs:71-99` | ~3h |
| 4.8 | Non-streaming REST (usar `RestChatResponse` dead code) | `src/server/mod.rs` | ~2h |
| 4.9 | PgVector conectado em AppState | `src/server/mod.rs:724` | ~4h |

**Critério de conclusão da Fase 4**: `curl localhost:3001/metrics` devolve contadores não-zero após requests; health check real para DB; sem dev keys em prod.

---

## 📋 Planeado — Fase 5: Integração Neutron (Target: Q2 2026)

> Dependente do Neutron v1.0. Descrito em detalhe em `docs/ADR.md` (ADR-007 a ADR-009).

- [ ] 5.1 `src/neutron/provenance.rs` — rastreamento de lineage de dados de treino
- [ ] 5.2 `src/neutron/consensus.rs` — verificação distribuída de model updates
- [ ] 5.3 `src/neutron/zkp.rs` — zero-knowledge proofs para inferência sensível
- [ ] 5.4 Hook em `ml-offload-api` — attestation de backends antes de inferência
- [ ] 5.5 Hook em `securellm-bridge` — audit logs imutáveis
- [ ] 5.6 Hook em `nlp.rs` (VectorStore) — verificação de fontes antes de RAG

**Impacto estimado**: +50-100ms latência por request (overhead criptográfico).

---

## 📋 Planeado — Fase 6: Integração ADR-Ledger (Target: Q2 2026)

> Descrito em `docs/ADR.md` (ADR-008).

- [ ] 6.1 Routing decisions auto-logged como ADR events
- [ ] 6.2 ADR enforcement em CI/CD (bloquear merge se violar ADR de segurança)
- [ ] 6.3 Semantic search sobre decisões arquiteturais via neoland TUI

---

## 📋 Planeado — Fase 7: Enterprise / Spectre (Target: Q3 2026)

> Descrito em `docs/ADR.md` (ADR-010).

- [ ] 7.1 Multi-tenancy com SLA-based backend selection
- [ ] 7.2 Distributed tracing completo com OpenTelemetry
- [ ] 7.3 Compliance automation (SOC2, ISO27001, LGPD)

---

## Métricas de Referência (2026-02-21)

| Métrica | Actual | Target Fase 4 | Target Fase 5 |
|---------|--------|---------------|---------------|
| Tests passando | 114/140 | 130/150 | 150/175 |
| Production Readiness | 65/100 | 80/100 | 90/100 |
| Test Coverage | ~60% | 70% | 80% |
| MTTR (rollback) | <5min | <5min | <5min |
| Inference latência (local) | 5-10 tok/s | 5-10 tok/s | +50-100ms (Neutron) |
| Config drift | 0 (Nix) | 0 (Nix) | 0 (Nix) |

---

## Como Contribuir / Desenvolver

```bash
# Entrar no dev shell Nix
nix develop

# Verificar ambiente
neoland doctor

# Build
cargo build

# Testes (sem deps externas)
cargo test

# Servidor local
neoland server

# Cliente TUI
neoland client
```

**Config local** (opcional — todos os defaults funcionam):
```toml
# neoland.toml
[server]
grpc_port = 50051
rest_port = 3001

[client]
server_url = "http://[::1]:50051"
ml_api_url = "http://localhost:8080"

[inference]
provider = "local"
temperature = 0.7
max_tokens = 2048
```
