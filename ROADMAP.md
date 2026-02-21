# Neoland — Roadmap

**Data de referência**: 2026-02-21
**Última actualização**: 2026-02-21
**Production Readiness**: 78/100

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

### Fase 3 — Disaster Recovery & Storage (100%)
- [x] 3.1 Backup/restore scripts (`scripts/backup/`)
- [x] 3.2 PersistentVectorStore com pgvector (`src/storage/vector_store.rs`) — código implementado
- [x] 3.3 AppState usa PersistentVectorStore em produção (pgvector opcional via `DATABASE_URL`)
- [x] 3.3b Tests para `server/mod.rs` — RateLimiter (5), lock ordering (2), gRPC stubs (4 ignored)
- [x] 3.3c Fill coverage gaps — `auth.rs` (2), `secrets.rs` (2) — suite: 114 passing, 26 ignored

### Fase 4 — Operações (100%)
- [x] 4.1 Prometheus metrics definidas (`src/metrics.rs`) — 20+ métricas
- [x] 4.1b Métricas gravadas: HTTP, gRPC, LLM, Auth, Rate Limit, Vector Store
- [x] 4.2 Correlation IDs + structured logging (`src/logging.rs`)
- [x] 4.3 Health checks Kubernetes-ready (`src/health.rs`)
- [x] 4.4 Config file (`src/config.rs`) — `neoland.toml` + env overrides + CLI flags
- [x] 4.5a `neoland doctor` — diagnóstica o ambiente
- [x] 4.5b PostgreSQL health check real (`src/health.rs` — pool ping + query test)
- [x] 4.6 LoadBalanced routing strategy — EMA latency tracking, circuit breakers
- [x] 4.7 Auth keys carregadas de Vault — `NEOLAND_REQUIRE_VAULT_KEYS=1` prod guard
- [x] 4.8 Non-streaming REST response — `stream:false` devolve JSON completo
- [x] 4.9 PgVector conectado em AppState — opcional via `DATABASE_URL`, fallback in-memory

---

## ✅ Fase 4 Concluída (2026-02-21)

**Critérios atingidos**:
- `curl localhost:3001/metrics` devolve contadores não-zero após requests
- Health check real para PostgreSQL (pool ping + query)
- `NEOLAND_REQUIRE_VAULT_KEYS=1` rejeita dev keys em produção
- LoadBalanced routing com EMA latency tracking
- Non-streaming REST e pgvector conectado ao AppState

## 🔄 Próximos Passos — Fase 5 (Target: Q2 2026)

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

| Métrica | Actual | Target Fase 5 | Target Fase 6 |
|---------|--------|---------------|---------------|
| Tests passando | 114 passing, 26 ignored | 130/160 | 150/175 |
| Production Readiness | 78/100 | 88/100 | 93/100 |
| Test Coverage | ~65% | 75% | 85% |
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
nix develop -c cargo build

# Testes (sem deps externas)
nix develop -c cargo test

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
