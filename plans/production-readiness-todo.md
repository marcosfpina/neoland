# Production Readiness TODO Plan

Data: 2026-07-21
Escopo: sanar as faltas encontradas na avaliacao de prontidao para producao, com foco inicial na TUI, endpoints, entrypoints, deploy e README.

## Objetivo

Deixar o projeto pronto para um release de producao auditavel:

- endpoints coerentes entre TUI, servidor, Docker, Helm e docs;
- entrypoints claros para binario, Docker, Python agents e Web;
- TUI validada em fluxo real, nao apenas render/unit tests;
- README legivel, honesto e alinhado ao estado atual;
- gates locais e CI capazes de bloquear regressao antes do deploy.

## P0 - Bloqueadores de Release

- [x] Corrigir autenticacao do live steering na TUI.
  - Problema: `post_agent_steer` usa `Authorization: Bearer <api_key>`, mas o middleware REST legado exige `X-API-Key`.
  - Aceite: `/steer`, Enter durante task ativa e Shift+Enter usam o mesmo header aceito por `/v1/agents/task`.
  - Verificacao: `post_agent_steer` (src/tui/mod.rs) agora envia `X-API-Key`; testes `e2e_agent_steer_requires_auth`, `e2e_agent_steer_rejects_bearer_header`, `e2e_agent_steer_with_auth` em `tests/rest_api_test.rs` — todos passam.

- [x] Separar endpoint REST e endpoint gRPC na TUI/CLI.
  - Problema: `server_url` default da TUI e REST e `try_grpc_fallback` tenta gRPC no mesmo URL.
  - Aceite: CLI/config tem URLs explicitas — `--server-url`/`NEOLAND_SERVER_URL` (REST, default `:3001`) e `--grpc-url`/`NEOLAND_GRPC_URL` (gRPC, default `:50051`), plumbed via `AppState.grpc_url` ate `try_grpc_fallback`.
  - Verificacao: `cargo test --lib cli::` e `cargo test --lib tui::` — 10 e 84 testes passando, incluindo o novo default de `grpc_url` no comando `client`.

- [x] Alinhar variavel do pipeline DSPy no Docker Compose.
  - Problema: compose define `NEOLAND_AGENTS_DSPY_URL`, mas o codigo le `NEOLAND_DSPY_URL`.
  - Aceite: control-plane em Docker aponta para `http://dspy-pipeline:8001`.
  - Verificacao: `docker compose config` (neoland/deploy/docker/docker-compose.yml e docker-compose.master.yml na raiz) confirmam `NEOLAND_DSPY_URL` renderizado corretamente.

- [x] Corrigir secrets/env vars no Helm.
  - Problema: `envFrom` injeta keys como `database-url` e `jwt-secret`, mas o binario espera `DATABASE_URL`, `NEOLAND_JWT_SECRET` e `NEOLAND_*_API_KEY`.
  - Aceite: Deployment recebe env vars com nomes validos e esperados pelo codigo.
  - Verificacao: `helm template` mostrando `DATABASE_URL`, `NEOLAND_JWT_SECRET`, `NEOLAND_ADMIN_API_KEY`, `NEOLAND_USER_API_KEY`, `NEOLAND_READONLY_API_KEY` — chaves agora `required` no chart (falha cedo se nao configuradas).

- [x] Ativar guard contra chaves de desenvolvimento em producao.
  - Problema: o app so recusa dev keys quando `NEOLAND_REQUIRE_VAULT_KEYS=1` esta definido.
  - Aceite: Helm/Docker/prod docs definem `NEOLAND_REQUIRE_VAULT_KEYS=1`; startup falha se cair em dev keys.
  - Verificacao: `NEOLAND_REQUIRE_VAULT_KEYS=1` agora default em `values.yaml` (`config.auth.requireVaultKeys: true`) e em ambos os `docker-compose.yml` (`${NEOLAND_REQUIRE_VAULT_KEYS:-1}`); confirmado via `helm template` e `docker compose config`.

- [x] Corrigir readiness/liveness no Helm.
  - Problema: readiness usa `/live`, que so confirma processo vivo.
  - Aceite: liveness usa `/live`; readiness usa `/ready` ou outro endpoint que represente prontidao operacional real.
  - Verificacao: `helm template --show-only templates/deployment-server.yaml` confirma `readinessProbe.httpGet.path: /ready`.

## P1 - TUI para Producao

- [ ] Rodar E2E real da TUI com `expect`.
  - Atual: `cargo test --lib tui::` passou com 84 testes; dump visual passou.
  - Pendente: smoke terminal com servidor real.
  - Verificacao: `tests/e2e/run_e2e.sh` ou teste equivalente atualizado.

- [ ] Atualizar E2E TUI para o fluxo agent-first atual.
  - Problema provavel: scripts antigos podem assumir fluxo legacy LLM/ml-offload.
  - Aceite: smoke cobre abrir TUI, enviar task, receber eventos SSE, usar `/why`, usar `/steer`, sair limpo.

- [ ] Validar UX em terminal estreito e comum.
  - Aceite: sem panic e sem texto ilegivel em pelo menos 80x24 e 120x30.
  - Verificacao: testes `ratatui::TestBackend` para tamanhos minimos.

- [ ] Garantir erro claro quando `NEOLAND_API_KEY` esta ausente.
  - Problema: TUI usa string vazia e falha depois no servidor.
  - Aceite: banner/notification acionavel antes de enviar task.

## P1 - EntryPoints e Runtime

- [ ] Documentar entrypoints reais.
  - Rust: `src/bin/neoland.rs`.
  - CLI: `neoland server`, `neoland client`, `neoland doctor`, `neoland test`.
  - Docker: `deploy/docker/entrypoint.sh` + `CMD ["server"]`.
  - Python agents: `agents/neoland_agents/app.py`.
  - Web: `web/src/main.rs` via Trunk ou bundle servido pelo control-plane.

- [ ] Validar `neoland` sem subcomando.
  - Atual: CLI assume `server`.
  - Aceite: README e help deixam isso explicito.

- [ ] Padronizar nomes de variaveis por superficie.
  - REST API.
  - gRPC API.
  - DSPy pipeline.
  - LLM gateway.
  - Database.
  - JWT/API keys.

## P1 - README e Docs

- [ ] Atualizar Release status do README.
  - Problema: numeros documentados divergem da validacao local.
  - Ultima validacao local: `cargo test --lib` = 301 passed, 18 ignored; `cargo test -p neoland-web` = 16 passed.

- [ ] Remover ou qualificar claim "Enterprise Ready / 94/100".
  - Aceite: README reflete status real: "release candidate", "prod blockers open" ou equivalente ate P0 fechar.

- [ ] Corrigir endpoints documentados dos agents Python.
  - Problema: README cita `/run` e `/sessions`, mas FastAPI real usa `/v1/pipeline/run` e `/v1/pipeline/session/{session_id}`.

- [ ] Criar secao curta "Production Checklist".
  - Deve listar: secrets, DB migrations, API keys, JWT secret, TLS/reverse proxy, readiness, backup, smoke, rollback.

- [ ] Separar Quick start dev de Deploy prod.
  - Aceite: dev local continua simples; prod nao mistura Docker/Helm/Nix sem criterio claro.

## P1 - Deploy Docker/Helm/Nix

- [ ] Docker Compose deve usar nomes de env que o binario realmente le.
- [ ] Docker Compose deve expor/verificar health de control-plane e dspy-pipeline.
- [ ] Helm deve renderizar secrets com `valueFrom.secretKeyRef` explicito ou keys env-validas.
- [ ] Helm deve definir API keys e JWT secret obrigatorios para prod.
- [ ] Helm deve alinhar UID/GID com a imagem Docker ou garantir permissao nos volumes.
- [ ] Nix package `neoland-full` deve ser validado como artefato release.

## P2 - Gates e Automacao

- [ ] Atualizar `scripts/validate-production-readiness.sh`.
  - Problema: script procura workflows antigos (`test.yml`, `lint.yml`, `build.yml`) que nao existem mais.
  - Aceite: script valida workflows reais (`ci.yml`, `validate-all.yml`, `secret-scan.yml`, `trivy.yml`, etc.).

- [ ] Adicionar gate de coerencia de endpoints.
  - Deve checar variaveis declaradas em Docker/Helm contra `Config::apply_env_overrides`.

- [ ] Adicionar teste para headers da TUI.
  - Deve impedir regressao `Authorization` vs `X-API-Key` nos fluxos legados de API key.

- [ ] Adicionar smoke local documentado.
  - Minimo: server sobe, `/health`, `/ready`, `/v1/agents/health`, TUI abre, task simples, `/steer`.

- [ ] Rodar preflight completo apos P0.
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test --lib`
  - `cargo test -p neoland-web`
  - `cd agents && poetry run pytest tests/ -m contract -v`
  - `nix build .#neoland-full`
  - smoke TUI/E2E

## Validacoes ja Executadas Nesta Avaliacao

- [x] `git diff --check`
- [x] `cargo fmt --check`
- [x] `cargo check --lib`
- [x] `cargo test --lib`
  - Resultado: 301 passed, 18 ignored.
- [x] `cargo test -p neoland-web`
  - Resultado: 16 passed.
- [x] `cargo test --lib tui::`
  - Resultado: 84 passed, 1 ignored.
- [x] `cargo test --lib tui::ui::tests::dump_visual -- --ignored --nocapture`
  - Resultado: passou e renderizou welcome, conversa completa e streaming.

## Criterio de Done para Prod

- [ ] Todos os P0 fechados.
- [ ] README alinhado ao estado real.
- [ ] Docker Compose smoke aprovado.
- [ ] Helm render/lint aprovado com secrets corretos.
- [ ] TUI E2E aprovado contra servidor real.
- [ ] CI principal verde.
- [ ] Plano de rollback documentado.
