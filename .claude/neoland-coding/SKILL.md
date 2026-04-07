---
name: neoland-coding
description: Coding partner for the Neoland AI agent platform. Directs all development dynamics: Rust control plane, Python DSPy pipeline, cross-boundary contracts, Ciclo-based cadence, documentation discipline, and test-first workflow. Activate whenever working on ~/master/neoland — covers cargo, mmap IPC, NATS, adr-ledger, NixOS modules, and any new Ciclo phase.
---

# Neoland Coding Partner

Parceiro de engenharia especializado na plataforma Neoland. Conhece a arquitetura inteira,
os contratos entre camadas, as regras de trabalho e o ritmo de entrega por Ciclos.

---

## Identidade do Projeto

| Campo | Valor |
|-------|-------|
| Repo | `~/master/neoland` |
| Binary | `neoland` |
| Control plane | Rust 2021 (tokio + axum + tonic + sqlx) |
| Agent pipeline | Python 3.13 + DSPy + FastAPI (:8001) |
| Storage | PostgreSQL + pgvector |
| Infra | NixOS modules declarativos |
| Secrets | HashiCorp Vault + sops-nix |
| IPC intra-host | mmap (`/run/neoland/agent-flags.shm`) |

**Arquivos canônicos de contexto** (sempre atualizados ao final de cada entrega):
- `~/master/neoland/CLAUDE.md` — regras, stack, contratos, fases
- `~/master/neoland/PROGRESS.md` — estado atual, score, tabela de ciclo

---

## Dinâmica de Trabalho

### Antes de qualquer modificação

1. Ler os arquivos relevantes — nunca modificar código que não foi lido
2. Rodar `cargo check --lib` para capturar o estado atual de compilação
3. Verificar se a mudança afeta contratos Rust ↔ Python (ver seção Contratos)

### Durante a implementação

- **Sem mocks** — testes usam LLM real e PostgreSQL real
- Preferir editar arquivos existentes a criar novos
- Não adicionar abstrações para uso único
- Não adicionar error handling para casos impossíveis
- Não adicionar comentários onde a lógica é auto-evidente
- Código inseguro (unsafe) só com bloco de comentário `// SAFETY:` explicando

### Após implementar

1. `cargo check --lib` — deve estar limpo
2. `cargo test --lib` — 100% passando antes de qualquer commit
3. Atualizar `CLAUDE.md` e `PROGRESS.md` com o que foi entregue
4. Criar ADR se a decisão afeta arquitetura ou contratos

---

## Regras Rust

```
SEMPRE dentro de nix develop — fora o linker quebra
```

| Regra | Detalhe |
|-------|---------|
| Shell obrigatório | `nix develop --command cargo <cmd>` |
| Check rápido | `cargo check --lib` |
| Testes | `cargo test --lib` — sem `--test-threads=1` exceto para agent_ |
| Sem `sqlx::query!` | usar `sqlx::query` + `.bind()` em `src/agents/` |
| LLM só via unified_client | `src/llm/unified_client.rs` — não criar outros pontos de acesso |
| Lock ordering CRÍTICO | engine primeiro, vector_store segundo — nunca inverter |
| Contratos | mudança em `src/agents/client.rs` → atualizar `schemas/api.py` |

### Estrutura de módulos (`src/agents/`)

```
flags.rs          ← SharedFlags (mmap layout, atomics)
mmap.rs           ← MmapRegion (open, flags(), flush())
client.rs         ← HTTP client + tipos espelho dos schemas Python
orchestrator.rs   ← lógica central de orquestração
session.rs        ← estado de sessão via PostgreSQL
escalation.rs     ← política de escalada entre agentes
checkpoint_store.rs
```

### Padrão de testes Rust

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;   // para testes de arquivo/mmap

    #[test]
    fn descricao_clara_do_que_valida() {
        // arrange
        // act
        // assert
    }
}
```

Testes de integração que requerem PostgreSQL ou LLM real ficam em `tests/` com `#[ignore]`
removível via variável de ambiente — não mockar.

---

## Regras Python

| Regra | Detalhe |
|-------|---------|
| Testes sem LLM | `pytest tests/ -m contract -v` — rodar primeiro |
| dspy.Assert | nos campos `confidence` (float) e `risk_level` (enum) — não remover |
| RAG limitado | 5 docs × 500 chars — não aumentar sem medir impacto |
| FastAPI | escuta em `127.0.0.1:8001` — nunca `0.0.0.0` sem config explícita |
| IPC | usar `AgentFlags` de `neoland_agents/ipc/flags.py` para sinalização intra-host |

### Layout binário mmap (contrato fixo)

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
Total   64    (1 cache line)
```

**Qualquer mudança no layout exige atualização simultânea em `flags.rs` e `flags.py`.**

---

## Contratos Rust ↔ Python

Os tipos em `src/agents/client.rs` espelham `agents/neoland_agents/schemas/api.py`.

| Endpoint | Direção | Descrição |
|----------|---------|-----------|
| `POST /v1/pipeline/run` | Rust → Python | Executa pipeline completo |
| `GET /v1/pipeline/session/{id}` | Rust → Python | Histórico de checkpoints |
| `GET /health` | Rust → Python | Health do pipeline |
| `POST /v1/agents/task` | Client → Rust | Envia task ao orchestrator |
| `GET /v1/agents/session/:id` | Client → Rust | Estado da sessão |

---

## Ciclos de Desenvolvimento

### Cadência

Cada Ciclo tem fases A, B, C, D. Fases são entregues sequencialmente.
Ao completar uma fase:
1. Testes passando
2. `CLAUDE.md` atualizado (fase marcada ✅)
3. `PROGRESS.md` atualizado (score + tabela)
4. ADR criado se houve decisão arquitetural
5. Commit descritivo com `feat(agents):` ou prefixo relevante

### Estado atual

| Ciclo | Fase | Status |
|-------|------|--------|
| 0 | Core pipeline | ✅ Fechado 2026-04-07 |
| 1 | A — mmap IPC | ✅ Entregue 2026-04-07 |
| 1 | B — NATS publisher | ⏳ |
| 1 | C — adr-ledger | ⏳ |
| 1 | D — Phantom scan | ⏳ |

---

## ADRs

Criar um ADR quando:
- Uma nova dependência é adicionada (crate ou package Python)
- Um contrato binário ou de API é estabelecido ou alterado
- Uma decisão de arquitetura tem trade-offs relevantes
- Uma fase de Ciclo é concluída

Arquivo: `docs/ADR/ADR-0XX-<slug>.md`
Formato mínimo: Contexto → Decisão → Alternativas consideradas → Consequências

ADRs existentes: ADR-011 a ADR-020 (ver `docs/ADR/`)

---

## Variáveis de Ambiente

| Variável | Padrão | Uso |
|----------|--------|-----|
| `DATABASE_URL` | — | PostgreSQL (obrigatório em prod) |
| `LLM_API_KEY` | — | API key do provider |
| `NEOLAND_LLM_PROVIDER` | `openai` | openai / deepseek / anthropic |
| `NEOLAND_LLM_MODEL` | `gpt-4o-mini` | modelo do provider |
| `NEOLAND_CHECKPOINT_DIR` | `/var/lib/neoland/checkpoints/adr` | ADRs JSON |
| `NEOLAND_PIPELINE_PORT` | `8001` | FastAPI |
| `NEOLAND_AGENTS_DSPY_URL` | `http://localhost:8001` | URL pipeline (Rust) |
| `NEOLAND_SHM_PATH` | `/run/neoland/agent-flags.shm` | mmap IPC |

---

## Problemas Conhecidos

| Problema | Causa | Workaround |
|----------|-------|-----------|
| linker quebra | `cargo` fora do `nix develop` | sempre usar `nix develop` |
| `sqlx::query!` falha | requer `DATABASE_URL` em compile time | usar `sqlx::query` + `.bind()` |
| `confidence` como string | DSPy 2.x ChainOfThought | `float()` no cast |
| FastAPI não sobe | LLM init síncrona no lifespan | checar provider antes de subir |

---

## Tom e Estilo de Resposta

- Comunicação em **português brasileiro** direto, sem rodeios
- Respostas curtas — ir direto ao ponto
- Antes de criar arquivos: ler o contexto existente
- Antes de propor mudanças: `cargo check --lib`
- Tabelas para comparar alternativas
- Sempre verificar se documentação foi atualizada ao final

---

## Checklist de Entrega por Fase

```
[ ] cargo check --lib — limpo
[ ] cargo test --lib  — 100% passando
[ ] CLAUDE.md         — fase marcada ✅, env vars, layout atualizado
[ ] PROGRESS.md       — score + tabela de ciclo atualizados
[ ] ADR criado        — se houve decisão arquitetural
[ ] Commit feito      — mensagem descritiva com prefixo correto
```
