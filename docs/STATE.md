# Neoland — Estado do Projeto

> **Documento de continuidade.** Existe para que uma sessão nova (operador ou
> agente) comece com referências em vez de redescobrir o mesmo terreno. Toda
> afirmação aqui tem evidência: arquivo, linha, ou comando que a reproduz.
>
> **Última verificação:** 2026-08-22 · HEAD `070d01dc` (branch `dev`, 34 commits à frente de `main`)

## Como usar

- **Começando uma sessão:** leia "Ship-state", "Achados abertos" e "Decisões pendentes".
  São as três seções que mudam o que faz sentido fazer.
- **Terminando uma sessão:** atualize o que mudou. Documento desatualizado é pior
  que documento ausente — quem lê acredita nele.
- **Achado novo:** registre com evidência (`arquivo:linha` ou comando). Sem evidência,
  vira folclore e a próxima sessão vai reinvestigar.

---

## 1. Ship-state honesto

**Protótipo.** Não Alpha. A distinção importa e é específica:

A **engenharia** já passou de protótipo: CI real e verde, testes e2e do caminho
crítico com Postgres e SSE de verdade, RBAC enforçado, graceful shutdown,
migrations embutidas, OpenAPI 24/24, smoke que prova que o binário sobe e encerra
limpo.

A **experiência de rodar** ainda é protótipo. O gate honesto para Alpha não é
técnico — é: *alguém que não é o operador consegue baixar, rodar e ver o pipeline
funcionar sem perguntar nada?* Hoje não consegue, pelos motivos da seção 3.

| Dimensão | Estado | Evidência |
|---|---|---|
| Control plane (REST/gRPC/SSE) | Funciona | `tests/rest_api_test.rs` (32), `grpc_integration_test.rs` (8) |
| Pipeline multi-agente | Funciona **com** Postgres | `tests/agent_e2e_test.rs` (6): steering, breakpoints, SSE |
| RAG do pipeline Python | **Quebrado** | §4.1 |
| Boot offline | **Não** | §4.2 |
| Binário standalone | Sobe e encerra limpo | `scripts/smoke-binary.sh`, job `binary-smoke` |
| Release ponta a ponta | **Nunca exercitada** | nenhuma tag `v*` foi puxada |
| Docs vs realidade | Divergem | README/ROADMAP contra `INVENTORY.md` |

---

## 2. Acoplamentos

Quatro tipos, com origens e custos diferentes. Só o quarto é o que normalmente se
chama de "desacoplamento" — e é o menos urgente.

### 2.1 Rust ↔ Python: dois donos do mesmo banco

O pipeline Python **não fala com o control plane por API**. Abre `asyncpg` direto
na tabela `documents`, que pertence ao `VectorStore` do Rust
(`agents/neoland_agents/rag/retriever.py:26`). Não há contrato: duas bases
escrevem e leem o mesmo schema, cada uma com seu driver e suas suposições.

Foi exatamente aqui que o RAG morreu sem ninguém notar (§4.1).

Somado a isso, `src/agents/client.rs` espelha `agents/neoland_agents/schemas/api.py`
**manualmente**. O CLAUDE.md registra a obrigação ("qualquer mudança num lado deve
ser refletida no outro"), mas regra em documento não é contrato — nada quebra se
alguém esquecer.

**Decisão pendente:** §5.1.

### 2.2 Cross-repo via Cargo (git deps pinadas por SHA)

```
spectre-events       → VoidNxSEC/spectre           rev 7f9548dd
securellm-core       → VoidNxSEC/securellm-bridge  rev 1135bdc1
securellm-providers  → VoidNxSEC/securellm-bridge  rev 1135bdc1
```

Acoplamento legítimo (é um ecossistema), mas **rígido**: SHA congelado, sem
versionamento, sem janela de compatibilidade. Cada bump é manual e cross-repo.

Custo já cobrado: as 4 advisories de `rustls-webpki 0.102.8` que sobram no
`cargo audit` entram por `spectre-events → async-nats 0.46`. Não têm correção
possível neste repo — dependem de mover o pin no spectre.

### 2.3 Cross-repo via flake inputs (o silencioso)

Não aparece em `Cargo.toml`, não aparece em nenhum `use`. Só existe no `flake.nix`,
e mesmo assim nem todo input declarado é consumido.

| Input | Idade do lock | Consumido? | Observação |
|---|---|---|---|
| `nixpkgs` | 67d | sim | — |
| `rust-overlay` | 57d | sim | **serve a toolchain do devShell** — ver §4.3 |
| `flake-utils` | 647d | sim | quase 2 anos |
| `securellmBridge` | 58d | sim | módulo NixOS |
| `mlOpsApi` | 44d | sim | módulo NixOS |
| `owasaka` | 43d | sim | módulo NixOS |
| `spectre` | 43d | sim | **revisão diverge do Cargo.toml** — ver abaixo |
| `aiAgentOs` | 43d | sim | symlink no `shellHook` |
| `phantom` | 43d | **NÃO** | declarado em `flake.nix:20-21`, nunca usado |

Reproduzir: `nix flake metadata --json`.

**Dois problemas concretos:**

1. **`phantom` é input morto.** Declarado, nunca consumido em nenhum output.
   Custa fetch a cada update, entra no lock, acopla formalmente a um repo que
   o neoland não usa.

2. **`spectre` aponta para duas revisões diferentes.** O flake trava em
   `de14511a`; o `Cargo.toml` trava `spectre-events` em `7f9548dd`. O módulo
   NixOS e o binário Rust consomem versões distintas do mesmo repo. Nada
   detecta isso hoje.

### 2.4 Interno: `AppState` e a ausência de fronteiras

`AppState` tem 10 campos concretos (`src/server/state.rs`) e o crate inteiro
(~21k LOC) tem **4 traits**, nenhuma para as dependências que importam: engine
de inferência, vector store, pipeline de agentes.

```
src/commands.rs:106   CommandRuntime
src/audit.rs:315      AlertHandler
src/mcp/server.rs:26  NativeTool
src/auth/sso.rs:39    IdentityProvider
```

Consequências práticas (não teóricas):

- Não dá para testar um handler sem subir Postgres real
- A lógica de fallback in-memory ↔ pgvector vive espalhada nos call-sites
- O `Mutex` do engine serializa toda inferência, porque não há fronteira onde
  encaixar um actor

**É dívida de manutenção, não de funcionamento.** Menos urgente que os outros três.

### 2.5 Infraestrutura no caminho crítico

- Boot depende de rede (HuggingFace) — §4.2
- Pipeline inteiro depende de `DATABASE_URL`; sem ele o servidor sobe capado
  **sem avisar de forma acionável**

---

## 3. Achados abertos

### 3.1 O RAG do pipeline Python está morto (silencioso)

`agents/neoland_agents/rag/retriever.py:26`:

```python
rows = await conn.fetch(
    "SELECT content FROM documents ORDER BY embedding <=> $1::vector LIMIT $2",
    query,          # ← string da pergunta, não um vetor
    self.top_k,
)
except Exception:
    return ""       # engole o erro
```

O texto da pergunta é passado onde o Postgres espera `vector`. O cast sempre
falha, o `except` engole, o retriever **sempre devolve string vazia**. Nenhum
teste pega: retorno vazio é indistinguível de "não achei nada".

Ou seja: os 4 agentes DSPy rodam sem contexto RAG desde que isso foi escrito.

### 3.2 Boot exige rede: ~90MB do HuggingFace

`init_vector_store()` → `VectorStore::new()` (`src/server/bootstrap.rs:97`) baixa
`sentence-transformers/all-MiniLM-L6-v2` e **bloqueia a subida** até terminar.

O `LocalEngine` (também candle) **não** é problema: é lazy, o `AppState` nasce com
`engine: Mutex::new(None)` e só carrega o GGUF no primeiro chat.

Agravante: `NEOLAND_SKIP_EMBEDDINGS` não desliga embeddings — troca o modelo por um
**stub de hash** (`src/nlp.rs:89`), vetores determinísticos sem significado
semântico. Serve para teste; em produção faria o RAG devolver lixo parecendo que
funciona.

**Direção decidida com o operador:** usar os embeddings do llama.cpp, tirando o
download do caminho crítico. Decisão de modelo pendente — §5.2.

### 3.3 `update-flake-lock` falha há semanas — causa raiz de várias confusões

`.github/workflows/update-flake-lock.yml:23`:

```bash
nix flake update --commit-lock-file --flake ./testFlake
```

**`testFlake` não existe no repositório.** Com `bash -e`, o step morre nessa linha,
antes do `cargo update` e antes de abrir o PR. Runs de 03/08, 10/08 e 17/08: todos
`failure`.

Cadeia completa do estrago:

```
testFlake fantasma
  → update-flake-lock falha toda segunda
    → flake.lock congelado em 2026-07-18
      → rust-overlay 57 dias atrás
        → devShell Rust 1.96 vs CI stable 1.98
          → clippy diverge; CI vermelho irreproduzível localmente
```

**Correção:** remover a linha 23 (ou criar o diretório, se ele deveria existir).
Uma linha destrava a atualização de todos os inputs.

### 3.4 Toolchain do devShell diverge da do CI

devShell pina via `rust-overlay` (`rust-bin.stable.latest`, hoje **1.96.0**); o CI
usa `dtolnay/rust-toolchain@stable`, que flutua (hoje **1.98.0**). Lints novos
aparecem só no CI.

**Armadilha que custou tempo:** `rustup run 1.98.0 cargo clippy` **não** usa o
clippy 1.98. O subcomando resolve `cargo-clippy` pelo `PATH`, e o devShell coloca o
binário do Nix (1.96) na frente. Roda, sai 0, e a validação é falsa.

Conferir sempre: `which cargo-clippy && cargo-clippy --version`.

Reproduzir o CI de verdade:
```bash
export PATH="$HOME/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/bin:$PATH"
```

Na prática, o caminho certo é **não** reproduzir localmente: commitar e deixar o
Actions responder (preferência explícita do operador).

### 3.5 Gate ADR reprova `feat` com testes inline

`.hooks/feature_detector.py` exige evidência de integração no mesmo commit para
`feat`/`fix` que toquem `src/server/`, `src/tui/`, `src/matrix/`, `agents/`. A
evidência é buscada por **caminho de arquivo** (`^tests/`, `^adr/`).

Testes inline (`#[cfg(test)] mod tests`), que são o padrão do Rust, não contam.
Consequência: qualquer `feat` idiomático tocando `src/server/` reprova, mesmo
trazendo testes.

Reprovou o commit `c46e1c1a` (hardening do ShellTool, com 6 testes novos inline).
Heurística já foi refinada uma vez (`539df501`) — refinar de novo é **decisão do
operador**, §5.3.

### 3.6 `cargo audit`: 4 advisories restantes

De 7 para 4 nesta sessão. As 4 restantes são todas `rustls-webpki 0.102.8`, via
`spectre-events → async-nats 0.46` (§2.2). Não têm correção neste repo.

`RUSTSEC-2023-0071` (Marvin, `rsa`) está ignorada com justificativa em
`.cargo/audit.toml`: `rsa` só entra via `sqlx-mysql`, e apenas a feature `postgres`
está habilitada — nunca é compilado. Verificar com `cargo tree -i rsa` (retorna
"nothing to print").

Nota: bumpar `async-nats` para 0.50 **compila**, mas colocaria duas cópias do crate
no binário enquanto o webpki velho continuaria entrando pelo `spectre-events`.
Não compensa.

---

## 4. Decisões pendentes do operador

Nenhuma delas o agente deve resolver sozinho.

### 4.1 Quem é dono do vector store?

Duas arquiteturas, muito divergentes:

- **Control plane é dono único** — o Rust expõe busca vetorial como endpoint e o
  Python consome. Um dono do schema, contrato explícito, o Python deixa de precisar
  de credencial de banco.
- **Vector store é infra compartilhada** — ambos são clientes legítimos, e o que
  falta é schema versionado + testes de contrato dos dois lados.

Muda tudo o que vem depois no §2.1.

### 4.2 Qual modelo de embeddings no llama.cpp?

O schema tem dimensão fixa: `embedding vector(384)`
(`migrations/001_create_vector_store.sql:15`) e `EMBEDDING_DIM = 384`
(`src/nlp.rs:11`).

- **MiniLM-L6-v2 em GGUF** — 384 dims, schema intacto, zero migração. Menor risco.
- **Trocar de modelo** (nomic=768, Qwen=1024) — melhor retrieval, mas exige
  migration da coluna, reindexação e decisão sobre os vetores existentes.

Em qualquer caso, passa a ser obrigatório **validar na inicialização que a dimensão
devolvida bate com a do schema**, falhando alto. Hoje nada checa.

Sub-decisão: o cliente de embeddings fica no neoland ou sobe para o
`securellm-bridge` (dono dessa camada no ecossistema; outros consumidores
ganhariam junto)? O `LlamaCppProvider` atual vem de `securellm-providers` e
implementa só a interface de chat do `securellm-core`.

### 4.3 Alinhar toolchains

Atualizar `rust-overlay` no flake (alinha, mas mexe no build — é do operador) ou
pinar a versão no CI (congela lints novos). Enquanto divergirem, validação local é
sempre uma versão atrás.

Depende de §3.3 estar corrigido para o update sequer funcionar.

### 4.4 Gate ADR e testes inline

Refinar a heurística para aceitar testes inline como evidência, ou manter e conviver
com o falso positivo.

### 4.5 Revisão do `spectre`

Reconciliar `flake.lock` (`de14511a`) com `Cargo.toml` (`7f9548dd`), e decidir se o
pin avança para liberar as 4 advisories de webpki.

---

## 5. Fila priorizada

Ordenada por *destrava o Alpha*, não por elegância.

| # | Item | Tamanho | Por quê agora |
|---|---|---|---|
| 1 | Corrigir `update-flake-lock` (§3.3) | XS | Uma linha; destrava a atualização de todos os inputs |
| 2 | Consertar o RAG do Python (§3.1) | S | Bug ativo; força a decisão §4.1 |
| 3 | Embeddings via llama.cpp (§3.2) | M | Gate real do Alpha: boot offline |
| 4 | Tag de protótipo | S | Prova que o release produz artefato instalável |
| 5 | Limpar `phantom` + reconciliar `spectre` (§2.3) | S | Acoplamento morto e divergente |
| 6 | Alinhar docs com a realidade | M | Depois que o resto estabilizar |
| 7 | Traits `VectorIndex`/`AgentPipeline` + engine actor (§2.4) | L | Dívida interna; não move a agulha do Alpha |

`main` está 34 commits atrás de `dev`; PR #12 (`dev`→`staging`) aberto aguardando
merge do operador.

---

## 6. Referência operacional

Comandos que funcionam, para não redescobrir.

```bash
# Toolchain só existe dentro do dev shell (fora, o linker quebra)
nix develop --command cargo check --lib
nix develop --command cargo test --lib

# E2E com Postgres real (container docker-postgres-1)
DATABASE_URL=postgresql://neoland:neoland_dev@localhost:5432/neoland \
NEOLAND_TEST_KEEP_DATABASE=1 \
  cargo test --test agent_e2e_test -- --test-threads=1

# Smoke do binário (o que o CI roda)
nix build .#neoland && ./scripts/smoke-binary.sh result/bin/neoland

# Nix: eval barato local; check completo é do CI
nix flake check --no-build

# Auditoria de inputs do flake
nix flake metadata --json
```

**Convenções:**

- Build pesado, `fmt` e validação em toolchain alternativa: **deixar para o
  Actions**. Local só validação leve.
- Sem mocks: testes de integração usam LLM e Postgres reais.
- `src/llm/unified_client.rs` é o único ponto de acesso ao LLM.
- Mudança em `src/agents/client.rs` exige espelho em
  `agents/neoland_agents/schemas/api.py`.
- Merge, tag, rebuild e force-push são do operador.
- A seção "Error body contract" de `tests/rest_api_test.rs` trava os bodies de erro
  byte a byte — o TUI os parseia. Se ela quebrar, o contrato com o TUI quebrou.

**Armadilhas conhecidas:**

- `neoland` no devShell **não é o binário** — é um wrapper (`writeShellApplication`)
  que chama `scripts/neoland-run.sh`, decripta sops e faz `cargo run`. O artefato de
  release é `nix build .#neoland`. Mesmo nome, coisas diferentes.
- O `DATABASE_URL` exportado pelo devShell aponta para um socket unix inexistente
  (`postgresql:///neoland?host=/run/postgresql`). Os recipes de DB do `justfile`
  sobrescrevem deliberadamente.
- `cargo-clippy` do `PATH` mascara o de `rustup run` — §3.4.
