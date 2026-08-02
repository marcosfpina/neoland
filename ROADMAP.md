# Neoland — Release Roadmap

**Versão atual**: v0.0.1  
**Atualizado em**: 2026-08-02  
**Fase**: release candidate — fechamento de prontidão para v0.1.0

Este documento é a fonte de verdade para direção e gates de release. Itens de
implementação descobertos durante o trabalho pertencem ao
[`plans/production-readiness-todo.md`](plans/production-readiness-todo.md) ou ao
backlog local; aqui ficam apenas marcos, evidências e bloqueios de lançamento.

## Estado atual

| Superfície | Estado comprovado | Limite atual |
|---|---|---|
| Control plane / CLI | REST, gRPC, SSE, RBAC, sessões, mTLS e multi-backend implementados | smoke completo ainda depende dos serviços externos |
| TUI | fluxo agent-first, steering, breakpoints, persistência e layout responsivo | smoke terminal com `expect` precisa ser modernizado e executado |
| Web Console | Leptos WASM conectado a REST/SSE, PWA, métricas e bundle Nix | cliente gRPC-web WASM foi adiado; REST/SSE é o caminho suportado |
| Desktop | shell Tauri, tray e notificações | derivation Nix apenas valida; instaladores nativos não são publicados |
| Operação | Nix, Docker Compose, Helm, probes, mTLS e runbooks de DR | render Helm, build Nix e smoke devem passar no ambiente de release |
| Compliance / comunidade | controles técnicos e documentação de segurança existem | auditoria SOC 2, revisão GDPR, canais públicos e soak são externos ao código |

## Marcos concluídos

### M1 — Honest Preview

- [x] Web Console consome REST e SSE reais.
- [x] Sessões persistidas aparecem na interface.
- [x] Build WASM faz parte do flake e do workspace Cargo.
- [x] README e screenshots representam a stack Rust/Leptos atual.
- [x] Testes unitários do Web Console estão integrados ao CI.

### M2 — Full Stack Beta

- [x] Bridge gRPC-web server-side com `tonic-web`.
- [x] Bundle Web estático servido pelo control plane (`--web-dist`).
- [x] PWA, IBM Plex Mono e painel de métricas.
- [x] Docker Compose e artefato `neoland-full` definidos.
- [x] Kubernetes Helm chart com HA, probes e secrets explícitos.
- [x] mTLS nativo nos listeners REST e gRPC.
- [x] Routing para llama.cpp e vLLM.

### M3 — Segurança e operação do release candidate

- [x] Endpoints REST e gRPC separados na configuração da TUI.
- [x] Live steering usa o mesmo contrato `X-API-Key` do control plane.
- [x] Docker e Helm recusam chaves de desenvolvimento por padrão.
- [x] Liveness usa `/live`; readiness usa `/ready`.
- [x] Refresh token é validado no PostgreSQL e gera novo access token.
- [x] Logout revoga a sessão persistida.
- [x] Callbacks OAuth persistem o refresh token antes de retorná-lo.
- [x] TUI avisa e bloqueia submissões sem `NEOLAND_API_KEY`.
- [x] Layout TUI coberto em 80×24 e 120×30.
- [x] Backup, restore, DR e rollback documentados.
- [x] Validador de production readiness aponta para os workflows e caminhos atuais.

## Gate técnico para v0.1.0

O release só pode ser tagueado quando todos os itens abaixo tiverem evidência da
mesma revisão:

- [x] `cargo fmt --check` (2026-08-02).
- [x] `cargo clippy --all-targets -- -D warnings` (2026-08-02).
- [x] `cargo test --workspace --lib` — 301 core + 16 web passaram; 18 ignorados (2026-08-02).
- [x] Contratos Python — 26/26 via `.venv/bin/python -m pytest tests/ -m contract -v` (2026-08-02).
- [x] Docker Compose renderiza com secrets obrigatórios preenchidos.
- [ ] `helm lint` e `helm template` com valores de produção de teste.
- [ ] `nix build .#neoland-full` e smoke do wrapper.
- [ ] Smoke TLS/mTLS.
- [ ] Smoke full-stack com PostgreSQL, DSPy e um LLM real.
- [ ] Smoke TUI agent-first: task, SSE, `/why`, `/steer` e saída limpa.
- [ ] Quickstart cronometrado em NixOS e Ubuntu.

O workflow canônico remoto é
[`validate-all.yml`](.github/workflows/validate-all.yml). O equivalente local é
`just validate-all`; gates que dependem de credenciais ou serviços devem registrar
data, ambiente e resultado, sem serem convertidos em “passou” por inferência.
Cada execução do workflow publica o artefato `release-evidence` e um resumo no
GitHub Actions; `gh run watch <run-id> --exit-status` é o caminho de monitoramento.

## Gate de lançamento público

Estes itens não podem ser concluídos apenas por uma alteração no repositório:

- [ ] Soak de 14 dias sem incidente P0, com início/fim e responsável registrados.
- [ ] Binários/instaladores Desktop assinados para Linux, macOS e Windows.
- [ ] Revisão jurídica GDPR e auditoria SOC 2 Type I concluídas por responsáveis nomeados.
- [ ] Release notes e guia de migração revisados para a tag final.
- [ ] Demo em vídeo publicada.
- [ ] GitHub Discussions e/ou Discord abertos, moderados e vinculados na documentação.
- [ ] Publicação do site, blog e anúncio coordenada.

## Sequência de fechamento

1. Tornar todos os gates técnicos verdes e anexar as evidências.
2. Gerar e assinar os artefatos Desktop nas três plataformas.
3. Executar o quickstart independente e iniciar o soak de 14 dias.
4. Obter sign-off de segurança/compliance e de operação.
5. Publicar documentação, canais da comunidade e tag `v0.1.0`.

## Pós-v0.1.0

- Analytics de ranking e confiança por estágio do pipeline.
- Automação mais profunda do ADR ledger.
- Cliente gRPC-web WASM, se trouxer vantagem mensurável sobre REST/SSE.
- Desktop offline com control plane embarcado, atalhos globais, auto-start e deep links.
- Analytics avançado, automação de ledger e cliente móvel nativo.

## Critério de encerramento deste roadmap

O roadmap estará concluído quando o gate técnico, o gate público e o soak estiverem
integralmente registrados, e a tag `v0.1.0` apontar para a mesma revisão validada.
Até lá, o status correto do produto é **release candidate**, não “enterprise ready”.

**Mantido por**: VoidNxSEC Team
