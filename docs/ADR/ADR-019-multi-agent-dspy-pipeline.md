# ADR-019: Multi-Agent DSPy Pipeline — Ciclo 0 Completo

**Status**: Accepted  
**Date**: 2026-04-07  
**Decision Makers**: marcosfpina, voidnxlabs Architecture  
**Fase**: Ciclo 0 — Core funcionando

---

## Contexto

A plataforma Neoland precisava de uma camada de raciocínio AI estruturada — não apenas um LLM
proxy genérico, mas um pipeline com perfis cognitivos distintos, auto-consciência de incerteza
(`unknowns`), controle de degradação de qualidade entre sessões, e checkpoints auditáveis
gerados automaticamente.

A decisão central: **Rust orquestra, Python raciocina.** O control plane Rust mantém a
performance, a segurança e o estado. O pipeline DSPy Python mantém a flexibilidade de
experimentação com LLMs e assinaturas de raciocínio.

---

## Decisão

Implementar um **pipeline multi-agent declarativo com 4 perfis cognitivos distintos**,
integrado ao control plane Rust via HTTP, com estado persistido em PostgreSQL e checkpoints
em formato ADR JSON.

### Perfis dos Agentes

| Agente | Perfil cognitivo | Execução |
|--------|-----------------|----------|
| **Junior** | Criativo, sem filtro, `unknowns` como auto-consciência estruturada | Sempre |
| **Senior** | Cético, refina, avalia risco real | Sempre |
| **Architect** | Soundness estrutural, composabilidade | Só se `escalate_to_architect=true` |
| **Tech-Leader** | Decisão final + ADR automático | Sempre |

### Stack técnico

- **DSPy 2.x** com `ChainOfThought` e `dspy.Assert` para enforcement de tipos em runtime
- **FastAPI** (:8001) como interface HTTP do pipeline Python
- **Rust control plane** (`src/agents/`) como orquestrador — nunca executa inferência
- **PostgreSQL** para estado de sessão e histórico de decisões
- **ADR JSON** como checkpoint automático de cada decisão do Tech-Leader

### API exposta pelo control plane

| Endpoint | Auth | Descrição |
|----------|------|-----------|
| `POST /v1/agents/task` | X-API-Key (User+) | Submete task ao pipeline |
| `GET /v1/agents/session/:id` | ReadOnly+ | Estado e histórico da sessão |
| `GET /v1/agents/health` | Público | Health do pipeline Python |

---

## Consequências

### Positivas

- **Pipeline completamente tipado**: contratos Rust ↔ Python espelhados via serde e Pydantic —
  qualquer drift de schema é detectado nos testes de contrato sem precisar de LLM
- **Raciocínio auditável**: cada decisão do Tech-Leader gera um ADR JSON com o raciocínio
  completo de todos os agentes, persistido em filesystem (e futuramente no adr-ledger)
- **Escalada condicional**: o Architect só entra quando o Senior sinaliza `escalate_to_architect=true`,
  evitando overhead em tasks de baixa complexidade
- **Degradação controlada**: o `EscalationPolicy` usa o histórico da sessão para decidir o
  ponto de entrada do pipeline — sessões com `defer` recente começam direto no Senior
- **Zero mocks**: testes de contrato validam schema roundtrip sem LLM; testes de sessão usam
  PostgreSQL real; testes de integração usam LLM real

### Negativas / Trade-offs

- **Latência adicional**: pipeline completo (sem Architect) ~10-30s dependendo do provider LLM
- **Dependência de serviço Python**: sem o FastAPI (:8001) rodando, o control plane retorna
  503 graciosamente mas não processa tasks de agentes
- **Estado em PostgreSQL**: requer `DATABASE_URL` configurado — em ausência, o orchestrator
  fica disabled (design intencional, não falha silenciosa)

### Futuro (Ciclo 1)

- Substituir HTTP Rust→Python por **mmap IPC intra-host** (zero-copy, sem serialização JSON
  entre agentes no mesmo host) — ver NEOLAND-PLATFORM.md
- Adicionar **NATS pub/sub** via spectre-events para streaming de eventos de pipeline em tempo real
- Integrar **adr-ledger** como subscriber NATS — assinatura secp256k1 + Merkle chain automáticos

---

## Implementação — Arquivos criados/modificados

```
MODIFICADOS:
  Cargo.toml                        uuid serde feature
  src/agents/client.rs              Serialize nos tipos de output
  src/agents/session.rs             Serialize no SessionState
  src/agents/orchestrator.rs        get_session() method
  src/server/mod.rs                 3 rotas + 3 handlers + AgentOrchestrator no AppState
  agents/pyproject.toml             Python 3.13
  neoland/flake.nix                 Python 3.13 devShell + venv automático + aliases

CRIADOS:
  tests/agent_contract_test.rs      9 testes — schema roundtrip sem dependências externas
  tests/agent_session_test.rs       4 testes — PostgreSQL real, sem LLM
  tests/agent_integration_test.rs   4 testes — end-to-end com LLM real
```

---

## Verificação

```bash
# Dentro de nix develop:
cargo test --lib                            # 137 testes — 0 falhas
cargo test --test agent_contract_test       # 9 testes de contrato — 0 falhas
cargo test --test agent_session_test        # 4 testes (requer DATABASE_URL)
cargo test --test agent_integration_test    # 4 testes (requer DSPy + LLM)

# Smoke test das rotas:
curl -s localhost:3001/v1/agents/health
curl -X POST localhost:3001/v1/agents/task \
  -H "X-API-Key: $NEOLAND_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"task": "Should we add circuit breaker to the LLM client?"}'
```

---

## Referências

- `NEOLAND-PLATFORM.md` — arquitetura completa e ciclos de evolução
- `src/agents/` — control plane Rust
- `agents/neoland_agents/` — pipeline Python DSPy
- `migrations/002_agent_sessions.sql` — schema do banco
- ADR-013 — Audit logging (eventos `AgentTaskStart`, `AgentDecision`, `AgentEscalation`)
