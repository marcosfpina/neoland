---
name: neoland-ops
description: >-
  Operational playbook for the Neoland project. Defines work conventions, daily
  rituals, task lifecycle, commit style, review standards, documentation discipline,
  and repetitive patterns. Activate when starting work, finishing a task, doing code
  review, writing commits, or asking "how should I do X here".
---

# Neoland Operations Playbook

Como a gente opera aqui. Convenções, rituais, checklist de task, padrões de commit,
review e documentação. Se for fazer qualquer coisa no Neoland, segue este playbook.

---

## Ritual de Início de Trabalho

Toda vez que sentar pra codar no Neoland:

```
[ ] cd ~/master/neoland
[ ] nix develop                          # SEMPRE — fora quebra linker
[ ] git --no-pager pull --rebase         # Trazer o que mudou
[ ] cargo check --lib                    # Confirmar build limpo
[ ] Abrir ROADMAP.md                     # Revisar o que está em progresso
[ ] Abrir CLAUDE.md                      # Revisar regras e contratos atuais
```

---

## Ciclo de Vida de uma Task

### 1. Antes de começar

- [ ] Entender o que a task pede (ler ROADMAP.md, ADRs relevantes)
- [ ] Identificar arquivos que serão tocados
- [ ] Verificar se afeta contratos Rust ↔ Python
- [ ] `cargo check --lib` — baseline limpo
- [ ] Se for feature nova: criar branch `feat/<nome-curto>`
- [ ] Se for fix: criar branch `fix/<nome-curto>`

### 2. Durante a implementação

- [ ] **Sem mocks** — testes usam LLM real e PostgreSQL real
- [ ] Preferir editar arquivos existentes a criar novos
- [ ] Não criar abstração pra uso único
- [ ] Não tratar erro impossível
- [ ] Não comentar o óbvio
- [ ] `unsafe` só com bloco `// SAFETY:` explicando o invariante
- [ ] Mudança em `src/agents/client.rs` → atualizar `schemas/api.py`
- [ ] Mudança no layout mmap → atualizar `flags.rs` E `flags.py`

### 3. Antes de considerar pronto

```
[ ] cargo check --lib                     # build limpo
[ ] cargo clippy --all-targets -- -D warnings  # zero warnings
[ ] cargo test --lib                      # 100% passando
[ ] cargo test --test rest_api_test       # E2E REST 22/22
[ ] cd agents && pytest -m contract -v    # Python contracts 26/26
[ ] just doctor                           # ok: true
```

### 4. Pra fechar a task

```
[ ] ROADMAP.md — marcar entrega, atualizar score se houver evidência
[ ] CLAUDE.md — atualizar se stack, contratos ou regras mudaram
[ ] ADR — criar se houve decisão arquitetural
[ ] Commit — mensagem no padrão (ver abaixo)
```

---

## Padrão de Commits

```
<tipo>(<escopo>): <mensagem curta em inglês, imperative mood>

<corpo opcional — o que, por que, não o como>
```

| Tipo | Quando usar |
|------|-------------|
| `feat` | Nova funcionalidade |
| `fix` | Correção de bug |
| `refactor` | Mudança de estrutura sem alterar comportamento |
| `test` | Adição ou melhoria de testes |
| `docs` | Documentação |
| `chore` | Tarefa operacional (build, deps, CI) |
| `security` | Hardening, fix de vulnerabilidade |
| `perf` | Otimização de performance |

| Escopo | Exemplos |
|--------|----------|
| `agents` | Pipeline, orchestration, session, escalation |
| `server` | API REST/gRPC, middleware, handlers |
| `tui` | Terminal UI |
| `auth` | Autenticação, RBAC |
| `audit` | Audit logging |
| `secrets` | Vault, SOPS |
| `nix` | Flake, NixOS modules, dev shell |
| `docs` | README, ROADMAP, ADRs |
| `pipeline` | DSPy, Python agents |

### Exemplos bons

```
feat(agents): add live steering breakpoint to orchestrator
fix(server): return 500 with log on session serialization failure
security(audit): add brute-force detection threshold config
test(pipeline): contract test for escalate_to_architect flag
docs(readme): align perf claims with load test evidence
```

### Exemplos ruins

```
fix bug
WIP
updates
fixed the thing that was broken in the session handler when serialization failed and it returned empty object silently
```

---

## Code Review

### O que revisar (checklist mental)

1. Build passa? `cargo check --lib` limpo?
2. Testes passam? `cargo test --lib` 100%?
3. Contratos respeitados? Se mexeu em `client.rs`, `schemas/api.py` foi atualizado?
4. Lock ordering: engine antes de vector_store — nunca inverteu?
5. mmap layout: mudou? `flags.rs` + `flags.py` sincronizados?
6. `sqlx::query!` não foi usado em `src/agents/`? (usar `sqlx::query` + `.bind()`)
7. LLM access: só via `unified_client.rs`?
8. Documentação: CLAUDE.md e ROADMAP.md atualizados?
9. ADR: se decisão arquitetural, foi criado?
10. `unwrap_or_default()` → trocou por `match` com 500 + log?

### Tom do review

- Direto, sem rodeios
- Apontar o problema, sugerir o fix
- Se o código está correto mas pode ser melhorado: "LGTM. Sugestão opcional: ..."
- Não bloquear por estilo a menos que viole regra explícita deste playbook

---

## Documentação

### O que sempre manter atualizado

| Arquivo | Quando atualizar |
|---------|-----------------|
| `ROADMAP.md` | Toda entrega — marcar ✅, atualizar score, registrar evidência |
| `CLAUDE.md` | Se stack, contratos, regras ou env vars mudaram |
| `README.md` | Se claims, métricas ou limitações mudaram |
| `docs/ADR/ADR-XXX.md` | Toda decisão arquitetural |
| `skills/README.md` | Se nova skill foi criada |

### ADRs

Criar um ADR quando:
- Nova dependência adicionada (crate ou package Python)
- Contrato binário ou de API estabelecido ou alterado
- Decisão arquitetural com trade-offs relevantes
- Fase de ciclo concluída

Formato: `docs/ADR/ADR-0XX-<slug>.md`

Estrutura mínima:
```markdown
# ADR-0XX: Título

## Contexto
## Decisão
## Alternativas consideradas
## Consequências
```

---

## Rotinas

### Diária (startup)

```
cd ~/master/neoland && nix develop
cargo check --lib
git --no-pager pull --rebase
# Revisar ROADMAP.md pra saber o que atacar hoje
```

### Por task

Seguir [Ciclo de Vida de uma Task](#ciclo-de-vida-de-uma-task) acima.

### Pré-release

```
just preflight
just smoke
# Verificar Production Criteria no ROADMAP.md
# Se todos ✅ → tag, release notes
```

---

## Regras Rápidas (memorizar)

```
 SEMPRE   nix develop — fora quebra linker
 SEMPRE   cargo check --lib antes de qualquer commit
 SEMPRE   testes 100% passando antes de commit
 SEMPRE   editar existente > criar novo
 NUNCA    mockar LLM ou PostgreSQL em testes
 NUNCA    sqlx::query! em src/agents/
 NUNCA    acessar LLM fora de unified_client.rs
 NUNCA    inverter lock ordering (engine → vector_store)
 NUNCA    subir score sem evidência
```

---

## Problemas Comuns e Resoluções

| Sintoma | Causa provável | Ação |
|---------|---------------|------|
| Linker quebra | `cargo` fora do `nix develop` | `nix develop --command cargo check --lib` |
| `sqlx::query!` erro de compilação | `DATABASE_URL` ausente em compile time | Trocar por `sqlx::query` + `.bind()` |
| Contratos quebrados | Mudou `client.rs` sem atualizar `api.py` | Sincronizar os dois lados |
| Testes Python falham | DSPy 3.x quebrou API (ex: `dspy.Assert`) | Verificar changelog DSPy, adaptar módulos |
| TUI trava em SSE | Pipeline morreu sem sinalizar | Timeout de 30s → `PipelineError` automático |
| Smoke falha em camada X | Serviço não está rodando na porta esperada | `neoland doctor --json` → verificar cada layer |

---

## Tom Geral

- Português brasileiro para comunicação interna
- Inglês para commits, código, ADRs, README e docs públicas
- Direto, sem firula, sem "talvez", sem "acho que"
- Evidência > opinião. Se não tem dado, vai medir
- Terminou a task? Atualizou ROADMAP.md? Se não, não terminou
