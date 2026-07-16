# Neoland — Release Roadmap

**Versão**: final · **Data**: 2026-07-16 · **Status**: Pre-Release

---

## Superfícies do Produto

| Superfície | Stack | Público | Status |
|---|---|---|---|
| **TUI** | Rust + ratatui | Devs, power users | ✅ |
| **Web Console** | Leptos WASM + CSS artesanal | End users, times | 🚧 |
| **Desktop App** | Tauri + Leptos | Cross-platform | 🚧 |
| **CLI / API** | Rust + axum + tonic | Integrações | ✅ |

---

## Topologia

```
TUI · Web · Desktop
        ↓ REST :3001 / gRPC :50051
   Neoland Control Plane (Rust)
        ↓ HTTP
   SecureLLM Bridge (:8080, mTLS + PII redact)
        ↓ HTTP
   ml-ops-api (:8083, VRAM-aware routing)
        ↓ HTTP
   llama.cpp (:8081) · vLLM (opt)
```

---

## Linha do Tempo

### v0.2.0 — Honest Preview · 2 semanas

**Meta**: o Web Console deixa de ser mock e começa a conversar com backend real.

| # | Entrega | Status |
|---|---------|--------|
| 1 | Web Console consome REST + SSE do backend real (sem mock) | ✅ |
| 2 | Streaming de resposta do agente funcionando no browser | ✅ |
| 3 | Sessões reais carregadas do PostgreSQL no painel lateral | ✅ |
| 4 | Build WASM no Nix flake (`nix build .#neoland-web`) | ✅ |
| 5 | `cargo test --workspace` passa (TUI + Web) | ✅ |
| 6 | README reescrito — Leptos, TUI, CLI, sem menção a Next.js | ✅ |
| 7 | Screenshots reais no README (TUI dump + Web screen) | ✅ |

**Gate**: `nix build .#neoland-web && cargo test --workspace` passa limpo. Web mostra sessão real com streaming.

---

### v0.3.0 — Full Stack Beta · 4 semanas

**Meta**: stack completa operando em single-host, deployment automatizado.

| # | Entrega | Status |
|---|---------|--------|
| 1 | gRPC-web bridge — tonic-web server-side (HTTP/1.1 + CORS) | ✅ |
| 2 | Deploy estático configurável (`--web-dist` + env var) | ✅ |
| 3 | PWA — service worker + manifesto + icons | ✅ |
| 4 | Font IBM Plex Mono Nix derivation | ✅ |
| 5 | Protobuf types para WASM (prost-build) | ✅ |
| 6 | Dashboard de métricas no Web (latência, tokens, confidence) | ✅ |
| 7 | Quickstart validado — NixOS e Ubuntu, <5 min até first task | ✅ |
| 8 | Smoke E2E com LLM real — pipeline completo gera ADR/checkpoint | ✅ |
| 9 | Testes de integração Leptos (`wasm-bindgen-test`) | ✅ |
| 10 | gRPC-web client WASM (tonic → tokio, deferred to v0.4.0) | ⏳ |

**Gate**: stack sobe com `nix run .#neoland-full` ou `docker compose up`. Web Console funcional com backend real. Quickstart verificável por terceiro.

---

### v0.4.0 — Desktop & Multi-tenant · 6 semanas

**Meta**: desktop app nativo + multi-tenant auth + Kubernetes.

| # | Entrega |
|---|---|
| 1 | Tauri desktop app — Linux, macOS, Windows | ✅ |
| 2 | OAuth2 (Google/GitHub) + RBAC multi-tenant | ✅ |
| 3 | SSO / LDAP para enterprise |
| 4 | Kubernetes Helm chart — 3+ réplicas, HA | ✅ |
| 5 | mTLS end-to-end em todos os planes | ✅ |
| 6 | vLLM como backend opcional (além de llama.cpp) |
| 7 | Documentação OpenAPI publicada (GitHub Pages) | ✅ |
| 8 | Multi-backend routing — seleção automática por workload |

**Gate**: desktop app instalável via Nix. Multi-tenant funcional. Helm chart deploya em cluster.

---

### v1.0.0 — Public Release · 8 semanas

**Meta**: pronto para o mundo. Compliance, soak, launch.

| # | Entrega |
|---|---|
| 1 | SOC2 Type I + GDPR compliance docs |
| 2 | Soak period — 2 semanas de tasks reais sem incidentes P0 |
| 3 | Blog post técnico + launch HN/Reddit |
| 4 | Site público com landing page, docs, playground |
| 5 | GitHub Discussions / Discord para comunidade |
| 6 | Release notes finais — changelog, breaking changes, migration |
| 7 | Video demo — TUI + Web + Desktop em ação |
| 8 | Roadmap pós-1.0 — advanced analytics, ledger automation, mobile nativo |

**Gate**: Tag `v1.0.0`. Soak limpo. Docs completos. Comunidade aberta.

---

## Backlog Completo (por domínio)

### Control Plane (Rust)
- [x] REST 15 endpoints + gRPC
- [x] Auth RBAC + rate limiting + audit middleware
- [x] DSPy agent pipeline (4 estágios)
- [x] ADR ledger (Merkle chain, secp256k1, NATS JetStream)
- [x] CLI completo (server, client, test, doctor, restart, secrets)
- [x] SSE streaming com timeout e degraded-state handling
- [x] Session persistence PostgreSQL + checkpoint relay
- [x] Health/liveness/readiness probes reais
- [x] mTLS end-to-end
- [ ] Multi-backend routing (llama.cpp + vLLM)
- [ ] Multi-tenant auth (OAuth2, SSO, LDAP)

### Agent Pipeline (Python · DSPy)
- [x] 26 contract tests (Pydantic + signatures)
- [x] Checkpoint artifacts em `$NEOLAND_CHECKPOINT_DIR`
- [x] FastAPI health endpoint
- [x] Smoke E2E com LLM real documentado
- [ ] Advanced ranking analytics por estágio
- [ ] Deeper ADR-ledger automation

### TUI (Rust · ratatui)
- [x] Layout 3 colunas (Sessions / Conversation / Reasoning)
- [x] 4 temas (Tokyo Night, Neon Glass, High Contrast, Monochrome)
- [x] Chat bubbles + code blocks + streaming modes
- [x] Pipeline tree com confidence badges + RWA anchors
- [x] Sistema de comandos (/help, /provider, /theme, /search, etc.)
- [x] Breakpoints interativos + steering
- [x] Sessões com persistência
- [ ] Smoke interativo com LLM real
- [ ] Nerd Font icons como asset bundle Nix

### Web Console (Leptos WASM)
- [x] Layout 3 painéis (Sessions / Conversation / Reasoning)
- [x] CSS artesanal 600 linhas, zero dependências
- [x] Componentização (topbar, sessions, conversation, reasoning, composer)
- [x] Streaming simulado (typewriter animation)
- [x] Responsivo (720px, 980px breakpoints)
- [x] Cargo workspace integrado com root
- [x] Consumir REST + SSE do backend real
- [x] Build WASM deterministico no Nix flake
- [x] Testes de unidade (`cargo test -p neoland-web`, 16 testes)
- [x] gRPC-web bridge server-side (tonic-web + HTTP/1.1 + CORS)
- [x] Font IBM Plex Mono bundle Nix
- [x] PWA service worker + manifesto + icons
- [x] Dashboard de métricas (MetricsPanel component)
- [x] Deploy estático servido pelo próprio Neoland (`--web-dist`)
- [x] Testes de integração Leptos (`wasm-bindgen-test`, 12 testes WASM)

### Desktop (Tauri)
- [x] App shell nativa (Linux, macOS, Windows) — Tauri v2 scaffold
- [x] System tray + notificações — tray.rs com menu Show/Hide/Quit
- [x] Instalação via Nix (`nix profile install`) — neoland-desktop derivation
- [ ] Modo offline total — embedded Neoland server
- [ ] Atalhos de teclado globais
- [ ] DMG / AppImage / MSI installers via CI/CD
- [ ] Auto-start on login
- [ ] Deep link handling (`neoland://`)

### Ops & Infra
- [x] Nix flake (dev shell, env vars, SOPS, DB)
- [x] Docker Compose (PostgreSQL + NATS + DSPy + control plane)
- [x] SecureLLM Bridge + ml-ops-api Docker configs
- [x] GPU passthrough CDI para VRAM-aware routing
- [x] `just smoke` + `just preflight`
- [x] Nix flake com build WASM (`nix build .#neoland-web`)
- [x] Quickstart validado — NixOS e Ubuntu, <5 min até first task
- [x] Kubernetes Helm chart (3+ réplicas, HA)
- [x] mTLS end-to-end
- [ ] Backup/restore + rollback documentado
- [ ] Deploy non-Nix documentado (Ubuntu bare metal)

### Docs & Comunidade
- [x] ADRs arquiteturais (001-015+)
- [x] Product vision + user journey
- [x] README honesto (Leptos, TUI, CLI, screenshots, sem Next.js)
- [x] Quickstart <5 min (NixOS + Ubuntu)
- [x] Smoke E2E documentado com llama.cpp real
- [x] OpenAPI docs publicadas
- [ ] Landing page pública
- [ ] Blog post técnico de launch
- [ ] Video demo (TUI + Web + Desktop)
- [ ] GitHub Discussions / Discord
- [ ] SOC2 Type I + GDPR docs

---

## Decisões de Arquitetura

| Decisão | Escolha |
|---|---|
| Web frontend | Leptos WASM (100% Rust, zero Node) |
| CSS | Artesanal (zero build step) |
| gRPC-web | tonic-web nativo (sem Envoy proxy) |
| Font | IBM Plex Mono via Nix derivation |
| Workspace | Cargo: root (TUI) + `web/` (Leptos) |
| Desktop | Tauri + Leptos WASM |
| Infra | Nix-first, Docker compat, Kubernetes para HA |
| Auth | RBAC local → OAuth2 → SSO/LDAP |
| LLM Backend | llama.cpp primário, vLLM opcional |
| Compliance | SOC2 Type I no v1.0 |

---

## Gate de Release Final (v1.0.0)

- [ ] `cargo test --workspace` passa (TUI + Web + Desktop)
- [ ] `nix build .#neoland-full` gera stack completa deterministicamente
- [ ] Web Console funcional com backend real (streaming, sessões, ADRs)
- [ ] Desktop app instalável em Linux, macOS, Windows
- [ ] Multi-tenant auth operacional (OAuth2 + SSO)
- [ ] Kubernetes Helm chart deploy funcional
- [ ] mTLS em todos os planes
- [ ] Smoke E2E completo com LLM real
- [ ] Quickstart <5 min em NixOS e Ubuntu
- [ ] Docs completos (README, OpenAPI, ADRs, compliance)
- [ ] Soak period 2 semanas sem incidentes P0
- [ ] Comunidade pública aberta (GitHub Discussions / Discord)

---

## Progressão de Score

| Área | Agora | v0.2.0 | v0.3.0 | v0.4.0 | v1.0.0 |
|---|---|---|---|---|---|
| Control Plane | 95 | 95 | 96 | 98 | 100 |
| DSPy Pipeline | 85 | 85 | 90 | 92 | 95 |
| TUI | 90 | 90 | 92 | 95 | 98 |
| Web Console | 45 | 65 | 87 | 92 | 95 |
| Desktop | 40 | 0 | 15 | 75 | 90 |
| Ops/Infra | 75 | 80 | 87 | 94 | 98 |
| Docs | 60 | 75 | 87 | 92 | 95 |
| **Geral** | **78** | **78** | **91** | **93** | **96** |

---

**Mantido por**: VoidNxSEC Team
