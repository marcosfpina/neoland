# Neoland Dev Book — Inventário Navegável

> Última atualização: 2026-08-06 · `cargo check --lib` ✅ · 275 testes ✅
> 
> Documento interno de referência para retomada do projeto após hiato.
> Descreve o estado real, sem inflar métricas.

---

## Entry Points: como entrar no projeto

### 1. Build & Test (sempre funciona)

```bash
cd ~/master/neoland
nix develop --command cargo check --lib     # build limpo
nix develop --command cargo test --lib      # 275 testes
nix develop --command cargo test --test rest_api_test  # 22 E2E
```

### 2. Subir o server (depende de PostgreSQL + SOPS)

```bash
# Precisa de:
# - PostgreSQL rodando com banco 'neoland'
# - SOPS configurado OU variáveis de ambiente manuais
nix develop --command cargo run -- server
# REST em :3001, gRPC em :50051
```

### 3. Pipeline DSPy (depende de LLM_API_KEY)

```bash
cd agents
poetry run python -m neoland_agents.app
# FastAPI em :8001
poetry run pytest tests/ -m contract -v   # 26 testes, não precisa de LLM
```

### 4. Smoke test completo (depende de tudo)

```bash
just doctor     # diagnóstico de cada layer
just smoke      # 9 camadas — precisa de Bridge + ml-ops + llama.cpp
```

---

## Mapa do Repositório: o que é cada diretório

```
neoland/
├── src/                    ★ CORE — control plane Rust (54 arquivos, ~17k LoC)
├── agents/                 ★ Pipeline DSPy Python (FastAPI, 4 estágios)
├── docs/ADR/               ★ 11 ADRs (ADR-011 a ADR-021) — conteúdo real
├── tests/                  Testes de integração/E2E Rust
├── proto/                  Definições protobuf (gRPC)
├── scripts/                smoke-full-stack.sh, release-preflight.sh
├── nix/                    NixOS modules
├── deploy/                 Configs de deploy
├── migrations/             SQL migrations (PostgreSQL)
├── .github/                CI/CD (Azure DevOps + GitHub Actions)
├── knowledge/              Base de conhecimento (RAG)
├── plans/                  Planos e specs antigos
├── desktop/                Experimentos TUI/desktop
├── secrets/                SOPS .env + tls/ (trial temporário)
├── securellm-bridge/       ⚠️ Vazio — gitignored, checkout externo esperado
├── matrix/                 ⚠️ NÃO EXISTE — frontend referenciado em docs
├── skills/                 Skills RAG (legado) — não confundir com .agents/skills/
├── .agents/skills/         ★ Skills do Zed (neoland-master, neoland-ops)
├── flake.nix               Nix dev shell
├── justfile                 Atalhos (just server, just smoke, etc.)
├── Cargo.toml              Dependências Rust
└── README.md               ⚠️ Desatualizado — 226 testes (são 275), frontend inexistente
```

---

## O que funciona vs o que range

### ✅ Sólido — pode confiar

| Componente | Evidência |
|-----------|-----------|
| Build Rust | `cargo check --lib` limpo |
| Clippy | 0 warnings com `-D warnings` |
| Testes unitários | 275 passed, 0 failed |
| E2E REST | 22/22 |
| Python contracts | 26/26 |
| Estrutura do código | 54 arquivos Rust, organizado |
| ADRs (docs) | 11 decisões documentadas com profundidade real |

### ⚠️ Range — precisa de atenção

| Componente | Problema |
|-----------|----------|
| **Frontend (Next.js)** | `matrix/frontend` não existe. README/ROADMAP referenciam mas não está no repo. Está em outro repositório? Foi removido? |
| **securellm-bridge** | Diretório gitignored vazio. Smoke test 9/9 só funciona com checkout externo. Não reproduzível só com clone. |
| **ADR toolchain** | ADRs existem como markdown, mas Merkle chain nunca inicializada. `.schema/adr.schema.json` não existe. `adr_list` retorna vazio. |
| **ROADMAP.md** | Desatualizado: 226 testes (são 275), frontend claims falsos. |
| **README.md** | Desatualizado: mesmas divergências. |
| **target/ cache** | Acumulou 50 GB e corrompeu. `cargo clean` resolveu mas é sintoma. |

### ❌ Não existe / Não funciona

| Componente | Detalhe |
|-----------|---------|
| Frontend | Diretório `matrix/` não existe |
| Merkle chain | Nunca inicializada |
| Smoke real | Dependente de serviços externos não versionados |

---

## Rotas de Navegação: "quero mexer em X"

### Quero mexer na API REST / gRPC

```
src/server/mod.rs     ← handlers, middleware, rotas
src/auth.rs           ← API keys, RBAC
src/validation.rs     ← input sanitization
src/audit.rs          ← audit logging
tests/rest_api_test.rs ← testes E2E
```

### Quero mexer no pipeline de agentes

```
src/agents/orchestrator.rs  ← orquestração (Rust)
src/agents/client.rs        ← HTTP client → Python
src/agents/session.rs       ← PostgreSQL sessions
src/agents/escalation.rs    ← política de escalada
agents/neoland_agents/      ← pipeline Python (DSPy)
agents/tests/               ← contract tests Python
```

### Quero mexer no TUI

```
src/tui/mod.rs       ← TUI principal (ratatui)
src/tui/ui.rs        ← renderização
src/bin/neoland.rs   ← CLI entrypoint (server | client | doctor | restart)
```

### Quero mexer em auth / secrets

```
src/auth.rs          ← API key validation, RBAC
src/secrets.rs       ← Vault → SOPS → env fallback
secrets/neoland.sops.env  ← dev secrets (age-encrypted)
docs/neoland-authentication.md
docs/neoland-vault-setup.md
docs/ADR/ADR-011-authentication-strategy.md
docs/ADR/ADR-012-secrets-management.md
```

### Quero mexer na infra / Nix

```
flake.nix            ← dev shell, dependências
nix/                 ← NixOS modules
justfile             ← atalhos
.sops.yaml           ← SOPS config
```

### Quero mexer nos ADRs

```
docs/ADR/            ← 11 ADRs em markdown (conteúdo real)
docs/neoland-adr.md  ← sumário
⚠️ Toolchain de Merkle chain não inicializada
```

### Quero mexer na topologia LLM

```
src/ml_offload/client.rs     ← cliente de inferência
src/llm/unified_client.rs    ← acesso unificado a LLMs
src/llm/proxy.rs             ← proxy local
⚠️ securellm-bridge/ é externo (gitignored)
⚠️ ml-ops-api é externo
```

---

## Coisas que provavelmente estão te incomodando

Baseado no que vi, aqui estão os pontos de fricção mais prováveis:

### 1. O frontend sumiu
O `matrix/frontend` era referenciado em todo lugar — README, ROADMAP, ADR-021. Não está no repo. Se foi movido pra outro repositório, precisa documentar. Se foi removido, precisa limpar as referências.

### 2. A topologia LLM é frágil
O smoke test de 9 camadas depende de 3 serviços externos (Bridge, ml-ops, llama.cpp) que não estão versionados no repo. Não dá pra clonar e rodar — precisa de setup manual com checkouts de outros repositórios.

### 3. O ADR Ledger é fake até agora
11 ADRs bem escritos em markdown, mas a infraestrutura de Merkle chain, assinaturas e governança nunca foi implementada. O README vende uma feature que é só documentação estática.

### 4. Métricas infladas
Score 99/100 no ROADMAP ignora que o frontend não existe e a topologia não é reproduzível. O score foi calculado num momento específico (2026-06-02) com a máquina do autor tendo tudo montado.

### 5. Build cache apodrece
50 GB de `target/` com paths de build antigos. O projeto compila várias deps nativas pesadas (candle, tokenizers, openssl, aws-lc) e o cache corrompe com o tempo.

### 6. Git tree sujo
`Cargo.lock` modificado, `skills/README.md` alterado, `secrets/tls/` com certs temporários staged, `.agents/` novo. Precisa de uma limpeza antes do próximo commit real.

---

## Plano de Ataque Sugerido

### Imediato (hoje) — sentir o app

```bash
# 1. Subir o server e bater nos endpoints
nix develop --command cargo run -- server
curl http://localhost:3001/live
curl http://localhost:3001/health
curl http://localhost:3001/metrics

# 2. Rodar o TUI
nix develop --command cargo run -- client

# 3. Rodar o doctor
nix develop --command cargo run -- doctor --json

# 4. Testar o pipeline DSPy (se tiver LLM_API_KEY)
cd agents && poetry run python -m neoland_agents.app
curl http://localhost:8001/health
```

Isso te dá o "feeling" do que está rodando de fato.

### Curto prazo (essa semana)

1. **Decidir o destino do frontend**: ele existe em outro repo? Vai ser recriado? Remover referências falsas do README/ROADMAP.
2. **Atualizar métricas**: ROADMAP.md com números reais (275 testes), status honesto do frontend.
3. **Limpar git tree**: decidir o que commitar e o que descartar.
4. **Documentar dependências externas**: Bridge, ml-ops, frontend — onde estão, como montar.

### Médio prazo

1. **Inicializar ADR toolchain** ou rebaixar claims no README.
2. **Script de bootstrap completo**: um comando que sobe tudo (ou documenta o que falta).
3. **CI que realmente roda**: os gates do README deveriam ser verificáveis por qualquer um.

---

## Checklist de Reentrada

```
[ ] nix develop --command cargo check --lib     ← confirma que build funciona
[ ] nix develop --command cargo test --lib      ← 275 testes
[ ] nix develop --command cargo run -- server   ← sobe e testa endpoints
[ ] Rever docs/ADR/                             ← relembrar decisões
[ ] Identificar o que está incomodando          ← fazer sua própria lista
[ ] Limpar git tree antes de começar a mexer    ← separar o que é trial do que é real
```
